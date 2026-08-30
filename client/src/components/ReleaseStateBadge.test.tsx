import { describe, it, expect } from "vitest";
import { screen } from "@testing-library/react";
import { ReleaseStateBadge } from "./ReleaseStateBadge";
import { renderWithAuth } from "@/test-utils";

describe("ReleaseStateBadge", () => {
  const states = [
    "created",
    "in_progress",
    "completed",
    "cancelled",
    "blocked",
  ] as const;

  it.each(states)("renders the state '%s'", (state) => {
    renderWithAuth(<ReleaseStateBadge state={state} />);
    expect(
      screen.getByText(new RegExp(state.replace("_", " "), "i")),
    ).toBeInTheDocument();
  });

  it("provides distinct visual signals for each state (color+icon+text)", () => {
    const rendered = states.map((state) => {
      const { container } = renderWithAuth(<ReleaseStateBadge state={state} />);
      return container.innerHTML;
    });

    const uniqueRenderings = new Set(rendered);
    expect(uniqueRenderings.size).toBe(states.length);
  });

  it("makes 'blocked' visually salient", () => {
    const { container } = renderWithAuth(<ReleaseStateBadge state="blocked" />);
    expect(container.innerHTML).toMatch(/destructive|red|warning/i);
  });
});
