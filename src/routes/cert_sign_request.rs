
use axum::{Json, http::StatusCode, response::IntoResponse, extract::Path};
use openssl::error::ErrorStack;
use openssl::rsa::Rsa;
use serde::{Deserialize, Serialize};
use crate::api_client::cloud_api::update_provisioning_status;

use hmac::{Hmac, Mac};
use jwt::{ VerifyWithKey, Token, Header};
use sha2::Sha256;
use std::fs;
use std::path::PathBuf;
use std::{collections::BTreeMap, ops::Deref};
use csr_signer::mk_ca_signed_cert;

use openssl::pkey::{PKey, Private};
use openssl::x509::{X509Req, X509Ref};

use openssl::x509::{X509};

use super::csr_signer;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct CertSignRequest {
    csr: String,
    serial_number: String,
    make: String,
    modal: String
}


pub async fn handle_cert_sign_request(Path(jwt):Path<String>, Json(body): Json<CertSignRequest>) ->  Result<String, AppErrors>  {
    println!("REQUEST BODY :: {:?}",body);
    //Parse csr and get device_id
    let device_id = parse_csr(&body.csr).unwrap();
    println!("FINAL DEVICE ID ::{} ",device_id);


    if decrypt_jwt(&jwt.to_string()).unwrap() != device_id {
        return Err(AppErrors::new(StatusCode::UNAUTHORIZED, "Request authentication failed"))
     }

     let req = X509Req::from_pem(body.csr.as_bytes()).unwrap();
    let refr = req.as_ref().deref().to_owned();

    //generate pkey
    let key = generate_pkey();
    println!("PRIVATE KEY :: {:?}",key);
    
   //sign certificate 
   let response = convert_x509req_to_x509ref(&req,&key).unwrap();
   println!("FINAL RESPONSE :: {:?}",response);

   //save to directory 
   let certificate_file = &response.to_pem().unwrap();
    let mut path = PathBuf::from("certs");
    fs::create_dir_all(&path).unwrap();
    path.push("signed_ca.pem");
    fs::write(path, certificate_file).unwrap();
   
    //convert in string to include in response
    let x = response.to_pem().unwrap();
    println!("{}", String::from_utf8_lossy(&x));
    Ok(String::from_utf8_lossy(&x).to_string())
}

fn generate_pkey() -> PKey<Private>{
    let rsa = Rsa::generate(2048).unwrap();
    let pkey = PKey::from_rsa(rsa).unwrap();
    pkey
}
fn parse_csr(csr: &str) -> Option<String> {
    let req = X509Req::from_pem(csr.as_bytes()).ok()?;
    let subject_name = req.subject_name();
    let common_name = subject_name.entries_by_nid(openssl::nid::Nid::COMMONNAME).next().map(|entry| entry.data().as_utf8().ok().map(|s| s.to_string())).flatten();
    common_name
}

pub fn decrypt_jwt(token:&str) -> Option<String> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(b"mineshp@mechasystems.com").ok()?;
    let verified_token: Token<Header, BTreeMap<String, String>, _> = VerifyWithKey::verify_with_key(token, &key).unwrap();
    let claims = verified_token.claims();
    println!("CLAIMS :: {}",claims["sub"]);
    Some(claims["sub"].to_string())
}

fn convert_x509req_to_x509ref(x509req: &X509Req, key: &PKey<Private>) -> Result<X509, openssl::error::ErrorStack> {
    let mut x509 = X509::builder()?;
    x509.set_version(2)?;
    x509.set_subject_name(x509req.subject_name())?;
    x509.set_pubkey(key)?;
    x509.sign(key, openssl::hash::MessageDigest::sha256())?;
    Ok(x509.build())
}