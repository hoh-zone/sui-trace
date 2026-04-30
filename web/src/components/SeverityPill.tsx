import { cn } from '@/lib/cn';

type Severity = 'critical' | 'high' | 'medium' | 'low' | 'info' | (string & {});

const map: Record<string, string> = {
  critical: 'bg-danger/15 border-danger/40 text-danger',
  high: 'bg-warn/15 border-warn/45 text-warn',
  medium: 'bg-warn/10 border-warn/30 text-warn',
  low: 'bg-bg-elev border-border text-fg-muted',
  info: 'bg-info/10 border-info/30 text-info',
};

export function SeverityPill({ value, className }: { value: Severity; className?: string }) {
  const k = (value || 'info').toLowerCase();
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1 px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wider border rounded',
        map[k] ?? map.info,
        className,
      )}
    >
      {value}
    </span>
  );
}

export function SeverityScore({ score }: { score: number }) {
  // 0–10 score → green→red gradient
  const pct = Math.min(100, Math.max(0, score * 10));
  const tone =
    score >= 7
      ? 'text-danger'
      : score >= 4
        ? 'text-warn'
        : score >= 1.5
          ? 'text-info'
          : 'text-ok';
  return (
    <div className="flex items-center gap-2">
      <span className={cn('mono font-semibold', tone)}>{score.toFixed(1)}</span>
      <div className="w-16 h-1.5 bg-bg-elev rounded overflow-hidden">
        <div
          className={cn(
            'h-full',
            score >= 7
              ? 'bg-danger'
              : score >= 4
                ? 'bg-warn'
                : score >= 1.5
                  ? 'bg-info'
                  : 'bg-ok',
          )}
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  );
}
