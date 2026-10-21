import { describe, expect, it } from "vitest";
import { bootstrap } from "../lib/bootstrap";

describe("greenfield isolation", () => {
  it("does not claim receiver data or production readiness", () => {
    expect(bootstrap.stage).toBe("G-00");
    expect(bootstrap.liveTraffic).toBe(false);
    expect(bootstrap.historyAvailable).toBe(false);
    expect(bootstrap.productionEnabled).toBe(false);
  });
});
