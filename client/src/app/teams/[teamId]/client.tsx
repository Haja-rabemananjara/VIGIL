"use client";

import { useEffect } from "react";
import { useParams, useRouter } from "next/navigation";

export function TeamClient() {
  const { teamId } = useParams<{ teamId: string }>();
  const router = useRouter();

  useEffect(() => {
    if (teamId) {
      router.replace(`/teams/${teamId}/incidents`);
    }
  }, [teamId, router]);

  return null;
}