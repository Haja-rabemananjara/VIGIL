"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { t } from "@/lib/i18n";
import { api } from "@/lib/api";
import { Pencil } from "lucide-react";
import { displayEmoji } from "@/components/Emoji";

export interface TimelineEntry {
  id: string;
  author_id: string;
  kind: "message" | "system";
  content: string;
  created_at: number;
  edited_at: number | null;
}

function formatDate(ts: number): string {
  return new Date(ts * 1000).toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

interface Props {
  entry: TimelineEntry;
  currentUserId?: string;
  displayName: (userId: string) => string;
  entryReactions: Record<string, string[]>;
  availableEmojis: string[];
  token: string | null;
  onEntryUpdated: (updated: TimelineEntry) => void;
  onReactionToggle: (entryId: string, emoji: string) => void;
}

export function TimelineEntryCard({
  entry,
  currentUserId,
  displayName,
  entryReactions,
  availableEmojis,
  token,
  onEntryUpdated,
  onReactionToggle,
}: Props) {
  const isOwnMessage =
    entry.kind === "message" && entry.author_id === currentUserId;

  const [isEditing, setIsEditing] = useState(false);
  const [editText, setEditText] = useState("");
  const [editLoading, setEditLoading] = useState(false);
  const [showPicker, setShowPicker] = useState(false);

  async function handleSave() {
    const content = editText.trim();
    if (!content) return;
    setEditLoading(true);
    try {
      const updated = await api<TimelineEntry>(`/timeline/${entry.id}`, {
        method: "PATCH",
        token,
        body: { content },
      });
      onEntryUpdated(updated);
      setIsEditing(false);
      setEditText("");
    } catch {
    } finally {
      setEditLoading(false);
    }
  }

  return (
    <div
      className={`rounded-lg border px-4 py-3 text-sm ${
        entry.kind === "system"
          ? "border-dashed bg-muted/30 text-muted-foreground"
          : "bg-card"
      }`}
    >
      {/* Header */}
      <div className="flex items-baseline justify-between gap-2">
        <span className="font-medium">
          {entry.kind === "system"
            ? t("timeline.system")
            : displayName(entry.author_id)}
        </span>
        <span className="flex items-center gap-1 text-xs text-muted-foreground">
          {formatDate(entry.created_at)}
          {entry.edited_at && (
            <span className="italic" title={formatDate(entry.edited_at)}>
              · {t("timeline.edited")}
            </span>
          )}
          {isOwnMessage && !isEditing && (
            <button
              onClick={() => {
                setIsEditing(true);
                setEditText(entry.content);
              }}
              className="ml-1 flex items-center gap-1 rounded px-1 text-muted-foreground hover:text-foreground focus:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <span>{t("action.edit")}</span>
              <Pencil className="h-3 w-3" />
            </button>
          )}
        </span>
      </div>

      {/* Content or edit mode */}
      {isEditing ? (
        <div className="mt-2 space-y-2">
          <textarea
            value={editText}
            onChange={(e) => setEditText(e.target.value)}
            rows={2}
            onKeyDown={(e) => {
              if (e.key === "Enter" && e.ctrlKey && !editLoading) handleSave();
              if (e.key === "Escape") {
                setIsEditing(false);
                setEditText("");
              }
            }}
            className="w-full rounded-md border bg-background px-3 py-2 text-sm focus:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            autoFocus
          />
          <div className="flex gap-2">
            <Button
              size="sm"
              onClick={handleSave}
              disabled={editLoading || !editText.trim()}
            >
              {t("action.save")}
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={() => {
                setIsEditing(false);
                setEditText("");
              }}
              disabled={editLoading}
            >
              {t("action.cancel")}
            </Button>
          </div>
        </div>
      ) : (
        <p className="mt-1">{entry.content}</p>
      )}

      {/* Reactions */}
      {entry.kind === "message" && (
        <div className="mt-2 flex flex-wrap items-center gap-1">
          {Object.entries(entryReactions).map(([emoji, userIds]) => {
            const reacted = userIds.includes(currentUserId ?? "");
            return (
              <button
                key={emoji}
                onClick={() => onReactionToggle(entry.id, emoji)}
                className={`inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs transition-colors ${
                  reacted
                    ? "border-primary bg-primary/10 text-primary"
                    : "border-border text-muted-foreground hover:border-primary/50"
                }`}
                title={userIds.map(displayName).join(", ")}
              >
                <span>{displayEmoji(emoji)}</span>
                <span>{userIds.length}</span>
              </button>
            );
          })}
          <button
            onClick={() => setShowPicker(!showPicker)}
            className="inline-flex items-center rounded-full border border-dashed px-2 py-0.5 text-xs text-muted-foreground hover:border-primary/50 hover:text-foreground"
          >
            +
          </button>
          {showPicker && (
            <div className="flex gap-1 rounded-md border bg-popover p-1 shadow-sm">
              {availableEmojis.map((emoji) => (
                <button
                  key={emoji}
                  onClick={() => {
                    onReactionToggle(entry.id, emoji);
                    setShowPicker(false);
                  }}
                  className="rounded px-1.5 py-0.5 text-sm hover:bg-muted"
                >
                  {displayEmoji(emoji)}
                </button>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}