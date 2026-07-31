"use client";

import { useEffect, useState } from "react";
import { useAuth } from "@/stores/auth";
import { api, ApiError } from "@/lib/api";
import { t } from "@/lib/i18n";
import { Button } from "@/components/ui/button";
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
import type { Rule } from "./client";

/** Shape of /about.json, narrowed to what the form needs. */
interface AboutResponse {
  server: {
    services: {
      name: string;
      actions: { name: string; description: string }[];
      reactions: {
        name: string;
        description: string;
        payload_example: string;
      }[];
    }[];
  };
}

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  teamId: string | undefined;
  rule: Rule | null;
  onSaved: (rule: Rule) => void;
}

function isPristinePayload(value: string): boolean {
  const trimmed = value.trim();
  return trimmed === "" || trimmed === "{}";
}

/** Re-indent a catalog example so source-side formatting */
function formatJson(raw: string): string {
  try {
    return JSON.stringify(JSON.parse(raw), null, 2);
  } catch {
    return raw;
  }
}

export function RuleFormDialog({
  open,
  onOpenChange,
  teamId,
  rule,
  onSaved,
}: Props) {
  const { token } = useAuth();

  const [about, setAbout] = useState<AboutResponse | null>(null);
  const [name, setName] = useState("");
  const [service, setService] = useState("");
  const [event, setEvent] = useState("");
  const [reactionType, setReactionType] = useState("");
  const [filters, setFilters] = useState("{}");
  const [payload, setPayload] = useState("{}");
  const [enabled, setEnabled] = useState(true);
  const [error, setError] = useState("");
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!open) return;
    api<AboutResponse>("/about.json")
      .then(setAbout)
      .catch(() => setError(t("common.error")));
  }, [open]);

  useEffect(() => {
    if (!open) return;
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setError("");
    setName(rule?.name ?? "");
    setService(rule?.trigger_service ?? "");
    setEvent(rule?.trigger_event ?? "");
    setReactionType(rule?.reaction_type ?? "");
    setFilters(JSON.stringify(rule?.trigger_filters ?? {}, null, 2));
    setPayload(JSON.stringify(rule?.reaction_payload ?? {}, null, 2));
    setEnabled(rule?.enabled ?? true);
  }, [open, rule]);

  const triggerServices =
    about?.server.services.filter((s) => s.actions.length > 0) ?? [];
  const events = triggerServices.find((s) => s.name === service)?.actions ?? [];
  const reactions =
    about?.server.services.flatMap((s) =>
      s.reactions.map((r) => ({ ...r, service: s.name })),
    ) ?? [];
  const selectedReaction = reactions.find((r) => r.name === reactionType);

  async function handleSave() {
    if (!teamId) return;
    if (!name.trim()) return setError(t("rules.form.error.emptyName"));
    if (!service || !event) return setError(t("rules.form.error.noTrigger"));
    if (!reactionType) return setError(t("rules.form.error.noReaction"));

    let parsedFilters: unknown;
    let parsedPayload: unknown;
    try {
      parsedFilters = JSON.parse(filters || "{}");
    } catch {
      return setError(t("rules.form.error.badFilters"));
    }
    try {
      parsedPayload = JSON.parse(payload || "{}");
    } catch {
      return setError(t("rules.form.error.badPayload"));
    }

    setError("");
    setSaving(true);
    try {
      const body = {
        name: name.trim(),
        enabled,
        trigger: { service, event, filters: parsedFilters },
        reaction: { type: reactionType, payload: parsedPayload },
      };
      const saved = rule
        ? await api<Rule>(`/teams/${teamId}/rules/${rule.id}`, {
            method: "PATCH",
            token,
            body,
          })
        : await api<Rule>(`/teams/${teamId}/rules`, {
            method: "POST",
            token,
            body,
          });
      onSaved(saved);
      onOpenChange(false);
    } catch (e) {
      setError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>
            {rule ? t("rules.form.editTitle") : t("rules.form.createTitle")}
          </DialogTitle>
          <DialogDescription>{t("rules.form.desc")}</DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="rule-name">{t("rules.form.nameLabel")}</Label>
            <Input
              id="rule-name"
              value={name}
              placeholder={t("rules.form.namePlaceholder")}
              onChange={(e) => setName(e.target.value)}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="rule-service">{t("rules.form.serviceLabel")}</Label>
            <select
              id="rule-service"
              value={service}
              onChange={(e) => {
                setService(e.target.value);
                setEvent("");
              }}
              className="h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-xs focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
            >
              <option value="">...</option>
              {triggerServices.map((s) => (
                <option key={s.name} value={s.name}>
                  {s.name}
                </option>
              ))}
            </select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="rule-event">{t("rules.form.eventLabel")}</Label>
            <select
              id="rule-event"
              value={event}
              disabled={!service}
              onChange={(e) => setEvent(e.target.value)}
              className="h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-xs focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none disabled:opacity-50"
            >
              <option value="">...</option>
              {events.map((a) => (
                <option key={a.name} value={a.name}>
                  {a.name}
                </option>
              ))}
            </select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="rule-filters">{t("rules.form.filtersLabel")}</Label>
            <textarea
              id="rule-filters"
              value={filters}
              rows={4}
              onChange={(e) => setFilters(e.target.value)}
              className="w-full rounded-md border border-input bg-transparent px-3 py-2 font-mono text-xs shadow-xs focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
            />
            <p className="text-xs text-muted-foreground">
              {t("rules.form.filtersHelp")}
            </p>
          </div>

          <div className="space-y-2">
            <Label htmlFor="rule-reaction">
              {t("rules.form.reactionLabel")}
            </Label>
            <select
              id="rule-reaction"
              value={reactionType}
              onChange={(e) => {
                const kind = e.target.value;
                setReactionType(kind);
                const example = reactions.find(
                  (r) => r.name === kind,
                )?.payload_example;
                if (example && isPristinePayload(payload)) {
                  setPayload(formatJson(example));
                }
              }}
              className="h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-xs focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
            >
              <option value="">...</option>
              {reactions.map((r) => (
                <option key={r.name} value={r.name}>
                  {r.service} {r.name}
                </option>
              ))}
            </select>
            {selectedReaction && (
              <p className="text-xs text-muted-foreground">
                {selectedReaction.description}
              </p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="rule-payload">{t("rules.form.payloadLabel")}</Label>
            <textarea
              id="rule-payload"
              value={payload}
              rows={6}
              onChange={(e) => setPayload(e.target.value)}
              className="w-full rounded-md border border-input bg-transparent px-3 py-2 font-mono text-xs shadow-xs focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
            />
            <p className="text-xs text-muted-foreground">
              {t("rules.form.payloadHelp")}
            </p>
          </div>

          <div className="flex items-center gap-2">
            <input
              id="rule-enabled"
              type="checkbox"
              checked={enabled}
              onChange={(e) => setEnabled(e.target.checked)}
              className="h-4 w-4 rounded border-input"
            />
            <Label htmlFor="rule-enabled">{t("rules.form.enabledLabel")}</Label>
          </div>

          {error && <p className="text-sm text-destructive">{error}</p>}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t("action.cancel")}
          </Button>
          <Button onClick={handleSave} disabled={saving}>
            {saving ? "..." : t("rules.form.save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
