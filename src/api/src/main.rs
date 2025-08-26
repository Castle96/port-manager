use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_web_lab::middleware::from_fn;
use prometheus::{IntCounter, register_int_counter};
use lazy_static::lazy_static;
use reservation::PortReservationManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

mod reservation;

#[derive(Serialize, Deserialize)]
struct User {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    role: String,
    exp: usize,
}

#[derive(Clone, Serialize, Deserialize)]
struct Task {
    id: i32,
    title: String,
    description: String,
    completed: bool,
}

#[derive(Serialize, Deserialize)]
struct CreateTaskRequest {
    title: String,
    description: String,
}

#[derive(Serialize, Deserialize)]
struct UpdateTaskRequest {
    title: String,
    description: String,
    completed: bool,
}

struct AppState {
    manager: Arc<PortReservationManager>,
}

lazy_static! {
    static ref RESERVE_COUNTER: IntCounter = register_int_counter!("reserve_requests_total", "Total reserve requests").unwrap();
    static ref RELEASE_COUNTER: IntCounter = register_int_counter!("release_requests_total", "Total release requests").unwrap();
}

// Simple in-memory rate limiter
struct RateLimiter {
    requests: Mutex<HashMap<String, (u32, Instant)>>,
    limit: u32,
    window: Duration,
}

impl RateLimiter {
    fn new(limit: u32, window_secs: u64) -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
            limit,
            window: Duration::from_secs(window_secs),
        }
    }

    fn check(&self, ip: String) -> bool {
        let mut reqs = self.requests.lock().unwrap();
        let now = Instant::now();
        let entry = reqs.entry(ip).or_insert((0, now));
        if now.duration_since(entry.1) > self.window {
            *entry = (1, now);
            true
        } else if entry.0 < self.limit {
            entry.0 += 1;
            true
        } else {
            false
        }
    }
}

async fn rate_limit_middleware(
    req: actix_web::dev::ServiceRequest,
    srv: &dyn actix_web::dev::Service<actix_web::dev::ServiceRequest, Response = actix_web::dev::ServiceResponse, Error = actix_web::Error, Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<actix_web::dev::ServiceResponse, actix_web::Error>> + Send>>>,
) -> Result<actix_web::dev::ServiceResponse, actix_web::Error> {
    let limiter = req.app_data::<Arc<RateLimiter>>().unwrap();
    let ip = req.connection_info().realip_remote_addr().unwrap_or("unknown").to_string();
    if limiter.check(ip) {
        srv.call(req).await
    } else {
        Ok(req.into_response(actix_web::HttpResponse::TooManyRequests().finish()))
    }
}

// Reserve a port
async fn reserve_port(data: web::Data<AppState>, info: web::Json<(u16, String)>) -> impl Responder {
    RESERVE_COUNTER.inc();
    let (port, service) = info.into_inner();
    match data.manager.reserve_port(port, service) {
        Ok(_) => {
            let _ = data.manager.save_to_file("reservations.json");
            HttpResponse::Ok().body("Reserved")
        },
        Err(e) => HttpResponse::BadRequest().body(format!("Error: {}", e)),
    }
}

// Release a port
async fn release_port(data: web::Data<AppState>, info: web::Json<u16>) -> impl Responder {
    RELEASE_COUNTER.inc();
    let port = info.into_inner();
    match data.manager.release_port(port) {
        Ok(_) => {
            let _ = data.manager.save_to_file("reservations.json");
            HttpResponse::Ok().body("Released")
        },
        Err(e) => HttpResponse::BadRequest().body(format!("Error: {}", e)),
    }
}

// Check reservation status
async fn status(data: web::Data<AppState>, port: web::Path<u16>) -> impl Responder {
    if data.manager.is_reserved(port.into_inner()) {
        HttpResponse::Ok().body("Reserved")
    } else {
        HttpResponse::Ok().body("Available")
    }
}
// List ports with advanced filtering
#[derive(Deserialize)]
struct PortFilter {
    query: Option<String>,
    protocol: Option<String>,
    state: Option<String>,
    port_start: Option<u16>,
    port_end: Option<u16>,
    tags: Option<Vec<String>>,
    user: Option<String>,
    process_name: Option<String>,
}
mod net;

async fn list_ports(filter: web::Query<PortFilter>) -> impl Responder {
        let all_ports = net::list_ports();
    let filtered: Vec<_> = all_ports.into_iter().filter(|p| {
        p.matches(
            filter.query.as_deref().unwrap_or("") ,
            filter.protocol.as_deref(),
            filter.state.as_deref(),
            match (filter.port_start, filter.port_end) {
                (Some(s), Some(e)) => Some((s, e)),
                _ => None,
            },
            filter.tags.as_deref(),
            filter.user.as_deref(),
        )
    }).collect();
    HttpResponse::Ok().json(filtered)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let limiter = Arc::new(RateLimiter::new(10, 60)); // 10 requests per 60 seconds per IP
    let manager = Arc::new(PortReservationManager::new());
    // Load reservations from file on startup
    let _ = manager.load_from_file("reservations.json");
    let prometheus = actix_web_prom::PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .build()
        .unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState { manager: manager.clone() }))
            .app_data(limiter.clone())
            .wrap(prometheus.clone())
            // TODO: Add proper rate limiting middleware for Actix Web 4.x
            .route("/reserve", web::post().to(reserve_port))
            .route("/release", web::post().to(release_port))
            .route("/status/{port}", web::get().to(status))
                .route("/ports", web::get().to(list_ports))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

