"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useRouteParams } from "@/lib/useRouteParams";

export function TeamClient() {
  const { teamId } = useRouteParams();
  const router = useRouter();

  useEffect(() => {
    if (teamId) {
      router.replace(`/teams/${teamId}/incidents`);
    }
  }, [teamId, router]);

  return null;
}
