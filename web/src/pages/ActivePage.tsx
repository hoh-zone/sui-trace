import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import ReactECharts from 'echarts-for-react';
import { Activity } from 'lucide-react';
import { Card } from '@/components/Card';
import { Loader, ErrorBlock, Empty } from '@/components/Loader';
import { Hash } from '@/components/Hash';
import { api } from '@/lib/api';
import { formatGas, formatNumber, shortAddr } from '@/lib/format';

export function ActivePage() {
  const [hours, setHours] = useState(24);
  const q = useQuery({
    queryKey: ['active', hours],
    queryFn: () => api.activeProjects(hours, 50),
    refetchInterval: 30_000,
  });

  return (
    <div className="space-y-6">
      <header className="flex items-end justify-between gap-4 flex-wrap">
        <div>
          <h1 className="text-xl font-semibold tracking-tight flex items-center gap-2">
            <Activity size={18} className="text-accent" /> Active packages
          </h1>
          <p className="text-sm text-fg-muted mt-1">
            Packages ranked by call volume in the selected window. Backed by event aggregation.
          </p>
        </div>
        <select
          value={hours}
          onChange={(e) => setHours(Number(e.target.value))}
          className="bg-bg-elev border border-border-subtle rounded px-2 py-1 text-xs"
        >
          <option value={1}>last 1h</option>
          <option value={24}>last 24h</option>
          <option value={168}>last 7d</option>
        </select>
      </header>

      <Card title={`Top 20 by calls · last ${hours}h`}>
        {q.isLoading ? (
          <Loader />
        ) : q.error ? (
          <ErrorBlock error={q.error} />
        ) : q.data?.rankings.length ? (
          <ReactECharts
            style={{ height: 480 }}
            option={{
              backgroundColor: 'transparent',
              tooltip: { trigger: 'axis' },
              grid: { left: 130, right: 20, top: 8, bottom: 24 },
              xAxis: { type: 'value', axisLabel: { color: '#9aa0b0', fontSize: 10 } },
              yAxis: {
                type: 'category',
                inverse: true,
                data: q.data.rankings.slice(0, 20).map((r) => shortAddr(r.package_id, 6)),
                axisLabel: { color: '#9aa0b0', fontSize: 10 },
              },
              series: [
                {
                  type: 'bar',
                  data: q.data.rankings.slice(0, 20).map((r) => r.calls),
                  itemStyle: { color: 'rgba(92,165,255,0.85)' },
                  name: 'Calls',
                },
              ],
            }}
          />
        ) : (
          <Empty />
        )}
      </Card>

      <Card title="Full ranking" noPadding>
        {q.data?.rankings.length ? (
          <table className="w-full text-sm">
            <thead className="text-fg-subtle text-[10px] uppercase tracking-wider bg-bg/40">
              <tr>
                <th className="text-left px-4 py-2 font-medium">Package</th>
                <th className="text-right font-medium">Calls</th>
                <th className="text-right font-medium">Unique callers</th>
                <th className="text-right pr-4 font-medium">Gas total</th>
              </tr>
            </thead>
            <tbody>
              {q.data.rankings.map((r) => (
                <tr key={r.package_id} className="border-t border-border-subtle">
                  <td className="px-4 py-2 max-w-[18rem]">
                    <Hash value={r.package_id} kind="package" copy={false} />
                  </td>
                  <td className="text-right mono">{formatNumber(r.calls)}</td>
                  <td className="text-right mono">{formatNumber(r.unique_callers)}</td>
                  <td className="text-right pr-4 mono">{formatGas(r.gas_total)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : null}
      </Card>
    </div>
  );
}
