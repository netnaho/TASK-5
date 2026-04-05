use rocket::request::{FromRequest, Outcome, Request};

/// Request guard that extracts the client IP address.
/// Checks X-Real-IP, X-Forwarded-For headers (set by reverse proxy), then falls back to socket address.
pub struct ClientIp(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientIp {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let ip = req.headers().get_one("X-Real-IP")
            .map(|s| s.to_string())
            .or_else(|| {
                req.headers().get_one("X-Forwarded-For")
                    .and_then(|s| s.split(',').next())
                    .map(|s| s.trim().to_string())
            })
            .or_else(|| req.client_ip().map(|ip| ip.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        Outcome::Success(ClientIp(ip))
    }
}
