import { describe, it, expect } from "vitest";
import { displayEmoji } from "./Emoji";

describe("displayEmoji", () => {
  it("maps +1 to thumbs up", () => {
    expect(displayEmoji("+1")).toBe("👍");
  });

  it("maps -1 to thumbs down", () => {
    expect(displayEmoji("-1")).toBe("👎");
  });

  it("maps eyes to eyes emoji", () => {
    expect(displayEmoji("eyes")).toBe("👀");
  });

  it("maps warning to warning emoji", () => {
    expect(displayEmoji("warning")).toBe("⚠️");
  });

  it("maps check to check emoji", () => {
    expect(displayEmoji("check")).toBe("✅");
  });

  it("maps fire to fire emoji", () => {
    expect(displayEmoji("fire")).toBe("🔥");
  });

  it("returns the key unchanged for unknown emojis", () => {
    expect(displayEmoji("rocket")).toBe("rocket");
    expect(displayEmoji("unknown")).toBe("unknown");
  });
});
