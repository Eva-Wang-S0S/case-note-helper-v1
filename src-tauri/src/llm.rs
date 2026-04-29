use crate::db::AppState;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: String,
}

pub fn apply_redaction(input: &str, redaction_list: &[String]) -> String {
    let mut result = input.to_string();
    for term in redaction_list {
        if !term.is_empty() && term != "[REDACTED NAME]" && term != "[REDACTED ORG]" && term != "[REDACTED LOCATION]" {
            let pattern = regex::escape(term);
            let re = regex::Regex::new(&format!("(?i){}", pattern)).unwrap();
            result = re.replace_all(&result, "[REDACTED]").to_string();
        }
    }
    result
}

pub async fn draft_note(
    state: &AppState,
    raw_input: &str,
) -> Result<String, String> {
    log::info!("Drafting case note with {} chars of input", raw_input.len());

    let settings = state.settings.read().await;

    let prompt = format!(
        "You are a social worker assistant helping to draft case notes. \
        Given the raw observations below, write a professional, structured case note. \
        Use clear headings and bullet points where appropriate. \
        Focus on facts, observations, and actions taken. \
        \n\nRAW OBSERVATIONS:\n{}\n\n\
        DRAFTED CASE NOTE:",
        raw_input
    );

    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let request = ChatRequest {
        model: settings.llm_model.clone(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: prompt,
        }],
        stream: false,
    };

    let mut url = settings.llm_endpoint.clone();
    if url.contains("localhost") || url.contains("127.0.0.1") {
    } else {
        log::warn!("LLM endpoint may not be localhost: {}", url);
    }

    let mut req_builder = client.post(&url);

    if !settings.llm_api_key.is_empty() && settings.llm_api_key != "ollama" {
        req_builder = req_builder.header("Authorization", format!("Bearer {}", settings.llm_api_key));
    }

    let response = req_builder
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("LLM request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        log::error!("LLM request failed: {} - {}", status, body);
        return Err(format!("LLM request failed: {}", status).into());
    }

    let chat_response: ChatResponse = response.json().await
        .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

    let content = chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();

    log::info!("Draft complete, {} chars returned", content.len());
    Ok(content)
}
