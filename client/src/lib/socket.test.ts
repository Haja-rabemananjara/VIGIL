import { describe, it, expect } from "vitest";
import { computeBackoff, wsUrl } from "./socket";

describe("computeBackoff", () => {
  it("starts at 1 second", () => {
    expect(computeBackoff(0)).toBe(1000);
  });

  it("doubles each attempt", () => {
    expect(computeBackoff(1)).toBe(2000);
    expect(computeBackoff(2)).toBe(4000);
    expect(computeBackoff(3)).toBe(8000);
    expect(computeBackoff(4)).toBe(16000);
  });

  it("caps at 30 seconds", () => {
    expect(computeBackoff(5)).toBe(30000);
    expect(computeBackoff(6)).toBe(30000);
    expect(computeBackoff(100)).toBe(30000);
  });
});

describe("wsUrl", () => {
  it("converts http to ws and appends token", () => {
    const url = wsUrl("abc123");
    expect(url).toContain("ws://");
    expect(url).toContain("/ws?token=abc123");
  });

  it("does not double the ws prefix", () => {
    const url = wsUrl("abc123");
    expect(url).not.toContain("ws://ws://");
  });
});
