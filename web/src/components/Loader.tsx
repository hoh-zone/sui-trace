import { cn } from '@/lib/cn';

export function Loader({ label = 'loading…', className }: { label?: string; className?: string }) {
  return (
    <div className={cn('flex items-center justify-center py-12 text-sm text-fg-subtle', className)}>
      <div className="flex items-center gap-2">
        <span className="live-dot" />
        <span className="animate-pulse">{label}</span>
      </div>
    </div>
  );
}

export function ErrorBlock({ error, onRetry }: { error: unknown; onRetry?: () => void }) {
  return (
    <div className="bg-danger/10 border border-danger/30 text-danger rounded-lg p-3 text-sm flex items-start justify-between gap-3">
      <div>
        <div className="font-medium">Request failed</div>
        <div className="text-xs mt-1 text-danger/80 mono break-all">
          {(error as Error)?.message ?? String(error)}
        </div>
      </div>
      {onRetry && (
        <button
          onClick={onRetry}
          className="text-xs border border-danger/40 px-2 py-1 rounded hover:bg-danger/15"
        >
          retry
        </button>
      )}
    </div>
  );
}

export function Empty({
  label = 'No data yet',
  hint,
  icon,
}: {
  label?: string;
  hint?: React.ReactNode;
  icon?: React.ReactNode;
}) {
  return (
    <div className="text-sm text-fg-subtle py-10 text-center border border-dashed border-border rounded-lg">
      {icon && <div className="flex justify-center mb-2 text-fg-subtle">{icon}</div>}
      <div className="text-fg-muted">{label}</div>
      {hint && <div className="text-xs mt-1">{hint}</div>}
    </div>
  );
}
