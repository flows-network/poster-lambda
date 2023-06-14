use serde_json::Value;
use std::collections::HashMap;

use lambda_flows::{request_received, send_response};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() {
    request_received(handler).await;
}

async fn handler(_qry: HashMap<String, Value>, _body: Vec<u8>) {
    let html = include_str!("index.html");
    let backurl = std::env::var("BACKEND_SERVICE_URL").expect("No url in env var");
    let html = html.replace("{BACKEND_SERVICE_URL}", &backurl);
    send_response(
        200,
        vec![(String::from("content-type"), String::from("text/html"))],
        html.as_bytes().to_vec(),
    );
}
