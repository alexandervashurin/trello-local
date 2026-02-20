use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

/// Конфигурация rate limiter
#[derive(Clone, Debug)]
pub struct RateLimiterConfig {
    pub max_requests: u32,
    pub window_duration: Duration,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            max_requests: 100, // 100 запросов
            window_duration: Duration::from_secs(60), // в минуту
        }
    }
}

/// Запись о запросах клиента
#[derive(Clone, Debug)]
struct ClientRequests {
    count: u32,
    window_start: Instant,
}

/// Состояние rate limiter
#[derive(Clone)]
pub struct RateLimiterState {
    clients: Arc<RwLock<HashMap<String, ClientRequests>>>,
    config: RateLimiterConfig,
}

impl RateLimiterState {
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(RateLimiterConfig::default())
    }
}

/// Middleware для rate limiting
pub async fn rate_limit_middleware(
    State(state): State<RateLimiterState>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    // Получаем IP адрес клиента
    let client_ip = request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .or_else(|| {
            request
                .extensions()
                .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                .map(|c| c.0.ip().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    let now = Instant::now();

    let mut clients = state.clients.write().await;

    let entry = clients.entry(client_ip.clone()).or_insert(ClientRequests {
        count: 0,
        window_start: now,
    });

    // Проверяем, истекло ли окно
    if now.duration_since(entry.window_start) > state.config.window_duration {
        entry.count = 0;
        entry.window_start = now;
    }

    // Увеличиваем счётчик
    entry.count += 1;

    // Проверяем лимит
    if entry.count > state.config.max_requests {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "Слишком много запросов. Попробуйте позже.".to_string(),
        ));
    }

    drop(clients);

    Ok(next.run(request).await)
}

/// Очистка старых записей (можно запускать в фоне)
pub async fn cleanup_old_entries(state: RateLimiterState) {
    loop {
        tokio::time::sleep(Duration::from_secs(300)).await; // Каждые 5 минут

        let mut clients = state.clients.write().await;
        let now = Instant::now();

        clients.retain(|_, client| {
            now.duration_since(client.window_start) < state.config.window_duration
        });
    }
}
