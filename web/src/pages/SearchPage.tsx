import { useQuery } from '@tanstack/react-query';
import { Link, useNavigate, useSearch } from '@tanstack/react-router';
import { useEffect } from 'react';
import { ArrowRight, Search } from 'lucide-react';
import { Card } from '@/components/Card';
import { Loader, ErrorBlock, Empty } from '@/components/Loader';
import { Hash } from '@/components/Hash';
import { Badge } from '@/components/Badge';
import { api } from '@/lib/api';
import { categoryColor, classifyTerm, shortAddr } from '@/lib/format';

export function SearchPage() {
  const { q } = useSearch({ from: '/search' });
  const navigate = useNavigate();

  // Fast path: a definite digest can jump straight to the tx page
  useEffect(() => {
    if (!q) return;
    const kind = classifyTerm(q);
    if (kind === 'tx') navigate({ to: '/tx/$digest', params: { digest: q }, replace: true });
  }, [q, navigate]);

  const result = useQuery({ queryKey: ['search', q], queryFn: () => api.search(q), enabled: !!q });

  if (!q) {
    return (
      <Empty
        label="Enter a search term"
        hint="Paste a digest, address, package id, or label name."
        icon={<Search size={20} />}
      />
    );
  }
  if (result.isLoading) return <Loader />;
  if (result.error) return <ErrorBlock error={result.error} />;
  const data = result.data!;

  return (
    <div className="space-y-6">
      <Card title={`Results for "${q}"`}>
        <div className="text-xs text-fg-subtle uppercase tracking-wider">Detected kind</div>
        <div className="mt-1 mb-4">
          <Badge variant="accent">{data.kind}</Badge>
        </div>

        <ul className="space-y-2 text-sm">
          {data.kind === 'address_or_object' && (
            <>
              <Suggestion to="/address/$addr" params={{ addr: q }} label="View as address" value={q} />
              <Suggestion to="/package/$id" params={{ id: q }} label="View as package" value={q} />
            </>
          )}
          {data.kind === 'digest' && (
            <Suggestion to="/tx/$digest" params={{ digest: q }} label="View as transaction" value={q} />
          )}
          {data.kind === 'unknown' && (
            <li className="text-xs text-fg-subtle">
              The term doesn't look like a 0x-prefixed digest or address. Try the labels list below.
            </li>
          )}
        </ul>
      </Card>

      <Card title={`Matching labels · ${data.labels.length}`}>
        {data.labels.length === 0 ? (
          <Empty label="No labels matched the term." />
        ) : (
          <ul className="space-y-2 text-sm">
            {data.labels.map((l) => (
              <li
                key={`${l.address}-${l.label}-${l.source}`}
                className="flex items-center justify-between gap-3 border-b border-border-subtle pb-2"
              >
                <div className="flex items-center gap-2 min-w-0">
                  <Hash value={l.address} kind="address" copy={false} />
                  <span className="text-fg-muted truncate">{l.label}</span>
                </div>
                <span className="flex items-center gap-2 shrink-0">
                  <span
                    className={`px-2 py-0.5 text-[11px] uppercase tracking-wider border rounded ${categoryColor(l.category)}`}
                  >
                    {l.category}
                  </span>
                  <span className="text-[10px] text-fg-subtle uppercase">{l.source}</span>
                </span>
              </li>
            ))}
          </ul>
        )}
      </Card>
    </div>
  );
}

interface SuggestionProps {
  to: '/address/$addr' | '/package/$id' | '/tx/$digest';
  params: { addr: string } | { id: string } | { digest: string };
  label: string;
  value: string;
}

function Suggestion({ to, params, label, value }: SuggestionProps) {
  return (
    <li>
      <Link
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        to={to as any}
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        params={params as any}
        className="flex items-center justify-between gap-2 border border-border-subtle rounded-md px-3 py-2 hover:border-accent/40 hover:bg-bg-elev"
      >
        <span className="text-fg">
          {label} <span className="mono text-accent">{shortAddr(value, 8)}</span>
        </span>
        <ArrowRight size={14} className="text-fg-subtle" />
      </Link>
    </li>
  );
}
