import { ChevronLeft, ChevronRight } from 'lucide-react';
import { cn } from '@/lib/cn';

interface PaginationProps {
  page: number;
  pageSize: number;
  hasNext: boolean;
  onChange: (page: number) => void;
  className?: string;
}

export function Pagination({ page, pageSize, hasNext, onChange, className }: PaginationProps) {
  return (
    <div className={cn('flex items-center justify-end gap-2 text-xs text-fg-muted', className)}>
      <span>
        page {page + 1} · {pageSize}/page
      </span>
      <button
        onClick={() => onChange(Math.max(0, page - 1))}
        disabled={page === 0}
        className="p-1.5 rounded border border-border-subtle hover:bg-bg-elev disabled:opacity-40 disabled:cursor-not-allowed"
      >
        <ChevronLeft size={14} />
      </button>
      <button
        onClick={() => onChange(page + 1)}
        disabled={!hasNext}
        className="p-1.5 rounded border border-border-subtle hover:bg-bg-elev disabled:opacity-40 disabled:cursor-not-allowed"
      >
        <ChevronRight size={14} />
      </button>
    </div>
  );
}
