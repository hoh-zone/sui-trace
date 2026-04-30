import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import ReactECharts from 'echarts-for-react';
import { Loader, ErrorBlock, Empty } from './Loader';
import { api } from '@/lib/api';
import { formatUsd } from '@/lib/format';

interface TvlSparklineProps {
  protocolId: string;
}

export default function TvlSparkline({ protocolId }: TvlSparklineProps) {
  const [hours, setHours] = useState<number>(24 * 7);
  const q = useQuery({
    queryKey: ['watch-tvl', protocolId, hours],
    queryFn: () => api.tvlHistory(protocolId, hours),
    refetchInterval: 60_000,
  });

  if (q.isLoading) return <Loader />;
  if (q.error) return <ErrorBlock error={q.error} onRetry={() => q.refetch()} />;
  const points = q.data?.history ?? [];
  if (!points.length)
    return (
      <Empty
        label="No TVL snapshots yet."
        hint={
          <>
            Set <code className="mono">defillama_slug</code> on this protocol so the
            poller starts collecting points (next tick within ~5 min).
          </>
        }
      />
    );

  return (
    <div>
      <div className="mb-2 flex items-center justify-between">
        <div className="text-xs text-fg-subtle">
          Latest{' '}
          <span className="mono text-fg">{formatUsd(points[points.length - 1].tvl_usd)}</span>
        </div>
        <div className="flex gap-1 text-xs">
          {[24, 24 * 7, 24 * 30].map((h) => (
            <button
              key={h}
              onClick={() => setHours(h)}
              className={`px-2 py-0.5 rounded border ${
                hours === h
                  ? 'border-accent text-accent'
                  : 'border-border-subtle text-fg-subtle hover:text-fg'
              }`}
            >
              {h === 24 ? '24h' : h === 24 * 7 ? '7d' : '30d'}
            </button>
          ))}
        </div>
      </div>
      <ReactECharts
        style={{ height: 280 }}
        option={{
          grid: { left: 56, right: 12, top: 16, bottom: 32 },
          tooltip: {
            trigger: 'axis',
            backgroundColor: '#0e101a',
            borderColor: '#2c3246',
            textStyle: { color: '#ebeef5' },
            valueFormatter: (v: number) => `$${v.toLocaleString()}`,
          },
          xAxis: {
            type: 'time',
            axisLine: { lineStyle: { color: '#2c3246' } },
            axisLabel: { color: '#7a8291' },
          },
          yAxis: {
            type: 'value',
            axisLabel: {
              color: '#7a8291',
              formatter: (v: number) =>
                v >= 1e9
                  ? `${(v / 1e9).toFixed(1)}B`
                  : v >= 1e6
                    ? `${(v / 1e6).toFixed(1)}M`
                    : v >= 1e3
                      ? `${(v / 1e3).toFixed(0)}k`
                      : `${v}`,
            },
            splitLine: { lineStyle: { color: '#1c2030' } },
          },
          series: [
            {
              type: 'line',
              smooth: true,
              showSymbol: false,
              lineStyle: { color: '#5ca5ff', width: 2 },
              areaStyle: {
                color: {
                  type: 'linear',
                  x: 0,
                  y: 0,
                  x2: 0,
                  y2: 1,
                  colorStops: [
                    { offset: 0, color: 'rgba(92,165,255,0.35)' },
                    { offset: 1, color: 'rgba(92,165,255,0.02)' },
                  ],
                },
              },
              data: points.map((p) => [p.timestamp, p.tvl_usd]),
            },
          ],
        }}
      />
    </div>
  );
}
