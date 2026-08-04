"use client";

import { useMemo } from "react";
import { Style, Avatar as DiceBearAvatar } from "@dicebear/core";
import avataaars from "@dicebear/styles/avataaars.json";
import Image from "next/image";

const style = new Style(avataaars);

interface UserAvatarProps {
  seed: string | null;
  displayName: string;
  size?: number;
  className?: string;
}

function getInitials(name: string): string {
  return name
    .split(/\s+/)
    .map((p) => p[0])
    .slice(0, 2)
    .join("")
    .toUpperCase();
}

export function UserAvatar({
  seed,
  displayName,
  size = 36,
  className = "",
}: UserAvatarProps) {
  const dataUri = useMemo(() => {
    if (!seed) return null;
    return new DiceBearAvatar(style, { seed, size }).toDataUri();
  }, [seed, size]);

  if (dataUri) {
    return (
      <Image
        src={dataUri}
        alt={displayName}
        width={size}
        height={size}
        className={`rounded-full ${className}`}
      />
    );
  }

  return (
    <div
      className={`flex items-center justify-center rounded-full bg-muted font-medium text-muted-foreground ${className}`}
      style={{ width: size, height: size, fontSize: size * 0.4 }}
    >
      {getInitials(displayName)}
    </div>
  );
}
