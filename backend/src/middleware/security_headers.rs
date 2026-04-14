use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};

/// Middleware для добавления security заголовков
pub async fn security_headers(request: Request, next: Next) -> Result<Response, StatusCode> {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Content Security Policy
    // Разрешаем:
    // - script: только свои скрипты ('self'), inline ('unsafe-inline' для Vanilla JS)
    // - style: свои стили, inline стили
    // - img: свои изображения, data: URI
    // - connect: свои API, blob:
    // - font: свои шрифты
    // - object: ничего ('none')
    // - base: свой origin
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static(
            "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' blob:; font-src 'self'; object-src 'none'; base-uri 'self';"
        ),
    );

    // Запрет контента в iframe (защита от clickjacking)
    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));

    // Защита от MIME-sniffing
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );

    // Referrer Policy - ограничиваем передачу referer
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Permissions Policy - отключаем ненужные функции браузера
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static(
            "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()"
        ),
    );

    // Cross-Origin Embedder Policy
    headers.insert(
        "Cross-Origin-Embedder-Policy",
        HeaderValue::from_static("require-corp"),
    );

    // Cross-Origin Opener Policy
    headers.insert(
        "Cross-Origin-Opener-Policy",
        HeaderValue::from_static("same-origin"),
    );

    // Cross-Origin Resource Policy
    headers.insert(
        "Cross-Origin-Resource-Policy",
        HeaderValue::from_static("same-origin"),
    );

    Ok(response)
}
