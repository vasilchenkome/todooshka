# Todooshka

Личный Kanban по рабочим проектам: backend на Rust (axum + SQLite), фронт на React + TypeScript.

Сейчас — этап 0: каркас, `/api/health` и проверка архитектурных границ. План — `docs/plans/mvp.md`
(локально, `docs/` в `.gitignore`).

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
| `RUST_LOG` | `info` | уровень логов |

## Проверки

```bash
cargo test
./scripts/check-boundaries.sh   # domain/application не тянут axum/sqlx/tokio
cd frontend && npm run build
```

## Docker

`Dockerfile` и `docker-compose.yml` — черновики этапа 0. Сборка и запуск на VM проверяются
на этапе 6; до этого локальный Docker не требуется.

## Статус

- [x] 0. Каркас
- [ ] 1. Данные и проекты
- [ ] 2. Задачи и порядок
- [ ] 3. Auth
- [ ] 4. Фронт: макет, проекты
- [ ] 5. Фронт: доска и DnD
- [ ] 6. Образ и VM
- [ ] 7. Полировка
