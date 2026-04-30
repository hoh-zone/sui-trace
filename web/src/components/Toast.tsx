import { createContext, useCallback, useContext, useEffect, useState } from 'react';
import { cn } from '@/lib/cn';

type ToastKind = 'info' | 'success' | 'warn' | 'danger';

interface ToastItem {
  id: number;
  kind: ToastKind;
  text: string;
}

interface ToastApi {
  push: (text: string, kind?: ToastKind) => void;
}

const ToastCtx = createContext<ToastApi>({ push: () => {} });

export function useToast() {
  return useContext(ToastCtx);
}

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [items, setItems] = useState<ToastItem[]>([]);

  const push = useCallback((text: string, kind: ToastKind = 'info') => {
    const id = Date.now() + Math.random();
    setItems((prev) => [...prev, { id, kind, text }]);
    setTimeout(() => setItems((prev) => prev.filter((t) => t.id !== id)), 2800);
  }, []);

  return (
    <ToastCtx.Provider value={{ push }}>
      {children}
      <div className="fixed top-4 right-4 z-50 flex flex-col items-end gap-2 pointer-events-none">
        {items.map((t) => (
          <Toast key={t.id} {...t} />
        ))}
      </div>
    </ToastCtx.Provider>
  );
}

function Toast({ kind, text }: ToastItem) {
  return (
    <div
      className={cn(
        'toast-in pointer-events-auto px-3 py-2 rounded-md text-xs font-medium border shadow-soft backdrop-blur',
        kind === 'success' && 'bg-ok/15 border-ok/40 text-ok',
        kind === 'warn' && 'bg-warn/15 border-warn/40 text-warn',
        kind === 'danger' && 'bg-danger/15 border-danger/40 text-danger',
        kind === 'info' && 'bg-bg-elev/95 border-border text-fg',
      )}
    >
      {text}
    </div>
  );
}

/** Hook used inside the providers — re-export so pages can fire one-off toasts. */
export function useFlash(text: string | null, kind: ToastKind = 'info') {
  const { push } = useToast();
  useEffect(() => {
    if (text) push(text, kind);
  }, [text, kind, push]);
}
