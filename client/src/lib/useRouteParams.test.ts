import { describe, it, expect, vi } from "vitest";
import { renderHook } from "@testing-library/react";
import { useRouteParams } from "./useRouteParams";

// Mock next/navigation
let mockPathname = "/";
vi.mock("next/navigation", () => ({
  usePathname: () => mockPathname,
}));

describe("useRouteParams", () => {
  it("returns empty object for the root path", () => {
    mockPathname = "/";
    const { result } = renderHook(() => useRouteParams());
    expect(result.current).toEqual({});
  });

  it("returns empty object for unrelated paths", () => {
    mockPathname = "/onboarding";
    const { result } = renderHook(() => useRouteParams());
    expect(result.current).toEqual({});
  });

  it("extracts teamId from /teams/:teamId", () => {
    mockPathname = "/teams/abc-123";
    const { result } = renderHook(() => useRouteParams());
    expect(result.current.teamId).toBe("abc-123");
    expect(result.current.incidentId).toBeUndefined();
    expect(result.current.releaseId).toBeUndefined();
  });

  it("extracts teamId + incidentId from an incident detail URL", () => {
    mockPathname = "/teams/abc-123/incidents/def-456";
    const { result } = renderHook(() => useRouteParams());
    expect(result.current.teamId).toBe("abc-123");
    expect(result.current.incidentId).toBe("def-456");
  });

  it("extracts teamId + releaseId from a release detail URL", () => {
    mockPathname = "/teams/abc-123/releases/xyz-789";
    const { result } = renderHook(() => useRouteParams());
    expect(result.current.teamId).toBe("abc-123");
    expect(result.current.releaseId).toBe("xyz-789");
  });

  it("returns only teamId on a listing page (no detail id)", () => {
    mockPathname = "/teams/abc-123/incidents";
    const { result } = renderHook(() => useRouteParams());
    expect(result.current.teamId).toBe("abc-123");
    expect(result.current.incidentId).toBeUndefined();
  });

  it("handles trailing slashes gracefully", () => {
    mockPathname = "/teams/abc-123/incidents/def-456/";
    const { result } = renderHook(() => useRouteParams());
    expect(result.current.teamId).toBe("abc-123");
    expect(result.current.incidentId).toBe("def-456");
  });
});
