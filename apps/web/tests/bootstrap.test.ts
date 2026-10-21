import { describe, expect, it } from "vitest";
import { bootstrap } from "../lib/bootstrap";

describe("oddělení nové aplikace od produkce V1", () => {
  it("nepředstírá dostupnost živých dat ani připravenost k produkci", () => {
    expect(bootstrap.stage).toBe("G-00");
    expect(bootstrap.liveTraffic).toBe(false);
    expect(bootstrap.historyAvailable).toBe(false);
    expect(bootstrap.productionEnabled).toBe(false);
  });
});
