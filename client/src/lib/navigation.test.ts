import { describe, it, expect } from "vitest";
import { postLoginDestination } from "./navigation";

describe("postLoginDestination", () => {
    it("returns the onboarding route while teams are not wired (VGL-014)", () => {
        expect(postLoginDestination()).toBe("/onboarding");
    });
});