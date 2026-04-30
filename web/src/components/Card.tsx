import { cn } from '@/lib/cn';

interface CardProps {
  title?: React.ReactNode;
  subtitle?: React.ReactNode;
  action?: React.ReactNode;
  children?: React.ReactNode;
  className?: string;
  bodyClassName?: string;
  noPadding?: boolean;
}

export function Card({
  title,
  subtitle,
  action,
  children,
  className,
  bodyClassName,
  noPadding,
}: CardProps) {
  return (
    <section
      className={cn(
        'bg-bg-subtle/70 backdrop-blur border border-border-subtle rounded-xl shadow-soft',
        className,
      )}
    >
      {(title || action) && (
        <header className="flex items-center justify-between px-4 py-3 border-b border-border-subtle gap-3">
          <div className="min-w-0">
            <h2 className="text-sm font-medium text-fg flex items-center gap-2">{title}</h2>
            {subtitle && <p className="text-xs text-fg-subtle mt-0.5">{subtitle}</p>}
          </div>
          {action && <div className="text-xs text-fg-muted shrink-0">{action}</div>}
        </header>
      )}
      <div className={cn(noPadding ? '' : 'p-4', bodyClassName)}>{children}</div>
    </section>
  );
}

interface StatCardProps {
  label: string;
  value: React.ReactNode;
  hint?: React.ReactNode;
  trend?: React.ReactNode;
  icon?: React.ReactNode;
  accent?: 'accent' | 'ok' | 'warn' | 'danger' | 'info';
}

const accentMap: Record<NonNullable<StatCardProps['accent']>, string> = {
  accent: 'from-accent/30 to-transparent',
  ok: 'from-ok/30 to-transparent',
  warn: 'from-warn/30 to-transparent',
  danger: 'from-danger/30 to-transparent',
  info: 'from-info/30 to-transparent',
};

export function StatCard({ label, value, hint, trend, icon, accent }: StatCardProps) {
  return (
    <div className="relative overflow-hidden bg-bg-subtle/70 backdrop-blur border border-border-subtle rounded-xl p-4 shadow-soft">
      {accent && (
        <div
          className={cn(
            'absolute -top-12 -right-12 w-32 h-32 rounded-full blur-3xl opacity-60 bg-gradient-to-br',
            accentMap[accent],
          )}
        />
      )}
      <div className="relative">
        <div className="flex items-center justify-between text-xs uppercase tracking-wide text-fg-subtle">
          <span className="flex items-center gap-1.5">
            {icon}
            {label}
          </span>
          {trend}
        </div>
        <div className="mt-2 text-[1.5rem] leading-tight font-semibold mono text-fg">{value}</div>
        {hint && <div className="mt-1 text-xs text-fg-subtle">{hint}</div>}
      </div>
    </div>
  );
}
