use axum::{
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;

static INDEX_HTML: &str = include_str!("../ui-assets/html/index.html");
static APP_CSS: &str = include_str!("../ui-assets/css/app.css");
static APP_JS: &str = include_str!("../ui-assets/js/app.js");

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/assets/app.css", get(app_css))
        .route("/assets/app.js", get(app_js));

    let addr = SocketAddr::from(([127, 0, 0, 1], 7070));
    let listener = TcpListener::bind(addr)
        .await
        .expect("failed to bind local AOXC Hub listener");

    println!("AOXC Hub running at http://127.0.0.1:7070");
    axum::serve(listener, app)
        .await
        .expect("AOXC Hub server terminated unexpectedly");
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn app_css() -> Response {
    (
        [("content-type", "text/css; charset=utf-8")],
        APP_CSS,
    )
        .into_response()
}

async fn app_js() -> Response {
    (
        [("content-type", "application/javascript; charset=utf-8")],
        APP_JS,
    )
        .into_response()
}
