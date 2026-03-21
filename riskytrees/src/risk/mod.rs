use std::{collections::HashMap, convert::TryFrom};

use serde_json::{Map, Value};

use crate::models;

const ATTACKER_LIKELIHOOD_MODEL_ID: &str = "b9ff54e0-37cf-41d4-80ea-f3a9b1e3af74";
const ATTACK_RISK_MODEL_ID: &str = "f1644cb9-b2a5-4abb-813f-98d0277e42f2";
const EVITA_MODEL_ID: &str = "bf4397f7-93ae-4502-a4a2-397f40f5cc49";

pub fn compute_node_risk(
    trees: &[models::ApiFullComputedTreeData],
    node_id: &str,
    risk_model: &str,
) -> Option<models::ApiGetNodeRiskResponseResult> {
    let computer = RiskComputer::new(trees);
    computer.compute_risk_for_node(node_id, risk_model)
}

struct RiskComputer {
    nodes: HashMap<String, models::ApiFullComputedNodeData>,
    parent_by_child: HashMap<String, String>,
}

impl RiskComputer {
    fn new(trees: &[models::ApiFullComputedTreeData]) -> Self {
        let mut nodes = HashMap::new();
        let mut parent_by_child = HashMap::new();

        for tree in trees {
            for node in &tree.nodes {
                for child_id in &node.children {
                    parent_by_child.insert(child_id.clone(), node.id.clone());
                }

                nodes.insert(node.id.clone(), node.clone());
            }
        }

        Self {
            nodes,
            parent_by_child,
        }
    }

    fn compute_risk_for_node(
        &self,
        node_id: &str,
        risk_model: &str,
    ) -> Option<models::ApiGetNodeRiskResponseResult> {
        match risk_model {
            ATTACKER_LIKELIHOOD_MODEL_ID => self.compute_attacker_likelihood(node_id, &[], None),
            ATTACK_RISK_MODEL_ID => self.compute_attack_risk(node_id, &[], None),
            EVITA_MODEL_ID => self.compute_evita_risk(node_id, &[], None),
            _ => None,
        }
    }

    fn is_node_computable(
        &self,
        node_id: &str,
        seen_node_ids: &[String],
        force_conditions: Option<&HashMap<String, bool>>,
    ) -> bool {
        if seen_node_ids.iter().any(|seen| seen == node_id) {
            return false;
        }

        let Some(node) = self.nodes.get(node_id) else {
            return false;
        };

        if let Some(force_conditions) = force_conditions {
            if let Some(force_value) = force_conditions.get(node_id) {
                return *force_value;
            }
        }

        let node_type = self.get_model_attribute_string(node, "node_type").unwrap_or_default();

        node.conditionAttribute.is_empty() || node.conditionResolved || node_type != "condition"
    }

    fn compute_attacker_likelihood(
        &self,
        node_id: &str,
        seen_node_ids: &[String],
        force_conditions: Option<&HashMap<String, bool>>,
    ) -> Option<models::ApiGetNodeRiskResponseResult> {
        if !self.is_node_computable(node_id, seen_node_ids, force_conditions) {
            return None;
        }

        let node = self.nodes.get(node_id)?;

        let result = if let Some(value) = self.get_model_attribute_number(node, "likelihoodOfSuccess") {
            value
        } else {
            let node_type = self.get_model_attribute_string(node, "node_type").unwrap_or_default();
            let child_likelihood_values: Vec<f64> = node
                .children
                .iter()
                .filter_map(|child_id| {
                    let next_seen = self.add_seen_node(seen_node_ids, node_id);
                    self.compute_attacker_likelihood(child_id, &next_seen, force_conditions)
                })
                .filter_map(|subres| self.get_computed_f64(&subres, "likelihoodOfSuccess"))
                .collect();

            if node_type == "and" {
                child_likelihood_values.iter().product()
            } else {
                let combined = child_likelihood_values.iter().fold(0.0, |acc, value| {
                    let joint_prob = acc * value;
                    acc + value - joint_prob
                });

                combined.min(1.0)
            }
        };

        Some(Self::build_result(
            "likelihoodOfSuccess",
            vec![("likelihoodOfSuccess", Value::from(result))],
        ))
    }

    fn compute_attack_risk(
        &self,
        node_id: &str,
        seen_node_ids: &[String],
        force_conditions: Option<&HashMap<String, bool>>,
    ) -> Option<models::ApiGetNodeRiskResponseResult> {
        let likelihood = self
            .compute_attacker_likelihood(node_id, seen_node_ids, force_conditions)
            .and_then(|result| self.get_computed_f64(&result, "likelihoodOfSuccess"))?;

        if !self.is_node_computable(node_id, seen_node_ids, force_conditions) {
            return None;
        }

        let node = self.nodes.get(node_id)?;
        let mut impact = self.get_model_attribute_number(node, "impactToDefender");

        if impact.is_none() {
            let child_impacts: Vec<f64> = node
                .children
                .iter()
                .filter_map(|child_id| {
                    let next_seen = self.add_seen_node(seen_node_ids, node_id);
                    self.compute_attack_risk(child_id, &next_seen, force_conditions)
                })
                .filter_map(|subres| self.get_computed_f64(&subres, "impactToDefender"))
                .collect();

            if !child_impacts.is_empty() {
                impact = Some(child_impacts.iter().sum());
            } else if let Some(parent_id) = self.parent_by_child.get(node_id) {
                let parent_seen: Vec<String> = seen_node_ids
                    .iter()
                    .filter(|seen| seen.as_str() != node_id)
                    .cloned()
                    .collect();

                impact = self
                    .compute_attack_risk(parent_id, &parent_seen, force_conditions)
                    .and_then(|result| self.get_computed_f64(&result, "impactToDefender"));
            }
        }

        let risk = impact.map(|value| value * likelihood);

        Some(Self::build_result(
            "risk",
            vec![
                ("likelihoodOfSuccess", Value::from(likelihood)),
                (
                    "impactToDefender",
                    impact.map(Value::from).unwrap_or(Value::Null),
                ),
                ("risk", risk.map(Value::from).unwrap_or(Value::Null)),
            ],
        ))
    }

    fn compute_evita_risk(
        &self,
        node_id: &str,
        seen_node_ids: &[String],
        force_conditions: Option<&HashMap<String, bool>>,
    ) -> Option<models::ApiGetNodeRiskResponseResult> {
        if !self.is_node_computable(node_id, seen_node_ids, force_conditions) {
            return None;
        }

        let node = self.nodes.get(node_id)?;

        let safety_impact = self.get_model_attribute_i32(node, "safetyImpact");
        let financial_impact = self.get_model_attribute_i32(node, "financialImpact");
        let privacy_impact = self.get_model_attribute_i32(node, "privacyImpact");
        let operational_impact = self.get_model_attribute_i32(node, "operationalImpact");

        let next_seen = self.add_seen_node(seen_node_ids, node_id);
        let child_values: Vec<models::ApiGetNodeRiskResponseResult> = node
            .children
            .iter()
            .filter_map(|child_id| self.compute_evita_risk(child_id, &next_seen, force_conditions))
            .collect();

        let node_type = self.get_model_attribute_string(node, "node_type").unwrap_or_default();

        let time = self
            .get_model_attribute_i32(node, "time")
            .or_else(|| self.get_inherited_evita_value(&child_values, &node_type, "time"));
        let expertise = self
            .get_model_attribute_i32(node, "expertise")
            .or_else(|| self.get_inherited_evita_value(&child_values, &node_type, "expertise"));
        let knowledge = self
            .get_model_attribute_i32(node, "knowledge")
            .or_else(|| self.get_inherited_evita_value(&child_values, &node_type, "knowledge"));
        let window_of_opportunity = self
            .get_model_attribute_i32(node, "windowOfOpportunity")
            .or_else(|| self.get_inherited_evita_value(&child_values, &node_type, "windowOfOpportunity"));
        let equipment_required = self
            .get_model_attribute_i32(node, "equipmentRequired")
            .or_else(|| self.get_inherited_evita_value(&child_values, &node_type, "equipmentRequired"));

        let total_attack_potential = match (
            time,
            expertise,
            knowledge,
            window_of_opportunity,
            equipment_required,
        ) {
            (Some(time), Some(expertise), Some(knowledge), Some(window), Some(equipment)) => {
                Some(time + expertise + knowledge + window + equipment)
            }
            _ => None,
        };

        let attack_probability = total_attack_potential.map(|total| {
            if total >= 25 {
                1
            } else if total >= 20 {
                2
            } else if total >= 14 {
                3
            } else if total >= 10 {
                2
            } else {
                1
            }
        });

        let risk_financial = Self::get_evita_risk_level(attack_probability, financial_impact);
        let risk_operational = Self::get_evita_risk_level(attack_probability, operational_impact);
        let risk_privacy = Self::get_evita_risk_level(attack_probability, privacy_impact);
        let risk_safety = Self::get_evita_risk_level(attack_probability, safety_impact);

        let single_value_display = format!(
            "RF{} RO{} RP{} RS{}",
            Self::format_optional_i32(risk_financial),
            Self::format_optional_i32(risk_operational),
            Self::format_optional_i32(risk_privacy),
            Self::format_optional_i32(risk_safety)
        );

        Some(Self::build_result(
            "singleValueDisplay",
            vec![
                ("safetyImpact", Self::optional_i32_to_value(safety_impact)),
                ("financialImpact", Self::optional_i32_to_value(financial_impact)),
                ("privacyImpact", Self::optional_i32_to_value(privacy_impact)),
                ("operationalImpact", Self::optional_i32_to_value(operational_impact)),
                ("time", Self::optional_i32_to_value(time)),
                ("expertise", Self::optional_i32_to_value(expertise)),
                ("knowledge", Self::optional_i32_to_value(knowledge)),
                (
                    "windowOfOpportunity",
                    Self::optional_i32_to_value(window_of_opportunity),
                ),
                (
                    "equipmentRequired",
                    Self::optional_i32_to_value(equipment_required),
                ),
                ("riskFinancial", Self::optional_i32_to_value(risk_financial)),
                ("riskOperational", Self::optional_i32_to_value(risk_operational)),
                ("riskPrivacy", Self::optional_i32_to_value(risk_privacy)),
                ("riskSafety", Self::optional_i32_to_value(risk_safety)),
                ("singleValueDisplay", Value::String(single_value_display)),
            ],
        ))
    }

    fn get_inherited_evita_value(
        &self,
        children: &[models::ApiGetNodeRiskResponseResult],
        node_type: &str,
        attribute: &str,
    ) -> Option<i32> {
        if children.is_empty() {
            return None;
        }

        let values: Option<Vec<i32>> = children
            .iter()
            .map(|child| self.get_computed_i32(child, attribute))
            .collect();

        let values = values?;

        if node_type == "and" {
            values.into_iter().max()
        } else if node_type == "or" {
            values.into_iter().min()
        } else {
            None
        }
    }

    fn get_model_attribute_number(
        &self,
        node: &models::ApiFullComputedNodeData,
        key: &str,
    ) -> Option<f64> {
        let attribute = node.modelAttributes.get(key)?;

        attribute
            .value_float
            .or_else(|| attribute.value_int.map(|value| value as f64))
    }

    fn get_model_attribute_i32(
        &self,
        node: &models::ApiFullComputedNodeData,
        key: &str,
    ) -> Option<i32> {
        node.modelAttributes.get(key)?.value_int
    }

    fn get_model_attribute_string(
        &self,
        node: &models::ApiFullComputedNodeData,
        key: &str,
    ) -> Option<String> {
        node.modelAttributes.get(key)?.value_string.clone()
    }

    fn get_computed_f64(
        &self,
        result: &models::ApiGetNodeRiskResponseResult,
        key: &str,
    ) -> Option<f64> {
        result.computed.get(key).and_then(Value::as_f64)
    }

    fn get_computed_i32(
        &self,
        result: &models::ApiGetNodeRiskResponseResult,
        key: &str,
    ) -> Option<i32> {
        result
            .computed
            .get(key)
            .and_then(Value::as_i64)
            .and_then(|value| i32::try_from(value).ok())
    }

    fn add_seen_node(&self, seen_node_ids: &[String], node_id: &str) -> Vec<String> {
        let mut next_seen = seen_node_ids.to_vec();
        next_seen.push(node_id.to_owned());
        next_seen
    }

    fn build_result(
        primary: &str,
        computed_entries: Vec<(&str, Value)>,
    ) -> models::ApiGetNodeRiskResponseResult {
        let mut computed = Map::new();

        for (key, value) in computed_entries {
            computed.insert(key.to_owned(), value);
        }

        models::ApiGetNodeRiskResponseResult {
            computed: Value::Object(computed),
            interface: models::ApiGetNodeRiskResponseInterface {
                primary: primary.to_owned(),
            },
        }
    }

    fn optional_i32_to_value(value: Option<i32>) -> Value {
        value.map(Value::from).unwrap_or(Value::Null)
    }

    fn get_evita_risk_level(
        combined_attack_probability: Option<i32>,
        severity: Option<i32>,
    ) -> Option<i32> {
        let combined_attack_probability = combined_attack_probability?;
        let severity = severity?;

        let evita_table = [
            [0, 0, 1, 2, 3],
            [0, 1, 2, 3, 4],
            [1, 2, 3, 4, 5],
            [2, 3, 4, 5, 6],
        ];

        if severity < 1 || combined_attack_probability < 1 {
            return None;
        }

        let severity_index = usize::try_from(severity - 1).ok()?;
        let probability_index = usize::try_from(combined_attack_probability - 1).ok()?;

        evita_table
            .get(severity_index)
            .and_then(|row| row.get(probability_index))
            .copied()
    }

    fn format_optional_i32(value: Option<i32>) -> String {
        match value {
            Some(value) => value.to_string(),
            None => "null".to_owned(),
        }
    }
}