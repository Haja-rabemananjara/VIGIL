import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";

// Mocks
vi.mock("@/lib/api", () => ({
  api: vi.fn(),
  ApiError: class ApiError extends Error {
    constructor(
      public status: number,
      public code: string,
      message: string,
    ) {
      super(message);
      this.name = "ApiError";
    }
  },
}));

const mockPush = vi.fn();
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: mockPush }),
}));

const mockUseRouteParams = vi.fn(() => ({ teamId: "team-1" }));
vi.mock("@/lib/useRouteParams", () => ({
  useRouteParams: () => mockUseRouteParams(),
}));

const mockUseAuth = vi.fn(() => ({
  token: "test-token",
  user: { id: "user-1", email: "alice@example.com", display_name: "Alice" },
}));
vi.mock("@/stores/auth", () => ({
  useAuth: () => mockUseAuth(),
}));

const mockUseVigilSocket = vi.fn(() => ({
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  lastEvent: null as any,
  reconnectCount: 0,
  status: "connected",
  send: vi.fn(),
}));
vi.mock("@/stores/socket", () => ({
  useVigilSocket: () => mockUseVigilSocket(),
}));

vi.mock("@/lib/navigation", () => ({
  saveLastTeam: vi.fn(),
}));

import { api } from "@/lib/api";
import { IncidentsClient } from "./client";

const mockApi = vi.mocked(api);

describe("IncidentsClient", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseRouteParams.mockReturnValue({ teamId: "team-1" });
    mockUseAuth.mockReturnValue({
      token: "test-token",
      user: {
        id: "user-1",
        email: "alice@example.com",
        display_name: "Alice",
      },
    });
    mockUseVigilSocket.mockReturnValue({
      lastEvent: null,
      reconnectCount: 0,
      status: "connected",
      send: vi.fn(),
    });
  });

  it("shows a loading state initially", () => {
    mockApi.mockImplementation(() => new Promise(() => {}));
    render(<IncidentsClient />);
    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  it("shows an error state when the fetch fails", async () => {
    mockApi.mockRejectedValue(new Error("network"));
    render(<IncidentsClient />);
    await waitFor(() => {
      expect(
        screen.getByText(/error|something went wrong/i),
      ).toBeInTheDocument();
    });
  });
});
