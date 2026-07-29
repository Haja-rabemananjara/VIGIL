import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  render,
  screen,
  fireEvent,
  waitFor,
  within,
} from "@testing-library/react";

// Mocks

const mockApi = vi.fn();
vi.mock("@/lib/api", () => {
  class ApiError extends Error {
    constructor(
      public status: number,
      public code: string,
      message: string,
    ) {
      super(message);
      this.name = "ApiError";
    }
  }
  return {
    api: (...args: unknown[]) => mockApi(...args),
    ApiError,
  };
});
import { ApiError } from "@/lib/api";

const mockPush = vi.fn();
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: mockPush, replace: vi.fn() }),
}));

vi.mock("@/stores/auth", () => ({
  useAuth: () => ({
    user: { id: "u1", display_name: "Alice", email: "alice@test.com" },
    token: "test-token",
  }),
}));

vi.mock("@/components/RequireAuth", () => ({
  RequireAuth: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));

vi.mock("@/components/UserMenu", () => ({
  UserMenu: () => <div data-testid="user-menu" />,
}));

import OnboardingPage from "./page";

const TEAM_FIXTURE = {
  id: "t1",
  name: "Team Alpha",
  role: "manager",
  created_at: "2024-01-01T00:00:00Z",
};

describe("OnboardingPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    mockApi.mockResolvedValue([]);
  });

  // Rendering

  it("renders welcome with user display name", () => {
    render(<OnboardingPage />);
    expect(screen.getByText(/Alice/)).toBeInTheDocument();
  });

  it("renders the user menu", () => {
    render(<OnboardingPage />);
    expect(screen.getByTestId("user-menu")).toBeInTheDocument();
  });

  it("hides back button when user has no teams", async () => {
    render(<OnboardingPage />);
    await waitFor(() => expect(mockApi).toHaveBeenCalled());
    expect(
      screen.queryByRole("button", { name: /back/i }),
    ).not.toBeInTheDocument();
  });

  // Existing teams

  it("shows back button when user has teams", async () => {
    mockApi.mockResolvedValueOnce([TEAM_FIXTURE]);
    render(<OnboardingPage />);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /back/i })).toBeInTheDocument(),
    );
  });

  it("back button navigates to first team", async () => {
    mockApi.mockResolvedValueOnce([TEAM_FIXTURE]);
    render(<OnboardingPage />);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /back/i })).toBeInTheDocument(),
    );
    fireEvent.click(screen.getByRole("button", { name: /back/i }));
    expect(mockPush).toHaveBeenCalledWith("/teams/t1/incidents");
  });

  it("back button uses last-viewed team from localStorage", async () => {
    localStorage.setItem("vigil_last_team", "t2");
    mockApi.mockResolvedValueOnce([
      TEAM_FIXTURE,
      { ...TEAM_FIXTURE, id: "t2", name: "Team B" },
    ]);
    render(<OnboardingPage />);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: /back/i })).toBeInTheDocument(),
    );
    fireEvent.click(screen.getByRole("button", { name: /back/i }));
    expect(mockPush).toHaveBeenCalledWith("/teams/t2/incidents");
  });

  // Create team dialog

  it("opens create dialog with a text input", async () => {
    render(<OnboardingPage />);
    fireEvent.click(screen.getByRole("button", { name: /create/i }));
    await waitFor(() => expect(screen.getByRole("dialog")).toBeInTheDocument());
    expect(
      within(screen.getByRole("dialog")).getByRole("textbox"),
    ).toBeInTheDocument();
  });

  it("shows error for empty team name", async () => {
    render(<OnboardingPage />);
    fireEvent.click(screen.getByRole("button", { name: /create/i }));
    await waitFor(() => expect(screen.getByRole("dialog")).toBeInTheDocument());
    const dialog = screen.getByRole("dialog");
    fireEvent.click(within(dialog).getByRole("button", { name: /create/i }));
    await waitFor(() =>
      expect(dialog.querySelector(".text-destructive")).toBeInTheDocument(),
    );
  });

  it("creates team and navigates on success", async () => {
    mockApi.mockResolvedValueOnce([]).mockResolvedValueOnce({
      id: "new-t",
      name: "My Team",
      role: "manager",
      created_at: "2024-01-01T00:00:00Z",
    });

    render(<OnboardingPage />);
    fireEvent.click(screen.getByRole("button", { name: /create/i }));
    await waitFor(() => expect(screen.getByRole("dialog")).toBeInTheDocument());
    const dialog = screen.getByRole("dialog");
    fireEvent.change(within(dialog).getByRole("textbox"), {
      target: { value: "My Team" },
    });
    fireEvent.click(within(dialog).getByRole("button", { name: /create/i }));

    await waitFor(() => {
      expect(mockApi).toHaveBeenCalledWith(
        "/teams",
        expect.objectContaining({
          method: "POST",
          body: { name: "My Team" },
        }),
      );
      expect(mockPush).toHaveBeenCalledWith("/teams/new-t/incidents");
    });
  });

  it("shows API error on create failure", async () => {
    mockApi
      .mockResolvedValueOnce([])
      .mockRejectedValueOnce(new ApiError(422, "validation", "Name too short"));

    render(<OnboardingPage />);
    fireEvent.click(screen.getByRole("button", { name: /create/i }));
    await waitFor(() => expect(screen.getByRole("dialog")).toBeInTheDocument());
    const dialog = screen.getByRole("dialog");
    fireEvent.change(within(dialog).getByRole("textbox"), {
      target: { value: "X" },
    });
    fireEvent.click(within(dialog).getByRole("button", { name: /create/i }));

    await waitFor(() =>
      expect(within(dialog).getByText("Name too short")).toBeInTheDocument(),
    );
  });

  it("shows generic error on non-ApiError create failure", async () => {
    mockApi
      .mockResolvedValueOnce([])
      .mockRejectedValueOnce(new Error("network"));

    render(<OnboardingPage />);
    fireEvent.click(screen.getByRole("button", { name: /create/i }));
    await waitFor(() => expect(screen.getByRole("dialog")).toBeInTheDocument());
    const dialog = screen.getByRole("dialog");
    fireEvent.change(within(dialog).getByRole("textbox"), {
      target: { value: "Test" },
    });
    fireEvent.click(within(dialog).getByRole("button", { name: /create/i }));

    await waitFor(() =>
      expect(
        within(dialog).getByText(/error|failed|wrong/i),
      ).toBeInTheDocument(),
    );
  });

  // Join team dialog

  it("opens join dialog with a text input", async () => {
    render(<OnboardingPage />);
    fireEvent.click(screen.getByRole("button", { name: /enter a code/i }));
    await waitFor(() => expect(screen.getByRole("dialog")).toBeInTheDocument());
    expect(
      within(screen.getByRole("dialog")).getByRole("textbox"),
    ).toBeInTheDocument();
  });

  it("shows error for empty join code", async () => {
    render(<OnboardingPage />);
    fireEvent.click(screen.getByRole("button", { name: /enter a code/i }));
    await waitFor(() => expect(screen.getByRole("dialog")).toBeInTheDocument());
    const dialog = screen.getByRole("dialog");
    fireEvent.click(within(dialog).getByRole("button", { name: /join/i }));
    await waitFor(() =>
      expect(dialog.querySelector(".text-destructive")).toBeInTheDocument(),
    );
  });

  it("joins team and navigates on success", async () => {
    mockApi.mockResolvedValueOnce([]).mockResolvedValueOnce({
      team_id: "joined-t",
      team_name: "Joined",
      role: "observer",
    });

    render(<OnboardingPage />);
    fireEvent.click(screen.getByRole("button", { name: /enter a code/i }));
    await waitFor(() => expect(screen.getByRole("dialog")).toBeInTheDocument());
    const dialog = screen.getByRole("dialog");
    fireEvent.change(within(dialog).getByRole("textbox"), {
      target: { value: "ABC123" },
    });
    fireEvent.click(within(dialog).getByRole("button", { name: /join/i }));

    await waitFor(() => {
      expect(mockApi).toHaveBeenCalledWith(
        "/teams/join",
        expect.objectContaining({
          method: "POST",
          body: { code: "ABC123" },
        }),
      );
      expect(mockPush).toHaveBeenCalledWith("/teams/joined-t/incidents");
    });
  });

  it("shows API error on join failure", async () => {
    mockApi
      .mockResolvedValueOnce([])
      .mockRejectedValueOnce(new ApiError(404, "not_found", "Invalid code"));

    render(<OnboardingPage />);
    fireEvent.click(screen.getByRole("button", { name: /enter a code/i }));
    await waitFor(() => expect(screen.getByRole("dialog")).toBeInTheDocument());
    const dialog = screen.getByRole("dialog");
    fireEvent.change(within(dialog).getByRole("textbox"), {
      target: { value: "BAD" },
    });
    fireEvent.click(within(dialog).getByRole("button", { name: /join/i }));

    await waitFor(() =>
      expect(within(dialog).getByText("Invalid code")).toBeInTheDocument(),
    );
  });

  it("uppercases the join code input", async () => {
    render(<OnboardingPage />);
    fireEvent.click(screen.getByRole("button", { name: /enter a code/i }));
    await waitFor(() => expect(screen.getByRole("dialog")).toBeInTheDocument());
    const input = within(screen.getByRole("dialog")).getByRole(
      "textbox",
    ) as HTMLInputElement;
    fireEvent.change(input, { target: { value: "abc123" } });
    expect(input.value).toBe("ABC123");
  });

  // Keyboard

  it("submits create dialog on Enter key", async () => {
    mockApi.mockResolvedValueOnce([]).mockResolvedValueOnce({
      id: "enter-t",
      name: "Enter Team",
      role: "manager",
      created_at: "2024-01-01T00:00:00Z",
    });

    render(<OnboardingPage />);
    fireEvent.click(screen.getByRole("button", { name: /create/i }));
    await waitFor(() => expect(screen.getByRole("dialog")).toBeInTheDocument());
    const input = within(screen.getByRole("dialog")).getByRole("textbox");
    fireEvent.change(input, { target: { value: "Enter Team" } });
    fireEvent.keyDown(input, { key: "Enter" });

    await waitFor(() =>
      expect(mockApi).toHaveBeenCalledWith(
        "/teams",
        expect.objectContaining({ method: "POST" }),
      ),
    );
  });

  it("submits join dialog on Enter key", async () => {
    mockApi.mockResolvedValueOnce([]).mockResolvedValueOnce({
      team_id: "enter-jt",
      team_name: "Joined",
      role: "observer",
    });

    render(<OnboardingPage />);
    fireEvent.click(screen.getByRole("button", { name: /enter a code/i }));
    await waitFor(() => expect(screen.getByRole("dialog")).toBeInTheDocument());
    const input = within(screen.getByRole("dialog")).getByRole("textbox");
    fireEvent.change(input, { target: { value: "XYZ" } });
    fireEvent.keyDown(input, { key: "Enter" });

    await waitFor(() =>
      expect(mockApi).toHaveBeenCalledWith(
        "/teams/join",
        expect.objectContaining({ method: "POST" }),
      ),
    );
  });
});
