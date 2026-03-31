use crate::{embed, environments::Environment, security, services::HubService};
use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
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
    let max_request_bytes = env_or_default("AOXCHUB_MAX_REQUEST_BYTES", 64 * 1024);
    let app = Router::new()
        .route("/", get(index))
        .route("/assets/{*path}", get(asset))
        .route("/api/state", get(state))
        .route("/api/environment", post(set_environment))
        .route("/api/binary/select", post(select_binary))
        .route("/api/binary/custom", post(add_custom_binary))
        .route("/api/execute", post(execute))
        .route("/api/jobs/{id}", get(job))
        .route("/api/jobs/{id}/stream", get(stream))
        .layer(DefaultBodyLimit::max(max_request_bytes))
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
async fn asset(Path(path): Path<String>) -> impl IntoResponse {
    if path == "app.js" {
        return (
            StatusCode::OK,
            [("content-type", "application/javascript; charset=utf-8")],
            embed::APP_JS,
        );
    }

    let css = match path.as_str() {
        "app.css" => Some(embed::APP_CSS),
        "reset.css" => Some(embed::RESET_CSS),
        "tokens.css" => Some(embed::TOKENS_CSS),
        "theme-mainnet.css" => Some(embed::THEME_MAINNET_CSS),
        "theme-testnet.css" => Some(embed::THEME_TESTNET_CSS),
        "layout.css" => Some(embed::LAYOUT_CSS),
        "header.css" => Some(embed::HEADER_CSS),
        "sidebar.css" => Some(embed::SIDEBAR_CSS),
        "hero.css" => Some(embed::HERO_CSS),
        "cards.css" => Some(embed::CARDS_CSS),
        "wallet.css" => Some(embed::WALLET_CSS),
        "actions.css" => Some(embed::ACTIONS_CSS),
        "banners.css" => Some(embed::BANNERS_CSS),
        "buttons.css" => Some(embed::BUTTONS_CSS),
        "badges.css" => Some(embed::BADGES_CSS),
        "forms.css" => Some(embed::FORMS_CSS),
        "panels.css" => Some(embed::PANELS_CSS),
        "terminal.css" => Some(embed::TERMINAL_CSS),
        "responsive.css" => Some(embed::RESPONSIVE_CSS),
        _ => None,
    };

    match css {
        Some(content) => (
            StatusCode::OK,
            [("content-type", "text/css; charset=utf-8")],
            content,
        ),
        None => (
            StatusCode::NOT_FOUND,
            [("content-type", "text/plain; charset=utf-8")],
            "asset not found",
        ),
    }
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

fn env_or_default(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(default)
}
