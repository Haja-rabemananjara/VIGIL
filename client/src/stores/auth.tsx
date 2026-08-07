"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import { api } from "@/lib/api";
import {
  setLanguage,
  getLanguage,
  initLanguage,
  type Language,
} from "@/lib/i18n";

const TOKEN_STORAGE_KEY = "vigil_token";

export interface User {
  id: string;
  email: string;
  display_name: string;
  language: string;
  avatar_seed: string | null;
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
  language: Language;
  changeLanguage: (lang: Language) => void;
  updateUser: (patch: Partial<User>) => void;
  signup: (
    email: string,
    password: string,
    displayName: string,
  ) => Promise<void>;
  signin: (email: string, password: string) => Promise<void>;
  signout: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | null>(null);

function readStoredToken(): string | null {
  if (typeof window === "undefined") return null;
  return localStorage.getItem(TOKEN_STORAGE_KEY);
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [token, setToken] = useState<string | null>(() => readStoredToken());
  const [isLoading, setIsLoading] = useState<boolean>(
    () => readStoredToken() !== null,
  );

  const [language, setLang] = useState<Language>("en");

  const changeLanguage = useCallback((lang: Language) => {
    setLanguage(lang);
    setLang(lang);
  }, []);

  const updateUser = useCallback((patch: Partial<User>) => {
    setUser((prev) => (prev ? { ...prev, ...patch } : prev));
  }, []);

  useEffect(() => {
    initLanguage();
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setLang(getLanguage());
  }, []);

  useEffect(() => {
    if (!token) return;

    let cancelled = false;

    api<User>("/me", { token })
      .then((u) => {
        if (!cancelled) {
          const stored = localStorage.getItem("vigil_language");
          if (!stored) {
            setLanguage(u.language as "en" | "fr");
            setLang(u.language as "en" | "fr");
          }
          setUser(u);
        }
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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const signin = useCallback(async (email: string, password: string) => {
    const res = await api<SigninResponse>("/auth/signin", {
      method: "POST",
      body: { email, password },
    });
    localStorage.setItem(TOKEN_STORAGE_KEY, res.token);
    setToken(res.token);
    const stored = localStorage.getItem("vigil_language");
    if (!stored) {
      setLanguage(res.user.language as "en" | "fr");
      setLang(res.user.language as "en" | "fr");
    }
    setUser(res.user);
  }, []);

  const signup = useCallback(
    async (email: string, password: string, display_name: string) => {
      await api<User>("/auth/signup", {
        method: "POST",
        body: { email, password, display_name },
      });
      await signin(email, password);
    },
    [signin],
  );

  const signout = useCallback(async () => {
    if (token) {
      try {
        await api("/auth/signout", { method: "POST", token });
      } catch {}
    }
    localStorage.removeItem(TOKEN_STORAGE_KEY);
    setUser(null);
    setToken(null);
  }, [token]);

  return (
    <AuthContext.Provider
      value={{
        user,
        token,
        isLoading,
        language,
        changeLanguage,
        updateUser,
        signup,
        signin,
        signout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return ctx;
}
