use std::sync::OnceLock;

static JWT_SECRET: OnceLock<Vec<u8>> = OnceLock::new();

pub fn get_jwt_secret() -> &'static [u8] {
    JWT_SECRET.get_or_init(|| {
        std::env::var("JWT_SECRET")
            .inspect_err(|_| {
                tracing::warn!(
                    "JWT_SECRET не установлен! Используйте уникальное значение в production"
                );
            })
            .unwrap_or_else(|_| {
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Время не может идти вспять")
                    .as_nanos();
                format!("dev-secret-{}", seed)
            })
            .into_bytes()
    })
}
