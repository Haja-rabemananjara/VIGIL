"use client";

import { useEffect, useState } from "react";
import { useAuth } from "@/stores/auth";
import { api } from "@/lib/api";
import { t } from "@/lib/i18n";
import { getLanguage } from "@/lib/i18n";
import { useRouteParams } from "@/lib/useRouteParams";

interface AuditEntry {
  id: string;
  actor_id: string | null;
  actor_name: string | null;
  action: string;
  entity_type: string;
  entity_id: string;
  metadata: Record<string, unknown>;
  created_at: string;
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString(getLanguage(), {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function AuditClient() {
  const { token } = useAuth();
  const { teamId } = useRouteParams();
  const [entries, setEntries] = useState<AuditEntry[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!token || !teamId) return;
    api<AuditEntry[]>(`/teams/${teamId}/audit?limit=100`, { token })
      .then(setEntries)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, [token, teamId]);

  if (loading) {
    return (
      <div className="p-6 text-muted-foreground">{t("common.loading")}</div>
    );
  }

  return (
    <div className="mx-auto max-w-4xl space-y-4 p-6">
      <h1 className="text-2xl font-semibold">{t("audit.title")}</h1>

      {entries.length === 0 ? (
        <p className="text-muted-foreground">{t("audit.empty")}</p>
      ) : (
        <div className="rounded-md border">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b bg-muted/50 text-left">
                <th className="px-4 py-2 font-medium">{t("audit.action")}</th>
                <th className="px-4 py-2 font-medium">{t("audit.entity")}</th>
                <th className="px-4 py-2 font-medium">{t("audit.actor")}</th>
                <th className="px-4 py-2 font-medium">{t("audit.date")}</th>
              </tr>
            </thead>
            <tbody>
              {entries.map((e) => {
                const target =
                  (e.metadata.target_name as string) ??
                  (e.metadata.name as string) ??
                  (e.metadata.title as string) ??
                  "";

                return (
                  <tr key={e.id} className="border-b last:border-0">
                    <td className="px-4 py-2">
                      {e.action.replaceAll("_", " ")}
                    </td>
                    <td className="px-4 py-2">{target || e.entity_type}</td>
                    <td className="px-4 py-2 text-muted-foreground">
                      {e.actor_name ?? "system"}
                    </td>
                    <td className="px-4 py-2 text-muted-foreground">
                      {formatDate(e.created_at)}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
