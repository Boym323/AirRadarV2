# G-00 bootstrap evidence

**Scope:** independent new GitHub repo, README and safety rules, Rust stub service, static Next.js placeholder, OpenAPI placeholder contract, feature acceptance matrix, V2-only CI, planned separate LXC.

**Out of scope:** actual live ADS-B, data tracking, MapLibre map, database creation, real alerts, V1 modification, LXC deployment, production DNS/proxy cutover.

**Validation:** initial CI `39657981782` failed Rust formatting; corrected build `39658242976` passed both Rust and web checks; follow-up `39658530749` produced pinned lockfiles. Final pinned-dependency CI status must be checked against its own SHA before marking `DONE`.

**Known blockers:**
- Lockfiles are committed from resolved successful CI bootstrap jobs. Confirm a new run uses `cargo --locked` and `npm ci` successfully on the final SHA.
- Review CI run on the exact commit and correct build/lint/typecheck issues.
- Owner provisioning and connectivity for the separate LXC are pending and should not be faked.
- Set explicit license only when selected.

**Result:** `PARTIAL` until lockfiles, CI results and LXC-independent G-00 verification are complete.
