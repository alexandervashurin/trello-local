# 🗂️ Trello Local — Локальный менеджер задач


**Trello Local** — это автономное веб-приложение для управления задачами, работающее **только в вашей локальной сети**. Никакого облака, никаких внешних зависимостей — всё работает на вашем сервере или компьютере.

> 🔒 **Идеально для закрытых сетей**: подходит для корпоративного использования, обучения или личных проектов без интернета.

---

## ✨ Особенности

- ✅ **Полностью автономно**: работает без интернета
- ✅ **Простота установки**: один бинарный файл + база данных
- ✅ **Drag-and-drop**: перетаскивайте карточки между списками
- ✅ **Статус выполнения**: помечайте задачи как "готово"
- ✅ **Резервное копирование**: Python-скрипт для бэкапов
- ✅ **Многоуровневая структура**: доски → списки → карточки
- 🌐 **Доступ из любой точки локальной сети**
- 👥 **Общие доски**: делитесь досками с другими пользователями
- 🔐 **Управление участниками**: назначайте роли и контролируйте доступ

---

## 🛠️ Требования

- **Rust** (для сборки) — [установка](https://rustup.rs/)
- **Python 3** (для бэкапов) — предустановлен в большинстве дистрибутивов Linux
- **SQLite** (встроен в приложение)
- **Linux** (протестировано на Ubuntu 22.04)

---

## ⚡ Быстрый старт

### 1. Клонируйте репозиторий

```bash
git clone https://github.com/alexandervashurin/trello-local.git
cd trello-local
cargo build --release
cargo run --manifest-path backend/Cargo.toml
http://<IP-адрес-вашего-сервера>:8080
```
### 2. 💾 Резервное копирование

```bash
python3 backup.py
```
Файлы сохраняются в папку  ```backups/``` с именем вида:
```trello_2026-01-20_14-30-00.db```

Для автоматизации добавьте в ```crontab```:

```bash
0 */6 * * * cd /путь/к/trello-local && python3 backup.py
```

### 3. 👥 Работа с пользователями и общими досками

#### Создание пользователей

```bash
# Создать первого пользователя
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{"username":"alice"}'

# Создать второго пользователя
curl -X POST http://localhost:8080/api/users \
  -H "Content-Type: application/json" \
  -d '{"username":"bob"}'

# Получить список всех пользователей
curl http://localhost:8080/api/users
```

#### Создание общих досок

```bash
# Создать личную доску (is_shared: false)
curl -X POST http://localhost:8080/api/boards \
  -H "Content-Type: application/json" \
  -d '{"title":"Мои задачи","is_shared":false}'

# Создать общую доску для команды
curl -X POST http://localhost:8080/api/boards \
  -H "Content-Type: application/json" \
  -d '{"title":"Команда разработки","is_shared":true}'
```

#### Управление участниками доски

```bash
# Добавить участника на доску (роль: member или owner)
curl -X POST http://localhost:8080/api/boards/1/members \
  -H "Content-Type: application/json" \
  -d '{"user_id":2,"role":"member"}'

# Получить список участников доски
curl http://localhost:8080/api/boards/1/members

# Удалить участника из доски
curl -X DELETE http://localhost:8080/api/boards/1/members/2
```

#### Получение досок пользователя

```bash
# Получить все доски пользователя (его собственные + общие)
curl http://localhost:8080/api/users/1/boards
```

### 4. 📂 Структура проекта
```
trello-local/
├── data/                # База данных SQLite
├── backups/             # Резервные копии
├── frontend/
│   ├── index.html       # Основной интерфейс
│   ├── style.css        # Стили
│   └── app.js           # Логика приложения
├── backend/
│   ├── Cargo.toml       # Зависимости Rust
│   └── src/             # Исходный код бэкенда
├── backup.py            # Скрипт резервного копирования
└── README.md            # Этот файл
```

### 5. 🔧 Ручная настройка (для разработки)
## Установка зависимостей

```bash
# Для Rust
rustup update
cargo install sqlx-cli --no-default-features --features rustls,sqlite

# Для Python
pip3 install requests
```

## Сборка в режиме разработки

```bash
cd backend
cargo run
```

### 6. ❓ Частые вопросы
## Почему приложение не запускается?
Убедитесь, что папка ```data/``` существует: ```mkdir -p data```
Проверьте права на запись: ```chmod -R u+rw data/```

## Как обновить приложение?
Остановите текущий сервер (```Ctrl+C```)
Скачайте новые файлы
Пересоберите: ```cargo build --release```
Запустите заново

## 7. Можно ли использовать на Windows?
Не собирал не пробовал, пробуйте!

### 📜 Лицензия
Этот проект распространяется под лицензией ```MIT``` — вы можете свободно использовать его в личных и коммерческих целях.

```
MIT License

Copyright (c) 2026 Ваше Имя

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

```


