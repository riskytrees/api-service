use crate::models;
use crate::expression_evaluator;

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
        attributes: rocket_contrib::json!({
            "hello": "world"
        })
    };

    assert_eq!(expression_evaluator::evaluate("1 == 1", &config), true);
    assert_eq!(expression_evaluator::evaluate("\"test\" == \"test\"", &config), true);
    assert_eq!(expression_evaluator::evaluate("config[\"hello\"] == \"world\"", &config), true);
    assert_eq!(expression_evaluator::evaluate("config[\"hello\"] == \"test\"", &config), false);

}

