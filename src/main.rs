use tiny_http::{Response, Server};
use urlencoding::decode;

const PORT: u16 = 8915;
const DOMAIN: &'static str = "0.0.0.0";
const OLLAMA_MODEL: &'static str = "llama3.2:1b";

/// Getting request from:
///  ```http://192.168.1.38:8915/ai-dictionary?text=${URLEncoder.encode(text, "UTF-8")}```
///
const URL_NAME: &str = "/ai-dictionary";
const TEXT_PARAM_NAME: &str = "text";

fn main() {
    let server = Server::http(format!("{}:{}", DOMAIN, PORT)).unwrap();

    for request in server.incoming_requests() {
        println!(
            " method {:?} , url {:?} , headers {:?} ",
            request.method(),
            request.url(),
            request.headers()
        );

        let mut split = request.url().split("?");
        let url = split.next().unwrap_or("");
        let text_param_value = split.last().unwrap_or("");
        if let Ok(decoded) = decode(&text_param_value[TEXT_PARAM_NAME.len()..]) {
            let decoded_text = decoded.lines().next().unwrap_or("");

            match url {
                URL_NAME => {
                    let response = Response::from_string(format!("I get word : {}", decoded_text));
                    let _ = request.respond(response);
                },
                _ => {
                    let response =
                        Response::from_string("Method is unsupported").with_status_code(400);
                    let _ = request.respond(response);
                },
            }
        } else {
            let response = Response::from_string("Failed to decode input").with_status_code(400);
            let _ = request.respond(response);
        }
    }
}
