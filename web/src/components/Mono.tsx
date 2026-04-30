import { Link } from '@tanstack/react-router';
import { cn } from '@/lib/cn';
import { shortAddr } from '@/lib/format';

interface AddressLinkProps {
  value: string;
  className?: string;
  short?: boolean;
}

export function AddressLink({ value, className, short = true }: AddressLinkProps) {
  return (
    <Link
      to="/address/$addr"
      params={{ addr: value }}
      className={cn('mono text-accent hover:underline', className)}
    >
      {short ? shortAddr(value) : value}
    </Link>
  );
}

export function PackageLink({ value, className, short = true }: AddressLinkProps) {
  return (
    <Link
      to="/package/$id"
      params={{ id: value }}
      className={cn('mono text-accent hover:underline', className)}
    >
      {short ? shortAddr(value) : value}
    </Link>
  );
}

export function TxLink({ value, className, short = true }: AddressLinkProps) {
  return (
    <Link
      to="/tx/$digest"
      params={{ digest: value }}
      className={cn('mono text-accent hover:underline', className)}
    >
      {short ? shortAddr(value) : value}
    </Link>
  );
}
