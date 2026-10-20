# Security Policy

## Project status

Nodlance is an early-stage scaffold and is **not production-ready**. It does not yet implement database management operations. Do not expose it to untrusted networks or use it to manage production databases.

## Reporting a vulnerability

Please do not disclose potential vulnerabilities in public issues. Use GitHub's private vulnerability reporting feature for the Nodlance repository if it is enabled. Until a private reporting channel is configured, contact the maintainers privately through an established channel and avoid including secrets or production data in the report.

Maintainers should configure a private reporting channel and a response process before the first public release.

## Security design priorities

- Strong identity for API users and agents; TLS/mTLS where appropriate.
- Explicit authorization and least-privilege execution.
- No arbitrary remote shell command API.
- Secret references instead of plaintext credentials in job payloads and logs.
- Durable audit records for privileged operations.
- Egress restrictions and SSRF defenses for discovered endpoints.
- Fencing and safe coordination as release gates for any automated failover.
