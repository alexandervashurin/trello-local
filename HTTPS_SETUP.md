# 🔒 Настройка HTTPS для Trello Local

Trello Local поддерживает работу через HTTPS с использованием reverse proxy (nginx или Caddy).

---

## 📋 Содержание

1. [Настройка nginx](#nginx)
2. [Настройка Caddy](#caddy)
3. [Переменные окружения](#переменные-окружения)
4. [Генерация самоподписанных сертификатов](#самоподписанные-сертификаты)

---

## nginx

### 1. Установка nginx

```bash
# Ubuntu/Debian
sudo apt update && sudo apt install nginx

# CentOS/RHEL
sudo yum install nginx
```

### 2. Конфигурация nginx

Создайте файл `/etc/nginx/sites-available/trello-local`:

```nginx
server {
    listen 80;
    server_name trello.yourdomain.com;
    
    # Перенаправление на HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name trello.yourdomain.com;

    # SSL сертификаты
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    # SSL настройки
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    ssl_prefer_server_ciphers off;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 1d;
    ssl_session_tickets off;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Content-Type-Options nosniff always;
    add_header X-Frame-Options DENY always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Forwarded-Host $host;
        proxy_set_header X-Forwarded-Port $server_port;
        
        # WebSocket поддержка (если понадобится)
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        
        # Таймауты
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
}
```

### 3. Включение конфигурации

```bash
# Создаём симлинк
sudo ln -s /etc/nginx/sites-available/trello-local /etc/nginx/sites-enabled/

# Проверяем конфигурацию
sudo nginx -t

# Перезапускаем nginx
sudo systemctl restart nginx
```

### 4. Автоматический SSL с Let's Encrypt

```bash
# Установка Certbot
sudo apt install certbot python3-certbot-nginx

# Получение сертификата
sudo certbot --nginx -d trello.yourdomain.com

# Автоматическое обновление
sudo certbot renew --dry-run
```

---

## Caddy

### 1. Установка Caddy

```bash
# Добавляем репозиторий
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list

# Установка
sudo apt update
sudo apt install caddy
```

### 2. Конфигурация Caddy

Создайте `/etc/caddy/Caddyfile`:

```
trello.yourdomain.com {
    reverse_proxy localhost:8080
    
    # Security headers
    header {
        Strict-Transport-Security "max-age=31536000; includeSubDomains"
        X-Content-Type-Options nosniff
        X-Frame-Options DENY
        X-XSS-Protection "1; mode=block"
        Referrer-Policy "strict-origin-when-cross-origin"
    }
}
```

### 3. Запуск Caddy

```bash
# Проверка конфигурации
caddy validate

# Перезапуск
sudo systemctl restart caddy
```

**Caddy автоматически получит и обновит SSL сертификаты!**

---

## Переменные окружения

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `PORT` | Порт для прослушивания | `8080` |
| `DATABASE_PATH` | Путь к базе данных | `./data/trello.db` |
| `JWT_SECRET` | Секретный ключ JWT | (генерируется) |
| `RUST_LOG` | Уровень логирования | `info` |
| `FRONTEND_DIR` | Путь к frontend | `/opt/trello-local/frontend` |

Пример `.env`:
```bash
PORT=8080
DATABASE_PATH=/var/lib/trello-local/trello.db
JWT_SECRET=your-secret-key-here
RUST_LOG=info
```

---

## Самоподписанные сертификаты

Для тестирования в локальной сети:

```bash
# Генерация приватного ключа
openssl genrsa -out key.pem 4096

# Генерация самоподписанного сертификата
openssl req -new -x509 -key key.pem -out cert.pem -days 365 \
  -subj "/C=RU/ST=Moscow/L=Moscow/O=Trello Local/CN=localhost"

# Установка прав
chmod 600 key.pem
chmod 644 cert.pem
```

**⚠️ Важно:** Самоподписанные сертификаты не доверяются браузерами по умолчанию. Используйте только для разработки!

---

## Проверка настройки

### 1. Проверка HTTPS

```bash
curl -I https://trello.yourdomain.com
```

Ожидаемый ответ:
```
HTTP/2 200 
strict-transport-security: max-age=31536000; includeSubDomains
x-content-type-options: nosniff
x-frame-options: DENY
```

### 2. Проверка редиректа HTTP → HTTPS

```bash
curl -I http://trello.yourdomain.com
```

Ожидаемый ответ:
```
HTTP/1.1 301 Moved Permanently
Location: https://trello.yourdomain.com/
```

---

## 🔗 Полезные ссылки

- [Mozilla SSL Configuration Generator](https://ssl-config.mozilla.org/)
- [SSL Labs Server Test](https://www.ssllabs.com/ssltest/)
- [Certbot Instructions](https://certbot.eff.org/)

---

**Последнее обновление:** 1 апреля 2026 г.
