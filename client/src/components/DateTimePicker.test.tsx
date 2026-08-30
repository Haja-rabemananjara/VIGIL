import { describe, it, expect, vi } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import { renderWithAuth } from "@/test-utils";
import { DateTimePicker } from "./DateTimePicker";

describe("DateTimePicker", () => {
  it("shows placeholder when no value", () => {
    const onChange = vi.fn();
    renderWithAuth(<DateTimePicker value="" onChange={onChange} />);
    expect(screen.getByText("Pick a date")).toBeInTheDocument();
  });

  it("displays formatted date when value is set", () => {
    const onChange = vi.fn();
    renderWithAuth(
      <DateTimePicker value="2026-09-15T14:30" onChange={onChange} />,
    );
    expect(screen.getByText(/09\/15\/2026 14:30/)).toBeInTheDocument();
  });

  it("opens calendar on click", () => {
    const onChange = vi.fn();
    renderWithAuth(<DateTimePicker value="" onChange={onChange} />);
    fireEvent.click(screen.getByText("Pick a date"));
    expect(screen.getByRole("grid")).toBeInTheDocument();
  });

  it("renders hour and minute selects in the popover", () => {
    const onChange = vi.fn();
    renderWithAuth(
      <DateTimePicker value="2026-09-15T14:30" onChange={onChange} />,
    );
    fireEvent.click(screen.getByText(/09\/15\/2026/));

    const selects = screen.getAllByRole("combobox");
    expect(selects.length).toBe(2);
  });

  it("calls onChange when hour is changed", () => {
    const onChange = vi.fn();
    renderWithAuth(
      <DateTimePicker value="2026-09-15T14:30" onChange={onChange} />,
    );
    fireEvent.click(screen.getByText(/09\/15\/2026/));

    const selects = screen.getAllByRole("combobox");
    fireEvent.change(selects[0], { target: { value: "08" } });
    expect(onChange).toHaveBeenCalled();
    expect(onChange.mock.calls[0][0]).toContain("T08:");
  });

  it("calls onChange when minute is changed", () => {
    const onChange = vi.fn();
    renderWithAuth(
      <DateTimePicker value="2026-09-15T14:30" onChange={onChange} />,
    );
    fireEvent.click(screen.getByText(/09\/15\/2026/));

    const selects = screen.getAllByRole("combobox");
    fireEvent.change(selects[1], { target: { value: "45" } });
    expect(onChange).toHaveBeenCalled();
    expect(onChange.mock.calls[0][0]).toContain(":45");
  });

  it("accepts an id prop", () => {
    const onChange = vi.fn();
    const { container } = renderWithAuth(
      <DateTimePicker value="" onChange={onChange} id="ban-date" />,
    );
    expect(container.querySelector("#ban-date")).toBeInTheDocument();
  });
});
