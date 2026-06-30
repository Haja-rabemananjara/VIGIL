"use client";

import {
  createContext,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { useAuth } from "./auth";
import { wsUrl, computeBackoff } from "@/lib/socket";

// TYPES
export type ConnectionStatus = "connecting" | "connected" | "disconnected";

/** A single event received from the server via WebSocket */
export interface WsEvent {
    type: string;
    [key: string]: unknown;
}

/**
 * Current connection state
 * The most recent event received. Changes trigger re-renders in consumers.
 * Increments after every successful reconnection.
 */
interface VigilSocketContextValue {
  status: ConnectionStatus;
  lastEvent: WsEvent | null;
  reconnectCount: number;
}

const VigilSocketContext = createContext<VigilSocketContextValue>({
  status: "disconnected",
  lastEvent: null,
  reconnectCount: 0,
});

export function useVigilSocket() {
  return useContext(VigilSocketContext);
}

// PROVIDER
export function VigilSocketProvider({ children }: { children: ReactNode }) {
  const { token } = useAuth();

  const [status, setStatus] = useState<ConnectionStatus>("disconnected");
  const [lastEvent, setLastEvent] = useState<WsEvent | null>(null);
  const [reconnectCount, setReconnectCount] = useState(0);

  const wsRef = useRef<WebSocket | null>(null);
  const attemptRef = useRef(0);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;

    if (!token) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setStatus("disconnected");
      return;
    }

    function connect() {
      if (!mountedRef.current) return;

      setStatus("connecting");

      const url = wsUrl(token!);
      const ws = new WebSocket(url);
      wsRef.current = ws;

      ws.onopen = () => {
        if (!mountedRef.current) return;
        attemptRef.current = 0;
        setStatus("connected");

        if (reconnectCount > 0 || attemptRef.current > 0) {
          setReconnectCount((prev) => prev + 1);
        }
      };

      ws.onmessage = (event) => {
        if (!mountedRef.current) return;
        try {
          const parsed: WsEvent = JSON.parse(event.data);
          setLastEvent({ ...parsed }); // new reference each time
        } catch {
        }
      };

      ws.onclose = () => {
        if (!mountedRef.current) return;
        setStatus("disconnected");
        wsRef.current = null;

        const delay = computeBackoff(attemptRef.current);
        attemptRef.current += 1;
        timerRef.current = setTimeout(connect, delay);
      };

      ws.onerror = () => {
        ws.close();
      };
    }

    connect();

    return () => {
      mountedRef.current = false;
      if (timerRef.current) clearTimeout(timerRef.current);
      if (wsRef.current) {
        wsRef.current.onclose = null;
        wsRef.current.close();
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [token]);

  return (
    <VigilSocketContext.Provider value={{ status, lastEvent, reconnectCount }}>
      {children}
    </VigilSocketContext.Provider>
  );
}