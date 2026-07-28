import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { AssignDialog } from "./AssignDialog";

const mockMembers = [
  { user_id: "user-1", display_name: "Alice", role: "responder" },
  { user_id: "user-2", display_name: "Bob", role: "manager" },
];

describe("AssignDialog", () => {
  const defaultProps = {
    open: true,
    eligibleMembers: mockMembers,
    loading: false,
    error: "",
    onAssign: vi.fn(),
    onClose: vi.fn(),
  };

  it("renders nothing when closed", () => {
    const { container } = render(
      <AssignDialog {...defaultProps} open={false} />,
    );
    expect(container.firstChild).toBeNull();
  });

  it("renders the dialog title when open", () => {
    render(<AssignDialog {...defaultProps} />);
    expect(screen.getByRole("heading")).toBeInTheDocument();
  });

  it("renders one button per eligible member", () => {
    render(<AssignDialog {...defaultProps} />);
    expect(screen.getByRole("button", { name: /alice/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /bob/i })).toBeInTheDocument();
  });

  it("shows the role next to each member", () => {
    render(<AssignDialog {...defaultProps} />);
    expect(screen.getByText(/\(responder\)/i)).toBeInTheDocument();
    expect(screen.getByText(/\(manager\)/i)).toBeInTheDocument();
  });

  it("calls onAssign with the user_id when a member is clicked", () => {
    const onAssign = vi.fn();
    render(<AssignDialog {...defaultProps} onAssign={onAssign} />);
    fireEvent.click(screen.getByRole("button", { name: /alice/i }));
    expect(onAssign).toHaveBeenCalledWith("user-1");
  });

  it("displays an empty state when there are no eligible members", () => {
    render(<AssignDialog {...defaultProps} eligibleMembers={[]} />);
    expect(screen.getByText(/no eligible/i)).toBeInTheDocument();
  });

  it("disables member buttons while loading", () => {
    render(<AssignDialog {...defaultProps} loading />);
    expect(screen.getByRole("button", { name: /alice/i })).toBeDisabled();
    expect(screen.getByRole("button", { name: /bob/i })).toBeDisabled();
  });

  it("shows an error message when provided", () => {
    render(<AssignDialog {...defaultProps} error="Assignment failed" />);
    expect(screen.getByText("Assignment failed")).toBeInTheDocument();
  });

  it("calls onClose when the cancel button is clicked", () => {
    const onClose = vi.fn();
    render(<AssignDialog {...defaultProps} onClose={onClose} />);
    fireEvent.click(screen.getByRole("button", { name: /cancel/i }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
