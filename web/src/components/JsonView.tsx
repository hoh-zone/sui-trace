import { useState } from 'react';
import { cn } from '@/lib/cn';
import { CopyButton } from './CopyButton';

interface JsonViewProps {
  value: unknown;
  collapsed?: boolean;
  className?: string;
  maxHeight?: number;
}

/** Lightweight pretty-printed JSON block with copy + collapse. No deps. */
export function JsonView({ value, collapsed = false, className, maxHeight = 480 }: JsonViewProps) {
  const [open, setOpen] = useState(!collapsed);
  const text = (() => {
    try {
      return JSON.stringify(value, null, 2);
    } catch {
      return String(value);
    }
  })();
  const lines = text.split('\n').length;

  return (
    <div className={cn('relative group rounded-md border border-border-subtle bg-bg/60', className)}>
      <div className="flex items-center justify-between px-3 py-1.5 border-b border-border-subtle text-xs text-fg-subtle">
        <button
          className="hover:text-fg"
          onClick={() => setOpen((o) => !o)}
          aria-expanded={open}
        >
          {open ? '▾' : '▸'} JSON · {lines} lines
        </button>
        <CopyButton value={text} silent />
      </div>
      {open && (
        <pre
          className="mono text-[12px] leading-relaxed p-3 overflow-auto"
          style={{ maxHeight }}
        >
          {colorise(text)}
        </pre>
      )}
    </div>
  );
}

/** Very small JSON syntax highlighter — strings/numbers/booleans/keys. */
function colorise(text: string) {
  // We split on a regex that captures keys, strings, numbers, booleans, null.
  const parts: React.ReactNode[] = [];
  const re =
    /("(?:\\.|[^"\\])*"\s*:)|("(?:\\.|[^"\\])*")|(\b-?\d+(?:\.\d+)?(?:e[+-]?\d+)?\b)|(\btrue\b|\bfalse\b|\bnull\b)/gi;
  let last = 0;
  let i = 0;
  for (const m of text.matchAll(re)) {
    if (m.index! > last) parts.push(text.slice(last, m.index!));
    if (m[1]) parts.push(<span key={i++} className="text-accent">{m[1]}</span>);
    else if (m[2]) parts.push(<span key={i++} className="text-ok">{m[2]}</span>);
    else if (m[3]) parts.push(<span key={i++} className="text-warn">{m[3]}</span>);
    else if (m[4]) parts.push(<span key={i++} className="text-info">{m[4]}</span>);
    last = m.index! + m[0].length;
  }
  if (last < text.length) parts.push(text.slice(last));
  return parts;
}
