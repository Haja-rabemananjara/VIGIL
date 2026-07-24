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
import {
  TimelineEntryCard,
  type TimelineEntry,
} from "@/components/TimelineEntryCard";

interface Incident {
  id: string;
  title: string;
  body: string;
  status: IncidentState;
  severity: Severity;
  created_by: string;
  created_at: number;
  assignee_id: string | null;
}

type Reactions = Record<string, Record<string, string[]>>;

interface Member {
  user_id: string;
  display_name: string;
  role: string;
}

function formatDate(ts: number): string {
  return new Date(ts * 1000).toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

const NEXT_TRANSITIONS: Record<IncidentState, IncidentState[]> = {
  open: ["acknowledged"],
  acknowledged: ["escalated", "resolved"],
  escalated: ["resolved"],
  resolved: [],
};

const TRANSITION_LABELS: Record<IncidentState, string> = {
  acknowledged: "incidents.detail.acknowledge",
  escalated: "incidents.detail.escalate",
  resolved: "incidents.detail.resolve",
  open: "incidents.detail.reopen",
};

export function IncidentDetailClient() {
  const { teamId, incidentId } = useRouteParams();
  const { token, user } = useAuth();
  const router = useRouter();

  const [incident, setIncident] = useState<Incident | null>(null);
  const [timeline, setTimeline] = useState<TimelineEntry[]>([]);
  const [members, setMembers] = useState<Member[]>([]);
  const [role, setRole] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  const [composerText, setComposerText] = useState("");
  const [composerLoading, setComposerLoading] = useState(false);
  const timelineEndRef = useRef<HTMLDivElement>(null);

  const [assignOpen, setAssignOpen] = useState(false);
  const [assignLoading, setAssignLoading] = useState(false);
  const [assignError, setAssignError] = useState("");
  const [transitionLoading, setTransitionLoading] = useState(false);

  const [reactions, setReactions] = useState<Reactions>({});
  const [availableEmojis, setAvailableEmojis] = useState<string[]>([]);

  const { lastEvent, reconnectCount, send } = useVigilSocket();
  const [watchers, setWatchers] = useState<string[]>([]);
  const [assignee, setAssignee] = useState<string | null>(null);

  useEffect(() => {
    if (!token || !user) return;
    Promise.all([
      api<Incident>(`/teams/${teamId}/incidents/${incidentId}`, { token }),
      api<{ entries: TimelineEntry[] }>(
        `/teams/${teamId}/incidents/${incidentId}/timeline`,
        { token },
      ),
      api<Member[]>(`/teams/${teamId}/members`, { token }),
      api<{ emojis: string[] }>(`/reactions/available`, { token }),
      api<{ reactions: Reactions }>(
        `/teams/${teamId}/incidents/${incidentId}/reactions`,
        { token },
      ),
    ])
      .then(([inc, tl, mem, emojiRes, reactionsRes]) => {
        setIncident(inc);
        setAssignee(inc.assignee_id);
        setTimeline(tl.entries);
        setMembers(mem);
        setAvailableEmojis(emojiRes.emojis);
        setReactions(reactionsRes.reactions);
        const me = mem.find((m) => m.user_id === user.id);
        setRole(me?.role ?? null);
      })
      .catch(() => setError(t("common.error")))
      .finally(() => setLoading(false));
  }, [token, teamId, incidentId, user]);

  useEffect(() => {
    timelineEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [timeline]);

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

  useEffect(() => {
    if (!lastEvent) return;

    const eventIncidentId = lastEvent.incident_id as string | undefined;

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
      if (changedUserId === user?.id) setRole(newRole);
      setMembers((prev) =>
        prev.map((m) =>
          m.user_id === changedUserId ? { ...m, role: newRole } : m,
        ),
      );
      return;
    }

    if (lastEvent.type === "incident_assigned") {
      if ((lastEvent.incident_id as string) === incidentId) {
        setAssignee(lastEvent.assigned_to as string);
      }
      return;
    }

    if (!incident) return;
    if (eventIncidentId !== incidentId) return;

    switch (lastEvent.type) {
      case "incident_state_changed": {
        setIncident((prev) =>
          prev
            ? { ...prev, status: lastEvent.new_state as IncidentState }
            : prev,
        );
        if ((lastEvent.by as string) !== user?.id && token) {
          api<{ entries: TimelineEntry[] }>(
            `/teams/${teamId}/incidents/${incidentId}/timeline`,
            { token },
          )
            .then((tl) => setTimeline(tl.entries))
            .catch(() => {});
        }
        break;
      }
      case "incident_escalated":
        setIncident((prev) =>
          prev
            ? { ...prev, severity: lastEvent.new_severity as Severity }
            : prev,
        );
        break;
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
          if (prev.some((e) => e.id === newEntry.id)) return prev;
          return [...prev, newEntry];
        });
        break;
      }
      case "timeline_entry_edited": {
        const entryId = lastEvent.entry_id as string;
        setTimeline((prev) =>
          prev.map((e) =>
            e.id === entryId
              ? {
                  ...e,
                  content: lastEvent.new_content as string,
                  edited_at: lastEvent.edited_at as number,
                }
              : e,
          ),
        );
        break;
      }
      case "reaction_added": {
        const entryId = lastEvent.entry_id as string;
        const emoji = lastEvent.emoji as string;
        const userId = lastEvent.user_id as string;
        setReactions((prev) => {
          const er = { ...(prev[entryId] || {}) };
          const users = [...(er[emoji] || [])];
          if (!users.includes(userId)) users.push(userId);
          er[emoji] = users;
          return { ...prev, [entryId]: er };
        });
        break;
      }
      case "reaction_removed": {
        const entryId = lastEvent.entry_id as string;
        const emoji = lastEvent.emoji as string;
        const userId = lastEvent.user_id as string;
        setReactions((prev) => {
          const er = { ...(prev[entryId] || {}) };
          const users = (er[emoji] || []).filter((id) => id !== userId);
          if (users.length === 0) {
            delete er[emoji];
          } else {
            er[emoji] = users;
          }
          return { ...prev, [entryId]: er };
        });
        break;
      }
      default:
        break;
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [lastEvent, incidentId, teamId, token]);

  // Presence
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

  async function handleTransition(toStatus: IncidentState) {
    if (!incident) return;
    setTransitionLoading(true);
    try {
      const updated = await api<Incident>(
        `/teams/${teamId}/incidents/${incidentId}/status`,
        { method: "PATCH", token, body: { status: toStatus } },
      );
      setIncident(updated);
      const tl = await api<{ entries: TimelineEntry[] }>(
        `/teams/${teamId}/incidents/${incidentId}/timeline`,
        { token },
      );
      setTimeline(tl.entries);
    } catch {
      // silent
    } finally {
      setTransitionLoading(false);
    }
  }

  function displayName(userId: string): string {
    const member = members.find((m) => m.user_id === userId);
    return member?.display_name ?? userId;
  }

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

  function handleEntryUpdated(updated: TimelineEntry) {
    setTimeline((prev) => prev.map((e) => (e.id === updated.id ? updated : e)));
  }

  async function handleToggleReaction(entryId: string, emoji: string) {
    const entryReactions = reactions[entryId] || {};
    const users = entryReactions[emoji] || [];
    const hasReacted = users.includes(user!.id);
    try {
      if (hasReacted) {
        await api(
          `/timeline/${entryId}/reactions/${encodeURIComponent(emoji)}`,
          { method: "DELETE", token },
        );
      } else {
        await api(`/timeline/${entryId}/reactions`, {
          method: "POST",
          token,
          body: { emoji },
        });
      }
    } catch {
      // silent
    }
  }

  const eligibleMembers = members.filter(
    (m) => m.role === "responder" || m.role === "manager",
  );

  if (loading) {
    return (
      <div className="p-6 text-muted-foreground">{t("common.loading")}</div>
    );
  }

  if (error || !incident) {
    return (
      <div className="p-6 text-destructive">{error || t("common.error")}</div>
    );
  }

  const nextTransitions = NEXT_TRANSITIONS[incident.status];
  const canAct = role === "responder" || role === "manager";
  const isManager = role === "manager";

  return (
    <>
      <div className="mx-auto max-w-3xl space-y-6 p-6">
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
              <div className="text-sm text-muted-foreground">
                {t("incidents.detail.assignee")}{" "}
                {assignee ? (
                  <span className="font-medium text-foreground">
                    {displayName(assignee)}
                  </span>
                ) : (
                  <span className="italic">
                    {t("incidents.detail.noAssignee")}
                  </span>
                )}
              </div>
            </div>

            {canAct && (
              <div className="flex items-center justify-between pt-2">
                <div className="flex flex-wrap gap-2">
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
                <TimelineEntryCard
                  key={entry.id}
                  entry={entry}
                  currentUserId={user?.id}
                  displayName={displayName}
                  entryReactions={reactions[entry.id] || {}}
                  availableEmojis={availableEmojis}
                  token={token}
                  onEntryUpdated={handleEntryUpdated}
                  onReactionToggle={handleToggleReaction}
                />
              ))}
              <div ref={timelineEndRef} />
            </div>
          )}

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
          <div className="w-full max-w-sm space-y-4 rounded-lg border bg-card p-6 shadow-lg">
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
