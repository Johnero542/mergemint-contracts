import { describe, expect, it } from "vitest";
import { shortenAddress } from "./format";

describe("shortenAddress boundary cases", () => {
  it("returns addresses shorter than 12 characters unchanged", () => {
    const address = "GABCDE1234"; // 10 chars
    expect(shortenAddress(address)).toBe(address);
  });

  it("returns addresses exactly 12 characters unchanged", () => {
    const address = "GABCDE123456"; // 12 chars
    expect(address).toHaveLength(12);
    expect(shortenAddress(address)).toBe(address);
  });

  it("shortens addresses longer than 12 characters", () => {
    const address = "GABCDE1234567"; // 13 chars
    expect(shortenAddress(address)).toBe("GABCDE…4567");
  });
});
