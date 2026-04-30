import { cn } from '@/lib/cn';

type Variant = 'default' | 'accent' | 'success' | 'warn' | 'danger' | 'info' | 'outline';

interface BadgeProps {
  children: React.ReactNode;
  className?: string;
  variant?: Variant;
}

const variants: Record<Variant, string> = {
  default: 'bg-bg-elev border-border text-fg-muted',
  accent: 'bg-accent/15 border-accent/40 text-accent',
  success: 'bg-ok/12 border-ok/40 text-ok',
  warn: 'bg-warn/12 border-warn/40 text-warn',
  danger: 'bg-danger/12 border-danger/40 text-danger',
  info: 'bg-info/12 border-info/40 text-info',
  outline: 'border-border text-fg-subtle',
};

export function Badge({ children, className, variant = 'default' }: BadgeProps) {
  return (
    <span
      className={cn(
        'inline-flex items-center px-1.5 py-0.5 text-[10px] uppercase tracking-wider font-semibold border rounded',
        variants[variant],
        className,
      )}
    >
      {children}
    </span>
  );
}
