import { describe, expect, it } from "vitest";
import { normalizeUnit } from "./layoutUnits";

describe("normalizeUnit", () => {
  it("keeps at most two decimal places", () => {
    expect(normalizeUnit(1)).toBe(1);
    expect(normalizeUnit(1.2)).toBe(1.2);
    expect(normalizeUnit(1.23)).toBe(1.23);
  });

  it("rounds extra decimal places", () => {
    expect(normalizeUnit(1.234)).toBe(1.23);
    expect(normalizeUnit(1.235)).toBe(1.24);
    expect(normalizeUnit(0.156)).toBe(0.16);
  });
});
