# AirRadar V2 agent rules

You are working on a **new greenfield project**, not the legacy AirRadar repository.

1. Read `docs/plan/GREENFIELD-BRIEF.md` and `contracts/feature-matrix.json` before implementing a new stage. One G-XX stage at a time; document implementation evidence.
2. Keep `Boym323/AirRadar` (V1), its PostgreSQL, LXC, ports, domain, CI/deploy and runtime configuration **untouched**. Do not copy the V1 source tree, secrets or production database.
3. The Rust engine will exclusively own live track state; never start a second independent receiver poller in Next.js. An optional provider/DB failure cannot stop the live path.
4. Do not claim to implement a KEEP feature until behavior is tested and its `feature-matrix.json` status is updated with proof. Do not reintroduce 99 excluded functions.
5. `G-00` is a non-live scaffold. Stubbed responses must explicitly report `NOT_IMPLEMENTED` and never resemble observed aircraft traffic.
6. Rust is for ingest, decoding, stream distribution and proven CPU-bound algorithms. TS is for web/UI. SQL is for set-oriented DB work. Other languages only with an evidence-backed ADR.
7. This repository uses a dedicated future V2 LXC. No production deploy, systemd changes, DNS/proxy changes, real alert delivery or database cleanup without explicit separate authorization.
8. V2 flight history starts empty and is sampled; never import or delete V1 history.
9. Add schema/contract, privacy, failure isolation, load and regression tests with each module. Record actual commands, commits and CI results; distinguish PASS, FAIL, NOT RUN and BLOCKED.
10. Do not commit environment files or secrets. Use independent V2 service/database credentials.
