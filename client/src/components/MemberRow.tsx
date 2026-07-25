"use client";

import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { t } from "@/lib/i18n";
import { MessageCircle } from "lucide-react";

export interface MemberView {
  user_id: string;
  display_name: string;
  email: string;
  role: string;
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
    <div className="flex items-center justify-between rounded-md border px-4 py-3">
      <div>
        <span className="font-medium">
          {member.display_name}
          {isMe && (
            <span className="ml-1 text-sm text-muted-foreground">
              {t("members.you")}
            </span>
          )}
        </span>
        <span className="ml-2 text-sm text-muted-foreground">
          {t(`members.role.${member.role}`)}
        </span>
      </div>

      <div className="flex gap-2">
        {/* Message button (not on self) */}
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

        {/* Manager actions (not on self, not on other managers) */}
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
