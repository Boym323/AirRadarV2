# Provisioning a dedicated AirRadar V2 LXC — plan only

**No LXC has been provisioned or modified by this repository bootstrap.** The owner will create the container on Proxmox. V1 must remain unchanged.

Suggested initial profile (not a measured requirement): Debian 13, x86_64, 4 vCPU, 8 GiB memory, 60 GiB root disk, unprivileged container, dedicated hostname/IP, restricted firewall ingress and outbound HTTPS as needed. Validate Proxmox/LXC kernel restrictions before deploying Rust/Node native dependencies.

Reserve names to avoid V1 collision:
- Service user: `airradarv2` (no interactive login).
- Source/app: `/opt/airradar-v2`.
- Data/log state: `/var/lib/airradar-v2`; make private.
- Engine bind: `127.0.0.1:4101` by default (expose only through a deliberate V2 proxy later).
- Web bind: port `4100` (initially private).
- systemd names: `airradarv2-engine.service`, `airradarv2-web.service` (to be created and security-reviewed at later stages).
- Database: independent `airradar_v2` or `airradar_v2_dev`, separate least-privilege user, **never V1 credentials**.

Do not copy the V1 `.env`, CI secrets, SSH keys, TLS private keys, directories, database or readsb feeder configuration. V2 can share a **physical PostgreSQL server** only after isolation/capacity analysis, not a V1 database.

Do not open a public V2 hostname or alter reverse proxy until shadow tests G-14 and deployment approval G-15. Backups: create a new V2-only schedule and test restore on isolated V2 test DB. Do not move existing V1 backups.

After the owner provisions the LXC, record actual CPU/RAM/disk, OS and network details privately, install stable Rust/Node safely, validate the engine and web without connecting to receiver feeds, and only then prepare separate V2 CI/CD.

## Exit criteria

V1 remains on its own LXC. No ports, databases, secrets, systemd names or GitHub release runners overlap. No real alerts or raw telemetry are forwarded by the empty G-00 app.
