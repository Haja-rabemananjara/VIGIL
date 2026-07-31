import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { IncidentHeader } from "./IncidentHeader";

const baseIncident = {
  title: "Database is down",
  body: "Primary DB is unreachable",
  status: "open" as const,
  severity: "critical" as const,
  created_by: "user-1",
  created_at: 1718000000,
};

describe("IncidentHeader", () => {
  const defaultProps = {
    incident: baseIncident,
    assignee: null,
    displayName: (id: string) => (id === "user-1" ? "Alice" : id),
    canAct: false,
    isManager: false,
    nextTransitions: [],
    transitionLoading: false,
    onTransition: vi.fn(),
    onOpenAssign: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the incident title", () => {
    render(<IncidentHeader {...defaultProps} />);
    expect(screen.getByText("Database is down")).toBeInTheDocument();
  });

  it("renders the incident body", () => {
    render(<IncidentHeader {...defaultProps} />);
    expect(screen.getByText("Primary DB is unreachable")).toBeInTheDocument();
  });

  it("does not render an empty body", () => {
    render(
      <IncidentHeader
        {...defaultProps}
        incident={{ ...baseIncident, body: "" }}
      />,
    );
    expect(
      screen.queryByText("Primary DB is unreachable"),
    ).not.toBeInTheDocument();
  });

  it("shows the creator's display name", () => {
    render(<IncidentHeader {...defaultProps} />);
    expect(screen.getByText("Alice")).toBeInTheDocument();
  });

  it("shows the assignee's display name when assigned", () => {
    render(
      <IncidentHeader
        {...defaultProps}
        assignee="user-2"
        displayName={(id) => (id === "user-2" ? "Bob" : id)}
      />,
    );
    expect(screen.getByText("Bob")).toBeInTheDocument();
  });

  it("shows an unassigned label when no assignee", () => {
    render(<IncidentHeader {...defaultProps} assignee={null} />);
    expect(screen.getByText(/unassigned/i)).toBeInTheDocument();
  });

  it("does not show action buttons when canAct is false", () => {
    render(
      <IncidentHeader
        {...defaultProps}
        canAct={false}
        nextTransitions={["acknowledged"]}
      />,
    );
    expect(
      screen.queryByRole("button", { name: /acknowledge/i }),
    ).not.toBeInTheDocument();
  });

  it("shows transition buttons when canAct is true", () => {
    render(
      <IncidentHeader
        {...defaultProps}
        canAct
        nextTransitions={["acknowledged"]}
      />,
    );
    expect(
      screen.getByRole("button", { name: /acknowledge/i }),
    ).toBeInTheDocument();
  });

  it("calls onTransition with the next status when a transition button is clicked", () => {
    const onTransition = vi.fn();
    render(
      <IncidentHeader
        {...defaultProps}
        canAct
        nextTransitions={["acknowledged", "resolved"]}
        onTransition={onTransition}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /acknowledge/i }));
    expect(onTransition).toHaveBeenCalledWith("acknowledged");
  });

  it("disables transition buttons while transitionLoading", () => {
    render(
      <IncidentHeader
        {...defaultProps}
        canAct
        nextTransitions={["acknowledged"]}
        transitionLoading
      />,
    );
    expect(screen.getByRole("button", { name: /acknowledge/i })).toBeDisabled();
  });

  it("shows the assign button only for managers", () => {
    const { rerender } = render(
      <IncidentHeader
        {...defaultProps}
        canAct
        isManager={false}
        nextTransitions={["acknowledged"]}
      />,
    );
    expect(
      screen.queryByRole("button", { name: /^assign$/i }),
    ).not.toBeInTheDocument();

    rerender(
      <IncidentHeader
        {...defaultProps}
        canAct
        isManager
        nextTransitions={["acknowledged"]}
      />,
    );
    expect(
      screen.getByRole("button", { name: /^assign$/i }),
    ).toBeInTheDocument();
  });

  it("calls onOpenAssign when assign button is clicked", () => {
    const onOpenAssign = vi.fn();
    render(
      <IncidentHeader
        {...defaultProps}
        canAct
        isManager
        nextTransitions={["acknowledged"]}
        onOpenAssign={onOpenAssign}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /^assign$/i }));
    expect(onOpenAssign).toHaveBeenCalledTimes(1);
  });

  it("uses distinct visual signals for state and severity badges", () => {
    const { container } = render(<IncidentHeader {...defaultProps} />);
    expect(container.innerHTML).toMatch(/open/i);
    expect(container.innerHTML).toMatch(/critical/i);
  });
});
