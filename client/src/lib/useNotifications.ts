"use client";

import { useVigilSocket } from "@/stores/socket";
import { useEffect } from "react";
import { notify } from "./platform";
import { useAuth } from "@/stores/auth";

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
        if (
          lastEvent.new_severity === "critical" &&
          lastEvent.by !== user?.id
        ) {
          notify(
            "Critical incident",
            "An incident has been escalated to critical severity.",
          );
        }
        break;
      }
      case "release_state_changed": {
        const state = lastEvent.new_state as string;
        const labels: Record<string, string> = {
          // created: "A new release has been created.",
          in_progress: "A release has started.",
          completed: "A release has been completed.",
          cancelled: "A release has been cancelled.",
          blocked: "A release has been blocked by an active incident.",
        };
        const message = labels[state];
        if (message) {
          notify("Release " + state.replace("_", " "), message);
        }
        break;
      }
      case "private_message_received": {
        if (lastEvent.from !== user?.id) {
          notify(
            "New message",
            (lastEvent.content as string) || "You received a private message.",
          );
        }
        break;
      }
      case "timeline_entry_added": {
        if (lastEvent.author_id !== user?.id) {
          notify(
            "New Timeline Entry",
            (lastEvent.content as string) || "A new entry was added.",
          );
        }
        break;
      }
      case "member_role_changed": {
        if (lastEvent.user_id === user?.id) {
          const role = lastEvent.new_role as string;
          const labels: Record<string, string> = {
            manager: "You are now Manager of this team.",
            responder: "You have been promoted to Responder.",
            observer: "Your role has been changed to Observer.",
          };
          const message = labels[role];
          if (message) {
            notify("Role updated", message);
          }
        }
        break;
      }
      case "rule_triggered": {
        notify("Rule triggered", `${lastEvent.rule_name}`);
        break;
      }
      case "rule_failed": {
        notify("Rule failed", `${lastEvent.rule_name}: ${lastEvent.error}`);
        break;
      }
    }
  }, [lastEvent, user?.id]);
}
