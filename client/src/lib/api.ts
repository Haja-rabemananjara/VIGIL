/**
 * HTTP client targeting the VIGIL Rust server.
 *
 * Responsibilities:
 *   - Inject the Authorization header for authenticated requests.
 *   - Parse the server's uniform error shape {error: {code, message}}.
 *   - Throw typed errors so callers can react (401 => redirect to signin,
 *     409 => show "email taken", etc).
 *
 * The server URL is read from NEXT_PUBLIC_API_URL, with a sane dev default.
 */

const API_URL =
    process.env.NEXT_PUBLIC_API_URL?.replace(/\/$/, "") ?? "http://localhost:8080";

/** Shape of the server's error responses (cf. Rust AppError::IntoResponse). */
export interface ApiErrorBody {
    error: {
        code: string;
        message: string;
    };
}

/**
 * Error thrown by api() when the server returns a non-2xx status.
 * Carries the HTTP status and the parsed server error body if available.
 */
export class ApiError extends Error {
    constructor(
        public status: number,
        public code: string,
        message: string,
    ) {
        super(message);
        this.name = "ApiError";
    }
}

interface RequestOptions {
    method?: "GET" | "POST" | "PATCH" | "DELETE";
    body?: unknown;
    token?: string | null;
}

/**
 * Core fetch wrapper. All other API helpers go through this.
 */
export async function api<T>(path: string, opts: RequestOptions = {}): Promise<T> {
    const { method = "GET", body, token } = opts;

    const headers: Record<string, string> = {};
    if (body !== undefined) headers["Content-Type"] = "application/json";
    if (token) headers["Authorization"] = `Bearer ${token}`;

    const response = await fetch(`${API_URL}${path}`, {
        method,
        headers,
        body: body !== undefined ? JSON.stringify(body) : undefined,
    });

    // 204 No Content: no body to parse (e.g. signout)
    if (response.status === 204) {
        return undefined as T;
    }

    const text = await response.text();
    const data = text ? JSON.parse(text) : null;

    if (!response.ok) {
        const errBody = data as ApiErrorBody | null;
        throw new ApiError(
        response.status,
        errBody?.error?.code ?? "unknown",
        errBody?.error?.message ?? `HTTP ${response.status}`,
        );
    }

    return data as T;
}