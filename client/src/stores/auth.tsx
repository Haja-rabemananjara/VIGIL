"use client";

/**
 * Authentication store, exposed via React Context.
 *
 * - Holds the current user and session token.
 * - Persists the token in localStorage (per architectural decision).
 * - Exposes signin/signup/signout to any component via useAuth().
 */

import {
    createContext,
    useCallback,
    useContext,
    useEffect,
    useState,
    type ReactNode,
} from "react";
import { api } from "@/lib/api";

const TOKEN_STORAGE_KEY = "vigil_token";

/** Public-safe user shape returned by /me, /auth/signup, /auth/signin. */
export interface User {
    id: string;
    email: string;
    display_name: string;
    language: string;
    created_at: number;
}

interface SigninResponse {
    token: string;
    user: User;
}

interface AuthContextValue {
    user: User | null;
    token: string | null;
    isLoading: boolean;
    signup: (email: string, password: string, displayName: string) => Promise<void>;
    signin: (email: string, password: string) => Promise<void>;
    signout: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | null>(null);

/**
 * Read the stored token synchronously at mount, before any render.
 * Returning null here (instead of using setState in an effect) avoids the
 * cascading-render warning from React 19's compiler.
 */
function readStoredToken(): string | null {
    if (typeof window === "undefined") return null;
    return localStorage.getItem(TOKEN_STORAGE_KEY);
}

export function AuthProvider({ children }: { children: ReactNode }) {
    const [user, setUser] = useState<User | null>(null);
    const [token, setToken] = useState<string | null>(() => readStoredToken());
    // We only stay in "loading" state if a token was found and needs validation.
    // No token => no async work to do => not loading.
    const [isLoading, setIsLoading] = useState<boolean>(() => readStoredToken() !== null);

    // Validate the restored token against /me.
    // If it's still valid, hydrate the user. If not, clear and move on.
    useEffect(() => {
        if (!token) return;

        let cancelled = false;

        api<User>("/me", { token })
        .then((u) => {
            if (!cancelled) setUser(u);
        })
        .catch(() => {
            if (!cancelled) {
            localStorage.removeItem(TOKEN_STORAGE_KEY);
            setToken(null);
            }
        })
        .finally(() => {
            if (!cancelled) setIsLoading(false);
        });

        return () => {
        cancelled = true;
        };
        // Run once at mount with the initial token. We don't re-run on token changes
        // here because signin() already sets user explicitly.
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []);

    const signin = useCallback(async (email: string, password: string) => {
        const res = await api<SigninResponse>("/auth/signin", {
        method: "POST",
        body: { email, password },
        });
        localStorage.setItem(TOKEN_STORAGE_KEY, res.token);
        setToken(res.token);
        setUser(res.user);
    }, []);

    const signup = useCallback(
        async (email: string, password: string, display_name: string) => {
        await api<User>("/auth/signup", {
            method: "POST",
            body: { email, password, display_name },
        });
        // Auto-signin after signup for a smooth UX.
        await signin(email, password);
        },
        [signin],
    );

    const signout = useCallback(async () => {
        if (token) {
        try {
            await api("/auth/signout", { method: "POST", token });
        } catch {
            // Server-side cleanup may fail (e.g. offline). Local cleanup still happens.
        }
        }
        localStorage.removeItem(TOKEN_STORAGE_KEY);
        setUser(null);
        setToken(null);
    }, [token]);

    return (
        <AuthContext.Provider value={{ user, token, isLoading, signup, signin, signout }}>
        {children}
        </AuthContext.Provider>
    );
}

/**
 * Access the auth state from any component.
 * Throws if used outside an <AuthProvider>.
 */
export function useAuth(): AuthContextValue {
    const ctx = useContext(AuthContext);
    if (!ctx) {
        throw new Error("useAuth must be used within an AuthProvider");
    }
    return ctx;
}