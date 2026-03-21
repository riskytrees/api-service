use crate::models;
use crate::expression_evaluator;
use crate::recommendations::convert_recommendations_to_list;
use crate::recommendations::recommend_steps_for_path;
use crate::risk::compute_node_risk;
use std::collections::HashMap;

#[test]
fn test_test_runner() {
    assert_eq!(1 + 2, 3);
}

#[test]
fn test_de_json_condition() {
    let res = expression_evaluator::de_json_condition("config[\"hello\"] == \"world\"");
    assert_eq!(res, "config_hello == \"world\"");

    let res2 = expression_evaluator::de_json_condition("config[\"hello\"] == config[\"hello\"]");
    assert_eq!(res2, "config_hello == config_hello");
}

#[test]
fn test_expression_evaluation() {
    let config = models::ApiProjectConfigResponseResult {
        id: "test".to_owned(),
        name: Some("Hello".to_owned()),
        attributes: serde_json::json!({
            "hello": "world",
            "other": false
        })
    };

    assert_eq!(expression_evaluator::evaluate("1 == 1", &config), true);
    assert_eq!(expression_evaluator::evaluate("\"test\" == \"test\"", &config), true);
    assert_eq!(expression_evaluator::evaluate("config[\"hello\"] == \"world\"", &config), true);
    assert_eq!(expression_evaluator::evaluate("config[\"hello\"] == \"test\"", &config), false);
    assert_eq!(expression_evaluator::evaluate("config[\"other\"] == false", &config), true);

}

#[tokio::test]
#[ignore]
async fn test_recommendations() {
    let path = vec!["Threats to RiskyTrees in next 5 years".to_string(), "Cost Flooding".to_string(), "Log flooding".to_string()];
    let result = recommend_steps_for_path(path).await;

    assert!(result.contains(","));

    let final_list = convert_recommendations_to_list(result);

    assert!(final_list.len() > 1);
}

fn model_attribute_string(value: &str) -> models::ModelAttribute {
    models::ModelAttribute {
        value_string: Some(value.to_owned()),
        value_int: None,
        value_float: None,
    }
}

fn model_attribute_int(value: i32) -> models::ModelAttribute {
    models::ModelAttribute {
        value_string: None,
        value_int: Some(value),
        value_float: None,
    }
}

fn model_attribute_float(value: f64) -> models::ModelAttribute {
    models::ModelAttribute {
        value_string: None,
        value_int: None,
        value_float: Some(value),
    }
}

fn make_node(
    id: &str,
    children: Vec<&str>,
    model_attributes: HashMap<String, models::ModelAttribute>,
    condition_resolved: bool,
) -> models::ApiFullComputedNodeData {
    models::ApiFullComputedNodeData {
        id: id.to_owned(),
        title: id.to_owned(),
        description: String::new(),
        modelAttributes: model_attributes,
        conditionAttribute: if condition_resolved {
            String::new()
        } else {
            "config['test'] == true".to_owned()
        },
        children: children.into_iter().map(|child| child.to_owned()).collect(),
        conditionResolved: condition_resolved,
    }
}

fn make_tree(nodes: Vec<models::ApiFullComputedNodeData>) -> models::ApiFullComputedTreeData {
    models::ApiFullComputedTreeData {
        title: "test".to_owned(),
        rootNodeId: "000".to_owned(),
        nodes,
    }
}

fn computed_f64(result: &models::ApiGetNodeRiskResponseResult, key: &str) -> f64 {
    result.computed.get(key).and_then(|value| value.as_f64()).unwrap()
}

fn computed_string(result: &models::ApiGetNodeRiskResponseResult, key: &str) -> String {
    result.computed.get(key).and_then(|value| value.as_str()).unwrap().to_owned()
}

#[test]
fn test_compute_attacker_likelihood_for_and_and_or_nodes() {
    let mut root_attributes = HashMap::new();
    root_attributes.insert("node_type".to_owned(), model_attribute_string("and"));

    let mut child_attributes = HashMap::new();
    child_attributes.insert("likelihoodOfSuccess".to_owned(), model_attribute_float(0.5));

    let tree = make_tree(vec![
        make_node("000", vec!["001", "002"], root_attributes.clone(), true),
        make_node("001", vec![], child_attributes.clone(), true),
        make_node("002", vec![], child_attributes.clone(), true),
    ]);

    let and_result = compute_node_risk(&[tree], "000", "b9ff54e0-37cf-41d4-80ea-f3a9b1e3af74").unwrap();
    assert_eq!(computed_f64(&and_result, "likelihoodOfSuccess"), 0.25);

    root_attributes.insert("node_type".to_owned(), model_attribute_string("or"));
    let or_tree = make_tree(vec![
        make_node("000", vec!["001", "002"], root_attributes, true),
        make_node("001", vec![], child_attributes.clone(), true),
        make_node("002", vec![], child_attributes, true),
    ]);

    let or_result = compute_node_risk(&[or_tree], "000", "b9ff54e0-37cf-41d4-80ea-f3a9b1e3af74").unwrap();
    assert_eq!(computed_f64(&or_result, "likelihoodOfSuccess"), 0.75);
}

#[test]
fn test_compute_attacker_likelihood_skips_unresolved_conditions() {
    let mut root_attributes = HashMap::new();
    root_attributes.insert("node_type".to_owned(), model_attribute_string("and"));

    let mut child_attributes = HashMap::new();
    child_attributes.insert("likelihoodOfSuccess".to_owned(), model_attribute_float(0.5));

    let mut condition_attributes = child_attributes.clone();
    condition_attributes.insert("node_type".to_owned(), model_attribute_string("condition"));

    let tree = make_tree(vec![
        make_node("000", vec!["001", "002"], root_attributes, true),
        make_node("001", vec![], child_attributes, true),
        make_node("002", vec![], condition_attributes, false),
    ]);

    let result = compute_node_risk(&[tree], "000", "b9ff54e0-37cf-41d4-80ea-f3a9b1e3af74").unwrap();
    assert_eq!(computed_f64(&result, "likelihoodOfSuccess"), 0.5);
}

#[test]
fn test_compute_attack_risk_with_integer_impact() {
    let mut root_attributes = HashMap::new();
    root_attributes.insert("node_type".to_owned(), model_attribute_string("or"));
    root_attributes.insert("impactToDefender".to_owned(), model_attribute_int(100));

    let mut child_attributes = HashMap::new();
    child_attributes.insert("likelihoodOfSuccess".to_owned(), model_attribute_float(0.5));

    let tree = make_tree(vec![
        make_node("000", vec!["001", "002"], root_attributes, true),
        make_node("001", vec![], child_attributes.clone(), true),
        make_node("002", vec![], child_attributes, true),
    ]);

    let result = compute_node_risk(&[tree], "000", "f1644cb9-b2a5-4abb-813f-98d0277e42f2").unwrap();

    assert_eq!(computed_f64(&result, "likelihoodOfSuccess"), 0.75);
    assert_eq!(computed_f64(&result, "impactToDefender"), 100.0);
    assert_eq!(computed_f64(&result, "risk"), 75.0);
}

#[test]
fn test_compute_attack_risk_skips_unresolved_conditions() {
    let mut root_attributes = HashMap::new();
    root_attributes.insert("node_type".to_owned(), model_attribute_string("and"));
    root_attributes.insert("impactToDefender".to_owned(), model_attribute_float(100.0));

    let mut child_attributes = HashMap::new();
    child_attributes.insert("likelihoodOfSuccess".to_owned(), model_attribute_float(0.5));

    let mut condition_attributes = child_attributes.clone();
    condition_attributes.insert("node_type".to_owned(), model_attribute_string("condition"));

    let tree = make_tree(vec![
        make_node("000", vec!["001", "002"], root_attributes, true),
        make_node("001", vec![], child_attributes, true),
        make_node("002", vec![], condition_attributes, false),
    ]);

    let result = compute_node_risk(&[tree], "000", "f1644cb9-b2a5-4abb-813f-98d0277e42f2").unwrap();

    assert_eq!(computed_f64(&result, "likelihoodOfSuccess"), 0.5);
    assert_eq!(computed_f64(&result, "risk"), 50.0);
}

#[test]
fn test_compute_evita_risk_for_and_and_or_nodes() {
    let mut root_attributes = HashMap::new();
    root_attributes.insert("node_type".to_owned(), model_attribute_string("and"));
    root_attributes.insert("safetyImpact".to_owned(), model_attribute_int(1));
    root_attributes.insert("privacyImpact".to_owned(), model_attribute_int(2));
    root_attributes.insert("financialImpact".to_owned(), model_attribute_int(3));
    root_attributes.insert("operationalImpact".to_owned(), model_attribute_int(4));

    let mut low_attack_potential = HashMap::new();
    low_attack_potential.insert("time".to_owned(), model_attribute_int(4));
    low_attack_potential.insert("expertise".to_owned(), model_attribute_int(4));
    low_attack_potential.insert("knowledge".to_owned(), model_attribute_int(4));
    low_attack_potential.insert("windowOfOpportunity".to_owned(), model_attribute_int(4));
    low_attack_potential.insert("equipmentRequired".to_owned(), model_attribute_int(4));

    let mut high_attack_potential = HashMap::new();
    high_attack_potential.insert("time".to_owned(), model_attribute_int(20));
    high_attack_potential.insert("expertise".to_owned(), model_attribute_int(20));
    high_attack_potential.insert("knowledge".to_owned(), model_attribute_int(20));
    high_attack_potential.insert("windowOfOpportunity".to_owned(), model_attribute_int(20));
    high_attack_potential.insert("equipmentRequired".to_owned(), model_attribute_int(20));

    let and_tree = make_tree(vec![
        make_node("000", vec!["001", "002"], root_attributes.clone(), true),
        make_node("001", vec![], low_attack_potential.clone(), true),
        make_node("002", vec![], high_attack_potential.clone(), true),
    ]);

    let and_result = compute_node_risk(&[and_tree], "000", "bf4397f7-93ae-4502-a4a2-397f40f5cc49").unwrap();
    assert_eq!(computed_string(&and_result, "singleValueDisplay"), "RF1 RO2 RP0 RS0");

    root_attributes.insert("node_type".to_owned(), model_attribute_string("or"));
    let or_tree = make_tree(vec![
        make_node("000", vec!["001", "002"], root_attributes, true),
        make_node("001", vec![], low_attack_potential, true),
        make_node("002", vec![], high_attack_potential, true),
    ]);

    let or_result = compute_node_risk(&[or_tree], "000", "bf4397f7-93ae-4502-a4a2-397f40f5cc49").unwrap();
    assert_eq!(computed_string(&or_result, "singleValueDisplay"), "RF2 RO3 RP1 RS0");
}

#[test]
fn test_compute_evita_risk_skips_unresolved_conditions() {
    let mut root_attributes = HashMap::new();
    root_attributes.insert("node_type".to_owned(), model_attribute_string("or"));
    root_attributes.insert("safetyImpact".to_owned(), model_attribute_int(1));
    root_attributes.insert("privacyImpact".to_owned(), model_attribute_int(2));
    root_attributes.insert("financialImpact".to_owned(), model_attribute_int(3));
    root_attributes.insert("operationalImpact".to_owned(), model_attribute_int(4));

    let mut low_attack_potential = HashMap::new();
    low_attack_potential.insert("time".to_owned(), model_attribute_int(4));
    low_attack_potential.insert("expertise".to_owned(), model_attribute_int(4));
    low_attack_potential.insert("knowledge".to_owned(), model_attribute_int(4));
    low_attack_potential.insert("windowOfOpportunity".to_owned(), model_attribute_int(4));
    low_attack_potential.insert("equipmentRequired".to_owned(), model_attribute_int(4));

    let mut blocked_attack_potential = HashMap::new();
    blocked_attack_potential.insert("time".to_owned(), model_attribute_int(20));
    blocked_attack_potential.insert("expertise".to_owned(), model_attribute_int(20));
    blocked_attack_potential.insert("knowledge".to_owned(), model_attribute_int(20));
    blocked_attack_potential.insert("windowOfOpportunity".to_owned(), model_attribute_int(20));
    blocked_attack_potential.insert("equipmentRequired".to_owned(), model_attribute_int(20));
    blocked_attack_potential.insert("node_type".to_owned(), model_attribute_string("condition"));

    let tree = make_tree(vec![
        make_node("000", vec!["001", "002"], root_attributes, true),
        make_node("001", vec![], low_attack_potential, true),
        make_node("002", vec![], blocked_attack_potential, false),
    ]);

    let result = compute_node_risk(&[tree], "000", "bf4397f7-93ae-4502-a4a2-397f40f5cc49").unwrap();
    assert_eq!(computed_string(&result, "singleValueDisplay"), "RF2 RO3 RP1 RS0");
}

