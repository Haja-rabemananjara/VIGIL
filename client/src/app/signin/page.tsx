"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { useAuth } from "@/stores/auth";
import { ApiError } from "@/lib/api";
import { t } from "@/lib/i18n";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { postLoginDestination } from "@/lib/navigation";

export default function SigninPage() {
  const { signin } = useAuth();
  const router = useRouter();

  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setSubmitting(true);
    try {
      await signin(email, password);
      const token = localStorage.getItem("vigil_token") ?? "";
      const dest = await postLoginDestination(token);
      router.replace(dest);
    } catch (err) {
      if (err instanceof ApiError && err.status === 401) {
        setError(t("auth.error.invalidCredentials"));
      } else {
        setError(t("common.error"));
      }
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <main className="flex min-h-screen items-center justify-center p-4">
      <Card className="w-full max-w-sm">
        <CardHeader>
          <CardTitle>{t("auth.signin.title")}</CardTitle>
          <CardDescription>{t("app.name")}</CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="email">{t("auth.signin.email")}</Label>
              <Input
                id="email"
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
                autoComplete="email"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="password">{t("auth.signin.password")}</Label>
              <Input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
                autoComplete="current-password"
              />
            </div>

            {error && (
              <p className="text-sm text-destructive" role="alert">
                {error}
              </p>
            )}

            <Button type="submit" className="w-full" disabled={submitting}>
              {submitting ? t("common.loading") : t("auth.signin.submit")}
            </Button>
          </form>

          <p className="mt-4 text-center text-sm text-muted-foreground">
            <Link href="/signup" className="underline hover:text-foreground">
              {t("auth.signin.switchToSignup")}
            </Link>
          </p>
        </CardContent>
      </Card>
    </main>
  );
}
