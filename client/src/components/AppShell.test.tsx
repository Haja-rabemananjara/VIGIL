import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { AppShell } from "./AppShell";

// Mocks
let mockPathname = "/onboarding";
vi.mock("next/navigation", () => ({
  usePathname: () => mockPathname,
}));

vi.mock("next/router", () => ({
  useRouter: () => ({ push: vi.fn(), replace: vi.fn() }),
}));

vi.mock("next/link", () => ({
  default: ({
    href,
    children,
  }: {
    href: string;
    children: React.ReactNode;
  }) => <a href={href}>{children}</a>,
}));

const mockUseAuth = vi.fn();
vi.mock("@/stores/auth", () => ({
  useAuth: () => mockUseAuth(),
}));

const mockUseVigilSocket = vi.fn();
vi.mock("@/stores/socket", () => ({
  useVigilSocket: () => mockUseVigilSocket(),
}));

vi.mock("@/lib/useNotifications", () => ({
  useNotifications: () => {},
}));

vi.mock("@/lib/api", () => ({
  api: vi.fn(),
}));

import { api } from "@/lib/api";
const mockApi = vi.mocked(api);

vi.mock("./UserMenu", () => ({
  UserMenu: () => <div data-testid="user-menu">UserMenu</div>,
}));

const mockUser = {
  id: "user-1",
  email: "alice@example.com",
  display_name: "Alice",
  language: "en",
  created_at: 1718000000,
};

const mockTeams = [
  {
    id: "team-1",
    name: "Team Alpha",
    role: "manager",
    created_at: "2026-01-01",
  },
  {
    id: "team-2",
    name: "Team Beta",
    role: "observer",
    created_at: "2026-01-02",
  },
];

describe("AppShell", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockPathname = "/onboarding";
    mockUseAuth.mockReturnValue({ token: "test-token", user: mockUser });
    mockUseVigilSocket.mockReturnValue({
      status: "connected",
      lastEvent: null,
    });
    mockApi.mockResolvedValue(mockTeams);
  });

  it("renders the app title in the header", () => {
    render(
      <AppShell>
        <div>Content</div>
      </AppShell>,
    );
    expect(screen.getByText(/vigil/i)).toBeInTheDocument();
  });

  it("renders children in the main area", () => {
    render(
      <AppShell>
        <div>Test content</div>
      </AppShell>,
    );
    expect(screen.getByText("Test content")).toBeInTheDocument();
  });

  it("renders the user menu", () => {
    render(
      <AppShell>
        <div>Content</div>
      </AppShell>,
    );
    expect(screen.getByTestId("user-menu")).toBeInTheDocument();
  });

  it("shows an empty state message when there are no teams", async () => {
    mockApi.mockResolvedValue([]);
    render(
      <AppShell>
        <div>Content</div>
      </AppShell>,
    );
    await waitFor(() => {
      expect(screen.getByText(/no teams yet/i)).toBeInTheDocument();
    });
  });

  it("lists the user's teams in the sidebar", async () => {
    render(
      <AppShell>
        <div>Content</div>
      </AppShell>,
    );
    await waitFor(() => {
      expect(screen.getByText("Team Alpha")).toBeInTheDocument();
      expect(screen.getByText("Team Beta")).toBeInTheDocument();
    });
  });

  it("shows sub-navigation only for the active team", async () => {
    mockPathname = "/teams/team-1/incidents";
    render(
      <AppShell>
        <div>Content</div>
      </AppShell>,
    );
    await waitFor(() => {
      expect(screen.getByText(/members/i)).toBeInTheDocument();
      expect(screen.getByText(/releases/i)).toBeInTheDocument();
      expect(screen.getByText(/rules/i)).toBeInTheDocument();
    });
  });

  it("does not fetch teams when no token is present", () => {
    mockUseAuth.mockReturnValue({ token: null, user: null });
    render(
      <AppShell>
        <div>Content</div>
      </AppShell>,
    );
    expect(mockApi).not.toHaveBeenCalled();
  });

  it("removes the team from the sidebar when the current user is kicked", async () => {
    mockUseVigilSocket.mockReturnValue({
      status: "connected",
      lastEvent: null,
    });
    const { rerender } = render(
      <AppShell>
        <div>Content</div>
      </AppShell>,
    );

    await waitFor(() => {
      expect(screen.getByText("Team Alpha")).toBeInTheDocument();
    });

    mockUseVigilSocket.mockReturnValue({
      status: "connected",
      lastEvent: {
        type: "member_kicked",
        team_id: "team-1",
        user_id: "user-1",
      },
    });
    rerender(
      <AppShell>
        <div>Content</div>
      </AppShell>,
    );

    await waitFor(() => {
      expect(screen.queryByText("Team Alpha")).not.toBeInTheDocument();
    });
  });

  it("shows the add-team link", () => {
    render(
      <AppShell>
        <div>Content</div>
      </AppShell>,
    );
    const addLink = screen.getByRole("link", {
      name: /create or join a team/i,
    });
    expect(addLink).toBeInTheDocument();
  });

  it("renders the connection indicator", () => {
    render(
      <AppShell>
        <div>Content</div>
      </AppShell>,
    );
    expect(screen.getByText(/connected/i)).toBeInTheDocument();
  });
});
