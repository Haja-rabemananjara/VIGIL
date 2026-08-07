"use client";

import { useEffect, useRef, useState } from "react";
import { useAuth } from "@/stores/auth";
import { api } from "@/lib/api";
import { getLanguage, t } from "@/lib/i18n";
import { Button } from "@/components/ui/button";
import { useVigilSocket } from "@/stores/socket";
import { ArrowLeft } from "lucide-react";
import { useRouter } from "next/navigation";
import { usePathname } from "next/navigation";

interface Message {
  id: string;
  sender_id: string;
  recipient_id: string;
  content: string;
  created_at: number;
}

interface UserInfo {
  id: string;
  display_name: string;
}

function formatDate(ts: number): string {
  return new Date(ts * 1000).toLocaleString(getLanguage(), {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function ConversationClient() {
  const pathname = usePathname();
  const otherUserId = pathname?.split("/")[2] ?? "";
  const { token, user } = useAuth();
  const router = useRouter();

  const [messages, setMessages] = useState<Message[]>([]);
  const [otherUser, setOtherUser] = useState<UserInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  const [composerText, setComposerText] = useState("");
  const [sending, setSending] = useState(false);

  const bottomRef = useRef<HTMLDivElement>(null);
  const lastTypingSent = useRef(0);
  const [isOtherTyping, setIsOtherTyping] = useState(false);
  const typingTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const { lastEvent, reconnectCount, send } = useVigilSocket();

  useEffect(() => {
    if (!token || !otherUserId) return;
    Promise.all([
      api<{ messages: Message[] }>(`/messages/${otherUserId}`, { token }),
      api<UserInfo>(`/users/${otherUserId}`, { token }).catch(() => null),
    ])
      .then(([conv, userInfo]) => {
        setMessages(conv.messages);
        if (userInfo) setOtherUser(userInfo);
      })
      .catch(() => setError(t("common.error")))
      .finally(() => setLoading(false));
  }, [token, otherUserId]);

  // Scroll to bottom on new messages
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // Re-fetch after reconnection
  useEffect(() => {
    if (reconnectCount > 0 && token && otherUserId) {
      api<{ messages: Message[] }>(`/messages/${otherUserId}`, { token })
        .then((conv) => setMessages(conv.messages))
        .catch(() => {});
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [reconnectCount]);

  useEffect(() => {
    if (!lastEvent || lastEvent.type !== "private_message_received") return;

    const from = lastEvent.from as string;
    const to = lastEvent.to as string;
    const involves =
      (from === user?.id && to === otherUserId) ||
      (from === otherUserId && to === user?.id);

    if (!involves) return;

    const newMsg: Message = {
      id: lastEvent.message_id as string,
      sender_id: from,
      recipient_id: to,
      content: lastEvent.content as string,
      created_at: lastEvent.at as number,
    };

    // eslint-disable-next-line react-hooks/set-state-in-effect
    setMessages((prev) => {
      if (prev.some((m) => m.id === newMsg.id)) return prev;
      return [...prev, newMsg];
    });
  }, [lastEvent, otherUserId, user?.id]);

  useEffect(() => {
    if (!lastEvent || lastEvent.type !== "user_typing") return;
    if ((lastEvent.from as string) !== otherUserId) return;

    // eslint-disable-next-line react-hooks/set-state-in-effect
    setIsOtherTyping(true);
    if (typingTimeoutRef.current) clearTimeout(typingTimeoutRef.current);
    typingTimeoutRef.current = setTimeout(() => setIsOtherTyping(false), 3000);
  }, [lastEvent, otherUserId]);

  // Cleanup timeout on unmount
  useEffect(() => {
    return () => {
      if (typingTimeoutRef.current) clearTimeout(typingTimeoutRef.current);
    };
  }, []);

  async function handleSend() {
    const content = composerText.trim();
    if (!content) return;
    setSending(true);
    try {
      const msg = await api<Message>(`/messages/${otherUserId}`, {
        method: "POST",
        token,
        body: { content },
      });
      setMessages((prev) => {
        if (prev.some((m) => m.id === msg.id)) return prev;
        return [...prev, msg];
      });
      setComposerText("");
    } catch {
      // silent
    } finally {
      setSending(false);
    }
  }

  function handleTyping() {
    const now = Date.now();
    if (now - lastTypingSent.current < 2500) return;
    lastTypingSent.current = now;
    send({ type: "typing", recipient_id: otherUserId });
  }

  const displayName = otherUser?.display_name ?? otherUserId ?? "";

  if (loading) {
    return (
      <div className="p-6 text-muted-foreground">{t("common.loading")}</div>
    );
  }

  if (error) {
    return <div className="p-6 text-destructive">{error}</div>;
  }

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="flex items-center gap-3 border-b px-6 py-3">
        <button
          onClick={() => router.back()}
          className="text-muted-foreground hover:text-foreground focus:outline-none focus-visible:ring-2 focus-visible:ring-ring rounded"
        >
          <ArrowLeft className="h-4 w-4" />
        </button>
        <h2 className="font-medium">{displayName}</h2>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-auto p-6 space-y-3">
        {messages.length === 0 ? (
          <p className="text-sm text-muted-foreground">{t("messages.empty")}</p>
        ) : (
          messages.map((msg) => {
            const isMine = msg.sender_id === user?.id;
            return (
              <div
                key={msg.id}
                className={`flex ${isMine ? "justify-end" : "justify-start"}`}
              >
                <div
                  className={`max-w-[70%] rounded-lg px-4 py-2 text-sm ${
                    isMine ? "bg-primary text-primary-foreground" : "bg-muted"
                  }`}
                >
                  <p>{msg.content}</p>
                  <p
                    className={`mt-1 text-xs ${
                      isMine
                        ? "text-primary-foreground/70"
                        : "text-muted-foreground"
                    }`}
                  >
                    {formatDate(msg.created_at)}
                  </p>
                </div>
              </div>
            );
          })
        )}
        <div ref={bottomRef} />
      </div>

      {isOtherTyping && (
        <p className="px-6 text-xs text-muted-foreground animate-pulse">
          {displayName} {t("messages.typing")}
        </p>
      )}

      {/* Composer */}
      <div className="flex gap-2 border-t px-6 py-3">
        <textarea
          value={composerText}
          onChange={(e) => {
            setComposerText(e.target.value);
            handleTyping();
          }}
          placeholder={t("messages.composer.placeholder")}
          rows={1}
          onKeyDown={(e) => {
            if (e.key === "Enter" && e.ctrlKey && !sending) {
              handleSend();
            }
          }}
          className="flex-1 rounded-md border bg-background px-3 py-2 text-sm focus:outline-none focus-visible:ring-2 focus-visible:ring-ring resize-none"
        />
        <Button onClick={handleSend} disabled={sending || !composerText.trim()}>
          {t("messages.send")}
        </Button>
      </div>
    </div>
  );
}
