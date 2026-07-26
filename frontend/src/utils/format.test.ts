import { describe, expect, it } from "vitest";
import { formatTokenAmount, toRawTokenAmount } from "./format";

describe("formatTokenAmount / toRawTokenAmount round-trip", () => {
  const cases: Array<{ raw: string; formatted: string }> = [
    { raw: "0", formatted: "0" }, // edge value: zero
    { raw: "10000000", formatted: "1" }, // whole number
    { raw: "1000000", formatted: "0.1" }, // trailing-zero fraction
    { raw: "12345670", formatted: "1.234567" }, // max-precision fraction (7 dp)
    { raw: "100", formatted: "0.00001" }, // small fraction, leading zero padding
    { raw: "123456789000000000", formatted: "12345678900" }, // very large amount
  ];

  it.each(cases)("formats raw $raw as $formatted", ({ raw, formatted }) => {
    expect(formatTokenAmount(raw)).toBe(formatted);
  });

  it.each(cases)("parses $formatted back to raw $raw", ({ raw, formatted }) => {
    expect(toRawTokenAmount(formatted)).toBe(raw);
  });

  it("round-trips arbitrary raw integers through format -> parse", () => {
    const rawValues = ["0", "1", "9999999", "10000001", "999999999999999"];
    for (const raw of rawValues) {
      expect(toRawTokenAmount(formatTokenAmount(raw))).toBe(raw);
    }
  });
});
