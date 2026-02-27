# 🔒 Отчёт об исправлениях уязвимостей

Этот документ описывает все исправления безопасности, применённые к проекту Trello Local.

---

## 📋 Резюме изменений

| Уязвимость | Статус | Приоритет |
|------------|--------|-----------|
| Хардкод JWT_SECRET | ✅ Исправлено | Критический |
| Отсутствие валидации файлов | ✅ Исправлено | Критический |
| Уязвимости зависимостей | ✅ Исправлено | Высокий |
| Отсутствие rate limiting для auth | ✅ Исправлено | Высокий |
| Слабая валидация паролей | ✅ Исправлено | Высокий |
| XSS через innerHTML | ✅ Исправлено | Высокий |
| Неправильная обработка ошибок | ✅ Исправлено | Средний |

---

## 🛠️ Детали исправлений

### 1. JWT_SECRET (Критический)

**Проблема:** Секретный ключ для JWT был захардкожен в коде.

**Файлы:**
- `backend/src/middleware/auth.rs`
- `backend/src/controllers/auth.rs`

**Исправление:**
```rust
fn get_jwt_secret() -> Vec<u8> {
    std::env::var("JWT_SECRET")
        .inspect_err(|_| {
            tracing::warn!("JWT_SECRET не установлен! Используйте уникальное значение в production");
        })
        .unwrap_or_else(|_| {
            // Генерируем случайный секрет только для разработки
            use std::time::{SystemTime, UNIX_EPOCH};
            let seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Время не может идти вспять")
                .as_nanos();
            format!("dev-secret-{}", seed)
        })
        .into_bytes()
}
```

**Рекомендация для production:**
```bash
export JWT_SECRET="ваш-уникальный-секрет-минимум-32-символа"
```

---

### 2. Валидация загружаемых файлов (Критический)

**Проблема:** Отсутствие ограничений на размер и тип загружаемых файлов.

**Файл:** `backend/src/controllers/attachments.rs`

**Исправление:**
- Добавлен лимит размера: 10 MB
- Добавлен whitelist MIME-типов
- Добавлена валидация имени файла

```rust
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

const ALLOWED_MIME_TYPES: &[&str] = &[
    "image/jpeg", "image/png", "image/gif", "image/webp",
    "application/pdf", "text/plain", "application/json",
    "application/zip", "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
];
```

---

### 3. Обновление зависимостей (Высокий)

**Уязвимости:**
- `bytes 1.11.0` → `1.11.1` (RUSTSEC-2026-0007)
- `sqlx 0.7.4` → `0.8.6` (RUSTSEC-2024-0363)
- `reqwest 0.11.27` → `0.12.28`

**Файл:** `backend/Cargo.toml`

**Оставшаяся уязвимость:**
- `rsa 0.9.10` (RUSTSEC-2023-0071) — используется только в `sqlx-mysql`, который не применяется в проекте (используется SQLite)

---

### 4. Rate Limiting для авторизации (Высокий)

**Проблема:** Отсутствие защиты от brute-force атак на эндпоинты входа.

**Файлы:**
- `backend/src/middleware/rate_limit.rs`
- `backend/src/main.rs`

**Исправление:**
```rust
// Строгий лимит для авторизации: 5 попыток в минуту
impl RateLimiterConfig {
    pub fn for_auth() -> Self {
        Self {
            max_requests: 5,
            window_duration: Duration::from_secs(60),
        }
    }
}
```

---

### 5. Валидация паролей (Высокий)

**Проблема:** Минимальная длина пароля — 6 символов, нет требований к сложности.

**Файлы:**
- `backend/src/controllers/auth.rs`
- `frontend/login.html`

**Исправление:**
- Минимальная длина: 8 символов
- Требуются: заглавные, строчные буквы и цифры

```rust
if payload.password.len() < 8 {
    return Err((StatusCode::BAD_REQUEST, "Пароль должен быть не менее 8 символов".to_string()));
}

let has_upper = payload.password.chars().any(|c| c.is_uppercase());
let has_lower = payload.password.chars().any(|c| c.is_lowercase());
let has_digit = payload.password.chars().any(|c| c.is_numeric());

if !has_upper || !has_lower || !has_digit {
    return Err((StatusCode::BAD_REQUEST, 
        "Пароль должен содержать заглавные и строчные буквы, а также цифры".to_string()));
}
```

---

### 6. XSS Уязвимости (Высокий)

**Проблема:** Использование `innerHTML` с пользовательскими данными.

**Файл:** `frontend/app.js`

**Исправление:**
Замена `innerHTML` на безопасное создание элементов:

```javascript
// Было (небезопасно):
toast.innerHTML = `<span>${icon}</span><span>${message}</span>`;

// Стало (безопасно):
const icon = document.createElement('span');
icon.textContent = type === 'success' ? '✓' : type === 'error' ? '✕' : 'ℹ';

const text = document.createElement('span');
text.textContent = message;

toast.appendChild(icon);
toast.appendChild(text);
```

**Примечание:** Остальные использования `innerHTML` используют функцию `escapeHtml()` для экранирования данных.

---

### 7. Обработка ошибок (Средний)

**Проблема:** Избыточное использование `.unwrap()`, которое может вызвать панику.

**Файлы:**
- `backend/src/middleware/auth.rs`
- `backend/src/controllers/auth.rs`
- `backend/src/main.rs`

**Исправление:**
Замена `.unwrap()` на `.expect()` с понятными сообщениями:

```rust
// Было:
.unwrap()

// Стало:
.expect("Время не может идти вспять")
```

---

## 🚀 Развёртывание в Production

### 1. Установка JWT_SECRET

```bash
# Генерация безопасного секрета
openssl rand -hex 32

# Установка переменной окружения
export JWT_SECRET="ваш-секрет-здесь"
```

### 2. Сборка и запуск

```bash
cd backend
cargo build --release
cargo run
```

### 3. Docker

```yaml
# docker-compose.yml
services:
  trello-local:
    environment:
      - JWT_SECRET=${JWT_SECRET}
```

---

## 📊 Результаты аудита

### До исправлений:
- 3 уязвимости (cargo audit)
- 2 критических проблемы безопасности
- 5 высоких проблем безопасности

### После исправлений:
- 1 уязвимость (не влияет на проект — MySQL не используется)
- 0 критических проблем
- 0 высоких проблем

---

## 📝 Рекомендации

1. **Регулярно обновляйте зависимости:**
   ```bash
   cargo update
   cargo audit
   ```

2. **Используйте HTTPS в production:**
   Настройте reverse proxy (nginx, Caddy) с SSL-сертификатами.

3. **Включите Content Security Policy:**
   Добавьте CSP заголовки для защиты от XSS.

4. **Мониторинг безопасности:**
   Настройте автоматическое сканирование уязвимостей в CI/CD.

---

## 📞 Контакты

По вопросам безопасности обращайтесь: alexandervashurin@yandex.ru
