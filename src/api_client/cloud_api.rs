use serde::{Deserialize, Serialize};
use serde_json::{json};
use std::env;

#[derive(Deserialize, Serialize)]
struct CompleteProvisioning {
    deviceId: String,
    cerId: String,
    certFingerprint: String,
    caFingerprint: String
}

pub async fn update_provisioning_status(device_id:&String) -> Result<serde_json::Value, reqwest::Error> {
println!("Device id is :: {}",device_id);
    let params_string = json!({
        "deviceId": "2223043857",
        "certId": "alpha",
        "certFingerprint": "beta",
        "caFingerprint": "gamma"
        });

let params: CompleteProvisioning = serde_json::from_str(&params_string.to_string()).unwrap();
    //let body = json!(params);
    let url = format!("https://api.v1.dev.mecha.build/api/v1/internal/devices/provision-requests/complete");
    let auth = format!("Basic {}", env::var("EMQX_AUTH").expect("Error"));    
    let client = reqwest::Client::new();
    let result = client
        .post(url)
        .json(&params)
        .header("AUTHORIZATION", auth)
        .header("CONTENT_TYPE", "application/json")
        .header("ACCEPT", "application/json")
        .send()
        .await;

        let response = match result {
            Ok(res) => res,
            Err(err) => return Err(err),
        };
    
        println!("TOPIC PUBLISH API COMPLETED 1111111");
        let resp = response.json::<serde_json::Value>().await;
        match resp {
            Ok(x) => Ok(x),
            Err(e) => return Err(e),
        }
}