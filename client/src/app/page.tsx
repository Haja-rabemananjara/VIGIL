"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { RequireAuth } from "@/components/RequireAuth";
import { postLoginDestination } from "@/lib/navigation";
import { t } from "@/lib/i18n";

export default function Home() {
  const router = useRouter();

  useEffect(() => {
    const token = localStorage.getItem("vigil_token");
    if (!token) return;
    postLoginDestination(token).then((dest) => router.replace(dest));
  }, [router]);

  return (
    <RequireAuth>
      <div className="flex h-screen items-center justify-center">
        <p className="text-muted-foreground">{t("common.loading")}</p>
      </div>
    </RequireAuth>
  );
}
