import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import ReactECharts from 'echarts-for-react';
import { Code2 } from 'lucide-react';
import { Card } from '@/components/Card';
import { Loader, ErrorBlock, Empty } from '@/components/Loader';
import { api } from '@/lib/api';
import { formatNumber } from '@/lib/format';

export function DeploymentsPage() {
  const [days, setDays] = useState(30);
  const q = useQuery({
    queryKey: ['deployments', days],
    queryFn: () => api.deploymentStats(days),
  });

  const total = q.data?.stats.reduce((s, d) => s + d.package_count, 0) ?? 0;
  const uniquePublishers =
    q.data?.stats.reduce((s, d) => Math.max(s, d.unique_publishers), 0) ?? 0;

  return (
    <div className="space-y-6">
      <header className="flex items-end justify-between gap-4 flex-wrap">
        <div>
          <h1 className="text-xl font-semibold tracking-tight flex items-center gap-2">
            <Code2 size={18} className="text-accent" /> Daily deployments
          </h1>
          <p className="text-sm text-fg-muted mt-1">
            Newly published Move packages and their unique publishers per day.
          </p>
        </div>
        <select
          value={days}
          onChange={(e) => setDays(Number(e.target.value))}
          className="bg-bg-elev border border-border-subtle rounded px-2 py-1 text-xs"
        >
          <option value={7}>last 7 days</option>
          <option value={30}>last 30 days</option>
          <option value={90}>last 90 days</option>
          <option value={180}>last 180 days</option>
        </select>
      </header>

      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <Stat label={`Packages · ${days}d`} value={formatNumber(total)} />
        <Stat label="Peak unique publishers/day" value={formatNumber(uniquePublishers)} />
        <Stat
          label="Average per day"
          value={q.data?.stats.length ? Math.round(total / q.data.stats.length) : '—'}
        />
        <Stat label="Days with data" value={q.data?.stats.length ?? '—'} />
      </div>

      <Card title="Daily volume">
        {q.isLoading ? (
          <Loader />
        ) : q.error ? (
          <ErrorBlock error={q.error} />
        ) : q.data?.stats.length ? (
          <ReactECharts
            style={{ height: 360 }}
            option={{
              backgroundColor: 'transparent',
              tooltip: { trigger: 'axis' },
              grid: { left: 40, right: 16, top: 28, bottom: 32 },
              legend: { textStyle: { color: '#9aa0b0' }, top: 0 },
              xAxis: {
                type: 'category',
                data: q.data.stats.map((s) => s.day),
                axisLabel: { color: '#9aa0b0', fontSize: 10 },
              },
              yAxis: {
                type: 'value',
                axisLabel: { color: '#9aa0b0', fontSize: 10 },
                splitLine: { lineStyle: { color: '#1c2034' } },
              },
              series: [
                {
                  type: 'bar',
                  name: 'Packages',
                  itemStyle: { color: 'rgba(92,165,255,0.85)' },
                  data: q.data.stats.map((s) => s.package_count),
                },
                {
                  type: 'line',
                  name: 'Publishers',
                  smooth: true,
                  itemStyle: { color: 'rgba(92,210,162,1)' },
                  data: q.data.stats.map((s) => s.unique_publishers),
                },
              ],
            }}
          />
        ) : (
          <Empty label="No deployments recorded yet — start the indexer." />
        )}
      </Card>

      <Card title="Daily table" noPadding>
        {q.data?.stats.length ? (
          <table className="w-full text-sm">
            <thead className="text-fg-subtle text-[10px] uppercase tracking-wider bg-bg/40">
              <tr>
                <th className="text-left px-4 py-2 font-medium">Day</th>
                <th className="text-right font-medium">Packages</th>
                <th className="text-right pr-4 font-medium">Unique publishers</th>
              </tr>
            </thead>
            <tbody>
              {q.data.stats
                .slice()
                .reverse()
                .map((s) => (
                  <tr key={s.day} className="border-t border-border-subtle">
                    <td className="px-4 py-2">{s.day}</td>
                    <td className="text-right mono">{formatNumber(s.package_count)}</td>
                    <td className="text-right pr-4 mono">{formatNumber(s.unique_publishers)}</td>
                  </tr>
                ))}
            </tbody>
          </table>
        ) : null}
      </Card>
    </div>
  );
}

function Stat({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className="bg-bg-subtle border border-border-subtle rounded-lg p-4">
      <div className="text-[10px] uppercase tracking-wider text-fg-subtle">{label}</div>
      <div className="mt-1.5 mono text-xl font-semibold">{value}</div>
    </div>
  );
}
