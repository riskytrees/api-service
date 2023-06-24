use std::collections::HashMap;

use crate::models::ApiProjectConfigResponseResult;

use evalexpr::*;
use rocket::http::ext::IntoCollection;

// Conditions may contain references to json object lookups. This should always
// be in the form:
//
// config["<key>"]...["<keyN>"] <OP> ...
//
// We want this converted to treat each lookup item as a single variable e.g.
// config_key_..._keyN <OP> ...
pub fn de_json_condition(condition: &str) -> String {
    let mut alerted_string = "".to_owned();

    let mut in_lookup_key = false;
    for c in condition.chars() {
        if c == '[' && in_lookup_key == false {
            in_lookup_key = true;
            alerted_string += "_";
        } else if c == ']' && in_lookup_key == true {
            in_lookup_key = false;
        } else if in_lookup_key == true && (c == '"' || c == '\'') {
            // Do nothing - skip quote characters
        } else {
            alerted_string.push(c);
        }
    }

    alerted_string
}

pub fn get_key_vals(obj: &serde_json::map::Map<String, serde_json::value::Value>) -> HashMap<String, serde_json::value::Value> {
    let mut result = HashMap::new();
    for (key, val) in obj.into_iter() {
        if val.is_object() {
            // Recurse
            let sub_obj = val.as_object().expect("Asserted");
            let sub_key_vals = get_key_vals(sub_obj);

            for (sub_key, sub_val) in sub_key_vals {
                result.insert(key.clone() + "_" + &sub_key, sub_val);
            }

        } else if val.is_number() || val.is_string() {
            result.insert(key.clone(), val.clone());
        }
    }

    result
}

pub fn evaluate(condition: &str, config: &ApiProjectConfigResponseResult) -> bool {
    // Empty conditions always resolve
    if condition.to_owned().len() == 0 {
        return true;
    }

    let normalized_condition = de_json_condition(condition);
    println!("Evaluate {}", normalized_condition);
    let mut context = HashMapContext::new();

    // Generate context map
    match config.attributes.as_object() {
        Some(obj) => {
            let key_vals = get_key_vals(obj);

            for (key, val) in key_vals {
                if val.is_number() {
                    context.set_value("config_".to_owned() + &key, val.as_f64().expect("Asserted").into()).expect("To always work");
                } else if val.is_string() {
                    context.set_value("config_".to_owned() + &key, val.as_str().expect("Asserted").into()).expect("To always work");
                }
            }

        },
        None => {
            eprintln!("Config wasn't an object!");
        }
    }

    match eval_boolean_with_context(&normalized_condition, &context) {
        Ok(res) => {
            res
        },
        Err(err) => {
            eprintln!("Eval error");
            return false;
        }
    }
}