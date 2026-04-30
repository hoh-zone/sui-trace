import { useEffect, useRef, useState } from 'react';

interface Options<T> {
  onMessage?: (msg: T) => void;
  parser?: (raw: string) => T | null;
  /** Reconnect with exponential backoff (default true). */
  reconnect?: boolean;
}

/** Subscribes to the API WS endpoint and exposes the latest payload + status. */
export function useWebSocket<T = unknown>(path: string, opts: Options<T> = {}) {
  const { onMessage, parser, reconnect = true } = opts;
  const [last, setLast] = useState<T | null>(null);
  const [status, setStatus] = useState<'connecting' | 'open' | 'closed'>('connecting');
  const ref = useRef<WebSocket | null>(null);
  const onMessageRef = useRef(onMessage);
  const parserRef = useRef(parser);
  onMessageRef.current = onMessage;
  parserRef.current = parser;

  useEffect(() => {
    let stopped = false;
    let attempt = 0;

    const connect = () => {
      if (stopped) return;
      const proto = window.location.protocol === 'https:' ? 'wss' : 'ws';
      const ws = new WebSocket(`${proto}://${window.location.host}${path}`);
      ref.current = ws;
      setStatus('connecting');
      ws.onopen = () => {
        attempt = 0;
        setStatus('open');
      };
      ws.onclose = () => {
        setStatus('closed');
        if (!reconnect) return;
        const delay = Math.min(15_000, 500 * 2 ** attempt++);
        setTimeout(connect, delay);
      };
      ws.onerror = () => ws.close();
      ws.onmessage = (e) => {
        try {
          const raw = typeof e.data === 'string' ? e.data : '';
          const parsed = parserRef.current ? parserRef.current(raw) : (JSON.parse(raw) as T);
          if (parsed != null) {
            setLast(parsed);
            onMessageRef.current?.(parsed);
          }
        } catch {
          /* ignore parse errors */
        }
      };
    };

    connect();
    return () => {
      stopped = true;
      ref.current?.close();
    };
  }, [path, reconnect]);

  return { last, status } as const;
}
