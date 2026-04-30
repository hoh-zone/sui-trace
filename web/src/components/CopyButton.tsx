import { Check, Copy } from 'lucide-react';
import { useState } from 'react';
import { cn } from '@/lib/cn';
import { useToast } from './Toast';

interface CopyButtonProps {
  value: string;
  className?: string;
  size?: number;
  silent?: boolean;
}

export function CopyButton({ value, className, size = 12, silent = false }: CopyButtonProps) {
  const [copied, setCopied] = useState(false);
  const { push } = useToast();
  return (
    <button
      type="button"
      title="Copy"
      onClick={async (e) => {
        e.stopPropagation();
        e.preventDefault();
        try {
          await navigator.clipboard.writeText(value);
          setCopied(true);
          if (!silent) push('Copied to clipboard', 'success');
          setTimeout(() => setCopied(false), 1200);
        } catch {
          push('Copy failed', 'danger');
        }
      }}
      className={cn(
        'inline-flex items-center justify-center text-fg-subtle hover:text-fg transition-colors',
        className,
      )}
    >
      {copied ? <Check size={size} className="text-ok" /> : <Copy size={size} />}
    </button>
  );
}
