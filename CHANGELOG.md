# Changelog

All notable changes to Todooshka are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Today view with overdue, today, and the next seven days; rows deep-link to the task dialog on the board.
- JSON export through `todooshka export` and authenticated `GET /api/export`, including projects, tasks, checklists, tags, comments, and attachment metadata without attachment bytes or account data.

### Changed

- Shared app header is sticky on Projects, the board, and Today.
- Board filters stay pinned; on desktop each column has its own scroll area and the page itself does not scroll.
- The next-seven-days filter now means tomorrow through day seven; today has a separate filter option.
- Done cards show the completion date instead of a relative due date.
- On mobile, the active column name and counter appear once in the status tabs, with the add action beside them.

## [0.1.2] - 2026-11-14

### Added

- Create tasks directly from each board column header. The form opens above the task list and returns focus to the add control when closed.
- Create projects from the Projects page header. The form opens before the project list instead of at the bottom of the page.

### Fixed

- Board columns now account for the app header and filter bar, keeping the add action and counters inside the viewport.
- Column titles and counters share the same baseline without extra space under the heading.

### Dependencies

- Updated Argon2 to 0.6, SQLx to 0.9, TypeScript to 7, and `@vitejs/plugin-react` to 6.

### Documentation

- Documented tag-based production deployment and the release flow in the README.

## [0.1.1] - 2026-11-14

### Added

- Automated production deployment on `v*` tags: reusable CI, GHCR image publishing, a dedicated SSH deploy user with a forced command, a pre-deploy database backup, health checks, and rollback on failure.

## [0.1.0] - 2026-11-14

### Added

- First public release under the MIT license.
- Projects and tasks with a three-column Kanban board and drag-and-drop movement.
- Planning fields: due date, priority, and time estimate.
- Subtasks and checklists, tags, search, and filters.
- Task comments and file attachments.
- Single-user authentication with Argon2id password hashing and an HTTP-only session cookie.
- Russian and English interface, including localized API errors.
- SQLite storage with backup and restore commands.
- A single binary serving both the API and the React application.
- Docker Compose deployment with Caddy, HTTPS, and images published to GHCR.

[Unreleased]: https://github.com/vasilchenkome/todooshka/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/vasilchenkome/todooshka/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/vasilchenkome/todooshka/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/vasilchenkome/todooshka/releases/tag/v0.1.0
