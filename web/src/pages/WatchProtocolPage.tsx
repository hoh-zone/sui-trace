import { useQuery } from '@tanstack/react-query';
import { useParams, Link } from '@tanstack/react-router';
import { lazy, Suspense } from 'react';
import {
  Activity,
  AlertOctagon,
  AlertTriangle,
  ArrowLeft,
  CheckCircle2,
  Code2,
  Globe,
  Hash as HashIcon,
  ShieldCheck,
  Wallet,
} from 'lucide-react';
import { Card } from '@/components/Card';
import { Loader, ErrorBlock, Empty } from '@/components/Loader';
import { Hash } from '@/components/Hash';
import { Badge } from '@/components/Badge';
import { Tabs } from '@/components/Tabs';
import { JsonView } from '@/components/JsonView';
import { CopyButton } from '@/components/CopyButton';
import { api, type CodeEvent, type ProtocolActivity } from '@/lib/api';
import { formatNumber, formatRelative, formatTime, formatUsd } from '@/lib/format';
import { cn } from '@/lib/cn';

const TvlSparkline = lazy(() => import('@/components/TvlSparkline'));

export function WatchProtocolPage() {
  const { id } = useParams({ from: '/watch/$id' });
  const q = useQuery({
    queryKey: ['watch-detail', id],
    queryFn: () => api.watchProtocol(id),
    refetchInterval: 30_000,
  });

  if (q.isLoading) return <Loader />;
  if (q.error) return <ErrorBlock error={q.error} onRetry={() => q.refetch()} />;
  const data = q.data!;
  const p = data.protocol;

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-2 text-xs text-fg-subtle">
        <Link to="/watch" className="hover:text-fg inline-flex items-center gap-1">
          <ArrowLeft size={12} /> watchlist
        </Link>
      </div>

      <header className="flex items-start gap-4 flex-wrap justify-between">
        <div className="flex items-start gap-3 min-w-0">
          <Logo url={p.logo_url} name={p.name} />
          <div className="min-w-0">
            <h1 className="text-2xl font-semibold tracking-tight flex items-center gap-2 flex-wrap">
              {p.name}
              <Badge variant={p.watched ? 'accent' : 'default'}>
                {p.watched ? 'watched' : 'inactive'}
              </Badge>
              <RiskPill level={p.risk_level} />
              <span className="text-xs text-fg-subtle uppercase tracking-wider">{p.category}</span>
            </h1>
            <div className="mt-1 text-xs text-fg-subtle flex items-center gap-2 flex-wrap">
              <span className="mono">{p.id}</span>
              <CopyButton value={p.id} silent />
              {p.website && (
                <a
                  href={p.website}
                  target="_blank"
                  rel="noreferrer noopener"
                  className="hover:text-fg inline-flex items-center gap-1"
                >
                  <Globe size={11} /> {new URL(p.website).host}
                </a>
              )}
              {p.defillama_slug && (
                <span>
                  llama:{' '}
                  <a
                    href={`https://defillama.com/protocol/${p.defillama_slug}`}
                    target="_blank"
                    rel="noreferrer noopener"
                    className="mono text-fg hover:text-accent"
                  >
                    {p.defillama_slug}
                  </a>
                </span>
              )}
              {p.contact && <span>contact: {p.contact}</span>}
            </div>
            {p.description && (
              <p className="mt-2 text-sm text-fg-muted max-w-2xl">{p.description}</p>
            )}
            {p.tags.length > 0 && (
              <div className="mt-2 flex flex-wrap gap-1">
                {p.tags.map((t) => (
                  <span
                    key={t}
                    className="text-[10px] px-1.5 py-0.5 rounded border border-border-subtle text-fg-subtle"
                  >
                    {t}
                  </span>
                ))}
              </div>
            )}
          </div>
        </div>
      </header>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <Stat
          label="TVL"
          value={data.tvl_latest ? formatUsd(data.tvl_latest.tvl_usd) : '—'}
          hint={data.tvl_latest ? formatRelative(data.tvl_latest.timestamp) : undefined}
        />
        <Stat label="Activity 24h" value={formatNumber(data.activity_24h)} hint="events" />
        <Stat
          label="Code events"
          value={formatNumber(data.code_events.length)}
          hint="all-time"
        />
        <Stat
          label="Packages tracked"
          value={formatNumber(p.package_ids.length)}
          hint={`${p.treasury_addresses.length + p.multisig_addresses.length} addresses`}
        />
      </div>

      <Tabs
        tabs={[
          {
            id: 'overview',
            label: 'Overview',
            content: <Overview data={data} />,
          },
          {
            id: 'code',
            label: (
              <span className="flex items-center gap-1.5">
                <Code2 size={12} /> Code events
              </span>
            ),
            badge: data.code_events.length,
            content: <CodeTimeline events={data.code_events} />,
          },
          {
            id: 'activity',
            label: (
              <span className="flex items-center gap-1.5">
                <Activity size={12} /> Activity
              </span>
            ),
            badge: data.activity.length,
            content: <ActivityList activity={data.activity} />,
          },
          {
            id: 'addresses',
            label: (
              <span className="flex items-center gap-1.5">
                <Wallet size={12} /> Addresses
              </span>
            ),
            badge: p.treasury_addresses.length + p.multisig_addresses.length,
            content: (
              <AddressesTab
                treasury={p.treasury_addresses}
                multisig={p.multisig_addresses}
                packages={p.package_ids}
              />
            ),
          },
          {
            id: 'tvl',
            label: 'TVL',
            content: <TvlPane id={p.id} />,
          },
          {
            id: 'raw',
            label: 'Raw',
            content: <JsonView value={data} />,
          },
        ]}
      />
    </div>
  );
}

/* -------------------------- panes -------------------------- */

function Overview({ data }: { data: NonNullable<Awaited<ReturnType<typeof api.watchProtocol>>> }) {
  const p = data.protocol;
  const recent = data.code_events.slice(0, 5);
  const lastActivity = data.activity[0];
  return (
    <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
      <Card title="Identity" className="lg:col-span-2">
        <dl className="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-3 text-sm">
          <Field label="Slug">
            <span className="mono">{p.id}</span>
          </Field>
          <Field label="Category">
            <span>{p.category}</span>
          </Field>
          <Field label="Risk level">
            <RiskPill level={p.risk_level} />
          </Field>
          <Field label="Priority">
            <span className="mono">{p.priority}</span>
          </Field>
          <Field label="Added">
            {formatTime(p.added_at)}{' '}
            <span className="text-xs text-fg-subtle">({formatRelative(p.added_at)})</span>
          </Field>
          <Field label="Last update">
            {formatTime(p.updated_at)}{' '}
            <span className="text-xs text-fg-subtle">({formatRelative(p.updated_at)})</span>
          </Field>
          {p.notes && (
            <Field label="Operator notes" className="sm:col-span-2">
              <span className="text-fg">{p.notes}</span>
            </Field>
          )}
        </dl>
      </Card>
      <Card title="Live snapshot">
        <ul className="text-sm space-y-2">
          <li className="flex items-center justify-between">
            <span className="text-fg-subtle">TVL</span>
            <span className="mono">
              {data.tvl_latest ? formatUsd(data.tvl_latest.tvl_usd) : '—'}
            </span>
          </li>
          <li className="flex items-center justify-between">
            <span className="text-fg-subtle">Activity 24h</span>
            <span className="mono">{formatNumber(data.activity_24h)}</span>
          </li>
          <li className="flex items-center justify-between">
            <span className="text-fg-subtle">Last code event</span>
            <span>
              {recent[0] ? (
                <SeverityChip severity={recent[0].severity}>
                  {recent[0].kind} v{recent[0].version} · {formatRelative(recent[0].happened_at)}
                </SeverityChip>
              ) : (
                <span className="text-fg-subtle">none</span>
              )}
            </span>
          </li>
          <li className="flex items-center justify-between">
            <span className="text-fg-subtle">Last on-chain event</span>
            <span className="text-xs">
              {lastActivity ? formatRelative(lastActivity.timestamp) : '—'}
            </span>
          </li>
        </ul>
      </Card>
    </div>
  );
}

function CodeTimeline({ events }: { events: CodeEvent[] }) {
  if (!events.length)
    return <Empty label="No code events recorded for this protocol yet." icon={<Code2 size={18} />} />;
  return (
    <ol className="relative border-l border-border-subtle ml-2 space-y-4">
      {events.map((e) => (
        <li key={e.id} className="pl-5 relative">
          <span
            className={cn(
              'absolute -left-1.5 top-1 w-3 h-3 rounded-full border-2',
              e.severity === 'critical'
                ? 'bg-danger border-danger'
                : e.severity === 'warning'
                  ? 'bg-warn border-warn'
                  : 'bg-info border-info',
            )}
          />
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant={e.kind === 'publish' ? 'accent' : 'default'}>
              {e.kind} v{e.version}
            </Badge>
            <SeverityChip severity={e.severity}>{e.severity}</SeverityChip>
            <span className="text-xs text-fg-subtle" title={formatTime(e.happened_at)}>
              {formatRelative(e.happened_at)}
            </span>
          </div>
          <div className="mt-1 text-xs text-fg-subtle flex flex-wrap items-center gap-x-3 gap-y-1">
            pkg <Hash value={e.package_id} kind="package" />
            {e.previous_id && (
              <>
                · prev <Hash value={e.previous_id} kind="package" copy={false} />
              </>
            )}
            {e.publish_tx && (
              <>
                · tx <Hash value={e.publish_tx} kind="tx" copy={false} />
              </>
            )}
            <>
              · by <Hash value={e.publisher} kind="address" copy={false} />
            </>
          </div>
          <CodeDiffSummary summary={e.summary} />
        </li>
      ))}
    </ol>
  );
}

function CodeDiffSummary({ summary }: { summary: CodeEvent['summary'] }) {
  const added = summary.modules_added ?? [];
  const removed = summary.modules_removed ?? [];
  const changed = summary.modules_changed ?? [];
  const empty = added.length + removed.length + changed.length === 0;
  if (empty) {
    return <p className="mt-1 text-xs text-fg-subtle italic">no module diff (publish)</p>;
  }
  return (
    <div className="mt-2 grid grid-cols-1 md:grid-cols-3 gap-2 text-xs">
      {added.length > 0 && (
        <DiffBox color="ok" label={`+${added.length} added`}>
          {added.map((m) => (
            <li key={m} className="mono truncate">
              {m}
            </li>
          ))}
        </DiffBox>
      )}
      {removed.length > 0 && (
        <DiffBox color="danger" label={`−${removed.length} removed`}>
          {removed.map((m) => (
            <li key={m} className="mono truncate">
              {m}
            </li>
          ))}
        </DiffBox>
      )}
      {changed.length > 0 && (
        <DiffBox color="warn" label={`~${changed.length} changed`}>
          {changed.map((c) => (
            <li key={c.module} className="mono truncate" title={`${c.prev_hash}\n→\n${c.new_hash}`}>
              {c.module}
            </li>
          ))}
        </DiffBox>
      )}
    </div>
  );
}

function DiffBox({
  color,
  label,
  children,
}: {
  color: 'ok' | 'danger' | 'warn';
  label: string;
  children: React.ReactNode;
}) {
  const map = {
    ok: 'border-ok/40 text-ok',
    danger: 'border-danger/40 text-danger',
    warn: 'border-warn/40 text-warn',
  } as const;
  return (
    <div className={cn('rounded border bg-bg/40 p-2', map[color])}>
      <div className="text-[10px] uppercase tracking-wider mb-1">{label}</div>
      <ul className="text-fg space-y-0.5 max-h-32 overflow-auto">{children}</ul>
    </div>
  );
}

function ActivityList({ activity }: { activity: ProtocolActivity[] }) {
  if (!activity.length)
    return <Empty label="No on-chain events recorded yet." icon={<Activity size={18} />} />;
  return (
    <ul className="divide-y divide-border-subtle border border-border-subtle rounded-lg">
      {activity.map((a) => (
        <li key={`${a.tx_digest}-${a.event_seq}`} className="px-3 py-2 text-sm flex items-center gap-3">
          <Badge variant="accent">{a.event_type.split('::').slice(-1)[0]}</Badge>
          <span className="text-xs text-fg-muted truncate mono">{a.event_type}</span>
          <span className="ml-auto text-xs text-fg-subtle whitespace-nowrap">
            {formatRelative(a.timestamp)}
          </span>
          <span className="text-xs text-fg-subtle">
            tx <Hash value={a.tx_digest} kind="tx" copy={false} />
          </span>
        </li>
      ))}
    </ul>
  );
}

function AddressesTab({
  treasury,
  multisig,
  packages,
}: {
  treasury: string[];
  multisig: string[];
  packages: string[];
}) {
  return (
    <div className="grid grid-cols-1 lg:grid-cols-3 gap-3">
      <AddrCard title="Packages" icon={<HashIcon size={14} />} addrs={packages} kind="package" />
      <AddrCard title="Treasury" icon={<Wallet size={14} />} addrs={treasury} kind="address" />
      <AddrCard
        title="Multisig / Admin"
        icon={<ShieldCheck size={14} />}
        addrs={multisig}
        kind="address"
      />
    </div>
  );
}

function AddrCard({
  title,
  icon,
  addrs,
  kind,
}: {
  title: string;
  icon: React.ReactNode;
  addrs: string[];
  kind: 'address' | 'package';
}) {
  return (
    <Card
      title={
        <span className="flex items-center gap-2">
          {icon} {title}
        </span>
      }
    >
      {addrs.length === 0 ? (
        <p className="text-sm text-fg-subtle">none</p>
      ) : (
        <ul className="space-y-1 text-sm">
          {addrs.map((a) => (
            <li key={a}>
              <Hash value={a} kind={kind} short={false} />
            </li>
          ))}
        </ul>
      )}
    </Card>
  );
}

function TvlPane({ id }: { id: string }) {
  // Render the TvlPage's chart for the chosen protocol via the shared
  // sparkline component.
  return (
    <Suspense fallback={<Loader />}>
      <TvlSparkline protocolId={id} />
    </Suspense>
  );
}

/* -------------------------- atoms -------------------------- */

function Logo({ url, name }: { url: string | null; name: string }) {
  if (url) {
    return (
      <img
        src={url}
        alt=""
        className="w-12 h-12 rounded-lg object-contain bg-bg-elev border border-border-subtle p-1"
      />
    );
  }
  return (
    <div className="w-12 h-12 rounded-lg bg-accent/15 border border-accent/40 text-accent text-base font-semibold flex items-center justify-center">
      {name.slice(0, 1).toUpperCase()}
    </div>
  );
}

function Stat({
  label,
  value,
  hint,
}: {
  label: string;
  value: React.ReactNode;
  hint?: string;
}) {
  return (
    <div className="border border-border-subtle rounded-lg px-4 py-3 bg-bg-subtle/40">
      <div className="text-[10px] uppercase tracking-wider text-fg-subtle">{label}</div>
      <div className="mt-1 text-xl font-semibold mono">{value}</div>
      {hint && <div className="text-xs text-fg-subtle">{hint}</div>}
    </div>
  );
}

function Field({
  label,
  className,
  children,
}: {
  label: string;
  className?: string;
  children: React.ReactNode;
}) {
  return (
    <div className={cn('border-l-2 border-border-subtle pl-3', className)}>
      <div className="text-[10px] uppercase tracking-wider text-fg-subtle">{label}</div>
      <div className="mt-1 text-sm flex items-center gap-1.5 flex-wrap min-w-0">{children}</div>
    </div>
  );
}

function RiskPill({ level }: { level: string }) {
  const map: Record<string, string> = {
    low: 'text-ok bg-ok/10 border-ok/40',
    medium: 'text-warn bg-warn/10 border-warn/40',
    high: 'text-danger bg-danger/10 border-danger/40',
    critical: 'text-danger bg-danger/15 border-danger/60',
    unknown: 'text-fg-subtle bg-bg-elev border-border-subtle',
  };
  return (
    <span
      className={cn(
        'text-[10px] px-1.5 py-0.5 rounded border uppercase tracking-wider',
        map[level] ?? map.unknown,
      )}
    >
      {level}
    </span>
  );
}

function SeverityChip({ severity, children }: { severity: string; children: React.ReactNode }) {
  const icon =
    severity === 'critical' ? (
      <AlertOctagon size={11} />
    ) : severity === 'warning' ? (
      <AlertTriangle size={11} />
    ) : (
      <CheckCircle2 size={11} />
    );
  const map: Record<string, string> = {
    critical: 'text-danger bg-danger/10 border-danger/40',
    warning: 'text-warn bg-warn/10 border-warn/40',
    info: 'text-info bg-info/10 border-info/30',
  };
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1 text-[10px] px-1.5 py-0.5 rounded border',
        map[severity] ?? 'text-fg-subtle border-border-subtle',
      )}
    >
      {icon}
      {children}
    </span>
  );
}
