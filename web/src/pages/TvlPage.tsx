import { useEffect, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import ReactECharts from 'echarts-for-react';
import { BarChart3, ExternalLink } from 'lucide-react';
import { Card } from '@/components/Card';
import { Loader, ErrorBlock, Empty } from '@/components/Loader';
import { Badge } from '@/components/Badge';
import { api } from '@/lib/api';
import { formatNumber, formatRelative, formatUsd } from '@/lib/format';

const WINDOW_OPTIONS = [
  { hours: 6, label: '6h' },
  { hours: 24, label: '24h' },
  { hours: 168, label: '7d' },
  { hours: 720, label: '30d' },
];

export function TvlPage() {
  const protocols = useQuery({ queryKey: ['protocols'], queryFn: api.protocols });
  const [protocol, setProtocol] = useState('');
  const [hours, setHours] = useState(24);

  // Default to first protocol once loaded
  useEffect(() => {
    if (!protocol && protocols.data?.protocols.length) {
      setProtocol(protocols.data.protocols[0].id);
    }
  }, [protocol, protocols.data]);

  const q = useQuery({
    queryKey: ['tvl', protocol, hours],
    queryFn: () => api.tvlHistory(protocol, hours),
    enabled: protocol.length > 0,
    refetchInterval: 60_000,
  });

  const points = q.data?.history ?? [];
  const latest = points[points.length - 1];
  const first = points[0];
  const drop =
    first && latest && first.tvl_usd > 0
      ? ((first.tvl_usd - latest.tvl_usd) / first.tvl_usd) * 100
      : 0;
  const proto = protocols.data?.protocols.find((p) => p.id === protocol);

  return (
    <div className="space-y-6">
      <header className="flex items-end justify-between gap-4 flex-wrap">
        <div>
          <h1 className="text-xl font-semibold tracking-tight flex items-center gap-2">
            <BarChart3 size={18} className="text-accent" /> TVL tracker
          </h1>
          <p className="text-sm text-fg-muted mt-1">
            Historical TVL polled from DefiLlama and combined with Sui-trace alerts.
          </p>
        </div>
        <div className="flex items-center gap-2">
          <select
            value={protocol}
            onChange={(e) => setProtocol(e.target.value)}
            className="bg-bg-elev border border-border-subtle rounded px-2 py-1 text-xs"
          >
            {protocols.data?.protocols.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name}
              </option>
            ))}
          </select>
          <select
            value={hours}
            onChange={(e) => setHours(Number(e.target.value))}
            className="bg-bg-elev border border-border-subtle rounded px-2 py-1 text-xs"
          >
            {WINDOW_OPTIONS.map((o) => (
              <option key={o.hours} value={o.hours}>
                {o.label}
              </option>
            ))}
          </select>
        </div>
      </header>

      {proto && (
        <Card>
          <div className="flex items-center gap-3 flex-wrap">
            <h2 className="text-lg font-medium">{proto.name}</h2>
            <Badge variant="outline">{proto.category}</Badge>
            <span className="text-xs text-fg-subtle mono">{proto.id}</span>
            {proto.website && (
              <a
                href={proto.website}
                target="_blank"
                rel="noopener noreferrer"
                className="text-xs text-accent hover:underline inline-flex items-center gap-1"
              >
                website <ExternalLink size={10} />
              </a>
            )}
            {proto.defillama_slug && (
              <a
                href={`https://defillama.com/protocol/${proto.defillama_slug}`}
                target="_blank"
                rel="noopener noreferrer"
                className="text-xs text-accent hover:underline inline-flex items-center gap-1"
              >
                defillama <ExternalLink size={10} />
              </a>
            )}
          </div>
          {latest && (
            <div className="mt-4 grid grid-cols-2 lg:grid-cols-4 gap-4">
              <Stat label="Latest TVL" value={formatUsd(latest.tvl_usd)} />
              <Stat label="As of" value={formatRelative(latest.timestamp)} />
              <Stat
                label="Window change"
                value={
                  <span className={drop > 0 ? 'text-warn' : 'text-ok'}>
                    {drop > 0 ? '−' : '+'}
                    {Math.abs(drop).toFixed(2)}%
                  </span>
                }
              />
              <Stat label="Samples" value={formatNumber(points.length)} />
            </div>
          )}
        </Card>
      )}

      <Card title="History">
        {q.isLoading ? (
          <Loader />
        ) : q.error ? (
          <ErrorBlock error={q.error} />
        ) : points.length ? (
          <ReactECharts
            style={{ height: 380 }}
            option={{
              backgroundColor: 'transparent',
              tooltip: {
                trigger: 'axis',
                valueFormatter: (v: number) => formatUsd(v),
              },
              grid: { left: 64, right: 16, top: 8, bottom: 32 },
              xAxis: { type: 'time', axisLabel: { color: '#9aa0b0', fontSize: 10 } },
              yAxis: {
                type: 'value',
                axisLabel: {
                  color: '#9aa0b0',
                  fontSize: 10,
                  formatter: (v: number) => formatUsd(v),
                },
                splitLine: { lineStyle: { color: '#1c2034' } },
              },
              series: [
                {
                  type: 'line',
                  smooth: true,
                  showSymbol: false,
                  data: points.map((p) => [p.timestamp, p.tvl_usd]),
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
                },
              ],
            }}
          />
        ) : (
          <Empty
            label={
              protocol
                ? 'No TVL samples yet — let the DefiLlama poller catch up.'
                : 'Pick a protocol to see history.'
            }
          />
        )}
      </Card>
    </div>
  );
}

function Stat({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className="bg-bg-subtle border border-border-subtle rounded-lg p-3">
      <div className="text-[10px] uppercase tracking-wider text-fg-subtle">{label}</div>
      <div className="mt-1 mono text-base">{value}</div>
    </div>
  );
}
