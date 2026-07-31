import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ConfirmDialog } from "./ConfirmDialog";

describe("ConfirmDialog", () => {
  const defaultProps = {
    open: true,
    onOpenChange: vi.fn(),
    title: "Delete something",
    description: "Are you sure?",
    confirmLabel: "Delete",
    onConfirm: vi.fn(),
  };

  it("renders the title when open", () => {
    render(<ConfirmDialog {...defaultProps} />);
    expect(screen.getByText("Delete something")).toBeInTheDocument();
  });

  it("renders the description when open", () => {
    render(<ConfirmDialog {...defaultProps} />);
    expect(screen.getByText("Are you sure?")).toBeInTheDocument();
  });

  it("does not render when closed", () => {
    render(<ConfirmDialog {...defaultProps} open={false} />);
    expect(screen.queryByText("Delete something")).not.toBeInTheDocument();
  });

  it("calls onConfirm when confirm button clicked", () => {
    const onConfirm = vi.fn();
    render(<ConfirmDialog {...defaultProps} onConfirm={onConfirm} />);
    fireEvent.click(screen.getByRole("button", { name: /delete/i }));
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it("uses destructive variant when destructive prop is true", () => {
    render(<ConfirmDialog {...defaultProps} destructive />);
    const button = screen.getByRole("button", { name: /delete/i });
    expect(button.className).toMatch(/destructive/i);
  });
});
