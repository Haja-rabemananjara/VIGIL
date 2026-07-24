const EMOJI_DISPLAY: Record<string, string> = {
  "+1": "👍",
  "-1": "👎",
  eyes: "👀",
  warning: "⚠️",
  check: "✅",
  fire: "🔥",
};

export function displayEmoji(key: string): string {
  return EMOJI_DISPLAY[key] ?? key;
}