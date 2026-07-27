"use client";

import { useEffect, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/stores/auth";
import { api, ApiError } from "@/lib/api";
import { t } from "@/lib/i18n";
import { type IncidentState } from "@/components/StateBadge";
import { type Severity } from "@/components/SeverityBadge";
import { useVigilSocket } from "@/stores/socket";
import { useRouteParams } from "@/lib/useRouteParams";
import {
  TimelineEntryCard,
  type TimelineEntry,
} from "@/components/TimelineEntryCard";
import { IncidentWatchers } from "@/components/IncidentWatchers";
import { IncidentHeader } from "@/components/IncidentHeader";
import { AssignDialog } from "@/components/AssignDialog";
import { TimelineComposer } from "@/components/TimelineComposer";

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

const NEXT_TRANSITIONS: Record<IncidentState, IncidentState[]> = {
  open: ["acknowledged"],
  acknowledged: ["escalated", "resolved"],
  escalated: ["resolved"],
  resolved: [],
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

    if (lastEvent.type === "presence_update") {
      if (
        lastEvent.resource_type === "incident" &&
        (lastEvent.resource_id as string) === incidentId
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
    if ((lastEvent.incident_id as string | undefined) !== incidentId) return;

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
    return members.find((m) => m.user_id === userId)?.display_name ?? userId;
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
    const users = reactions[entryId]?.[emoji] || [];
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

  const canAct = role === "responder" || role === "manager";
  const isManager = role === "manager";
  const eligibleMembers = members.filter(
    (m) => m.role === "responder" || m.role === "manager",
  );

  return (
    <>
      <div className="mx-auto max-w-3xl space-y-6 p-6">
        <button
          onClick={() => router.push(`/teams/${teamId}/incidents`)}
          className="text-sm text-muted-foreground hover:text-foreground focus:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          {t("incidents.detail.backToList")}
        </button>

        <IncidentWatchers watchers={watchers} displayName={displayName} />

        <IncidentHeader
          incident={incident}
          assignee={assignee}
          displayName={displayName}
          canAct={canAct}
          isManager={isManager}
          nextTransitions={NEXT_TRANSITIONS[incident.status]}
          transitionLoading={transitionLoading}
          onTransition={handleTransition}
          onOpenAssign={() => setAssignOpen(true)}
        />

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
            <TimelineComposer
              value={composerText}
              loading={composerLoading}
              onChange={setComposerText}
              onSubmit={handlePost}
            />
          )}
        </div>
      </div>

      <AssignDialog
        open={assignOpen}
        eligibleMembers={eligibleMembers}
        loading={assignLoading}
        error={assignError}
        onAssign={handleAssign}
        onClose={() => {
          setAssignOpen(false);
          setAssignError("");
        }}
      />
    </>
  );
}
