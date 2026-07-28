import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { RequireAuth } from "./RequireAuth";

const mockReplace = vi.fn();
vi.mock("next/navigation", () => ({
  useRouter: () => ({ replace: mockReplace }),
}));

const mockUseAuth = vi.fn();
vi.mock("@/stores/auth", () => ({
  useAuth: () => mockUseAuth(),
}));

const mockUser = {
  id: "user-1",
  email: "alice@example.com",
  display_name: "Alice",
  language: "en",
  created_at: 1718000000,
};

describe("RequireAuth", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders children when a user is present and not loading", () => {
    mockUseAuth.mockReturnValue({ user: mockUser, isLoading: false });
    render(
      <RequireAuth>
        <div>Protected content</div>
      </RequireAuth>,
    );
    expect(screen.getByText("Protected content")).toBeInTheDocument();
  });

  it("shows a loading state when isLoading is true", () => {
    mockUseAuth.mockReturnValue({ user: null, isLoading: true });
    render(
      <RequireAuth>
        <div>Protected content</div>
      </RequireAuth>,
    );
    expect(screen.queryByText("Protected content")).not.toBeInTheDocument();
    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  it("does not render children when there is no user", () => {
    mockUseAuth.mockReturnValue({ user: null, isLoading: false });
    render(
      <RequireAuth>
        <div>Protected content</div>
      </RequireAuth>,
    );
    expect(screen.queryByText("Protected content")).not.toBeInTheDocument();
  });

  it("redirects to /signin when no user and not loading", () => {
    mockUseAuth.mockReturnValue({ user: null, isLoading: false });
    render(
      <RequireAuth>
        <div>Protected content</div>
      </RequireAuth>,
    );
    expect(mockReplace).toHaveBeenCalledWith("/signin");
  });

  it("does not redirect while loading", () => {
    mockUseAuth.mockReturnValue({ user: null, isLoading: true });
    render(
      <RequireAuth>
        <div>Protected content</div>
      </RequireAuth>,
    );
    expect(mockReplace).not.toHaveBeenCalled();
  });

  it("does not redirect when the user is loaded", () => {
    mockUseAuth.mockReturnValue({ user: mockUser, isLoading: false });
    render(
      <RequireAuth>
        <div>Protected content</div>
      </RequireAuth>,
    );
    expect(mockReplace).not.toHaveBeenCalled();
  });
});
