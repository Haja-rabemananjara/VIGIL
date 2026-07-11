"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/stores/auth";
import { api, ApiError } from "@/lib/api";
import { t } from "@/lib/i18n";
import {
  ReleaseStateBadge,
  type ReleaseState,
} from "@/components/ReleaseStateBadge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useVigilSocket } from "@/stores/socket";
import { useRouteParams } from "@/lib/useRouteParams";

interface ReleaseRow {
  id: string;
  title: string;
  status: ReleaseState;
  created_by: string;
  created_at: number;
  updated_at: number;
  progress: { completed: number; total: number };
}

const STATUS_OPTIONS: ReleaseState[] = [
  "created",
  "in_progress",
  "completed",
  "cancelled",
  "blocked",
];

function formatDate(ts: number): string {
  return new Date(ts * 1000).toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function ReleasesClient() {
  const { teamId } = useRouteParams();
  const { token, user } = useAuth();
  const router = useRouter();

  const [releases, setReleases] = useState<ReleaseRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [role, setRole] = useState<string | null>(null);
  const [statusFilter, setStatusFilter] = useState<string>("");

  // Create dialog
  const [createOpen, setCreateOpen] = useState(false);
  const [createTitle, setCreateTitle] = useState("");
  const [createBody, setCreateBody] = useState("");
  const [stepInputs, setStepInputs] = useState<string[]>(["build", "staging", "production"]);
  const [createError, setCreateError] = useState("");
  const [createLoading, setCreateLoading] = useState(false);

  const { lastEvent, reconnectCount } = useVigilSocket();

  // Fetch releases
  async function fetchReleases() {
    if (!token) return;
    setLoading(true);
    setError("");
    try {
      const query = statusFilter ? `?status=${statusFilter}` : "";
      const data = await api<ReleaseRow[]>(
        `/teams/${teamId}/releases${query}`,
        { token },
      );
      setReleases(data);
    } catch {
      setError(t("common.error"));
    } finally {
      setLoading(false);
    }
  }

  // Fetch role
  useEffect(() => {
    if (!token || !user) return;
    api<{ user_id: string; role: string }[]>(
      `/teams/${teamId}/members`,
      { token },
    )
      .then((members) => {
        const me = members.find((m) => m.user_id === user.id);
        setRole(me?.role ?? null);
      })
      .catch(() => {});
  }, [token, teamId, user]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    fetchReleases();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [token, teamId, statusFilter]);

  // Re-fetch after WS reconnect
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    if (reconnectCount > 0) fetchReleases();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [reconnectCount]);

  // Real-time: release state changes
  useEffect(() => {
    if (!lastEvent) return;
    if (lastEvent.team_id !== teamId) return;

    if (lastEvent.type === "release_state_changed") {
      const eventId = lastEvent.release_id as string;
      const newState = lastEvent.new_state as string;

      // eslint-disable-next-line react-hooks/set-state-in-effect
      setReleases((prev) => {
        const exists = prev.some((r) => r.id === eventId);
        if (exists) {
          return prev.map((r) =>
            r.id === eventId ? { ...r, status: newState as ReleaseState } : r,
          );
        }
        return prev;
      });

      // New release created: fetch full data (same pattern as incidents)
      if (newState === "created") {
        api<ReleaseRow[]>(
          `/teams/${teamId}/releases`,
          { token: token! },
        )
          .then((data) => setReleases(data))
          .catch(() => {});
      }
    }

    if (
      lastEvent.type === "release_incident_linked" ||
      lastEvent.type === "release_incident_unlinked"
    ) {
      const affectedReleaseId = lastEvent.release_id as string;
      // Refetch la release concernée pour récupérer sa liste d'incidents à jour
      api<ReleaseRow>(
        `/teams/${teamId}/releases/${affectedReleaseId}`,
        { token: token! },
      )
        .then((updated) => {
          setReleases((prev) =>
            prev.map((r) => (r.id === affectedReleaseId ? updated : r)),
          );
        })
        .catch(() => {});
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [lastEvent, teamId]);

  // Create dialog handlers
  function handleCreateOpenChange(next: boolean) {
    setCreateOpen(next);
    if (!next) {
      setCreateTitle("");
      setCreateBody("");
      setStepInputs(["build", "staging", "production"]);
      setCreateError("");
    }
  }

  function handleStepChange(idx: number, value: string) {
    setStepInputs((prev) => prev.map((s, i) => (i === idx ? value : s)));
  }

  function handleAddStep() {
    setStepInputs((prev) => [...prev, ""]);
  }

  function handleRemoveStep(idx: number) {
    setStepInputs((prev) => prev.filter((_, i) => i !== idx));
  }

  async function handleCreate() {
    const trimmedTitle = createTitle.trim();
    if (!trimmedTitle) {
      setCreateError(t("releases.create.error.emptyTitle"));
      return;
    }
    const steps = stepInputs.map((s) => s.trim()).filter(Boolean);
    if (steps.length === 0) {
      setCreateError(t("releases.create.error.noSteps"));
      return;
    }
    setCreateLoading(true);
    setCreateError("");
    try {
      const release = await api<ReleaseRow>(`/teams/${teamId}/releases`, {
        method: "POST",
        token,
        body: { title: trimmedTitle, body: createBody, steps },
      });
      setReleases((prev) => [release, ...prev]);
      handleCreateOpenChange(false);
      router.push(`/teams/${teamId}/releases/${release.id}`);
    } catch (e) {
      setCreateError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setCreateLoading(false);
    }
  }

  return (
    <>
      <div className="space-y-4 p-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-semibold">{t("releases.title")}</h1>
          {role === "manager" && (
            <Button onClick={() => setCreateOpen(true)}>
              {t("releases.new")}
            </Button>
          )}
        </div>

        {/* Status filter */}
        <div className="flex items-center gap-2">
          <Label htmlFor="filter-status">{t("releases.filter.status")}</Label>
          <select
            id="filter-status"
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value)}
            className="rounded-md border bg-background px-2 py-1 text-sm"
          >
            <option value="">{t("releases.filter.all")}</option>
            {STATUS_OPTIONS.map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </select>
        </div>

        {/* Content */}
        {loading ? (
          <p className="text-muted-foreground">{t("common.loading")}</p>
        ) : error ? (
          <p className="text-destructive">{error}</p>
        ) : releases.length === 0 ? (
          <Card>
            <CardContent className="py-12 text-center text-muted-foreground">
              {t("releases.empty")}
            </CardContent>
          </Card>
        ) : (
          <div className="space-y-2">
            {releases.map((release) => (
              <button
                key={release.id}
                onClick={() =>
                  router.push(`/teams/${teamId}/releases/${release.id}`)
                }
                className="w-full rounded-lg border bg-card px-4 py-3 text-left transition-colors hover:bg-muted/50 focus:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                <div className="flex items-center justify-between gap-4">
                  <span className="flex-1 truncate font-medium">
                    {release.title}
                  </span>
                  <div className="flex shrink-0 items-center gap-2">
                    <ReleaseStateBadge state={release.status} />
                    <span className="text-xs text-muted-foreground">
                      {release.progress.completed}/{release.progress.total}
                    </span>
                    <span className="text-xs text-muted-foreground">
                      {formatDate(release.created_at)}
                    </span>
                  </div>
                </div>
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Create release dialog */}
      <Dialog open={createOpen} onOpenChange={handleCreateOpenChange}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("releases.create.dialogTitle")}</DialogTitle>
            <DialogDescription>
              {t("releases.create.dialogDesc")}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="rel-title">
                {t("releases.create.titleLabel")}
              </Label>
              <Input
                id="rel-title"
                value={createTitle}
                onChange={(e) => setCreateTitle(e.target.value)}
                placeholder={t("releases.create.titlePlaceholder")}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="rel-body">
                {t("releases.create.bodyLabel")}
              </Label>
              <textarea
                id="rel-body"
                value={createBody}
                onChange={(e) => setCreateBody(e.target.value)}
                placeholder={t("releases.create.bodyPlaceholder")}
                rows={2}
                className="w-full rounded-md border bg-background px-3 py-2 text-sm focus:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              />
            </div>
            <div className="space-y-2">
              <Label>{t("releases.create.stepsLabel")}</Label>
              {stepInputs.map((step, idx) => (
                <div key={idx} className="flex items-center gap-2">
                  <span className="text-xs text-muted-foreground w-5">
                    {idx + 1}.
                  </span>
                  <Input
                    value={step}
                    onChange={(e) => handleStepChange(idx, e.target.value)}
                    placeholder={t("releases.create.stepPlaceholder")}
                    className="flex-1"
                  />
                  {stepInputs.length > 1 && (
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleRemoveStep(idx)}
                      className="text-muted-foreground"
                    >
                      ×
                    </Button>
                  )}
                </div>
              ))}
              <Button
                variant="outline"
                size="sm"
                onClick={handleAddStep}
                className="w-full"
              >
                {t("releases.create.addStep")}
              </Button>
            </div>
            {createError && (
              <p className="text-sm text-destructive">{createError}</p>
            )}
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => handleCreateOpenChange(false)}
            >
              {t("releases.create.cancel")}
            </Button>
            <Button onClick={handleCreate} disabled={createLoading}>
              {createLoading ? "…" : t("releases.create.submit")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}