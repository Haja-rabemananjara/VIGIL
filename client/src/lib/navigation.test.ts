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

  it("returns the first team's incidents page when user has teams", async () => {
    mockApi.mockResolvedValue([
      { id: "aaa-111", name: "Alpha", role: "manager", created_at: "" },
    ]);
    const dest = await postLoginDestination("fake-token");
    expect(dest).toBe("/teams/aaa-111/incidents");
  });

  it("falls back to /onboarding on network error", async () => {
    mockApi.mockRejectedValue(new Error("network"));
    const dest = await postLoginDestination("fake-token");
    expect(dest).toBe("/onboarding");
  });

  it("returns the last visited team if stored", async () => {
    localStorage.setItem("vigil_last_team", "bbb-222");
    mockApi.mockResolvedValue([
      { id: "aaa-111", name: "Alpha", role: "manager", created_at: "" },
      { id: "bbb-222", name: "Beta", role: "observer", created_at: "" },
    ]);
    const dest = await postLoginDestination("fake-token");
    expect(dest).toBe("/teams/bbb-222/incidents");
  });
});
