use axum::Router;
mod qoute_of_day;
pub mod provisioning_request;
pub mod cert_sign_request;
pub mod mqtt_publisher;
pub mod csr_signer;

use qoute_of_day::qoute_of_day;
use provisioning_request::{ device_provisioning};
use cert_sign_request::handle_cert_sign_request;
use axum :: {routing::{get, post, put}, body::Body};

pub async fn create_routes() -> Router<(),Body> {
    Router::new().route("/", get(qoute_of_day))
    .route("/provision/request", post(device_provisioning))
    .route("/cert/sign/:jwt", put(handle_cert_sign_request))
}

