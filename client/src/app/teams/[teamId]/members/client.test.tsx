import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";

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

import { api } from "@/lib/api";
import { MembersClient } from "./client";

const mockApi = vi.mocked(api);

const membersAsManager = [
  {
    user_id: "user-1",
    display_name: "Alice",
    email: "alice@example.com",
    role: "manager",
    joined_at: "2026-01-01T00:00:00Z",
  },
  {
    user_id: "user-2",
    display_name: "Bob",
    email: "bob@example.com",
    role: "responder",
    joined_at: "2026-01-02T00:00:00Z",
  },
  {
    user_id: "user-3",
    display_name: "Charlie",
    email: "charlie@example.com",
    role: "observer",
    joined_at: "2026-01-03T00:00:00Z",
  },
];

const membersAsObserver = [
  {
    user_id: "user-1",
    display_name: "Alice",
    email: "alice@example.com",
    role: "observer",
    joined_at: "2026-01-01T00:00:00Z",
  },
  {
    user_id: "user-2",
    display_name: "Bob",
    email: "bob@example.com",
    role: "manager",
    joined_at: "2026-01-02T00:00:00Z",
  },
];

describe("MembersClient", () => {
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
    render(<MembersClient />);
    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  it("shows an error state when the fetch fails", async () => {
    mockApi.mockRejectedValue(new Error("network"));
    render(<MembersClient />);
    await waitFor(() => {
      expect(
        screen.getByText(/error|something went wrong/i),
      ).toBeInTheDocument();
    });
  });

  it("renders one row per member", async () => {
    mockApi.mockResolvedValueOnce(membersAsManager);
    render(<MembersClient />);
    await waitFor(() => {
      expect(screen.getByText("Alice")).toBeInTheDocument();
      expect(screen.getByText("Bob")).toBeInTheDocument();
      expect(screen.getByText("Charlie")).toBeInTheDocument();
    });
  });

  it("shows the invite button when user is manager", async () => {
    mockApi.mockResolvedValueOnce(membersAsManager);
    render(<MembersClient />);
    await waitFor(() => {
      const buttons = screen.getAllByRole("button");
      const inviteBtn = buttons.find((b) =>
        /^invite/i.test(b.textContent ?? ""),
      );
      expect(inviteBtn).toBeDefined();
    });
  });

  it("does not show the invite button when user is not manager", async () => {
    mockApi.mockResolvedValueOnce(membersAsObserver);
    render(<MembersClient />);
    await waitFor(() => {
      expect(screen.getByText("Alice")).toBeInTheDocument();
    });
    const buttons = screen.queryAllByRole("button");
    const inviteBtn = buttons.find((b) => /^invite/i.test(b.textContent ?? ""));
    expect(inviteBtn).toBeUndefined();
  });

  it("always shows the leave button", async () => {
    mockApi.mockResolvedValueOnce(membersAsManager);
    render(<MembersClient />);
    await waitFor(() => {
      const buttons = screen.getAllByRole("button");
      const leaveBtn = buttons.find((b) => /^leave/i.test(b.textContent ?? ""));
      expect(leaveBtn).toBeDefined();
    });
  });

  it("opens the invitation dialog when invite button is clicked", async () => {
    mockApi.mockResolvedValueOnce(membersAsManager);
    render(<MembersClient />);
    await waitFor(() => expect(screen.getByText("Alice")).toBeInTheDocument());

    const buttons = screen.getAllByRole("button");
    const inviteBtn = buttons.find((b) =>
      /^invite/i.test(b.textContent ?? ""),
    )!;
    fireEvent.click(inviteBtn);

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: /generate/i }),
      ).toBeInTheDocument();
    });
  });

  it("generates an invitation code when generate is clicked", async () => {
    mockApi
      .mockResolvedValueOnce(membersAsManager)
      .mockResolvedValueOnce({ code: "ABC123" });

    render(<MembersClient />);
    await waitFor(() => expect(screen.getByText("Alice")).toBeInTheDocument());

    const buttons = screen.getAllByRole("button");
    const inviteBtn = buttons.find((b) =>
      /^invite/i.test(b.textContent ?? ""),
    )!;
    fireEvent.click(inviteBtn);

    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: /generate/i }),
      ).toBeInTheDocument(),
    );

    fireEvent.click(screen.getByRole("button", { name: /generate/i }));

    await waitFor(() => {
      expect(mockApi).toHaveBeenCalledWith(
        "/teams/team-1/invitations",
        expect.objectContaining({ method: "POST" }),
      );
    });
  });

  it("opens the leave confirmation dialog when leave is clicked", async () => {
    mockApi.mockResolvedValueOnce(membersAsManager);
    render(<MembersClient />);
    await waitFor(() => expect(screen.getByText("Alice")).toBeInTheDocument());

    const buttons = screen.getAllByRole("button");
    const leaveBtn = buttons.find((b) => /^leave/i.test(b.textContent ?? ""))!;
    fireEvent.click(leaveBtn);

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: /cancel/i }),
      ).toBeInTheDocument();
    });
  });
});
