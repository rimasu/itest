use axum::{
    extract::Request, http::StatusCode, middleware::{self, Next}, response::{IntoResponse, Json, Response}, routing::get, Router
};
use serde_json::{Value, json};
use tokio::net::TcpListener;

async fn hello() -> Json<Value> {
    Json(json!({"message": "Hello, World!"}))
}

async fn log_http_version(request: Request, next: Next) -> Response {
    let version = request.version();
    match version {
        axum::http::Version::HTTP_2 => {
            next.run(request).await
        }
        _ => {
            println!("HTTP/1.x request blocked");
            (
                StatusCode::HTTP_VERSION_NOT_SUPPORTED,
                Json(json!({
                    "error": "This server only accepts HTTP/2 connections",
                    "received_version": format!("{:?}", version)
                })),
            )
                .into_response()
        }
    }
}

pub async fn server_main() {
    let app = Router::new()
        .route("/", get(hello))
        .layer(middleware::from_fn(log_http_version));

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    println!("Server running on http://127.0.0.1:3000");
    println!("HTTP version will be logged for each request");

    axum::serve(listener, app).await.unwrap();
}
