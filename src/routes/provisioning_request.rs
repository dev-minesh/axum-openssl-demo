
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use sha256::{digest};

use hmac::{Hmac, Mac};
use jwt::{SignWithKey};
use sha2::Sha256;
use std::collections::BTreeMap;

use crate::routes::mqtt_publisher;



#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceProvisioningRequestType {
    code: String,
    device_id: String,
}

#[derive(Debug)]
pub struct AppErrors {
    code: StatusCode,
    message:String
}

impl AppErrors {
    pub fn new(code:StatusCode, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }
}

impl IntoResponse for AppErrors {
    fn into_response(self) -> axum::response::Response {
        (
            self.code,
            Json(self.message)
        )
        .into_response()
    }
}


fn validate_luhn(num:&str) -> bool {
    luhn::valid(num)
}
pub async fn device_provisioning(Json(body):Json<DeviceProvisioningRequestType>) ->  Result<(), AppErrors>  {
    println!("{}", body.code);

    if !validate_luhn(&body.code) {
        return Err(AppErrors::new(StatusCode::BAD_REQUEST, "Code validation failed"))
    }

    //Validate length of code
    if body.code.len() < 6 || body.code.len() > 6 {
       return Err(AppErrors::new(StatusCode::BAD_REQUEST, "Code length must be 6"))
    }


    //Generate SHA256
    let device_id_hash = generate_hash(&body.device_id);
    println!("Device id hash :: {}",device_id_hash);

    //Generate CERT sign url
    let cert_sign_url = generate_cert_sign_url(&body.device_id);
    println!("cert url {}",cert_sign_url);

    
    // Create a JSON payload
    let payload = json!({
        "device_id": body.device_id,
        "cert_sign_url": cert_sign_url
    });

    //connect and publish message to mqtt
    mqtt_publisher::mqtt_pub(payload);

    Ok(())

}

pub fn generate_cert_sign_url(device_id:&str) -> String {

    //access env variables 
    let mut request_url = env::var("CERT_SIGN_URL_RAW").unwrap().to_owned();
   
   // let request_id = generate_request_id(device_id);
   let request_jwt = generate_jwt(device_id);
   
    request_url.push_str(&request_jwt);
    request_url.to_string()
}

pub fn generate_jwt(device_id:&str) -> String {
    let key: Hmac<Sha256> = Hmac::new_from_slice(b"mineshp@mechasystems.com").unwrap();
    let mut claims = BTreeMap::new();
    claims.insert("sub", device_id);
    
    let token_str = claims.sign_with_key(&key).unwrap();
    token_str
}

pub fn generate_request_id(device_id:&str) -> String {
    //generate request_id sha256 of device_id + salt (36 char key)
    let salt_key: String= env::var("SALT_KEY").unwrap().to_owned();
    let mut device_id_owned = device_id.to_owned();
    device_id_owned.push_str(&salt_key);
    let request_id_hash = generate_hash(&device_id_owned);
    request_id_hash
}

pub fn generate_hash(raw_key:&str) -> String {
    digest(raw_key.to_string())
}