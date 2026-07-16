"use client";

import { usePathname } from "next/navigation";

interface RouteParams {
  teamId?: string;
  incidentId?: string;
  releaseId?: string;
}

/**
 * Static-export-safe replacement for useParams().
 *
 * In a static export served by a plain file server (Tauri, nginx),
 * dynamic pages are served from the "placeholder" build output, whose
 * RSC payload contains params frozen at build time. useParams() would
 * therefore return "placeholder" instead of the real URL segment.
 *
 * This hook derives params from the actual browser URL instead.
 */
export function useRouteParams(): RouteParams {
  const pathname = usePathname();
  const segments = (pathname ?? "").split("/").filter(Boolean);

  const params: RouteParams = {};
  if (segments[0] === "teams" && segments[1]) {
    params.teamId = segments[1];
    if (segments[2] === "incidents" && segments[3]) {
      params.incidentId = segments[3];
    }
    if (segments[2] === "releases" && segments[3]) {
      params.releaseId = segments[3];
    }
  }
  return params;
}