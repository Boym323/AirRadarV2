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
assert.match(canary, /airradarv2-g04-canary\.service/);
assert.match(canary, /127\.0\.0\.1:4114/);
console.log("PASS: main auto-deploy nemění G-03 a G-04 je oddělený opt-in unit.");
