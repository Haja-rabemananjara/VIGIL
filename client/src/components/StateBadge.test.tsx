import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { StateBadge, type IncidentState } from "./StateBadge";

describe("StateBadge", () => {
    const states: IncidentState[] = ["open", "acknowledged", "escalated", "resolved"];

    it.each(states)("renders text label for state '%s'", (state) => {
        render(<StateBadge state={state} />);
        const badge = screen.getByText(/.+/);
        expect(badge).toBeInTheDocument();
    });

    it.each(states)("renders an icon for state '%s' (svg present)", (state) => {
        const { container } = render(<StateBadge state={state} />);
        const svg = container.querySelector("svg");
        expect(svg).toBeInTheDocument();
    });

    it("renders distinct labels per state", () => {
        const labels = states.map((s) => {
        const { container } = render(<StateBadge state={s} />);
        return container.textContent;
        });
        const unique = new Set(labels);
        expect(unique.size).toBe(states.length);
    });
});