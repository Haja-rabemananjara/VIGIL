"use client";

import { useEffect, useState } from "react";
import { useAuth } from "@/stores/auth";
import { api, ApiError } from "@/lib/api";
import { t } from "@/lib/i18n";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ConfirmDialog } from "@/components/ConfirmDialog";

/** Subset of /about.json */
interface AboutResponse {
  server: {
    services: {
      name: string;
      connectable: boolean;
    }[];
  };
}

interface ServiceConnection {
  id: string;
  service: string;
  created_at: number;
  updated_at: number;
}

export function ServicesClient() {
  const { token } = useAuth();

  const [connectable, setConnectable] = useState<string[]>([]);
  const [connections, setConnections] = useState<ServiceConnection[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [actionError, setActionError] = useState("");

  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [pending, setPending] = useState<string | null>(null);
  const [disconnectTarget, setDisconnectTarget] = useState<string | null>(null);

  useEffect(() => {
    if (!token) return;
    let cancelled = false;

    Promise.all([
      api<AboutResponse>("/about.json"),
      api<ServiceConnection[]>("/me/services", { token }),
    ])
      .then(([about, mine]) => {
        if (cancelled) return;
        setConnectable(
          about.server.services.filter((s) => s.connectable).map((s) => s.name),
        );
        setConnections(mine);
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
  }, [token]);

  async function handleConnect(service: string) {
    const draft = (drafts[service] ?? "").trim();
    if (!draft) {
      setActionError(t("services.error.emptyToken"));
      return;
    }
    setActionError("");
    setPending(service);
    try {
      const created = await api<ServiceConnection>(`/me/services/${service}`, {
        method: "POST",
        token,
        body: { token: draft },
      });
      setConnections((prev) => [
        ...prev.filter((c) => c.service !== service),
        created,
      ]);
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
      await api(`/me/services/${service}`, { method: "DELETE", token });
      setConnections((prev) => prev.filter((c) => c.service !== service));
    } catch (e) {
      setActionError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setDisconnectTarget(null);
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
      <div className="mx-auto max-w-2xl space-y-4 p-6">
        <div>
          <h1 className="text-2xl font-semibold">{t("services.title")}</h1>
          <p className="text-sm text-muted-foreground">
            {t("services.subtitle")}
          </p>
        </div>

        {actionError && (
          <p className="text-sm text-destructive">{actionError}</p>
        )}

        {connectable.length === 0 && (
          <p className="text-muted-foreground">{t("services.empty")}</p>
        )}

        {connectable.map((service) => {
          const connection = connections.find((c) => c.service === service);
          const inputId = `token-${service}`;

          return (
            <Card key={service}>
              <CardHeader className="flex flex-row items-center justify-between">
                <CardTitle className="capitalize">{service}</CardTitle>
                <span
                  className={
                    connection
                      ? "text-sm text-muted-foreground"
                      : "text-sm text-muted-foreground"
                  }
                >
                  {connection
                    ? t("services.connected")
                    : t("services.notConnected")}
                </span>
              </CardHeader>
              <CardContent>
                {connection ? (
                  <div className="flex items-center justify-between gap-4">
                    <p className="text-sm text-muted-foreground">
                      {t("services.connected")}{" "}
                      {new Date(connection.created_at * 1000).toLocaleString()}
                    </p>
                    <Button
                      variant="destructive"
                      onClick={() => setDisconnectTarget(service)}
                    >
                      {t("services.disconnect")}
                    </Button>
                  </div>
                ) : (
                  <div className="space-y-2">
                    <Label htmlFor={inputId}>{t("services.tokenLabel")}</Label>
                    <div className="flex gap-2">
                      <Input
                        id={inputId}
                        type="password"
                        value={drafts[service] ?? ""}
                        placeholder={t("services.tokenPlaceholder")}
                        onChange={(e) =>
                          setDrafts((prev) => ({
                            ...prev,
                            [service]: e.target.value,
                          }))
                        }
                      />
                      <Button
                        onClick={() => handleConnect(service)}
                        disabled={pending === service}
                      >
                        {pending === service ? "…" : t("services.connect")}
                      </Button>
                    </div>
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
        title={t("services.disconnect.title").replace(
          "{service}",
          disconnectTarget ?? "",
        )}
        description={t("services.disconnect.desc").replace(
          "{service}",
          disconnectTarget ?? "",
        )}
        confirmLabel={t("services.disconnect")}
        destructive
        onConfirm={handleDisconnect}
      />
    </>
  );
}
