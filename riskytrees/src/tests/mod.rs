use crate::models;
use crate::expression_evaluator;
use crate::recommendations::convert_recommendations_to_list;
use crate::recommendations::recommend_steps_for_path;

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
async fn test_recommendations() {
    let path = vec!["Threats to RiskyTrees in next 5 years".to_string(), "Cost Flooding".to_string(), "Log flooding".to_string()];
    let result = recommend_steps_for_path(path).await;

    assert!(result.contains(","));

    let final_list = convert_recommendations_to_list(result);

    assert!(final_list.len() > 1);
}

