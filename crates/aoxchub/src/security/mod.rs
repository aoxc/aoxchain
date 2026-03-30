use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};

pub async fn localhost_only(req: Request, next: Next) -> Result<Response, StatusCode> {
    if let Some(addr) = req.extensions().get::<std::net::SocketAddr>()
        && !addr.ip().is_loopback()
    {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(next.run(req).await)
}
