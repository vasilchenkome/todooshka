# Todooshka

Личный Kanban по рабочим проектам: backend на Rust (axum + SQLite), фронт на React + TypeScript.

Сейчас сделаны этапы 0–5: API проектов и задач, вход по cookie-сессии, проекты и доска
с drag&drop. Следующий шаг — Docker-образ и проверка на Linux VM. План —
`docs/plans/mvp.md` (локально, `docs/` в `.gitignore`).

## Структура

| Крейт | Слой |
|---|---|
| `todooshka-domain` | сущности, инварианты, переходы статусов |
| `todooshka-app` | use cases и порты к инфраструктуре |
| `todooshka-sqlite` | репозитории, транзакции, миграции |
| `todooshka-http` | axum-роутер, DTO, middleware, статика |
| `todooshka` | composition root, конфигурация, запуск |

`frontend/` — Vite + React, в разработке проксирует `/api` на бэкенд.

## Разработка

```bash
# backend: http://127.0.0.1:8080
cargo run

# frontend: http://localhost:5173
cd frontend
npm install
npm run dev
```

Окружение:

| Переменная | По умолчанию | Назначение |
|---|---|---|
| `BIND_ADDR` | `127.0.0.1:8080` | адрес сервера; в контейнере `0.0.0.0:8080` |
| `DATABASE_URL` | `sqlite://todooshka.db` | файл базы; схема обязательна (`sqlite://`), иначе старт падает |
| `COOKIE_SECURE` | `false` | `true` — когда приложение за HTTPS: cookie получает флаг `Secure` |
| `RUST_LOG` | `info` | уровень логов |
| `LOG_FORMAT` | `text` | `json` — структурированные логи для контейнера |

## Пользователи

Учётная запись создаётся только из CLI, до или во время работы сервера на той же базе:

```bash
todooshka user add -u kirill      # пароль вводится скрыто и дважды
todooshka user passwd -u kirill   # смена пароля отзывает все сессии
todooshka user list
todooshka healthcheck             # 0, если /api/health/ready отвечает 200
todooshka backup ./copy.db        # копия базы + integrity_check
```

Для скриптов и контейнера пароль передаётся одной строкой: `todooshka user add -u kirill --password-stdin`.
Всё, кроме `/api/health` и `/api/auth/login`, требует cookie сессии (`HttpOnly`, `SameSite=Lax`, 30 дней).

## Проверки

```bash
cargo test
./scripts/check-boundaries.sh   # domain/application не тянут axum/sqlx/tokio
cd frontend && npm run build
```

## Docker и запуск на VM

Один бинарь отдаёт и API, и интерфейс: отдельный nginx не нужен, статика вшита в release-сборку.
Образ собирается на Linux VM, где есть Docker; на рабочем ПК Docker не требуется.

```bash
# на VM, из каталога репозитория
docker compose build
docker compose up -d
docker compose ps            # app должен стать healthy
```

Первый запуск: создать пользователя (пароль вводится скрыто, дважды).

```bash
docker compose exec app todooshka user add -u kirill
```

Сервис слушает `http://<ip-vm>:8080`; порт и тег образа задаются в `.env` рядом с compose:
`TODOOSHKA_PORT` (по умолчанию 8080), `TODOOSHKA_TAG` (по умолчанию 0.1.0), `COOKIE_SECURE`.
Приложение в контейнере пишет JSON-логи (`LOG_FORMAT=json`):

```bash
docker compose logs -f app
```

### Обновление и откат

```bash
TODOOSHKA_TAG=0.2.0 docker compose build
TODOOSHKA_TAG=0.2.0 docker compose up -d
# откат на прошлый тег
TODOOSHKA_TAG=0.1.0 docker compose up -d
```

### Бэкап и восстановление

Копия снимается через `VACUUM INTO` и проверяется `PRAGMA integrity_check`; существующий файл
не перезаписывается. Данные лежат в томе `todooshka-data` (`/data/todooshka.db`).

```bash
docker compose exec app todooshka backup /data/backups/todooshka-$(date +%F-%H%M).db
```

Восстановление — только на остановленном сервисе:

```bash
docker compose stop app
docker compose run --rm --entrypoint sh app -c 'rm -f /data/todooshka.db-wal /data/todooshka.db-shm'
docker compose cp todooshka-2026-11-14-1200.db app:/data/todooshka.db
docker compose start app
docker compose exec app todooshka user list   # база читается, пользователи на месте
```

### HTTPS (профиль tls, по умолчанию выключен)

В `docker-compose.yml` есть профиль `tls` с Caddy. Включается, когда есть DNS-имя:

```bash
TODOOSHKA_DOMAIN=todo.example.org COOKIE_SECURE=true docker compose --profile tls up -d
```

Caddy получает сертификат сам, приложение остаётся на внутреннем порту 8080.

## Статус

- [x] 0. Каркас
- [x] 1. Данные и проекты (CRUD, счётчики, HTTP-API)
- [x] 2. Задачи и порядок (move в транзакции, перенумерация)
- [x] 3. Auth (CLI пользователей, Argon2id, cookie-сессии, лимит попыток)
- [x] 4. Фронт: макет и проекты (вход, список, создание, архив)
- [x] 5. Фронт: доска и DnD (перенос между колонками, диалог задачи, вкладки на телефоне)
- [ ] 6. Образ и VM — образ и compose готовы, проверка на VM впереди
- [ ] 7. Полировка
