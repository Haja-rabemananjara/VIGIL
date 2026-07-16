"use client";

import { useVigilSocket } from "@/stores/socket";
import { useEffect } from "react";
import { notify } from "./platform";
import { useAuth } from "@/stores/auth";

/**
 * Central desktop notification dispatcher.
 * Watches WebSocket events and fires native OS notifications on the
 * three required triggers: incident assignment, critical escalation,
 * release blocked. Mounted once in AppShell.
 */
export function useNotifications() {
  const { lastEvent } = useVigilSocket();
  const { user } = useAuth();

  useEffect(() => {
    if (!lastEvent) return;

    switch (lastEvent.type) {
      case "incident_assigned": {
        if (lastEvent.assigned_to === user?.id) {
          notify("Incident assigned", "You have been assigned to an incident.");
        }
        break;
      }
      case "incident_escalated": {
        if (lastEvent.new_severity === "critical") {
          notify(
            "Critical incident",
            "An incident has been escalated to critical severity.",
          );
        }
        break;
      }
      case "release_state_changed": {
        if (lastEvent.new_state === "blocked") {
          notify(
            "Release blocked",
            "A release has been blocked by an active incident.",
          );
        }
        break;
      }
    }
  }, [lastEvent, user?.id]);
}