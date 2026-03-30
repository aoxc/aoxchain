use crate::{embed, environments::Environment, security, services::HubService};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::{Html, IntoResponse, Sse},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::json;
use std::{convert::Infallible, net::SocketAddr, time::Duration};
use tokio::net::TcpListener;

#[derive(Deserialize)]
struct EnvRequest {
    environment: Environment,
}

#[derive(Deserialize)]
struct ExecuteRequest {
    command_id: String,
    confirm: bool,
}

#[derive(Deserialize)]
struct BinarySelectRequest {
    binary_id: String,
}

#[derive(Deserialize)]
struct CustomBinaryRequest {
    path: String,
}

pub async fn serve(service: HubService) -> Result<(), std::io::Error> {
    let app = Router::new()
        .route("/", get(index))
        .route("/assets/app.css", get(css))
        .route("/assets/app.js", get(js))
        .route("/api/state", get(state))
        .route("/api/environment", post(set_environment))
        .route("/api/binary/select", post(select_binary))
        .route("/api/binary/custom", post(add_custom_binary))
        .route("/api/execute", post(execute))
        .route("/api/jobs/{id}", get(job))
        .route("/api/jobs/{id}/stream", get(stream))
        .layer(middleware::from_fn(security::localhost_only))
        .with_state(service);

    let addr = SocketAddr::from(([127, 0, 0, 1], 7070));
    let listener = TcpListener::bind(addr).await?;
    println!("AOXCHub listening on http://127.0.0.1:7070");
    axum::serve(listener, app).await
}

async fn index() -> Html<&'static str> {
    Html(embed::INDEX_HTML)
}
async fn css() -> impl IntoResponse {
    (
        [("content-type", "text/css; charset=utf-8")],
        embed::APP_CSS,
    )
}
async fn js() -> impl IntoResponse {
    (
        [("content-type", "application/javascript; charset=utf-8")],
        embed::APP_JS,
    )
}

async fn state(State(service): State<HubService>) -> Json<crate::domain::HubStateView> {
    Json(service.state().await)
}

async fn set_environment(
    State(service): State<HubService>,
    Json(req): Json<EnvRequest>,
) -> Json<serde_json::Value> {
    service.set_environment(req.environment).await;
    Json(json!({"ok": true}))
}

async fn select_binary(
    State(service): State<HubService>,
    Json(req): Json<BinarySelectRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    service.set_binary(req.binary_id).await.map_err(to_http)?;
    Ok(Json(json!({"ok": true})))
}

async fn add_custom_binary(
    State(service): State<HubService>,
    Json(req): Json<CustomBinaryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    service.add_custom_binary(req.path).await.map_err(to_http)?;
    Ok(Json(json!({"ok": true})))
}

async fn execute(
    State(service): State<HubService>,
    Json(req): Json<ExecuteRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    if !req.confirm {
        return Err((
            StatusCode::BAD_REQUEST,
            String::from("explicit confirmation is required"),
        ));
    }
    let id = service.execute(req.command_id).await.map_err(to_http)?;
    Ok(Json(json!({"ok": true, "job_id": id})))
}

async fn job(
    Path(id): Path<String>,
    State(service): State<HubService>,
) -> Result<Json<crate::domain::JobStatus>, StatusCode> {
    service
        .runner
        .get_job(&id)
        .await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn stream(
    Path(id): Path<String>,
    State(service): State<HubService>,
) -> Result<
    Sse<impl futures_core::Stream<Item = Result<axum::response::sse::Event, Infallible>>>,
    StatusCode,
> {
    let mut rx = service
        .runner
        .subscribe(&id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(line) => yield Ok(axum::response::sse::Event::default().data(line)),
                Err(_) => break,
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    };
    Ok(Sse::new(stream))
}

fn to_http(err: crate::errors::HubError) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, err.to_string())
}
