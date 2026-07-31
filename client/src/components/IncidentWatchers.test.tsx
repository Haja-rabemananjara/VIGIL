import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { IncidentWatchers } from "./IncidentWatchers";

describe("IncidentWatchers", () => {
  const displayName = (id: string) => {
    const names: Record<string, string> = {
      "user-1": "Alice Smith",
      "user-2": "Bob Jones",
      "user-3": "Charlie",
    };
    return names[id] ?? id;
  };

  it("renders nothing when watchers list is empty", () => {
    const { container } = render(
      <IncidentWatchers watchers={[]} displayName={displayName} />,
    );
    expect(container.firstChild).toBeNull();
  });

  it("renders the watching label when there are watchers", () => {
    render(
      <IncidentWatchers watchers={["user-1"]} displayName={displayName} />,
    );
    expect(screen.getByText(/watching/i)).toBeInTheDocument();
  });

  it("renders one avatar per watcher", () => {
    render(
      <IncidentWatchers
        watchers={["user-1", "user-2", "user-3"]}
        displayName={displayName}
      />,
    );
    expect(screen.getByText("AS")).toBeInTheDocument();
    expect(screen.getByText("BJ")).toBeInTheDocument();
    expect(screen.getByText("C")).toBeInTheDocument();
  });

  it("shows the full name as title (tooltip) on each avatar", () => {
    render(
      <IncidentWatchers watchers={["user-1"]} displayName={displayName} />,
    );
    const avatar = screen.getByTitle("Alice Smith");
    expect(avatar).toBeInTheDocument();
  });

  it("uses first two initials for multi-word names", () => {
    render(
      <IncidentWatchers watchers={["user-1"]} displayName={displayName} />,
    );
    expect(screen.getByText("AS")).toBeInTheDocument();
  });

  it("uses first letter only for single-word names", () => {
    render(
      <IncidentWatchers watchers={["user-3"]} displayName={displayName} />,
    );
    expect(screen.getByText("C")).toBeInTheDocument();
  });

  it("uppercases initials", () => {
    const dn = () => "alice smith";
    render(<IncidentWatchers watchers={["user-x"]} displayName={dn} />);
    expect(screen.getByText("AS")).toBeInTheDocument();
  });
});
