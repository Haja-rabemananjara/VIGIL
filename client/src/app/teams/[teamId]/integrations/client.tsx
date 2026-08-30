"use client";

import { useEffect, useState } from "react";
import { useAuth } from "@/stores/auth";
import { api, ApiError } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { useRouteParams } from "@/lib/useRouteParams";
import { Copy, Check } from "lucide-react";
import { t, type TranslationKey } from "@/lib/i18n";

interface TeamConnection {
  id: string;
  team_id: string;
  service: string;
  created_by: string;
  created_at: number;
  updated_at: number;
}

interface ConnectResponse {
  connection: TeamConnection;
  webhook_url?: string;
}

const SERVICES: {
  name: string;
  label: string;
  tokenLabel: TranslationKey;
  tokenPlaceholder: TranslationKey;
  help: TranslationKey;
}[] = [
  {
    name: "github",
    label: "GitHub",
    tokenLabel: "integrations.github.secretLabel",
    tokenPlaceholder: "integrations.github.secretPlaceholder",
    help: "integrations.github.help",
  },
  {
    name: "discord",
    label: "Discord",
    tokenLabel: "integrations.discord.urlLabel",
    tokenPlaceholder: "integrations.discord.urlPlaceholder",
    help: "integrations.discord.help",
  },
];

export function IntegrationsClient() {
  const { token } = useAuth();
  const { teamId } = useRouteParams();

  const [connections, setConnections] = useState<TeamConnection[]>([]);
  const [webhookUrls, setWebhookUrls] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [actionError, setActionError] = useState("");

  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [pending, setPending] = useState<string | null>(null);
  const [disconnectTarget, setDisconnectTarget] = useState<string | null>(null);
  const [copied, setCopied] = useState<string | null>(null);

  useEffect(() => {
    if (!token || !teamId) return;
    let cancelled = false;

    api<TeamConnection[]>(`/teams/${teamId}/connections`, { token })
      .then((data) => {
        if (!cancelled) setConnections(data);
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
  }, [token, teamId]);

  async function handleConnect(service: string) {
    const draft = (drafts[service] ?? "").trim();
    if (!draft) {
      setActionError(t("integrations.error.empty"));
      return;
    }
    setActionError("");
    setPending(service);
    try {
      const result = await api<ConnectResponse>(
        `/teams/${teamId}/connections/${service}`,
        {
          method: "POST",
          token,
          body: { token: draft },
        },
      );
      setConnections((prev) => [
        ...prev.filter((c) => c.service !== service),
        result.connection,
      ]);
      if (result.webhook_url) {
        setWebhookUrls((prev) => ({ ...prev, [service]: result.webhook_url! }));
      }
      setDrafts((prev) => ({ ...prev, [service]: "" }));
    } catch (e) {
      setActionError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setPending(null);
    }
  }

  async function handleDisconnect() {
    if (!disconnectTarget) return;
    const service = disconnectTarget;
    setActionError("");
    try {
      await api(`/teams/${teamId}/connections/${service}`, {
        method: "DELETE",
        token,
      });
      setConnections((prev) => prev.filter((c) => c.service !== service));
      setWebhookUrls((prev) => {
        const next = { ...prev };
        delete next[service];
        return next;
      });
    } catch (e) {
      setActionError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setDisconnectTarget(null);
    }
  }

  function handleCopy(text: string, service: string) {
    navigator.clipboard.writeText(text).then(() => {
      setCopied(service);
      setTimeout(() => setCopied(null), 2000);
    });
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
      <div className="mx-auto max-w-2xl space-y-4 p-6">
        <div>
          <h1 className="text-2xl font-semibold">{t("integrations.title")}</h1>
          <p className="text-sm text-muted-foreground">
            {t("integrations.subtitle")}
          </p>
        </div>

        {actionError && (
          <p className="text-sm text-destructive">{actionError}</p>
        )}

        {SERVICES.map((svc) => {
          const connection = connections.find((c) => c.service === svc.name);
          const webhookUrl = webhookUrls[svc.name];
          const inputId = `token-${svc.name}`;

          return (
            <Card key={svc.name}>
              <CardHeader className="flex flex-row items-center justify-between">
                <CardTitle>{svc.label}</CardTitle>
                <span className="text-sm text-muted-foreground">
                  {connection
                    ? t("integrations.connected")
                    : t("integrations.notConnected")}
                </span>
              </CardHeader>
              <CardContent className="space-y-3">
                {connection ? (
                  <>
                    <p className="text-sm text-muted-foreground">
                      {t("integrations.connectedSince")}{" "}
                      {new Date(
                        connection.created_at * 1000,
                      ).toLocaleDateString()}
                    </p>

                    {webhookUrl && (
                      <div className="space-y-1">
                        <Label>{t("integrations.webhookUrl")}</Label>
                        <div className="flex gap-2">
                          <Input
                            value={webhookUrl}
                            readOnly
                            className="font-mono text-xs"
                          />
                          <Button
                            variant="outline"
                            size="icon"
                            onClick={() => handleCopy(webhookUrl, svc.name)}
                          >
                            {copied === svc.name ? (
                              <Check className="h-4 w-4" />
                            ) : (
                              <Copy className="h-4 w-4" />
                            )}
                          </Button>
                        </div>
                        <p className="text-xs text-muted-foreground">
                          {t(svc.help)}
                        </p>
                      </div>
                    )}

                    <div className="flex justify-end">
                      <Button
                        variant="destructive"
                        onClick={() => setDisconnectTarget(svc.name)}
                      >
                        {t("integrations.disconnect")}
                      </Button>
                    </div>
                  </>
                ) : (
                  <div className="space-y-2">
                    <Label htmlFor={inputId}>{t(svc.tokenLabel)}</Label>
                    <div className="flex gap-2">
                      <Input
                        id={inputId}
                        type="password"
                        value={drafts[svc.name] ?? ""}
                        placeholder={t(svc.tokenPlaceholder)}
                        onChange={(e) =>
                          setDrafts((prev) => ({
                            ...prev,
                            [svc.name]: e.target.value,
                          }))
                        }
                      />
                      <Button
                        onClick={() => handleConnect(svc.name)}
                        disabled={pending === svc.name}
                      >
                        {pending === svc.name
                          ? "..."
                          : t("integrations.connect")}
                      </Button>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {t(svc.help)}
                    </p>
                  </div>
                )}
              </CardContent>
            </Card>
          );
        })}
      </div>

      <ConfirmDialog
        open={!!disconnectTarget}
        onOpenChange={(open) => {
          if (!open) setDisconnectTarget(null);
        }}
        title={t("integrations.disconnect.title").replace(
          "{service}",
          disconnectTarget ?? "",
        )}
        description={t("integrations.disconnect.desc").replace(
          "{service}",
          disconnectTarget ?? "",
        )}
        confirmLabel={t("integrations.disconnect")}
        destructive
        onConfirm={handleDisconnect}
      />
    </>
  );
}
