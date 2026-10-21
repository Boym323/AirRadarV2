# G-00 bootstrap: verified implementation

**Software stage:** DONE. **New LXC provisioning/deployment:** NOT STARTED. **Live radar and G-01+ features:** NOT IMPLEMENTED.

- Repository: https://github.com/Boym323/AirRadarV2
- Verified source SHA: `99df0456c4e57b8192db9b41020453abcd14c8f6`
- CI: https://github.com/Boym323/AirRadarV2/actions/runs/39659205202
- CI result: **rust-engine PASS; nextjs-web PASS**, including lockfile-pinned installs, Rust fmt, Clippy, unit tests, Next.js lint/typecheck/unit tests/build and actual HTTP smoke tests.
- Initial failed CI `39657981782` was due to Rust formatting and was corrected. Follow-ups `39658242976`, `39658530749` also passed; the latter produced the committed reproducible `Cargo.lock` and `apps/web/package-lock.json`.

**Implemented:** isolated new repository; Rust/Tokio/Axum HTTP stub (bootstrap health, explicitly NOT_IMPLEMENTED aircraft API and SSE placeholder with keepalive); new Next.js static development website; 79 KEEP + 99 REMOVE feature matrix; OpenAPI baseline; LXC provisioning specification; standalone V2-only CI without deploy permissions.

**Not implemented:** actual readsb/BEAST/ADS-B ingest, live state, working map, PostgreSQL schema/queries, flight history, real notifications, production domain, service units, LXC provisioning or deployment. API stub responses are not proof of live functionality.

**Environment restrictions:** V1 code, CI, database, reverse proxy and original LXC were not edited. V2 will get a new Proxmox LXC, separate least-privilege PostgreSQL credentials, isolated ports and its own release process later.

**Pending outside G-00 software scope:** owner creates the LXC; provide its network/OS access for a separately authorized setup. Choose an explicit project license before publishing redistributed builds. G-01 must add detailed feature acceptance tests and deterministic sanitized fixtures.

**Decision:** GO for implementing **G-01** in the isolated repository; **NO-GO for live/production deployment** or representing the scaffold as a functional radar.
