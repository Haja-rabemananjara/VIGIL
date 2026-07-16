"use client";

import { useEffect, useState } from "react";
import { useAuth } from "@/stores/auth";
import { api, ApiError } from "@/lib/api";
import { t } from "@/lib/i18n";
import { useRouteParams } from "@/lib/useRouteParams";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { RuleFormDialog } from "./RuleFormDialog";
import { useVigilSocket } from "@/stores/socket";

export interface Rule {
  id: string;
  team_id: string;
  name: string;
  enabled: boolean;
  trigger_service: string;
  trigger_event: string;
  trigger_filters: Record<string, unknown>;
  reaction_type: string;
  reaction_payload: Record<string, unknown>;
  created_by: string;
  created_at: number;
  updated_at: number;
}

interface Member {
  user_id: string;
  role: string;
}

interface Execution {
    key: string;
    ruleName: string;
    reactionType: string
    error: string | null;
    at: number;
}

export function RulesClient() {
  const { teamId } = useRouteParams();
  const { token, user } = useAuth();

  const [rules, setRules] = useState<Rule[]>([]);
  const [isManager, setIsManager] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [actionError, setActionError] = useState("");
  const [deleteTarget, setDeleteTarget] = useState<Rule | null>(null);
  const [formOpen, setFormOpen] = useState(false);
  const [editTarget, setEditTarget] = useState<Rule | null>(null);
  const [executions, setExecutions] = useState<Execution[]>([]);
  const { lastEvent } = useVigilSocket();

  useEffect(() => {
    if (!token) return;
    let cancelled = false;

    Promise.all([
      api<Rule[]>(`/teams/${teamId}/rules`, { token }),
      api<Member[]>(`/teams/${teamId}/members`, { token }),
    ])
      .then(([fetchedRules, members]) => {
        if (cancelled) return;
        setRules(fetchedRules);
        setIsManager(
          members.find((m) => m.user_id === user?.id)?.role === "manager",
        );
      })
      .catch(() => {
        if (!cancelled) setError(t("common.error"));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [token, teamId, user?.id]);

    useEffect(() => {
    if (!lastEvent) return;
    if (lastEvent.type !== "rule_triggered" && lastEvent.type !== "rule_failed") {
      return;
    }
    if (lastEvent.team_id !== teamId) return;

    // The event carries everything we display, so no refetch here.
    const entry: Execution = {
      key: `${lastEvent.rule_id as string}-${Date.now()}`,
      ruleName: lastEvent.rule_name as string,
      reactionType: lastEvent.reaction_type as string,
      error: lastEvent.type === "rule_failed" ? (lastEvent.error as string) : null,
      at: Date.now(),
    };
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setExecutions((prev) => [entry, ...prev].slice(0, 20));
  }, [lastEvent, teamId]);

  function handleSaved(saved: Rule) {
    setRules((prev) => {
      const exists = prev.some((r) => r.id === saved.id);
      return exists
        ? prev.map((r) => (r.id === saved.id ? saved : r))
        : [saved, ...prev];
    });
  }

  async function handleToggle(rule: Rule) {
    setActionError("");
    try {
      const updated = await api<Rule>(`/teams/${teamId}/rules/${rule.id}`, {
        method: "PATCH",
        token,
        body: { enabled: !rule.enabled },
      });
      setRules((prev) => prev.map((r) => (r.id === rule.id ? updated : r)));
    } catch (e) {
      setActionError(e instanceof ApiError ? e.message : t("common.error"));
    }
  }

  async function handleDelete() {
    if (!deleteTarget) return;
    const id = deleteTarget.id;
    setActionError("");
    try {
      await api(`/teams/${teamId}/rules/${id}`, { method: "DELETE", token });
      setRules((prev) => prev.filter((r) => r.id !== id));
    } catch (e) {
      setActionError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setDeleteTarget(null);
    }
  }

  if (loading) {
    return (
      <div className="p-6 text-muted-foreground">{t("common.loading")}</div>
    );
  }

  if (error) {
    return <div className="p-6 text-destructive">{error}</div>;
  }

  return (
    <>
      <div className="space-y-4 p-6">
        <div className="flex items-start justify-between">
          <div>
            <h1 className="text-2xl font-semibold">{t("rules.title")}</h1>
            <p className="text-sm text-muted-foreground">
              {t("rules.subtitle")}
            </p>
          </div>
          {isManager && (
            <Button
              onClick={() => {
                setEditTarget(null);
                setFormOpen(true);
              }}
            >
              {t("rules.new")}
            </Button>
          )}
        </div>

        {actionError && (
          <p className="text-sm text-destructive">{actionError}</p>
        )}

        {rules.length === 0 ? (
          <p className="text-muted-foreground">{t("rules.empty")}</p>
        ) : (
          <div className="space-y-2">
            {rules.map((rule) => (
              <Card key={rule.id}>
                <CardContent className="flex items-center justify-between gap-4 py-4">
                  <div className="min-w-0 space-y-1">
                    <div className="flex items-center gap-2">
                      <span className="font-medium">{rule.name}</span>
                      <span
                        className={
                          rule.enabled
                            ? "rounded-full bg-success/15 px-2 py-0.5 text-xs text-success"
                            : "rounded-full bg-muted px-2 py-0.5 text-xs text-muted-foreground"
                        }
                      >
                        {rule.enabled ? t("rules.enabled") : t("rules.disabled")}
                      </span>
                    </div>
                    <p className="truncate text-sm text-muted-foreground">
                      <span className="font-medium">{t("rules.trigger")}</span>{" "}
                      {rule.trigger_service}.{rule.trigger_event}
                      {" : "}
                      <span className="font-medium">{t("rules.reaction")}</span>{" "}
                      {rule.reaction_type}
                    </p>
                  </div>

                  {isManager && (
                    <div className="flex shrink-0 gap-2">
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => {
                          setEditTarget(rule);
                          setFormOpen(true);
                        }}
                      >
                        {t("rules.form.editTitle")}
                      </Button>

                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => handleToggle(rule)}
                      >
                        {rule.enabled ? t("rules.disabled") : t("rules.enabled")}
                      </Button>
                      <Button
                        size="sm"
                        variant="destructive"
                        onClick={() => setDeleteTarget(rule)}
                      >
                        {t("rules.delete.confirm")}
                      </Button>
                    </div>
                  )}
                </CardContent>
              </Card>
            ))}
          </div>
        )}

        {!isManager && (
          <p className="text-sm text-muted-foreground">
            {t("rules.managerOnly")}
          </p>
        )}

                <div className="space-y-2 pt-4">
          <h2 className="text-sm font-medium">{t("rules.activity.title")}</h2>
          {executions.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              {t("rules.activity.empty")}
            </p>
          ) : (
            <div className="space-y-1">
              {executions.map((exec) => (
                <div
                  key={exec.key}
                  className="flex items-start gap-2 rounded-md border px-3 py-2 text-sm"
                >
                  <span
                    className={
                      exec.error
                        ? "shrink-0 rounded-full bg-destructive/15 px-2 py-0.5 text-xs text-destructive"
                        : "shrink-0 rounded-full bg-success/15 px-2 py-0.5 text-xs text-success"
                    }
                  >
                    {exec.error
                      ? t("rules.activity.failed")
                      : t("rules.activity.ok")}
                  </span>
                  <div className="min-w-0">
                    <p className="truncate">
                      <span className="font-medium">{exec.ruleName}</span>{" "}
                      <span className="text-muted-foreground">
                        {exec.reactionType}
                      </span>
                    </p>
                    {exec.error && (
                      <p className="truncate text-xs text-destructive">
                        {exec.error}
                      </p>
                    )}
                  </div>
                  <span className="ml-auto shrink-0 text-xs text-muted-foreground">
                    {new Date(exec.at).toLocaleTimeString()}
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      <RuleFormDialog
        open={formOpen}
        onOpenChange={setFormOpen}
        teamId={teamId}
        rule={editTarget}
        onSaved={handleSaved}
      />
      <ConfirmDialog
        open={!!deleteTarget}
        onOpenChange={(open) => {
          if (!open) setDeleteTarget(null);
        }}
        title={t("rules.delete.title")}
        description={t("rules.delete.desc").replace(
          "{name}",
          deleteTarget?.name ?? "",
        )}
        confirmLabel={t("rules.delete.confirm")}
        destructive
        onConfirm={handleDelete}
      />
    </>
  );
}