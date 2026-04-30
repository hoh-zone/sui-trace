import { useState } from 'react';
import { cn } from '@/lib/cn';

export interface TabSpec {
  id: string;
  label: React.ReactNode;
  badge?: React.ReactNode;
  content: React.ReactNode;
}

interface TabsProps {
  tabs: TabSpec[];
  initial?: string;
  className?: string;
}

export function Tabs({ tabs, initial, className }: TabsProps) {
  const [active, setActive] = useState(initial ?? tabs[0]?.id);
  const current = tabs.find((t) => t.id === active) ?? tabs[0];
  return (
    <div className={className}>
      <div role="tablist" className="flex items-center gap-1 border-b border-border-subtle">
        {tabs.map((t) => {
          const isActive = t.id === current?.id;
          return (
            <button
              key={t.id}
              role="tab"
              aria-selected={isActive}
              onClick={() => setActive(t.id)}
              className={cn(
                'inline-flex items-center gap-1.5 px-3 py-2 text-xs uppercase tracking-wide -mb-px border-b-2',
                isActive
                  ? 'text-fg border-accent'
                  : 'text-fg-subtle border-transparent hover:text-fg hover:border-border',
              )}
            >
              {t.label}
              {t.badge != null && (
                <span className="text-[10px] mono px-1.5 py-0.5 rounded bg-bg-elev border border-border-subtle text-fg-muted">
                  {t.badge}
                </span>
              )}
            </button>
          );
        })}
      </div>
      <div className="pt-4 animate-in">{current?.content}</div>
    </div>
  );
}
