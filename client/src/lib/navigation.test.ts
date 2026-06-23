import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@/lib/api", () => ({
  api: vi.fn(),
}));

import { postLoginDestination } from "@/lib/navigation";
import { api } from "@/lib/api";

const mockApi = vi.mocked(api);

describe("postLoginDestination", () => {
  beforeEach(() => {
    vi.resetAllMocks();
    localStorage.clear();
  });

  it("returns /onboarding when the user has no teams", async () => {
    mockApi.mockResolvedValue([]);
    const dest = await postLoginDestination("fake-token");
    expect(dest).toBe("/onboarding");
  });

  // TODO(VGL-034): update this test when /teams/:id/incidents exists
  it("returns /onboarding even with teams (team pages not built yet)", async () => {
    mockApi.mockResolvedValue([
      { id: "aaa-111", name: "Alpha", role: "manager", created_at: "" },
    ]);
    const dest = await postLoginDestination("fake-token");
    expect(dest).toBe("/onboarding");
  });

  it("falls back to /onboarding on network error", async () => {
    mockApi.mockRejectedValue(new Error("network"));
    const dest = await postLoginDestination("fake-token");
    expect(dest).toBe("/onboarding");
  });
});