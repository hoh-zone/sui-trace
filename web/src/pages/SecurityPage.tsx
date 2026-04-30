import { useQuery } from '@tanstack/react-query';
import { ShieldAlert } from 'lucide-react';
import { useState } from 'react';
import ReactECharts from 'echarts-for-react';
import { Card } from '@/components/Card';
import { Loader, ErrorBlock, Empty } from '@/components/Loader';
import { Hash } from '@/components/Hash';
import { SeverityPill } from '@/components/SeverityPill';
import { api, type Severity } from '@/lib/api';
import { formatRelative } from '@/lib/format';

const SEV_ORDER: Severity[] = ['critical', 'high', 'medium', 'low', 'info'];
const SEV_COLOR: Record<Severity, string> = {
  critical: '#f87180',
  high: '#f5b85c',
  medium: '#f5b85c',
  low: '#9aa0b0',
  info: '#82a8ff',
};

export function SecurityPage() {
  const [days, setDays] = useState(30);
  const [sevFilter, setSevFilter] = useState<Severity | 'all'>('all');

  const recent = useQuery({
    queryKey: ['sec-recent'],
    queryFn: () => api.recentSecurityFindings(100),
    refetchInterval: 30_000,
  });
  const score = useQuery({
    queryKey: ['sec-score', days],
    queryFn: () => api.securityScoreboard(days),
  });

  const filtered = (recent.data?.findings ?? []).filter(
    (f) => sevFilter === 'all' || f.finding.severity === sevFilter,
  );

  const sevCounts = score.data?.severity_counts ?? [];
  const totalFindings = sevCounts.reduce((s, c) => s + c.count, 0);

  return (
    <div className="space-y-6">
      <header className="flex items-end justify-between gap-4 flex-wrap">
        <div>
          <h1 className="text-xl font-semibold tracking-tight flex items-center gap-2">
            <ShieldAlert size={18} className="text-warn" /> Security feed
          </h1>
          <p className="text-sm text-fg-muted mt-1">
            Latest static-analysis findings and a scoreboard of the most-triggered rules.
          </p>
        </div>
        <select
          value={days}
          onChange={(e) => setDays(Number(e.target.value))}
          className="bg-bg-elev border border-border-subtle rounded px-2 py-1 text-xs"
        >
          {[7, 30, 90].map((n) => (
            <option key={n} value={n}>
              window {n}d
            </option>
          ))}
        </select>
      </header>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <Card title={`Severity mix · ${days}d`} subtitle={`${totalFindings} findings total`}>
          {score.isLoading ? (
            <Loader />
          ) : sevCounts.length ? (
            <ReactECharts
              style={{ height: 220 }}
              option={{
                backgroundColor: 'transparent',
                tooltip: { trigger: 'item' },
                legend: { textStyle: { color: '#9aa0b0' }, bottom: 0, type: 'scroll' },
                series: [
                  {
                    type: 'pie',
                    radius: ['45%', '75%'],
                    avoidLabelOverlap: true,
                    label: { color: '#9aa0b0', formatter: '{b}\n{c}' },
                    data: SEV_ORDER.map((sev) => ({
                      value: sevCounts.find((c) => c.severity === sev)?.count ?? 0,
                      name: sev,
                      itemStyle: { color: SEV_COLOR[sev] },
                    })).filter((d) => d.value > 0),
                  },
                ],
              }}
            />
          ) : (
            <Empty />
          )}
        </Card>

        <Card title={`Top triggered rules · ${days}d`} className="lg:col-span-2" noPadding>
          {score.isLoading ? (
            <div className="p-4"><Loader /></div>
          ) : score.data?.rule_rankings.length ? (
            <table className="w-full text-sm">
              <thead className="text-left text-fg-subtle text-[10px] uppercase tracking-wider bg-bg/40">
                <tr>
                  <th className="px-4 py-2 font-medium">Rule</th>
                  <th className="font-medium">Severity</th>
                  <th className="font-medium text-right">Hits</th>
                  <th className="px-4 font-medium" />
                </tr>
              </thead>
              <tbody>
                {score.data.rule_rankings.map((r) => {
                  const max = Math.max(...score.data!.rule_rankings.map((x) => x.hits), 1);
                  const pct = (r.hits / max) * 100;
                  return (
                    <tr key={r.rule_id} className="border-t border-border-subtle">
                      <td className="px-4 py-2">
                        <span className="mono text-xs text-fg-muted">{r.rule_id}</span>{' '}
                        <span>{r.rule_name}</span>
                      </td>
                      <td>
                        <SeverityPill value={r.severity} />
                      </td>
                      <td className="text-right mono text-xs">{r.hits}</td>
                      <td className="px-4 w-32">
                        <div className="h-1 bg-bg-elev rounded">
                          <div
                            className="h-full rounded"
                            style={{ width: `${pct}%`, background: SEV_COLOR[r.severity] }}
                          />
                        </div>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          ) : (
            <div className="p-4"><Empty /></div>
          )}
        </Card>
      </div>

      <Card
        title="Recent findings"
        action={
          <select
            value={sevFilter}
            onChange={(e) => setSevFilter(e.target.value as Severity | 'all')}
            className="bg-bg-elev border border-border-subtle rounded px-2 py-1 text-xs"
          >
            <option value="all">all severities</option>
            {SEV_ORDER.map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </select>
        }
        noPadding
      >
        {recent.isLoading ? (
          <div className="p-4"><Loader /></div>
        ) : recent.error ? (
          <div className="p-4"><ErrorBlock error={recent.error} onRetry={() => recent.refetch()} /></div>
        ) : filtered.length ? (
          <ul className="divide-y divide-border-subtle">
            {filtered.map((rf) => (
              <li
                key={`${rf.package_id}-${rf.finding.rule_id}-${rf.scanned_at}`}
                className="px-4 py-3 text-sm"
              >
                <div className="flex items-center gap-2 mb-1 flex-wrap">
                  <SeverityPill value={rf.finding.severity} />
                  <span className="mono text-xs text-fg-muted">{rf.finding.rule_id}</span>
                  <span className="font-medium">{rf.finding.rule_name}</span>
                  <span className="text-xs text-fg-subtle ml-auto">{formatRelative(rf.scanned_at)}</span>
                </div>
                <div className="text-fg-muted text-xs">
                  package <Hash value={rf.package_id} kind="package" copy={false} /> ·{' '}
                  <span className="mono">{rf.finding.location}</span>
                </div>
                <p className="text-fg mt-1">{rf.finding.message}</p>
                {rf.finding.suggestion && (
                  <p className="text-fg-subtle text-xs mt-1">→ {rf.finding.suggestion}</p>
                )}
              </li>
            ))}
          </ul>
        ) : (
          <div className="p-4"><Empty label="No findings yet." /></div>
        )}
      </Card>
    </div>
  );
}
