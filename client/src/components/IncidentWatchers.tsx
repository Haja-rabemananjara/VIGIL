"use client";

import { t } from "@/lib/i18n";

interface Props {
  watchers: string[];
  displayName: (userId: string) => string;
}

export function IncidentWatchers({ watchers, displayName }: Props) {
  if (watchers.length === 0) return null;

  return (
    <div className="flex items-center gap-2">
      <span className="text-xs text-muted-foreground">
        {t("presence.watching")}
      </span>
      <div className="flex -space-x-2">
        {watchers.map((userId) => {
          const name = displayName(userId);
          const initials = name
            .split(" ")
            .map((w) => w[0])
            .join("")
            .slice(0, 2)
            .toUpperCase();
          return (
            <div
              key={userId}
              title={name}
              className="flex h-7 w-7 items-center justify-center rounded-full border-2 border-background bg-primary text-[10px] font-medium text-primary-foreground"
            >
              {initials}
            </div>
          );
        })}
      </div>
    </div>
  );
}
