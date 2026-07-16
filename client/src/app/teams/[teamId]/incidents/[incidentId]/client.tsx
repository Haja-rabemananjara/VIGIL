"use client";

import { useEffect, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/stores/auth";
import { api, ApiError } from "@/lib/api";
import { t } from "@/lib/i18n";
import { StateBadge, type IncidentState } from "@/components/StateBadge";
import { SeverityBadge, type Severity } from "@/components/SeverityBadge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useVigilSocket } from "@/stores/socket";
import { useRouteParams } from "@/lib/useRouteParams";

interface Incident {
  id: string;
  title: string;
  body: string;
  status: IncidentState;
  severity: Severity;
  created_by: string;
  created_at: number;
  acknowledged_at: number | null;
  escalated_at: number | null;
  resolved_at: number | null;
  assignee_id: string | null;
}

interface TimelineEntry {
  id: string;
  author_id: string;
  kind: "message" | "system";
  content: string;
  created_at: number;
  edited_at: number | null;
}

interface Member {
  user_id: string;
  display_name: string;
  role: string;
}

// HELPERS
function formatDate(ts: number): string {
  return new Date(ts * 1000).toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

// Which transitions are available from a given state
const NEXT_TRANSITIONS: Record<IncidentState, IncidentState[]> = {
  open: ["acknowledged"],
  acknowledged: ["escalated", "resolved"],
  escalated: ["resolved"],
  resolved: [],
};

// Label for each transition button
const TRANSITION_LABELS: Record<IncidentState, string> = {
  acknowledged: "incidents.detail.acknowledge",
  escalated: "incidents.detail.escalate",
  resolved: "incidents.detail.resolve",
  open: "incidents.detail.reopen",
};

// COMPONENTS

export function IncidentDetailClient() {
  const { teamId, incidentId } = useRouteParams();
  const { token, user } = useAuth();
  const router = useRouter();

  // Data
  const [incident, setIncident] = useState<Incident | null>(null);
  const [timeline, setTimeline] = useState<TimelineEntry[]>([]);
  const [members, setMembers] = useState<Member[]>([]);
  const [role, setRole] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  // Timeline composer
  const [composerText, setComposerText] = useState("");
  const [composerLoading, setComposerLoading] = useState(false);
  const timelineEndRef = useRef<HTMLDivElement>(null);

  // Assign dialog
  const [assignOpen, setAssignOpen] = useState(false);
  const [assignLoading, setAssignLoading] = useState(false);
  const [assignError, setAssignError] = useState("");

  // Transition loading state
  const [transitionLoading, setTransitionLoading] = useState(false);

  const { lastEvent, reconnectCount, send } = useVigilSocket();
  const [watchers, setWatchers] = useState<string[]>([]);
  const [assignee, setAssignee] = useState<string | null>(null);

  // Fetch everything
  useEffect(() => {
    if (!token || !user) return;
    Promise.all([
      api<Incident>(`/teams/${teamId}/incidents/${incidentId}`, { token }),
      api<{ entries: TimelineEntry[] }>(
        `/teams/${teamId}/incidents/${incidentId}/timeline`,
        { token },
      ),
      api<Member[]>(`/teams/${teamId}/members`, { token }),
    ])
      .then(([inc, tl, mem]) => {
        setIncident(inc);
        setAssignee(inc.assignee_id);
        setTimeline(tl.entries);
        setMembers(mem);
        const me = mem.find((m) => m.user_id === user.id);
        setRole(me?.role ?? null);
      })
      .catch(() => setError(t("common.error")))
      .finally(() => setLoading(false));
  }, [token, teamId, incidentId, user]);

  // Scroll timeline to bottom on new entries
  useEffect(() => {
    timelineEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [timeline]);

  // Re-fetch everything after reconnection
  useEffect(() => {
    if (reconnectCount > 0 && token) {
      Promise.all([
        api<Incident>(`/teams/${teamId}/incidents/${incidentId}`, { token }),
        api<{ entries: TimelineEntry[] }>(
          `/teams/${teamId}/incidents/${incidentId}/timeline`,
          { token },
        ),
      ])
        .then(([inc, tl]) => {
          setIncident(inc);
          setAssignee(inc.assignee_id);
          setTimeline(tl.entries);
        })
        .catch(() => {});
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [reconnectCount]);

  // React to real-time events for this incident
  useEffect(() => {
    if (!lastEvent) return;

    const eventIncidentId = lastEvent.incident_id as string | undefined;

    // Presence doesn't need the incident to be loaded
    if (lastEvent.type === "presence_update") {
      const eventResourceId = lastEvent.resource_id as string;
      if (
        lastEvent.resource_type === "incident" &&
        eventResourceId === incidentId
      ) {
        // eslint-disable-next-line react-hooks/set-state-in-effect
        setWatchers(lastEvent.watchers as string[]);
      }
      return;
    }

    if (lastEvent.type === "member_role_changed") {
      const changedUserId = lastEvent.user_id as string;
      const newRole = lastEvent.new_role as string;
      if (changedUserId === user?.id) {
        setRole(newRole);
      }
      // Also update the members list (for assign dialog display names)
      setMembers((prev) =>
        prev.map((m) =>
          m.user_id === changedUserId ? { ...m, role: newRole } : m,
        ),
      );
      return;
    }

    if (lastEvent.type === "incident_assigned") {
      const eventIncidentId = lastEvent.incident_id as string;
      if (eventIncidentId === incidentId) {
        setAssignee(lastEvent.assigned_to as string);
      }
      return;
    }

    // All other events need the incident loaded
    if (!incident) return;
    if (eventIncidentId !== incidentId) return;

    switch (lastEvent.type) {
      case "incident_state_changed": {
        setIncident((prev) =>
          prev
            ? { ...prev, status: lastEvent.new_state as IncidentState }
            : prev,
        );
        const actor = lastEvent.by as string;
        if (actor !== user?.id && token) {
          api<{ entries: TimelineEntry[] }>(
            `/teams/${teamId}/incidents/${incidentId}/timeline`,
            { token },
          )
            .then((tl) => setTimeline(tl.entries))
            .catch(() => {});
        }
        break;
      }
      case "incident_escalated": {
        setIncident((prev) =>
          prev
            ? { ...prev, severity: lastEvent.new_severity as Severity }
            : prev,
        );
        break;
      }
      case "timeline_entry_added": {
        const newEntry: TimelineEntry = {
          id: lastEvent.entry_id as string,
          author_id: lastEvent.author_id as string,
          kind: "message",
          content: lastEvent.content as string,
          created_at: lastEvent.at as number,
          edited_at: null,
        };
        setTimeline((prev) => {
          // Avoid duplicates (the author already added it optimistically or via refetch)
          if (prev.some((e) => e.id === newEntry.id)) return prev;
          return [...prev, newEntry];
        });
        break;
      }
      default:
        break;
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [lastEvent, incidentId, teamId, token]);

  // Presence: tell the server we're watching this incident
  useEffect(() => {
    if (!teamId || !incidentId) return;

    send({
      type: "watch",
      resource_type: "incident",
      resource_id: incidentId,
      team_id: teamId,
    });

    return () => {
      send({
        type: "unwatch",
        resource_type: "incident",
        resource_id: incidentId,
        team_id: teamId,
      });
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [teamId, incidentId]);

  // Transition
  async function handleTransition(toStatus: IncidentState) {
    if (!incident) return;
    setTransitionLoading(true);
    try {
      const updated = await api<Incident>(
        `/teams/${teamId}/incidents/${incidentId}/status`,
        {
          method: "PATCH",
          token,
          body: { status: toStatus },
        },
      );
      setIncident(updated);
      // Refresh timeline to show the new system entry
      const tl = await api<{ entries: TimelineEntry[] }>(
        `/teams/${teamId}/incidents/${incidentId}/timeline`,
        { token },
      );
      setTimeline(tl.entries);
    } catch {
      // Silent — could add a toast here later
    } finally {
      setTransitionLoading(false);
    }
  }

  // Display Name on the Incidents (creator and author)
  function displayName(userId: string): string {
    const member = members.find((m) => m.user_id === userId);
    return member?.display_name ?? userId;
  }

  // Assign
  async function handleAssign(userId: string) {
    setAssignLoading(true);
    setAssignError("");
    try {
      await api(`/teams/${teamId}/incidents/${incidentId}/assign`, {
        method: "POST",
        token,
        body: { user_id: userId },
      });
      setAssignOpen(false);
      setAssignee(userId);
    } catch (e) {
      setAssignError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setAssignLoading(false);
    }
  }

  // Timeline post
  async function handlePost() {
    const content = composerText.trim();
    if (!content) return;
    setComposerLoading(true);
    try {
      const entry = await api<TimelineEntry>(
        `/teams/${teamId}/incidents/${incidentId}/timeline`,
        { method: "POST", token, body: { content } },
      );
      setTimeline((prev) => {
        if (prev.some((e) => e.id === entry.id)) return prev;
        return [...prev, entry];
      });
      setComposerText("");
    } catch {
      // silent
    } finally {
      setComposerLoading(false);
    }
  }

  // Eligible responders for assign dialog
  const eligibleMembers = members.filter(
    (m) => m.role === "responder" || m.role === "manager",
  );

  // Render

  if (loading) {
    return (
      <>
        <div className="p-6 text-muted-foreground">{t("common.loading")}</div>
      </>
    );
  }

  if (error || !incident) {
    return (
      <>
        <div className="p-6 text-destructive">{error || t("common.error")}</div>
      </>
    );
  }

  const nextTransitions = NEXT_TRANSITIONS[incident.status];
  const canAct = role === "responder" || role === "manager";
  const isManager = role === "manager";

  return (
    <>
      <div className="mx-auto max-w-3xl space-y-6 p-6">
        {/* Back (back to incidents) link */}
        <button
          onClick={() => router.push(`/teams/${teamId}/incidents`)}
          className="text-sm text-muted-foreground hover:text-foreground focus:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          {t("incidents.detail.backToList")}
        </button>

        {/* Watchers */}
        {watchers.length > 0 && (
          <div className="flex items-center gap-2">
            <span className="text-xs text-muted-foreground">
              {t("presence.watching")}
            </span>
            <div className="flex -space-x-2">
              {watchers.map((userId) => {
                const name = displayName(userId);
                const initials = name
                  .split(" ")
                  .map((w) => w[0])
                  .join("")
                  .slice(0, 2)
                  .toUpperCase();
                return (
                  <div
                    key={userId}
                    title={name}
                    className="flex h-7 w-7 items-center justify-center rounded-full border-2 border-background bg-primary text-[10px] font-medium text-primary-foreground"
                  >
                    {initials}
                  </div>
                );
              })}
            </div>
          </div>
        )}

        {/* Header card */}
        <Card>
          <CardHeader>
            <div className="flex items-start justify-between gap-4">
              <CardTitle className="text-xl">{incident.title}</CardTitle>
              <div className="flex shrink-0 items-center gap-2">
                <StateBadge state={incident.status} />
                <SeverityBadge severity={incident.severity} />
              </div>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            {incident.body && (
              <p className="text-sm text-muted-foreground">{incident.body}</p>
            )}

            <div className="text-sm text-muted-foreground">
              {t("incidents.detail.createdBy")}{" "}
              <span className="text-foreground">
                {displayName(incident.created_by)}
              </span>
              {" · "}
              {formatDate(incident.created_at)}
              {assignee && (
                <div className="text-sm text-muted-foreground">
                  {t("incidents.detail.assignee")}{" "}
                  <span className="text-foreground font-medium">
                    {displayName(assignee)}
                  </span>
                </div>
              )}
              {!assignee && (
                <div className="text-sm text-muted-foreground">
                  {t("incidents.detail.assignee")}{" "}
                  <span className="italic">
                    {t("incidents.detail.noAssignee")}
                  </span>
                </div>
              )}
            </div>

            {/* Actions (only visible to Responder+) */}
            {canAct && (
              <div className="flex items-center justify-between pt-2">
                <div className="flex flex-wrap gap-2">
                  {/* Transition buttons */}
                  {nextTransitions.map((toStatus) => (
                    <Button
                      key={toStatus}
                      size="sm"
                      variant={toStatus === "resolved" ? "default" : "outline"}
                      disabled={transitionLoading}
                      onClick={() => handleTransition(toStatus)}
                    >
                      {t(TRANSITION_LABELS[toStatus])}
                    </Button>
                  ))}
                </div>
                {/* Assign button (Manager only) */}
                {isManager && (
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => setAssignOpen(true)}
                  >
                    {t("incidents.detail.assign")}
                  </Button>
                )}
              </div>
            )}
          </CardContent>
        </Card>

        {/* Timeline */}
        <div className="space-y-3">
          <h2 className="text-lg font-medium">Timeline</h2>
          {timeline.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              {t("timeline.empty")}
            </p>
          ) : (
            <div className="space-y-2">
              {timeline.map((entry) => (
                <div
                  key={entry.id}
                  className={`rounded-lg border px-4 py-3 text-sm ${
                    entry.kind === "system"
                      ? "border-dashed bg-muted/30 text-muted-foreground"
                      : "bg-card"
                  }`}
                >
                  <div className="flex items-baseline justify-between gap-2">
                    <span className="font-medium">
                      {entry.kind === "system"
                        ? t("timeline.system")
                        : displayName(entry.author_id)}
                    </span>
                    <span className="text-xs text-muted-foreground">
                      {formatDate(entry.created_at)}
                      {entry.edited_at && (
                        <span className="ml-1 italic">
                          · {t("timeline.edited")}
                        </span>
                      )}
                    </span>
                  </div>
                  <p className="mt-1">{entry.content}</p>
                </div>
              ))}
              <div ref={timelineEndRef} />
            </div>
          )}

          {/* Composer (Responder+ only) */}
          {canAct && (
            <div className="flex gap-2 pt-2">
              <textarea
                value={composerText}
                onChange={(e) => setComposerText(e.target.value)}
                placeholder={t("timeline.composer.placeholder")}
                rows={2}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && e.ctrlKey && !composerLoading) {
                    handlePost();
                  }
                }}
                className="flex-1 rounded-md border bg-background px-3 py-2 text-sm focus:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              />
              <Button
                onClick={handlePost}
                disabled={composerLoading || !composerText.trim()}
              >
                {t("timeline.composer.submit")}
              </Button>
            </div>
          )}
        </div>
      </div>

      {/* Assign dialog */}
      {assignOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-full max-w-sm rounded-lg border bg-card p-6 shadow-lg space-y-4">
            <h3 className="font-semibold">
              {t("incidents.assign.dialogTitle")}
            </h3>
            <p className="text-sm text-muted-foreground">
              {t("incidents.assign.dialogDesc")}
            </p>
            {eligibleMembers.length === 0 ? (
              <p className="text-sm text-muted-foreground">
                {t("incidents.assign.noEligible")}
              </p>
            ) : (
              <div className="space-y-2">
                {eligibleMembers.map((m) => (
                  <button
                    key={m.user_id}
                    onClick={() => handleAssign(m.user_id)}
                    disabled={assignLoading}
                    className="w-full rounded-md border px-4 py-2 text-left text-sm hover:bg-muted focus:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                  >
                    {m.display_name}{" "}
                    <span className="text-muted-foreground">({m.role})</span>
                  </button>
                ))}
              </div>
            )}
            {assignError && (
              <p className="text-sm text-destructive">{assignError}</p>
            )}
            <div className="flex justify-end">
              <Button
                variant="outline"
                onClick={() => {
                  setAssignOpen(false);
                  setAssignError("");
                }}
              >
                {t("action.cancel")}
              </Button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
