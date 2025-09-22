use std::env;

use axum::{
    Router,
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::get,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{Pool, Postgres};
use tokio::net::TcpListener;

async fn hello() -> Json<Value> {
    Json(json!({"message": "Hello, World!"}))
}

#[derive(Deserialize, Debug)]
struct KeyParams {
    set: Option<String>,
}

async fn key_value_pair(
    Path(key): Path<String>,
    Query(params): Query<KeyParams>,
    State(state): State<ServerState>,
) -> Json<Value> {
    if let Some(set) = &params.set {
        sqlx::query!("INSERT INTO key_value_pair (key, value) VALUES($1, $2) ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value", &key, set)
            .execute(&state.pool)
            .await.unwrap();

        println!("{:?}={:?}", key, params.set);
        Json(json!({"message": "Hello, World!"}))
    } else {
        let record = sqlx::query!("SELECT value FROM key_value_pair WHERE key=$1", &key)
            .fetch_one(&state.pool)
            .await
            .unwrap();

        Json(json!({"key": key.to_owned(), "value":  record.value.to_owned()}))
    }
}

async fn force_http2_only(request: Request, next: Next) -> Response {
    let version = request.version();
    match version {
        axum::http::Version::HTTP_2 => next.run(request).await,
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

async fn connect_to_database(db_url: &str) -> Pool<Postgres> {
    println!("Connecting to database");
    let pool = Pool::<Postgres>::connect(db_url).await.unwrap();
    pool
}

#[derive(Clone)]
pub struct ServerState {
    pool: Pool<Postgres>,
}

pub async fn server_main() {
    let db_url = env::var("EXAMPLE_DATABASE_URL").unwrap();

    let pool = connect_to_database(&db_url).await;
    let state = ServerState { pool };

    let app = Router::new()
        .route("/", get(hello))
        .route("/{key}", get(key_value_pair))
        .layer(middleware::from_fn(force_http2_only))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
