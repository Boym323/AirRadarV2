# G-00 bootstrap evidence

**Scope:** independent new GitHub repo, README and safety rules, Rust stub service, static Next.js placeholder, OpenAPI placeholder contract, feature acceptance matrix, V2-only CI, planned separate LXC.

**Out of scope:** actual live ADS-B, data tracking, MapLibre map, database creation, real alerts, V1 modification, LXC deployment, production DNS/proxy cutover.

**Initial validation state:** `NOT RUN` (toolchain unavailable in authoring environment; GitHub Actions is the subsequent source of evidence). Do not mark `DONE` until all checks pass on a matching SHA.

**Known blockers:**
- Generate and commit `Cargo.lock` and `apps/web/package-lock.json`, then use `cargo --locked` and `npm ci`; CI currently generates them during execution.
- Review CI run on the exact commit and correct build/lint/typecheck issues.
- Owner provisioning and connectivity for the separate LXC are pending and should not be faked.
- Set explicit license only when selected.

**Result:** `PARTIAL` until lockfiles, CI results and LXC-independent G-00 verification are complete.
