import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";

vi.mock("@/lib/api", () => ({
  api: vi.fn(),
}));

vi.mock("@/stores/auth", () => ({
  useAuth: () => ({ token: "test-token" }),
}));

vi.mock("@/lib/useRouteParams", () => ({
  useRouteParams: () => ({ teamId: "team-1" }),
}));

vi.mock("next/navigation", () => ({
  useRouter: () => ({ back: vi.fn(), push: vi.fn(), replace: vi.fn() }),
  usePathname: () => "/teams/team-1/audit",
}));

import { api } from "@/lib/api";
import { AuditClient } from "./client";

const mockApi = vi.mocked(api);

describe("AuditClient", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("shows loading state", () => {
    mockApi.mockImplementation(() => new Promise(() => {}));
    render(<AuditClient />);
    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  it("shows empty state", async () => {
    mockApi.mockResolvedValueOnce([]);
    render(<AuditClient />);
    await waitFor(() => {
      expect(screen.getByText(/no activity|aucune/i)).toBeInTheDocument();
    });
  });

  it("renders audit entries", async () => {
    mockApi.mockResolvedValueOnce([
      {
        id: "a1",
        actor_id: "u1",
        actor_name: "Alice",
        action: "member_kicked",
        entity_type: "team_member",
        entity_id: "u2",
        metadata: { target_name: "Bob" },
        created_at: "2026-08-04T12:00:00Z",
      },
      {
        id: "a2",
        actor_id: "u1",
        actor_name: "Alice",
        action: "release_cancelled",
        entity_type: "release",
        entity_id: "r1",
        metadata: { title: "v1.2.0" },
        created_at: "2026-08-04T11:00:00Z",
      },
    ]);

    render(<AuditClient />);

    await waitFor(() => {
      expect(screen.getByText("member kicked")).toBeInTheDocument();
      expect(screen.getByText("Bob")).toBeInTheDocument();
      expect(screen.getByText("release cancelled")).toBeInTheDocument();
      expect(screen.getByText("v1.2.0")).toBeInTheDocument();
      expect(screen.getAllByText("Alice")).toHaveLength(2);
    });
  });
});
