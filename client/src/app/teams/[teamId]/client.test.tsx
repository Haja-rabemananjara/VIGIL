import { describe, it, expect, vi, beforeEach } from "vitest";
import { render } from "@testing-library/react";

const mockReplace = vi.fn();
vi.mock("next/navigation", () => ({
  useRouter: () => ({ replace: mockReplace }),
}));

const mockUseRouteParams = vi.fn();
vi.mock("@/lib/useRouteParams", () => ({
  useRouteParams: () => mockUseRouteParams(),
}));

import { TeamClient } from "./client";

describe("TeamClient (redirect)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("redirects to the incidents page for the current team", () => {
    mockUseRouteParams.mockReturnValue({ teamId: "abc-123" });
    render(<TeamClient />);
    expect(mockReplace).toHaveBeenCalledWith("/teams/abc-123/incidents");
  });

  it("does not redirect when there is no teamId in the URL", () => {
    mockUseRouteParams.mockReturnValue({});
    render(<TeamClient />);
    expect(mockReplace).not.toHaveBeenCalled();
  });

  it("renders nothing", () => {
    mockUseRouteParams.mockReturnValue({ teamId: "abc-123" });
    const { container } = render(<TeamClient />);
    expect(container.firstChild).toBeNull();
  });
});
