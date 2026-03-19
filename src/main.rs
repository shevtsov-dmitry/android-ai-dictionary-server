use reqwest::Client;
use serde::Deserialize;
use tiny_http::{Response, Server};
use urlencoding::decode;

const PORT: u16 = 8915;
const DOMAIN: &str = "0.0.0.0";
const OLLAMA_MODEL: &str = "llama3.2:1b";


const URL_NAME: &str = "/ai-dictionary";
const TEXT_PARAM_NAME: &str = "text";

#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

#[tokio::main]
async fn main() {
    let server = Server::http(format!("{}:{}", DOMAIN, PORT)).unwrap();

    for request in server.incoming_requests() {
        println!(
            "method {:?}, url {:?}, headers {:?}",
            request.method(),
            request.url(),
            request.headers()
        );

        let full_url = request.url();
        let mut parts = full_url.splitn(2, '?');
        let path = parts.next().unwrap_or("");
        let query = parts.next().unwrap_or("");

        if path != URL_NAME {
            let response = Response::from_string("Method is unsupported").with_status_code(400);
            let _ = request.respond(response);
            continue;
        }

        let encoded_text = query
            .split('&')
            .find_map(|pair| {
                let mut kv = pair.splitn(2, '=');
                let key = kv.next()?;
                let value = kv.next()?;
                if key == TEXT_PARAM_NAME {
                    Some(value)
                } else {
                    None
                }
            });

        let Some(encoded_text) = encoded_text else {
            let response = Response::from_string("Missing text parameter").with_status_code(400);
            let _ = request.respond(response);
            continue;
        };

        let decoded_text = match decode(encoded_text) {
            Ok(decoded) => decoded.into_owned(),
            Err(_) => {
                let response =
                    Response::from_string("Failed to decode input").with_status_code(400);
                let _ = request.respond(response);
                continue;
            }
        };

        match make_request(&decoded_text).await {
            Ok(answer) => {
                println!("ollama res {}", answer);
                let response = Response::from_string(answer).with_status_code(200);
                let _ = request.respond(response);
            }
            Err(err) => {
                eprintln!("ollama error: {}", err);
                let response =
                    Response::from_string("Failed to get response from Ollama").with_status_code(500);
                let _ = request.respond(response);
            }
        }
    }
}

async fn make_request(content_to_define: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let prompt = PROMPT.replace("{content}", content_to_define);

    let res = client
        .post("http://localhost:11434/api/generate")
        .json(&serde_json::json!({
            "model": OLLAMA_MODEL,
            "prompt": prompt,
            "stream": false
        }))
        .send()
        .await?;

    let ollama_response: OllamaGenerateResponse = res.json().await?;
    Ok(ollama_response.response)
}