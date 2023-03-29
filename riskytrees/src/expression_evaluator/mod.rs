use crate::models::ApiProjectConfigResponseResult;

use evalexpr::*;

pub fn evaluate(condition: &str, config: &ApiProjectConfigResponseResult) -> bool {
    println!("Evaluate {}", condition);
    let context = context_map! {
        "a" => 6,
    }.expect("To work");

    match eval_boolean_with_context(condition, &context) {
        Ok(res) => {
            res
        },
        Err(err) => {
            eprintln!("Eval error");
            return true;
        }
    }
}