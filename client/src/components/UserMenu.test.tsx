import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { UserMenu } from "./UserMenu";

const mockSignout = vi.fn();
const mockUseAuth = vi.fn();

vi.mock("@/stores/auth", () => ({
  useAuth: () => mockUseAuth(),
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

const mockUser = {
  id: "user-1",
  email: "alice@example.com",
  display_name: "Alice Smith",
  language: "en",
  created_at: 1718000000,
};

describe("UserMenu", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockUseAuth.mockReturnValue({ user: mockUser, signout: mockSignout });
  });

  it("returns null when no user is loaded", () => {
    mockUseAuth.mockReturnValue({ user: null, signout: mockSignout });
    const { container } = render(<UserMenu />);
    expect(container.firstChild).toBeNull();
  });

  it("renders the user's initials in the avatar", () => {
    render(<UserMenu />);
    expect(screen.getByText("AS")).toBeInTheDocument();
  });

  it("shows only one initial for a single-word display name", () => {
    mockUseAuth.mockReturnValue({
      user: { ...mockUser, display_name: "Alice" },
      signout: mockSignout,
    });
    render(<UserMenu />);
    expect(screen.getByText("A")).toBeInTheDocument();
  });

  it("uppercases initials", () => {
    mockUseAuth.mockReturnValue({
      user: { ...mockUser, display_name: "alice smith" },
      signout: mockSignout,
    });
    render(<UserMenu />);
    expect(screen.getByText("AS")).toBeInTheDocument();
  });

  it("renders the dropdown trigger button", () => {
    render(<UserMenu />);
    expect(screen.getByRole("button")).toBeInTheDocument();
  });
});
