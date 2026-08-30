"use client";

import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { t, TranslationKey } from "@/lib/i18n";
import { MessageCircle } from "lucide-react";
import { UserAvatar } from "@/components/UserAvatar";

export interface MemberView {
  user_id: string;
  display_name: string;
  email: string;
  role: string;
  avatar_seed: string | null;
  joined_at: string;
}

interface Props {
  member: MemberView;
  isMe: boolean;
  isManager: boolean;
  onPromote: (userId: string) => void;
  onDemote: (userId: string) => void;
  onTransfer: (member: MemberView) => void;
  onKick: (member: MemberView) => void;
  onBan: (member: MemberView) => void;
}

export function MemberRow({
  member,
  isMe,
  isManager,
  onPromote,
  onDemote,
  onTransfer,
  onKick,
  onBan,
}: Props) {
  const router = useRouter();
  const isMemberManager = member.role === "manager";

  return (
    <div className="flex flex-col gap-2 rounded-md border px-4 py-3 sm:flex-row sm:items-center sm:justify-between">
      <div className="flex items-center gap-3 min-w-0">
        <UserAvatar
          seed={member.avatar_seed}
          displayName={member.display_name}
          size={32}
          className="shrink-0"
        />
        <div className="min-w-0">
          <span className="font-medium truncate block">
            {member.display_name}
            {isMe && (
              <span className="ml-1 text-sm text-muted-foreground">
                {t("members.you")}
              </span>
            )}
          </span>
          <span className="text-sm text-muted-foreground">
            {t(`members.role.${member.role}` as TranslationKey)}
          </span>
        </div>
      </div>

      <div className="flex flex-wrap gap-2 sm:shrink-0">
        {!isMe && (
          <Button
            size="sm"
            variant="ghost"
            onClick={() => router.push(`/messages/${member.user_id}`)}
            title={t("members.message")}
          >
            <MessageCircle className="h-4 w-4" />
          </Button>
        )}

        {isManager && !isMe && !isMemberManager && (
          <>
            {member.role === "observer" ? (
              <Button
                size="sm"
                variant="outline"
                onClick={() => onPromote(member.user_id)}
              >
                {t("members.promote")}
              </Button>
            ) : (
              <Button
                size="sm"
                variant="outline"
                onClick={() => onDemote(member.user_id)}
              >
                {t("members.demote")}
              </Button>
            )}
            <Button
              size="sm"
              variant="outline"
              onClick={() => onTransfer(member)}
            >
              {t("members.transfer")}
            </Button>
            <Button size="sm" variant="outline" onClick={() => onKick(member)}>
              {t("members.kick")}
            </Button>
            <Button
              size="sm"
              variant="destructive"
              onClick={() => onBan(member)}
            >
              {t("members.ban")}
            </Button>
          </>
        )}
      </div>
    </div>
  );
}
