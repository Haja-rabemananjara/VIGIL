import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { ReleaseStateBadge } from "./ReleaseStateBadge";

describe("ReleaseStateBadge", () => {
  const states = [
    "created",
    "in_progress",
    "completed",
    "cancelled",
    "blocked",
  ] as const;

  it.each(states)("renders the state '%s'", (state) => {
    render(<ReleaseStateBadge state={state} />);
    expect(
      screen.getByText(new RegExp(state.replace("_", " "), "i")),
    ).toBeInTheDocument();
  });

  it("provides distinct visual signals for each state (color+icon+text)", () => {
    const rendered = states.map((state) => {
      const { container } = render(<ReleaseStateBadge state={state} />);
      return container.innerHTML;
    });

    const uniqueRenderings = new Set(rendered);
    expect(uniqueRenderings.size).toBe(states.length);
  });

  it("makes 'blocked' visually salient", () => {
    const { container } = render(<ReleaseStateBadge state="blocked" />);
    expect(container.innerHTML).toMatch(/destructive|red|warning/i);
  });
});
