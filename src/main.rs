
use device_provisioning_server::run;
use dotenv::dotenv;
use std::env;
#[tokio::main]
 async fn main() {
    dotenv().ok();
    for (key, value) in env::vars() {
        println!("{}: {}", key, value);
    }
    run().await;
}