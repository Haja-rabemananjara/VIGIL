const API_URL =
  process.env.NEXT_PUBLIC_API_URL?.replace(/\/$/, "") ??
  "http://localhost:8080";

export function wsUrl(token: string): string {
  const base = API_URL.replace(/^http/, "ws");
  return `${base}/ws?token=${token}`;
}

/**
 * Exponential backoff: 1s, 2s, 4s, ... 30s...
 * @param attempt : zero-based retry count
 * @returns delay in milliseconds
 */
export function computeBackoff(attempt: number): number {
  const base = 1000;
  const max = 30000;
  return Math.min(base * Math.pow(2, attempt), max);
}
