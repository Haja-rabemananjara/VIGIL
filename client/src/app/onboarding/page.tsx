"use client";

import { RequireAuth } from "@/components/RequireAuth";
import { useAuth } from "@/stores/auth";
import { t } from "@/lib/i18n";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";

export default function OnboardingPage() {
  const { user } = useAuth();

  return (
    <RequireAuth>
      <main className="flex min-h-screen items-center justify-center p-4">
        <div className="w-full max-w-md space-y-6">
          <div className="text-center">
            <h1 className="text-2xl font-semibold">
              {t("onboarding.welcome")}, {user?.display_name}
            </h1>
            <p className="mt-2 text-muted-foreground">
              {t("onboarding.subtitle")}
            </p>
          </div>

          <div className="grid gap-4">
            <Card>
              <CardHeader>
                <CardTitle>{t("onboarding.create.title")}</CardTitle>
                <CardDescription>{t("onboarding.create.desc")}</CardDescription>
              </CardHeader>
              <CardContent>
                <Button className="w-full" disabled>
                  {t("onboarding.create.action")}
                </Button>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>{t("onboarding.join.title")}</CardTitle>
                <CardDescription>{t("onboarding.join.desc")}</CardDescription>
              </CardHeader>
              <CardContent>
                <Button className="w-full" variant="outline" disabled>
                  {t("onboarding.join.action")}
                </Button>
              </CardContent>
            </Card>
          </div>
        </div>
      </main>
    </RequireAuth>
  );
}