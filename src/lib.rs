pub mod routes;
mod api_client; 
use routes::create_routes;

pub async fn run() {
    println!("server initialized!");

    let app = create_routes().await;
    
     // run it with hyper on localhost:3000
     axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
     .serve(app.into_make_service())
     .await
     .unwrap();
}

