import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import GitHubCallbackPage from "./page";

vi.mock("@/lib/api", () => ({
  api: vi.fn(),
}));

vi.mock("@/lib/navigation", () => ({
  postLoginDestination: vi.fn(),
}));

vi.mock("@/lib/i18n", () => ({
  t: (key: string) => key,
  setLanguage: vi.fn(),
}));

describe("GitHubCallbackPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    window.history.pushState({}, "", "/");
  });

  it("shows an error when authorization code is missing", () => {
    window.history.pushState({}, "", "/auth/github/callback");

    render(<GitHubCallbackPage />);

    expect(screen.getByText("Missing authorization code")).toBeInTheDocument();
  });
});
