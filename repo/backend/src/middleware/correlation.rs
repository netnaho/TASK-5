use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Request, Data};
use uuid::Uuid;

pub struct CorrelationId;

#[rocket::async_trait]
impl Fairing for CorrelationId {
    fn info(&self) -> Info {
        Info {
            name: "Correlation ID",
            kind: Kind::Request,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        let cid = req.headers().get_one("X-Correlation-Id")
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        req.local_cache(|| cid);
    }
}

pub fn get_correlation_id(req: &rocket::Request<'_>) -> String {
    req.local_cache(|| Uuid::new_v4().to_string()).clone()
}
