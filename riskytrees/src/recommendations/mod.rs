use aws_sdk_bedrockruntime::operation::invoke_model;
use aws_sdk_bedrockruntime::primitives::Blob;


pub async fn recommend_steps_for_path(current_steps: Vec<String>) -> String {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_bedrockruntime::Client::new(&config);

    println!("{:?}", config.region());

    let guardrail_identifier = "arn:aws:bedrock:us-east-2:891377076852:guardrail/2sfcr35ii256";
    let model_identifier = "arn:aws:bedrock:us-east-2:891377076852:inference-profile/us.amazon.nova-micro-v1:0";

    let steps = current_steps.join(" -> ");
    let system_prompt = "You are a security engineer who is an expert in analyzing computer systems and determining how to defend systems from hackers.\\nYou like to model attacks in a top-down fashion, where threats start high-level and ambiguous and you slowly answer \\\"how\\\" until a very specific set of steps is listed. In other words, you're building an \\\"attack tree\\\".\\nI will give you a series of steps through a tree. You must propose how an attacker could accomplish the last step listed by suggesting one or more new steps.\\nYou need to be succinct with responses. Each proposed step should contain fewer than 10 words. Present the results as a list.";

    let body_as_str = &format!("{{
        \"messages\": [
            {{
                \"role\": \"user\",
                \"content\": [{{\"text\": \"{}\"}}]
            }}
        ],
        \"inferenceConfig\": {{
            \"maxTokens\": 512,
            \"temperature\": 0.5,
            \"topP\": 0.9
        }}
    }}", system_prompt.to_owned() + "\\nHere are the current steps: " + &steps);
        println!("{}", body_as_str);

    let body: serde_json::Value = serde_json::from_str(body_as_str).expect("JSON");


    let body_string = body.to_string();
    let body_bytes = body_string.as_bytes();
    let blob = Blob::new(body_bytes);

    let result = client.invoke_model()
        .guardrail_identifier(guardrail_identifier)
        .guardrail_version("DRAFT")
        .model_id(model_identifier)
        .body(blob)
        .send()
        .await;

    match result {
        Ok(res) => {
            let response_body = res.body();
            match String::from_utf8(response_body.as_ref().to_vec()) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to parse response body: {}", e);
                    "Error".to_string()
                }
            }
        },
        Err(err) => {
            eprintln!("{:?}", err.raw_response());
            return "Error".to_string();
        }
    }
}