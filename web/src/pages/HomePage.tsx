import { useQuery } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import {
  Activity,
  AlertTriangle,
  ArrowRight,
  Boxes,
  Clock,
  Code2,
  Layers,
  ShieldAlert,
  Zap,
} from 'lucide-react';
import ReactECharts from 'echarts-for-react';
import { Card, StatCard } from '@/components/Card';
import { Loader, ErrorBlock, Empty } from '@/components/Loader';
import { Hash } from '@/components/Hash';
import { Badge } from '@/components/Badge';
import { Sparkline } from '@/components/Sparkline';
import { SeverityPill } from '@/components/SeverityPill';
import { api } from '@/lib/api';
import { formatGas, formatNumber, formatRelative, formatTime } from '@/lib/format';
import { useWebSocket } from '@/hooks/useWebSocket';

export function HomePage() {
  const overview = useQuery({
    queryKey: ['network-overview'],
    queryFn: api.networkOverview,
    refetchInterval: 6_000,
  });
  const throughput = useQuery({
    queryKey: ['throughput-60'],
    queryFn: () => api.throughput(60),
    refetchInterval: 30_000,
  });
  const latestTx = useQuery({
    queryKey: ['tx-latest-12'],
    queryFn: () => api.latestTxs(12),
    refetchInterval: 6_000,
  });
  const recentCps = useQuery({
    queryKey: ['cp-recent-8'],
    queryFn: () => api.recentCheckpoints(8),
    refetchInterval: 6_000,
  });
  const recentPkgs = useQuery({
    queryKey: ['pkg-recent-8'],
    queryFn: () => api.recentPackages(8),
    refetchInterval: 30_000,
  });
  const findings = useQuery({
    queryKey: ['sec-recent-6'],
    queryFn: () => api.recentSecurityFindings(6),
    refetchInterval: 30_000,
  });
  const active = useQuery({
    queryKey: ['active-1h-8'],
    queryFn: () => api.activeProjects(1, 8),
    refetchInterval: 30_000,
  });
  const alerts = useQuery({
    queryKey: ['alerts-feed-6'],
    queryFn: () => api.alertsFeed(6),
    refetchInterval: 30_000,
  });

  const live = useWebSocket<{ type?: string; data?: unknown }>('/ws');

  const cp = overview.data?.checkpoint;
  const tps = (() => {
    const pts = throughput.data?.points ?? [];
    const last = pts.slice(-5).reduce((s, p) => s + p.tx_count, 0);
    return Math.round(last / 60);
  })();

  return (
    <div className="space-y-6">
      <header className="flex items-end justify-between gap-4 flex-wrap">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Sui mainnet overview</h1>
          <p className="text-sm text-fg-muted mt-1 max-w-2xl">
            Real-time activity, security signals and developer-centric metrics for the Sui blockchain.
          </p>
        </div>
        <div className="flex items-center gap-2 text-xs">
          <span className="flex items-center gap-1.5 text-fg-muted">
            <span className={`live-dot ${live.status === 'open' ? '' : 'warn'}`} />
            live stream {live.status}
          </span>
        </div>
      </header>

      <div className="grid grid-cols-2 lg:grid-cols-6 gap-4">
        <StatCard
          accent="accent"
          icon={<Clock size={12} />}
          label="Latest checkpoint"
          value={cp ? `#${formatNumber(cp.sequence_number)}` : '—'}
          hint={cp ? formatRelative(cp.timestamp_ms) : 'syncing…'}
        />
        <StatCard
          accent="info"
          icon={<Layers size={12} />}
          label="Epoch"
          value={cp?.epoch ?? '—'}
          hint={cp ? `mainnet · ${formatTime(cp.timestamp_ms)}` : '—'}
        />
        <StatCard
          accent="ok"
          icon={<Zap size={12} />}
          label="TPS · 5m"
          value={Number.isFinite(tps) ? tps : '—'}
          trend={
            <Sparkline
              data={(throughput.data?.points ?? []).map((p) => p.tx_count)}
              width={64}
              height={24}
            />
          }
          hint={`${formatNumber(overview.data?.tx_24h ?? 0)} tx in 24h`}
        />
        <StatCard
          accent="warn"
          icon={<Boxes size={12} />}
          label="Packages 24h"
          value={formatNumber(overview.data?.packages_24h ?? 0)}
          hint={`${formatNumber(overview.data?.packages_total ?? 0)} total`}
        />
        <StatCard
          accent="danger"
          icon={<ShieldAlert size={12} />}
          label="Findings (recent)"
          value={findings.data?.findings.length ?? 0}
          hint="latest security scans"
        />
        <StatCard
          accent="info"
          icon={<Activity size={12} />}
          label="Network total tx"
          value={formatNumber(cp?.network_total_transactions ?? 0)}
          hint="cumulative"
        />
      </div>

      {/* Throughput chart */}
      <Card
        title={
          <span className="flex items-center gap-2">
            <Zap size={14} className="text-accent" /> Throughput · last 60 min
          </span>
        }
        subtitle="Transactions confirmed per minute"
        action={<span className="text-fg-subtle">auto-refresh 30s</span>}
      >
        {throughput.isLoading ? (
          <Loader />
        ) : throughput.error ? (
          <ErrorBlock error={throughput.error} />
        ) : throughput.data?.points.length ? (
          <ReactECharts
            style={{ height: 220 }}
            option={{
              backgroundColor: 'transparent',
              tooltip: { trigger: 'axis' },
              grid: { left: 40, right: 16, top: 12, bottom: 24 },
              xAxis: {
                type: 'time',
                axisLabel: { color: '#9aa0b0', fontSize: 10 },
                axisLine: { lineStyle: { color: '#2a3046' } },
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
                  data: throughput.data.points.map((p) => [p.bucket, p.tx_count]),
                  areaStyle: {
                    color: {
                      type: 'linear',
                      x: 0,
                      y: 0,
                      x2: 0,
                      y2: 1,
                      colorStops: [
                        { offset: 0, color: 'rgba(92,165,255,0.5)' },
                        { offset: 1, color: 'rgba(92,165,255,0)' },
                      ],
                    },
                  },
                  itemStyle: { color: 'rgba(92,165,255,1)' },
                  lineStyle: { width: 1.5 },
                },
              ],
            }}
          />
        ) : (
          <Empty label="No throughput samples yet — start the indexer." />
        )}
      </Card>

      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
        {/* Recent transactions */}
        <Card
          className="xl:col-span-2"
          title={<span className="flex items-center gap-2"><Activity size={14} /> Recent transactions</span>}
          action={<Link to="/" className="text-accent hover:underline">live ↻</Link>}
          noPadding
        >
          {latestTx.isLoading ? (
            <div className="p-4">
              <Loader />
            </div>
          ) : latestTx.error ? (
            <div className="p-4"><ErrorBlock error={latestTx.error} /></div>
          ) : latestTx.data?.transactions.length ? (
            <table className="w-full text-sm">
              <thead className="text-left text-fg-subtle text-[10px] uppercase tracking-wider bg-bg/40">
                <tr>
                  <th className="px-4 py-2 font-medium">Digest</th>
                  <th className="font-medium">Sender</th>
                  <th className="font-medium">Status</th>
                  <th className="font-medium text-right">Gas</th>
                  <th className="px-4 font-medium text-right">Time</th>
                </tr>
              </thead>
              <tbody>
                {latestTx.data.transactions.map((t) => (
                  <tr key={t.digest} className="border-t border-border-subtle">
                    <td className="px-4 py-2 max-w-[12rem]">
                      <Hash value={t.digest} kind="tx" copy={false} />
                    </td>
                    <td className="max-w-[12rem]">
                      <Hash value={t.sender} kind="address" copy={false} />
                    </td>
                    <td>
                      <Badge variant={t.status === 'success' ? 'success' : 'danger'}>{t.status}</Badge>
                    </td>
                    <td className="mono text-xs text-fg-muted text-right">{formatGas(t.gas_used)}</td>
                    <td className="px-4 text-xs text-fg-subtle text-right whitespace-nowrap">
                      {formatRelative(t.timestamp)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : (
            <div className="p-4"><Empty label="No transactions yet — start the indexer." /></div>
          )}
        </Card>

        {/* Recent checkpoints */}
        <Card
          title={<span className="flex items-center gap-2"><Clock size={14} /> Recent checkpoints</span>}
          action={<Link to="/checkpoints" className="text-accent hover:underline">all <ArrowRight size={12} className="inline" /></Link>}
          noPadding
        >
          {recentCps.isLoading ? (
            <div className="p-4"><Loader /></div>
          ) : recentCps.error ? (
            <div className="p-4"><ErrorBlock error={recentCps.error} /></div>
          ) : recentCps.data?.checkpoints.length ? (
            <ul className="divide-y divide-border-subtle text-sm">
              {recentCps.data.checkpoints.map((cp) => (
                <li key={cp.sequence_number} className="px-4 py-2 flex items-center justify-between gap-3">
                  <Link to="/checkpoint/$seq" params={{ seq: String(cp.sequence_number) }} className="text-accent hover:underline mono">
                    #{cp.sequence_number}
                  </Link>
                  <span className="flex items-center gap-2">
                    <Badge variant="outline">epoch {cp.epoch}</Badge>
                    <span className="text-xs text-fg-subtle">{formatRelative(cp.timestamp_ms)}</span>
                  </span>
                </li>
              ))}
            </ul>
          ) : (
            <div className="p-4"><Empty /></div>
          )}
        </Card>
      </div>

      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
        <Card
          title={<span className="flex items-center gap-2"><Code2 size={14} /> Newly published packages</span>}
          action={<Link to="/packages" className="text-accent hover:underline">all <ArrowRight size={12} className="inline" /></Link>}
          noPadding
        >
          {recentPkgs.isLoading ? (
            <div className="p-4"><Loader /></div>
          ) : recentPkgs.data?.packages.length ? (
            <ul className="divide-y divide-border-subtle text-sm">
              {recentPkgs.data.packages.map((p) => (
                <li key={p.id} className="px-4 py-2 flex items-center justify-between gap-3">
                  <Hash value={p.id} kind="package" copy={false} />
                  <span className="flex items-center gap-2 text-xs">
                    <Badge variant="outline">{p.modules_count} mod</Badge>
                    <span className="text-fg-subtle">{formatRelative(p.published_at)}</span>
                  </span>
                </li>
              ))}
            </ul>
          ) : (
            <div className="p-4"><Empty /></div>
          )}
        </Card>

        <Card
          title={<span className="flex items-center gap-2"><Activity size={14} /> Top active · 1h</span>}
          action={<Link to="/analytics/active" className="text-accent hover:underline">full ranking</Link>}
          noPadding
        >
          {active.isLoading ? (
            <div className="p-4"><Loader /></div>
          ) : active.data?.rankings.length ? (
            <ol className="divide-y divide-border-subtle text-sm">
              {active.data.rankings.map((r, i) => (
                <li key={r.package_id} className="px-4 py-2 flex items-center gap-3">
                  <span className="text-fg-subtle text-xs w-5 text-right">{i + 1}</span>
                  <Hash value={r.package_id} kind="package" copy={false} className="flex-1 truncate" />
                  <span className="text-xs text-fg-muted shrink-0">{formatNumber(r.calls)} calls</span>
                </li>
              ))}
            </ol>
          ) : (
            <div className="p-4"><Empty label="No activity yet." /></div>
          )}
        </Card>

        <Card
          title={<span className="flex items-center gap-2"><ShieldAlert size={14} /> Latest security findings</span>}
          action={<Link to="/security" className="text-accent hover:underline">all</Link>}
          noPadding
        >
          {findings.isLoading ? (
            <div className="p-4"><Loader /></div>
          ) : findings.data?.findings.length ? (
            <ul className="divide-y divide-border-subtle text-sm">
              {findings.data.findings.map((f) => (
                <li key={`${f.package_id}-${f.finding.rule_id}-${f.scanned_at}`} className="px-4 py-2.5">
                  <div className="flex items-center gap-2 mb-1">
                    <SeverityPill value={f.finding.severity} />
                    <span className="text-fg-muted text-xs">{f.finding.rule_id}</span>
                  </div>
                  <div className="flex items-center justify-between gap-2">
                    <span className="text-fg-muted text-xs truncate">{f.finding.rule_name}</span>
                    <Hash value={f.package_id} kind="package" copy={false} className="text-xs" />
                  </div>
                </li>
              ))}
            </ul>
          ) : (
            <div className="p-4"><Empty label="No findings yet." /></div>
          )}
        </Card>
      </div>

      <Card
        title={<span className="flex items-center gap-2"><AlertTriangle size={14} className="text-warn" /> Public alerts feed</span>}
        action={<Link to="/alerts" className="text-accent hover:underline">manage watchlists</Link>}
      >
        {alerts.isLoading ? (
          <Loader />
        ) : alerts.data?.alerts.length ? (
          <ul className="space-y-2 text-sm">
            {alerts.data.alerts.map((a) => (
              <li key={a.id} className="border border-border-subtle rounded-md p-3">
                <div className="flex items-center justify-between gap-2">
                  <span className="flex items-center gap-2">
                    <Badge variant={a.delivered ? 'success' : 'warn'}>{a.delivered ? 'delivered' : 'pending'}</Badge>
                    <strong className="text-fg">{a.payload?.title ?? a.rule_id}</strong>
                  </span>
                  <span className="text-xs text-fg-subtle">{formatRelative(a.fired_at)}</span>
                </div>
                {a.payload?.body && (
                  <p className="mt-1 text-fg-muted text-xs whitespace-pre-wrap">{a.payload.body}</p>
                )}
              </li>
            ))}
          </ul>
        ) : (
          <Empty label="No alerts fired yet — quiet day on Sui." />
        )}
      </Card>
    </div>
  );
}
