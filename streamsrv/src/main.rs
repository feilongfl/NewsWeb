use axum::{
    extract::{Json, Query},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct StreamParams {
    channel: Option<String>,
}

#[derive(Deserialize)]
struct PublishRequest {
    channel: String,
    message: String,
}

async fn stream(Query(params): Query<StreamParams>) -> impl IntoResponse {
    let channel = params.channel.unwrap_or_else(|| "updates".into());
    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("text/event-stream"),
    );
    headers.insert("Grip-Hold", HeaderValue::from_static("stream"));
    headers.insert("Grip-Channel", HeaderValue::from_str(&channel).unwrap());
    (headers, "")
}

async fn publish(Json(payload): Json<PublishRequest>) -> impl IntoResponse {
    let client = reqwest::Client::new();
    let url = format!("http://localhost:5561/publish/{}", payload.channel);
    let body = json!({ "items": [{ "body": payload.message }] });
    match client.post(url).json(&body).send().await {
        Ok(resp) if resp.status().is_success() => StatusCode::OK,
        _ => StatusCode::BAD_GATEWAY,
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/stream", get(stream))
        .route("/publish", post(publish));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
