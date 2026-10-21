# ADR 0001 — Standalone AirRadar V2

**Decision:** Start in a separate GitHub repository and dedicated future LXC. Rust/Tokio/Axum will own live ingest and canonical RAM state. Next.js/React/MapLibre renders UI only. PostgreSQL/SQLx handles bounded sampled history and indexed queries. Do not migrate old flight history.

**Motivation:** V1 accumulated coupled web, ingest, history, geometry, status and notification concerns, and a large CSS/React surface. V2 must maintain a single state authority and avoid putting DB/provider work in the readsb → RAM → SSE critical path.

**Boundaries:** The Rust HTTP/SSE engine is the source of truth. Web code must not implement separate readsb polling or flight history writes. Prefer versioned OpenAPI contracts. Preserve public privacy, OGN DDB fail-closed semantics, ATC uncertainty, local/network provenance and explicit service degradation.

**Alternatives:** All-TypeScript rewrite (rejected as preferred default for low-level BEAST and CPU-bound hot paths, not because TypeScript is categorically slow); several microservices and queues (rejected as needless operational overhead before profiling).

**Risks:** Rust introduces new build/deployment expertise, backend/frontend version coordination, and two runtime processes. Test graceful shutdown, SSE backpressure, API versioning, memory bounds, container resource consumption and rollback before running production.

**State:** Accepted as a starting design; revisit based on measured evidence, not assumptions.
