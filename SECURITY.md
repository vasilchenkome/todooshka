# Security policy

## Supported versions

The `main` branch and the latest published container image receive fixes. Older tags are kept
for rollback but are not maintained.

## Reporting a vulnerability

Please **do not** open a public GitHub issue for an unfixed security problem.

Use GitHub's private vulnerability reporting for this repository, or email
[kirill@vasilchenko.me](mailto:kirill@vasilchenko.me). Include:

- affected version or commit;
- reproduction steps or a proof of concept;
- impact you expect;
- any suggested fix, if you have one.

I will acknowledge the report as soon as I can and coordinate a fix and disclosure. This is a
personal project, so response times are best-effort, not a commercial SLA.

## Scope

The application is a single-user, self-hosted Kanban. Relevant reports include authentication
bypass, session fixation, SQL injection, path traversal, stored XSS, unsafe file handling, and
exposure of one user's data to another. Issues that require an already compromised host,
physical access, or a malicious administrator are generally out of scope.

Please do not test against installations you do not own. There is no bug bounty.
