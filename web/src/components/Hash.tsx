import { Link } from '@tanstack/react-router';
import { cn } from '@/lib/cn';
import { shortAddr } from '@/lib/format';
import { CopyButton } from './CopyButton';

type Kind = 'tx' | 'address' | 'package' | 'checkpoint' | 'object';

interface HashProps {
  value: string;
  kind?: Kind;
  short?: boolean | number;
  className?: string;
  copy?: boolean;
  bare?: boolean;
}

/** Single canonical way to render a hash/address/digest in the UI:
 *  short by default, monospaced, optionally copy-able and linked. */
export function Hash({ value, kind, short = true, className, copy = true, bare = false }: HashProps) {
  const visible = typeof short === 'number' ? shortAddr(value, short) : short ? shortAddr(value) : value;
  const isLink = !!kind && kind !== 'object' && !bare;
  const inner = (
    <span className={cn('mono break-all', isLink && 'text-accent hover:underline', className)}>
      {visible}
    </span>
  );
  return (
    <span className="inline-flex items-center gap-1.5 max-w-full">
      {isLink ? <LinkFor kind={kind!} value={value}>{inner}</LinkFor> : inner}
      {copy && <CopyButton value={value} silent />}
    </span>
  );
}

function LinkFor({ kind, value, children }: { kind: Kind; value: string; children: React.ReactNode }) {
  switch (kind) {
    case 'tx':
      return (
        <Link to="/tx/$digest" params={{ digest: value }}>
          {children}
        </Link>
      );
    case 'address':
      return (
        <Link to="/address/$addr" params={{ addr: value }}>
          {children}
        </Link>
      );
    case 'package':
      return (
        <Link to="/package/$id" params={{ id: value }}>
          {children}
        </Link>
      );
    case 'checkpoint':
      return (
        <Link to="/checkpoint/$seq" params={{ seq: value }}>
          {children}
        </Link>
      );
    default:
      return <>{children}</>;
  }
}
