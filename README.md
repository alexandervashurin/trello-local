# 🗂️ Trello Local — Локальный менеджер задач

**Trello Local** — это автономное веб-приложение для управления задачами, работающее **только в вашей локальной сети**. Никакого облака, никаких внешних зависимостей — всё работает на вашем сервере или компьютере.

> 🔒 **Идеально для закрытых сетей**: подходит для корпоративного использования, обучения или личных проектов без интернета.

---

## ✨ Особенности

- ✅ **Полностью автономно**: работает без интернета
- ✅ **Простота установки**: один бинарный файл + база данных
- ✅ **Drag-and-drop**: перетаскивайте карточки между списками
- ✅ **Статус выполнения**: помечайте задачи как "готово"
- ✅ **Комментарии**: обсуждайте задачи в карточках
- ✅ **Многоуровневая структура**: доски → списки → карточки
- 🌐 **Доступ из любой точки локальной сети**
- 👥 **Общие доски**: делитесь досками с другими пользователями
- 🔐 **Управление участниками**: назначайте роли и контролируйте доступ
- 🔑 **Регистрация/авторизация**: JWT-токены, хэширование паролей (bcrypt)
- 🔍 **Поиск**: быстрый поиск по доскам
- 🐳 **Docker**: готовая контейнеризация

---

## 🛠️ Требования

- **Rust** (для сборки) — [установка](https://rustup.rs/)
- **SQLite** (встроен в приложение)
- **Linux** (протестировано на Ubuntu 22.04)
- **Docker** (опционально) — [установка](https://docs.docker.com/get-docker/)

---

## 🐳 Запуск в Docker

### Быстрый старт

```bash
# Сборка и запуск
docker-compose up -d

# Просмотр логов
docker-compose logs -f

# Остановка
docker-compose down
```

Приложение будет доступно по адресу: http://localhost:8080

### Ручная сборка Docker

```bash
# Сборка образа
docker build -t trello-local .

# Запуск контейнера
docker run -d -p 8080:8080 -v trello-data:/app/backend/data --name trello-local trello-local
```

### Тома

Данные сохраняются в Docker volume `trello-data`:
- `/app/backend/data/trello.db` — база данных

Для экспорта данных:
```bash
docker run --rm -v trello-data:/data -v $(pwd):/backup ubuntu tar czf /backup/trello-backup.tar.gz -C /data .
```

---

## ⚡ Быстрый старт (без Docker)

### 1. Клонируйте репозиторий

```bash
git clone https://github.com/alexandervashurin/trello-local.git
cd trello-local
```

### 2. Сборка и запуск

```bash
cd backend
cargo build --release
cargo run
```

Сервер запустится на **http://localhost:8080**

### 3. Регистрация первого пользователя

1. Откройте http://localhost:8080
2. Вас перенаправит на страницу входа
3. Нажмите **"Зарегистрироваться"**
4. Введите имя пользователя и пароль (минимум 6 символов)
5. После регистрации вы будете перенаправлены на главную страницу

---

## 📋 API Reference

### Авторизация

```bash
# Регистрация
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123"}'

# Вход
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123"}'
```

Ответ содержит JWT-токен (срок действия — 7 дней):
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "user_id": 1,
  "username": "alice"
}
```

### Доски

```bash
# Получить все доски (с поиском)
curl "http://localhost:8080/api/boards"
curl "http://localhost:8080/api/boards?search=Проект"

# Создать доску
curl -X POST http://localhost:8080/api/boards \
  -H "Content-Type: application/json" \
  -d '{"title":"Моя доска","is_shared":false}'

# Обновить доску
curl -X PATCH http://localhost:8080/api/boards/1 \
  -H "Content-Type: application/json" \
  -d '{"title":"Новое название","is_shared":true}'

# Удалить доску
curl -X DELETE http://localhost:8080/api/boards/1
```

### Списки

```bash
# Создать список
curl -X POST http://localhost:8080/api/boards/1/lists \
  -H "Content-Type: application/json" \
  -d '{"title":"В работе"}'

# Обновить список
curl -X PATCH http://localhost:8080/api/lists/1 \
  -H "Content-Type: application/json" \
  -d '{"title":"Готово"}'

# Удалить список
curl -X DELETE http://localhost:8080/api/lists/1
```

### Карточки

```bash
# Создать карточку
curl -X POST http://localhost:8080/api/lists/1/cards \
  -H "Content-Type: application/json" \
  -d '{"title":"Задача","content":"Описание задачи"}'

# Обновить карточку (изменить название, описание, переместить)
curl -X PATCH http://localhost:8080/api/cards/1 \
  -H "Content-Type: application/json" \
  -d '{"title":"Новое название","content":"Новое описание","list_id":2,"done":true}'

# Удалить карточку
curl -X DELETE http://localhost:8080/api/cards/1
```

### Комментарии

```bash
# Получить комментарии к карточке
curl http://localhost:8080/api/cards/1/comments

# Добавить комментарий
curl -X POST http://localhost:8080/api/cards/1/comments \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"content":"Текст комментария"}'

# Обновить комментарий (только автор)
curl -X PATCH http://localhost:8080/api/comments/1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"content":"Новый текст"}'

# Удалить комментарий (только автор)
curl -X DELETE http://localhost:8080/api/comments/1 \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### Участники

```bash
# Получить участников доски
curl http://localhost:8080/api/boards/1/members

# Добавить участника
curl -X POST http://localhost:8080/api/boards/1/members \
  -H "Content-Type: application/json" \
  -d '{"user_id":2,"role":"member"}'

# Удалить участника
curl -X DELETE http://localhost:8080/api/boards/1/members/2
```

### Пользователи

```bash
# Получить всех пользователей
curl http://localhost:8080/api/users

# Найти пользователя по имени
curl "http://localhost:8080/api/users?username=alice"

# Создать пользователя (без пароля, для совместимости)
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{"username":"bob"}'

# Получить доски пользователя
curl http://localhost:8080/api/users/1/boards
```

---

## 📂 Структура проекта

```
trello-local/
├── backend/
│   ├── Cargo.toml           # Зависимости Rust
│   ├── Cargo.lock           # Заблокированные версии
│   ├── src/
│   │   ├── main.rs          # Точка входа (MVC)
│   │   ├── lib.rs           # Библиотека (для тестов)
│   │   ├── db.rs            # Подключение к БД
│   │   ├── controllers/     # Контроллеры (логика)
│   │   │   ├── auth.rs      # Авторизация
│   │   │   ├── boards.rs    # Доски
│   │   │   ├── lists.rs     # Списки
│   │   │   ├── cards.rs     # Карточки
│   │   │   ├── users.rs     # Пользователи
│   │   │   ├── comments.rs  # Комментарии
│   │   │   └── mod.rs
│   │   ├── models/          # Модели данных
│   │   │   ├── board.rs
│   │   │   ├── card.rs
│   │   │   ├── list.rs
│   │   │   ├── user.rs
│   │   │   ├── comment.rs   # Комментарии
│   │   │   └── mod.rs
│   │   ├── views/           # Представления (DTO)
│   │   │   ├── auth_view.rs
│   │   │   ├── board_view.rs
│   │   │   └── mod.rs
│   │   └── middleware/      # Middleware
│   │       ├── auth.rs      # JWT аутентификация
│   │       └── mod.rs
│   └── tests/
│       └── integration_tests.rs  # Интеграционные тесты
├── frontend/
│   ├── index.html           # Основной интерфейс
│   ├── login.html           # Страница входа/регистрации
│   ├── style.css            # Стили
│   └── app.js               # Логика приложения
├── data/                    # База данных SQLite
├── Dockerfile               # Docker образ
├── docker-compose.yml       # Docker Compose
├── .dockerignore            # Исключения Docker
└── README.md                # Этот файл
```

---

## 🧪 Тесты

```bash
cd backend
cargo test
```

Запускается 8 интеграционных тестов:
- `test_create_board` — создание доски
- `test_get_boards` — получение списка досок
- `test_create_list` — создание списка
- `test_create_card` — создание карточки
- `test_auth_register` — регистрация пользователя
- `test_auth_login` — вход пользователя
- `test_search_boards` — поиск досок
- `test_comments` — комментарии к карточкам

---

## 🔧 Настройка

### Переменные окружения

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `DATABASE_PATH` | Путь к базе данных | `/app/backend/data/trello.db` |
| `RUST_LOG` | Уровень логирования | `info` |

### Изменение секретного ключа JWT

В production измените `JWT_SECRET` в файле `backend/src/views/auth_view.rs`:

```rust
const JWT_SECRET: &[u8] = b"trello-local-secret-key-change-in-production-2024";
```

Или используйте переменную окружения.

---

## ❓ Частые вопросы

### Почему приложение не запускается?

1. Убедитесь, что папка `data/` существует: `mkdir -p data`
2. Проверьте права на запись: `chmod -R u+rw data/`
3. Проверьте логи: `cargo run 2>&1 | tail -50`

### Как обновить приложение?

1. Остановите сервер (`Ctrl+C` или `docker-compose down`)
2. Обновите код: `git pull`
3. Пересоберите: `cargo build --release` или `docker-compose build`
4. Запустите заново

### Как сбросить базу данных?

```bash
# Удалить базу данных
rm data/trello.db

# При следующем запуске создастся новая
cargo run
```

### Можно ли использовать на Windows?

Да, соберите проект в Windows:
```bash
cargo build --release
```

Или используйте Docker Desktop.

### Как настроить HTTPS?

Для production используйте reverse proxy (nginx, Caddy):

**nginx пример:**
```nginx
server {
    listen 443 ssl;
    server_name trello.local;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

---

## 📜 Лицензия

MIT License

Copyright (c) 2026 Александр Вашурин

Разрешается бесплатное использование, копирование, модификация и распространение ПО.

[Полный текст лицензии](LICENSE)

---

## 🤝 Вклад в проект

1. Fork репозитория
2. Создайте ветку: `git checkout -b feature/new-feature`
3. Внесите изменения: `git commit -m 'Add new feature'`
4. Отправьте: `git push origin feature/new-feature`
5. Создайте Pull Request

---

## 📞 Контакты

- GitHub: [@alexandervashurin](https://github.com/alexandervashurin)
- Email: alexandervashurin@yandex.ru
