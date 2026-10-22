#!/usr/bin/env node
// G-03.08: statický síťový kontrakt; reálné průchody přes proxy jsou další brána.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
const vite = readFileSync("apps/web/vite.config.ts", "utf8");
const nginx = readFileSync("deploy/nginx-g03.example.conf", "utf8");
for (const fragment of ['AIRRADAR_V2_G03_API === "1"', '127.0.0.1:4101',
                        '127.0.0.1:4102', 'target: apiTarget']) {
  assert.ok(vite.includes(fragment), "Vite: " + fragment);
}
for (const fragment of ['location = /api/v1/stream', 'proxy_pass http://127.0.0.1:4102',
                        'proxy_http_version 1.1', 'proxy_buffering off', 'proxy_cache off',
                        'proxy_read_timeout 75s', 'send_timeout 30s', 'gzip off',
                        'X-Accel-Buffering "no"', 'no-cache, no-transform',
                        'listen 127.0.0.1:8082']) {
  assert.ok(nginx.includes(fragment), "Nginx: " + fragment);
}
assert.ok(!nginx.includes('listen 0.0.0.0'));
assert.ok(!nginx.includes('listen [::]'));
console.log("PASS G-03.08: oddělený Vite opt-in a nebufferovaný SSE Nginx vzor.");
