# AirRadar V2

**Greenfield development — not production-ready.** This repository is independent of [AirRadar V1](https://github.com/Boym323/AirRadar). The existing V1 application, databases and LXC remain unchanged.

## Direction

- Live aircraft engine: **Rust / Tokio / Axum**.
- Web UI: **TypeScript / Next.js / React / MapLibre** (map work begins in G-06).
- Storage: independent **PostgreSQL** database with SQLx (begins in G-05).
- One canonical live track owner, bounded SSE, sampled asynchronous history, no dependence on DB to serve live traffic.
- **79 KEEP** features are targets; **99 REMOVE** are excluded. All statuses currently say NOT_STARTED or EXCLUDED, because this is only a scaffold.

Read [the complete V2 build brief](docs/plan/GREENFIELD-BRIEF.md) and [AGENTS.md](AGENTS.md) before contributing.

## G-00 bootstrap scope

The engine implements only `/api/v1/health`, a clearly marked not-implemented aircraft response and a non-live SSE placeholder with a heartbeat. The website is a static placeholder. **There is no live ADS-B reception, real map, history persistence or alert sending.**

### Prerequisites

Rust stable (2024 edition, 1.85+), Node.js 22.18+, npm 10+. Development runs on a workstation or dedicated **V2 LXC**, never in the V1 production checkout.

### Run backend

~~~sh
cargo run -p airradarv2-engine
# curl http://127.0.0.1:4101/api/v1/health
~~~

### Run web

~~~sh
cd apps/web
npm ci
npm run dev
# http://127.0.0.1:4100
~~~

### Quality gates

~~~sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd apps/web && npm run lint && npm run typecheck && npm test && npm run build
~~~

Pinned `Cargo.lock` and `apps/web/package-lock.json` are tracked. CI uses `cargo --locked` and `npm ci` and never runs a deploy job. G-00 is **PARTIAL** until the exact committed SHA passes CI and runtime smoke checks are evidenced.

## Isolation

- Engine default bind `127.0.0.1:4101`.
- Web default port `4100`.
- No shared V1 runtime files, credentials, database, DNS, proxy, GitHub Actions deploy runner or production service.
- No real receiver input until an explicitly approved read-only shadow test.
- No V1 flight history import. V2 begins with an empty, separately owned DB when G-05 is implemented.

See [dedicated LXC setup](docs/runbooks/LXC-SETUP.md). **No production deploy is configured.**
