import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { acceptsDelta, persistable, validObservation, type Observation } from "../lib/contract";
const data = JSON.parse(readFileSync(resolve(process.cwd(), "../../contracts/fixtures/observations.json"), "utf8")) as {
  cases: { name: string; valid: boolean; persistable: boolean; observation: Observation }[];
};
describe("G-01 sdílené Rust / TypeScript golden případy", () => {
  for (const c of data.cases) {
    it(c.name, () => {
      expect(validObservation(c.observation)).toBe(c.valid);
      expect(persistable(c.observation)).toBe(c.persistable);
    });
  }
  it("odmítá chybějící SSE seq a restart epoch", () => {
    expect(acceptsDelta("a", 7, "a", 8)).toBe(true);
    expect(acceptsDelta("a", 7, "a", 9)).toBe(false);
    expect(acceptsDelta("a", 7, "b", 8)).toBe(false);
  });
  it("JSON a BEAST stejné stanice nemění identitu letadla", () => {
    expect(data.cases[0].observation.aircraftKey).toBe(data.cases[1].observation.aircraftKey);
    expect(data.cases[0].observation.receiverId).toBe(data.cases[1].observation.receiverId);
    expect(data.cases[0].observation.transport).not.toBe(data.cases[1].observation.transport);
  });
});
