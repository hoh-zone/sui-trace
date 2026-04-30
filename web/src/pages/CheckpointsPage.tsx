import { useQuery } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { Clock } from 'lucide-react';
import { Card } from '@/components/Card';
import { Loader, ErrorBlock, Empty } from '@/components/Loader';
import { Badge } from '@/components/Badge';
import { Hash } from '@/components/Hash';
import { api } from '@/lib/api';
import { formatNumber, formatRelative, formatTime } from '@/lib/format';

export function CheckpointsPage() {
  const q = useQuery({
    queryKey: ['cps-list'],
    queryFn: () => api.recentCheckpoints(50),
    refetchInterval: 6_000,
  });

  return (
    <div className="space-y-4">
      <header>
        <h1 className="text-xl font-semibold tracking-tight flex items-center gap-2">
          <Clock size={18} className="text-accent" /> Recent checkpoints
        </h1>
        <p className="text-sm text-fg-muted mt-1">
          Latest checkpoints ingested by the indexer. Click any row to drill into its transactions.
        </p>
      </header>
      <Card noPadding>
        {q.isLoading ? (
          <div className="p-4"><Loader /></div>
        ) : q.error ? (
          <div className="p-4"><ErrorBlock error={q.error} onRetry={() => q.refetch()} /></div>
        ) : q.data?.checkpoints.length ? (
          <table className="w-full text-sm">
            <thead className="text-left text-fg-subtle text-[10px] uppercase tracking-wider bg-bg/40">
              <tr>
                <th className="px-4 py-2 font-medium">Sequence</th>
                <th className="font-medium">Digest</th>
                <th className="font-medium">Epoch</th>
                <th className="font-medium text-right">Network total</th>
                <th className="px-4 font-medium text-right">Time</th>
              </tr>
            </thead>
            <tbody>
              {q.data.checkpoints.map((cp) => (
                <tr key={cp.sequence_number} className="border-t border-border-subtle hover:bg-bg-elev/30">
                  <td className="px-4 py-2">
                    <Link
                      to="/checkpoint/$seq"
                      params={{ seq: String(cp.sequence_number) }}
                      className="mono text-accent hover:underline"
                    >
                      #{cp.sequence_number}
                    </Link>
                  </td>
                  <td className="max-w-[18rem]">
                    <Hash value={cp.digest} copy={false} />
                  </td>
                  <td>
                    <Badge variant="outline">epoch {cp.epoch}</Badge>
                  </td>
                  <td className="text-right mono text-xs text-fg-muted">
                    {formatNumber(cp.network_total_transactions)}
                  </td>
                  <td
                    className="px-4 text-right text-xs text-fg-subtle whitespace-nowrap"
                    title={formatTime(cp.timestamp_ms)}
                  >
                    {formatRelative(cp.timestamp_ms)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <div className="p-4"><Empty label="No checkpoints indexed yet." /></div>
        )}
      </Card>
    </div>
  );
}
