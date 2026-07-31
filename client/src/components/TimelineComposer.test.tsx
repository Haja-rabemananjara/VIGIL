import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { TimelineComposer } from "./TimelineComposer";

describe("TimelineComposer", () => {
  const defaultProps = {
    value: "",
    loading: false,
    onChange: vi.fn(),
    onSubmit: vi.fn(),
  };

  it("renders the textarea with the current value", () => {
    render(<TimelineComposer {...defaultProps} value="hello" />);
    const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
    expect(textarea.value).toBe("hello");
  });

  it("calls onChange when the textarea changes", () => {
    const onChange = vi.fn();
    render(<TimelineComposer {...defaultProps} onChange={onChange} />);
    fireEvent.change(screen.getByRole("textbox"), {
      target: { value: "new text" },
    });
    expect(onChange).toHaveBeenCalledWith("new text");
  });

  it("submits on Ctrl+Enter when not loading", () => {
    const onSubmit = vi.fn();
    render(
      <TimelineComposer {...defaultProps} value="hello" onSubmit={onSubmit} />,
    );
    fireEvent.keyDown(screen.getByRole("textbox"), {
      key: "Enter",
      ctrlKey: true,
    });
    expect(onSubmit).toHaveBeenCalledTimes(1);
  });

  it("does not submit on plain Enter", () => {
    const onSubmit = vi.fn();
    render(
      <TimelineComposer {...defaultProps} value="hello" onSubmit={onSubmit} />,
    );
    fireEvent.keyDown(screen.getByRole("textbox"), { key: "Enter" });
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("does not submit on Ctrl+Enter when loading", () => {
    const onSubmit = vi.fn();
    render(
      <TimelineComposer
        {...defaultProps}
        value="hello"
        loading
        onSubmit={onSubmit}
      />,
    );
    fireEvent.keyDown(screen.getByRole("textbox"), {
      key: "Enter",
      ctrlKey: true,
    });
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("disables the submit button when value is empty", () => {
    render(<TimelineComposer {...defaultProps} value="" />);
    const button = screen.getByRole("button");
    expect(button).toBeDisabled();
  });

  it("disables the submit button when value is whitespace only", () => {
    render(<TimelineComposer {...defaultProps} value="   " />);
    expect(screen.getByRole("button")).toBeDisabled();
  });

  it("disables the submit button when loading", () => {
    render(<TimelineComposer {...defaultProps} value="hello" loading />);
    expect(screen.getByRole("button")).toBeDisabled();
  });

  it("enables the submit button when value is non-empty and not loading", () => {
    render(<TimelineComposer {...defaultProps} value="hello" />);
    expect(screen.getByRole("button")).not.toBeDisabled();
  });

  it("calls onSubmit when the button is clicked", () => {
    const onSubmit = vi.fn();
    render(
      <TimelineComposer {...defaultProps} value="hello" onSubmit={onSubmit} />,
    );
    fireEvent.click(screen.getByRole("button"));
    expect(onSubmit).toHaveBeenCalledTimes(1);
  });
});
