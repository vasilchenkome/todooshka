# Contributing

Thanks for taking a look at Todooshka. This is a small personal project, so please keep changes
focused and discuss larger features before writing a lot of code.

## Development setup

Requirements: Rust 1.99+, Node.js 26+, npm.

```bash
# backend
cargo run

# frontend, in a second terminal
cd frontend
npm install
npm run dev
```

The backend uses `sqlite://todooshka.db` by default. Create a user with
`cargo run -- user add -u kirill`. In development Vite proxies `/api` to
`http://127.0.0.1:8080`.

## Before opening a pull request

Run all checks from the repository root:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-boundaries.sh
cd frontend && npm run build
```

A pull request should:

- keep `domain` and `application` free of axum, sqlx, tokio, and HTTP types;
- add or update tests for behavior changes;
- avoid secrets, database files, and personal data;
- describe the problem and the chosen fix, not just the diff.

## Commits and pull requests

Small, focused commits are easiest to review. Conventional prefixes such as `feat:`, `fix:`,
`docs:`, `test:`, and `build:` are welcome. If a change affects the database schema, include a
new numbered migration rather than editing an existing one.

## Security

Do not report security issues in a public issue. See [SECURITY.md](SECURITY.md).

## License

By contributing, you agree that your contribution is licensed under the MIT license in
[LICENSE](LICENSE).
