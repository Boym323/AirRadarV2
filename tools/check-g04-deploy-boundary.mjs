import fs from "node:fs";
import assert from "node:assert/strict";

const root = new URL("..", import.meta.url).pathname;
const files = ["deploy/install-main-dev-auto.sh", "deploy/main-dev-auto-pull.py",
  "deploy/install-dev-bundle.sh", "deploy/activate-g03-main-autodeploy.sh"];
for (const file of files) {
  const text = fs.readFileSync(`${root}${file}`, "utf8");
  assert.equal(text.includes("--g04-dev-history"), false, `${file}: G-04 leakage`);
}
const g03 = fs.readFileSync(`${root}deploy/install-dev-bundle.sh`, "utf8");
assert.match(g03, /ExecStart=\/opt\/airradar-v2\/current\/airradarv2-engine --g03-live-api/);
const canary = fs.readFileSync(`${root}deploy/systemd/airradarv2-g04-canary.service`, "utf8");
const rollback = fs.readFileSync(`${root}tools/rollback_g04_canary.sh`, "utf8");
assert.match(canary, /airradarv2-g04-canary\.service/);
assert.match(canary, /127\.0\.0\.1:4114/);
assert.match(canary, /Restart=no/);
assert.match(canary, /RuntimeMaxSec=1875/);
assert.doesNotMatch(canary, /monitor_g04_canary\.sh/);
assert.match(canary, /ExecStart=\/opt\/airradar-v2\/canary\/releases\/e186c54f8304c3e437f6ed9a54c6bd31722dde7c\/airradarv2-engine --g04-dev-history/);
assert.match(canary, /CPUQuota=50%/);
assert.match(canary, /MemoryMax=2G/);
assert.doesNotMatch(canary, /fe0ac66fdebea25d21a2d211cd74cab922dd7d05/);
assert.match(canary, /RuntimeMaxSec=1875/);
const canaryEnvExample = fs.readFileSync(`${root}deploy/systemd/g04-canary.env.example`, "utf8");
assert.doesNotMatch(canaryEnvExample, /^AIRRADAR_G04_DB_MONITOR_SSH_TARGET=/m);
assert.doesNotMatch(canary, /\/opt\/airradar-v2\/current\/airradarv2-engine/);
assert.match(canary, /releases\/e186c54f8304c3e437f6ed9a54c6bd31722dde7c\/airradarv2-engine/);
assert.match(rollback, /systemctl stop airradarv2-g04-canary\.service/);
assert.match(rollback, /systemctl is-active --quiet airradarv2-engine\.service/);
assert.doesNotMatch(rollback, /systemctl restart airradarv2-engine\.service/);
assert.doesNotMatch(rollback, /\b(rm|DROP|pg_restore)\b/);
console.log("PASS: main auto-deploy nemění G-03 a G-04 je oddělený opt-in unit.");
