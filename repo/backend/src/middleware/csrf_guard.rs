//! CSRF defense via Origin header verification.
//!
//! # Threat model
//!
//! This application uses Bearer tokens in the `Authorization` header (not
//! cookies), which inherently mitigates classic CSRF attacks — browsers do not
//! automatically attach custom headers to cross-origin requests. However, as
//! defense-in-depth, this fairing logs warnings when state-changing requests
//! arrive with a mismatched `Origin` header.
//!
//! The primary CSRF defense stack is:
//! 1. Bearer token authentication (not cookie-based) — browsers do not auto-attach
//!    the `Authorization` header to cross-origin requests.
//! 2. CORS allowlist restricted to a single origin (`ALLOWED_ORIGIN` env var),
//!    blocking cross-origin preflight for state-changing methods.
//! 3. This fairing as defense-in-depth: logs mismatched Origin headers on
//!    state-changing requests for security monitoring.
//!
//! Requests without an `Origin` header (e.g., curl, server-to-server, non-browser)
//! are allowed through since they rely on the Bearer token for authentication.

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Method;
use rocket::{Data, Request};

pub struct CsrfOriginCheck;

#[rocket::async_trait]
impl Fairing for CsrfOriginCheck {
    fn info(&self) -> Info {
        Info {
            name: "CSRF Origin Check",
            kind: Kind::Request,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        // Only check state-changing methods
        let method = req.method();
        if method == Method::Get || method == Method::Head || method == Method::Options {
            return;
        }

        // If no Origin header, the request is non-browser. Auth enforced by Bearer token.
        let origin = match req.headers().get_one("Origin") {
            Some(o) => o,
            None => return,
        };

        let allowed = std::env::var("ALLOWED_ORIGIN")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        if origin != allowed {
            tracing::warn!(
                origin = origin,
                allowed = %allowed,
                method = %method,
                uri = %req.uri(),
                "CSRF: state-changing request from unexpected origin"
            );
        }
    }
}
