"use client";

import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import { setLanguage } from "@/lib/i18n";
import { t } from "@/lib/i18n";
import { postLoginDestination } from "@/lib/navigation";

interface OAuthResponse {
  token: string;
  user: {
    id: string;
    email: string;
    display_name: string;
    language: string;
    avatar_seed: string | null;
    created_at: number;
  };
}

export default function GitHubCallbackPage() {
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const code = params.get("code");

    if (!code) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setError("Missing authorization code");
      return;
    }

    api<OAuthResponse>(`/auth/oauth/github/callback?code=${code}&state=ok`)
      .then(async (res) => {
        localStorage.setItem("vigil_token", res.token);
        const stored = localStorage.getItem("vigil_language");
        if (!stored) {
          setLanguage(res.user.language as "en" | "fr");
        }
        const dest = await postLoginDestination(res.token);
        window.location.href = dest;
      })
      .catch(() => {
        setError(t("common.error"));
      });
  }, []);

  if (error) {
    return (
      <main className="flex min-h-screen items-center justify-center p-4">
        <p className="text-destructive">{error}</p>
      </main>
    );
  }

  return (
    <main className="flex min-h-screen items-center justify-center p-4">
      <p className="text-muted-foreground">{t("common.loading")}</p>
    </main>
  );
}
