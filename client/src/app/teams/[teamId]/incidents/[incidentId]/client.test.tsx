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

const mockUseRouteParams = vi.fn(() => ({
  teamId: "team-1",
  incidentId: "inc-1",
}));
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
import { IncidentDetailClient } from "./client";

const mockApi = vi.mocked(api);

const mockIncident = {
  id: "inc-1",
  title: "Database is down",
  body: "Primary DB unreachable",
  status: "open",
  severity: "critical",
  created_by: "user-1",
  created_at: 1718000000,
  assignee_id: null,
};

const mockTimeline = {
  entries: [
    {
      id: "entry-1",
      author_id: "user-1",
      kind: "message",
      content: "Investigating stuff",
      created_at: 1718000100,
      edited_at: null,
    },
  ],
};

const mockMembers = [
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
];

const mockEmojis = { emojis: ["+1", "-1", "fire"] };
const mockReactions = { reactions: {} };

function setupHappyPath() {
  mockApi
    .mockResolvedValueOnce(mockIncident)
    .mockResolvedValueOnce(mockTimeline)
    .mockResolvedValueOnce(mockMembers)
    .mockResolvedValueOnce(mockEmojis)
    .mockResolvedValueOnce(mockReactions);
}

Element.prototype.scrollIntoView = vi.fn();

describe("IncidentDetailClient", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseRouteParams.mockReturnValue({
      teamId: "team-1",
      incidentId: "inc-1",
    });
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
    render(<IncidentDetailClient />);
    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  it("shows an error state when the fetch fails", async () => {
    mockApi.mockRejectedValue(new Error("network"));
    render(<IncidentDetailClient />);
    await waitFor(() => {
      expect(
        screen.getByText(/error|something went wrong/i),
      ).toBeInTheDocument();
    });
  });

  it("renders the incident title and body", async () => {
    setupHappyPath();
    render(<IncidentDetailClient />);
    await waitFor(() => {
      expect(screen.getByText("Database is down")).toBeInTheDocument();
      expect(screen.getByText("Primary DB unreachable")).toBeInTheDocument();
    });
  });

  it("renders the timeline entries", async () => {
    setupHappyPath();
    render(<IncidentDetailClient />);
    await waitFor(() => {
      const matches = screen.getAllByText("Investigating stuff");
      expect(matches.length).toBeGreaterThan(0);
    });
  });

  it("shows a composer for responders and managers", async () => {
    setupHappyPath();
    render(<IncidentDetailClient />);
    await waitFor(() => {
      const textareas = document.querySelectorAll("textarea");
      expect(textareas.length).toBeGreaterThan(0);
    });
  });

  it("has a back button", async () => {
    setupHappyPath();
    render(<IncidentDetailClient />);
    await waitFor(() => {
      const buttons = screen.getAllByRole("button");
      const backBtn = buttons.find((b) =>
        /back to list|back to incidents|^back$/i.test(b.textContent ?? ""),
      );
      expect(backBtn).toBeDefined();
    });
  });
});
