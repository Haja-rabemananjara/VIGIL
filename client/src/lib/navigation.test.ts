import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@/lib/api", () => ({
  api: vi.fn(),
}));

import { postLoginDestination } from "./navigation";
import { api } from "./api";

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

  it("returns first team's incidents page by default", async () => {
    mockApi.mockResolvedValue([
      { id: "aaa-111", name: "Alpha", role: "manager", created_at: "" },
      { id: "bbb-222", name: "Beta", role: "observer", created_at: "" },
    ]);

    const dest = await postLoginDestination("fake-token");
    expect(dest).toBe("/teams/aaa-111/incidents");
  });

  it("prefers the last active team from localStorage", async () => {
    localStorage.setItem("vigil_last_team", "bbb-222");
    mockApi.mockResolvedValue([
      { id: "aaa-111", name: "Alpha", role: "manager", created_at: "" },
      { id: "bbb-222", name: "Beta", role: "observer", created_at: "" },
    ]);

    const dest = await postLoginDestination("fake-token");
    expect(dest).toBe("/teams/bbb-222/incidents");
  });

  it("ignores a stale last team not in the list", async () => {
    localStorage.setItem("vigil_last_team", "deleted-team");
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
});
