import { useQuery } from '@tanstack/react-query';
import { Boxes } from 'lucide-react';
import { useState } from 'react';
import { Card } from '@/components/Card';
import { Loader, ErrorBlock, Empty } from '@/components/Loader';
import { Badge } from '@/components/Badge';
import { Hash } from '@/components/Hash';
import { api } from '@/lib/api';
import { formatRelative, formatTime } from '@/lib/format';

export function PackagesPage() {
  const [limit, setLimit] = useState(50);
  const q = useQuery({
    queryKey: ['pkgs-list', limit],
    queryFn: () => api.recentPackages(limit),
    refetchInterval: 30_000,
  });

  return (
    <div className="space-y-4">
      <header className="flex items-end justify-between gap-4 flex-wrap">
        <div>
          <h1 className="text-xl font-semibold tracking-tight flex items-center gap-2">
            <Boxes size={18} className="text-accent" /> Recent packages
          </h1>
          <p className="text-sm text-fg-muted mt-1">
            Newly published Move packages. Each one is automatically scanned by the security worker.
          </p>
        </div>
        <select
          value={limit}
          onChange={(e) => setLimit(Number(e.target.value))}
          className="bg-bg-elev border border-border-subtle rounded px-2 py-1 text-xs"
        >
          {[25, 50, 100, 200].map((n) => (
            <option key={n} value={n}>
              show {n}
            </option>
          ))}
        </select>
      </header>
      <Card noPadding>
        {q.isLoading ? (
          <div className="p-4"><Loader /></div>
        ) : q.error ? (
          <div className="p-4"><ErrorBlock error={q.error} onRetry={() => q.refetch()} /></div>
        ) : q.data?.packages.length ? (
          <table className="w-full text-sm">
            <thead className="text-left text-fg-subtle text-[10px] uppercase tracking-wider bg-bg/40">
              <tr>
                <th className="px-4 py-2 font-medium">Package</th>
                <th className="font-medium">Publisher</th>
                <th className="font-medium text-right">Modules</th>
                <th className="font-medium">Source</th>
                <th className="px-4 font-medium text-right">Published</th>
              </tr>
            </thead>
            <tbody>
              {q.data.packages.map((p) => (
                <tr key={p.id} className="border-t border-border-subtle hover:bg-bg-elev/30">
                  <td className="px-4 py-2 max-w-[18rem]">
                    <Hash value={p.id} kind="package" copy={false} />
                  </td>
                  <td className="max-w-[14rem]">
                    <Hash value={p.publisher} kind="address" copy={false} />
                  </td>
                  <td className="text-right mono text-xs text-fg-muted">{p.modules_count}</td>
                  <td>
                    <Badge variant={p.source_verified ? 'success' : 'default'}>
                      {p.source_verified ? 'verified' : 'unverified'}
                    </Badge>
                  </td>
                  <td
                    className="px-4 text-right text-xs text-fg-subtle whitespace-nowrap"
                    title={formatTime(p.published_at)}
                  >
                    {formatRelative(p.published_at)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <div className="p-4"><Empty label="No packages indexed yet." /></div>
        )}
      </Card>
    </div>
  );
}
