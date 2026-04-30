import { useQuery } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { Globe, Layers } from 'lucide-react';
import ReactECharts from 'echarts-for-react';
import { Card, StatCard } from '@/components/Card';
import { Loader, Empty } from '@/components/Loader';
import { Badge } from '@/components/Badge';
import { api } from '@/lib/api';
import { formatNumber, formatRelative, formatTime } from '@/lib/format';

export function NetworkPage() {
  const overview = useQuery({
    queryKey: ['net-overview'],
    queryFn: api.networkOverview,
    refetchInterval: 6_000,
  });
  const throughput = useQuery({
    queryKey: ['net-throughput-24'],
    queryFn: () => api.throughput(60 * 24),
    refetchInterval: 60_000,
  });
  const cps = useQuery({
    queryKey: ['net-cps'],
    queryFn: () => api.recentCheckpoints(20),
    refetchInterval: 6_000,
  });
  const protocols = useQuery({
    queryKey: ['net-protocols'],
    queryFn: api.protocols,
  });

  const cp = overview.data?.checkpoint;
  const totalTxLast24h = (throughput.data?.points ?? []).reduce((s, p) => s + p.tx_count, 0);

  return (
    <div className="space-y-6">
      <header>
        <h1 className="text-xl font-semibold tracking-tight flex items-center gap-2">
          <Globe size={18} className="text-accent" /> Sui network
        </h1>
        <p className="text-sm text-fg-muted mt-1">
          Live network state, throughput and registered protocols.
        </p>
      </header>

      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          accent="accent"
          label="Latest checkpoint"
          value={cp ? `#${formatNumber(cp.sequence_number)}` : '—'}
          hint={cp ? formatRelative(cp.timestamp_ms) : '—'}
        />
        <StatCard
          accent="info"
          label="Epoch"
          value={cp?.epoch ?? '—'}
          hint={cp ? formatTime(cp.timestamp_ms) : '—'}
        />
        <StatCard
          accent="ok"
          label="Tx · indexed 24h"
          value={formatNumber(totalTxLast24h)}
          hint="from per-minute aggregation"
        />
        <StatCard
          accent="warn"
          label="Network total tx"
          value={formatNumber(cp?.network_total_transactions ?? 0)}
          hint="cumulative on-chain"
        />
      </div>

      <Card title="Throughput · 24h (indexed by sui-trace)">
        {throughput.isLoading ? (
          <Loader />
        ) : throughput.data?.points.length ? (
          <ReactECharts
            style={{ height: 280 }}
            option={{
              backgroundColor: 'transparent',
              tooltip: { trigger: 'axis' },
              grid: { left: 48, right: 16, top: 12, bottom: 28 },
              xAxis: {
                type: 'time',
                axisLabel: { color: '#9aa0b0', fontSize: 10 },
              },
              yAxis: {
                type: 'value',
                axisLabel: { color: '#9aa0b0', fontSize: 10 },
                splitLine: { lineStyle: { color: '#1c2034' } },
              },
              series: [
                {
                  type: 'line',
                  smooth: true,
                  showSymbol: false,
                  lineStyle: { color: 'rgba(92,165,255,1)', width: 1.5 },
                  areaStyle: {
                    color: {
                      type: 'linear',
                      x: 0,
                      y: 0,
                      x2: 0,
                      y2: 1,
                      colorStops: [
                        { offset: 0, color: 'rgba(92,165,255,0.4)' },
                        { offset: 1, color: 'rgba(92,165,255,0)' },
                      ],
                    },
                  },
                  data: throughput.data.points.map((p) => [p.bucket, p.tx_count]),
                },
              ],
            }}
          />
        ) : (
          <Empty label="No throughput samples yet." />
        )}
      </Card>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card
          title={<span className="flex items-center gap-2"><Layers size={14} /> Recent checkpoints</span>}
          noPadding
        >
          {cps.isLoading ? (
            <div className="p-4"><Loader /></div>
          ) : cps.data?.checkpoints.length ? (
            <ul className="divide-y divide-border-subtle text-sm">
              {cps.data.checkpoints.map((c) => (
                <li key={c.sequence_number} className="px-4 py-2 flex items-center justify-between gap-3">
                  <Link
                    to="/checkpoint/$seq"
                    params={{ seq: String(c.sequence_number) }}
                    className="mono text-accent hover:underline"
                  >
                    #{c.sequence_number}
                  </Link>
                  <span className="flex items-center gap-2 text-xs">
                    <Badge variant="outline">epoch {c.epoch}</Badge>
                    <span className="text-fg-subtle">{formatRelative(c.timestamp_ms)}</span>
                  </span>
                </li>
              ))}
            </ul>
          ) : (
            <div className="p-4"><Empty /></div>
          )}
        </Card>

        <Card title="Tracked protocols" subtitle="Configured in the protocols table" noPadding>
          {protocols.isLoading ? (
            <div className="p-4"><Loader /></div>
          ) : protocols.data?.protocols.length ? (
            <ul className="divide-y divide-border-subtle text-sm">
              {protocols.data.protocols.map((p) => (
                <li key={p.id} className="px-4 py-2 flex items-center justify-between gap-3">
                  <div>
                    <div className="font-medium">{p.name}</div>
                    <div className="text-xs text-fg-subtle mono">{p.id}</div>
                  </div>
                  <div className="flex items-center gap-2 text-xs">
                    <Badge variant="outline">{p.category}</Badge>
                    {p.website && (
                      <a
                        href={p.website}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-accent hover:underline"
                      >
                        site ↗
                      </a>
                    )}
                  </div>
                </li>
              ))}
            </ul>
          ) : (
            <div className="p-4"><Empty label="No protocols registered yet." /></div>
          )}
        </Card>
      </div>
    </div>
  );
}
