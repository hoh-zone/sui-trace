import { useQuery, useQueryClient, useMutation } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { useState } from 'react';
import {
  Activity,
  AlertOctagon,
  AlertTriangle,
  ArrowUpRight,
  CheckCircle2,
  Code2,
  GitBranch,
  Globe,
  KeyRound,
  Pencil,
  Plus,
  Search as SearchIcon,
  ShieldCheck,
  Trash2,
  Wallet,
} from 'lucide-react';
import { Card } from '@/components/Card';
import { Badge } from '@/components/Badge';
import { Loader, ErrorBlock, Empty } from '@/components/Loader';
import { Hash } from '@/components/Hash';
import { useToast } from '@/components/Toast';
import { api, type CodeEvent, type Protocol, type ProtocolCard } from '@/lib/api';
import { formatNumber, formatRelative, formatUsd } from '@/lib/format';
import { cn } from '@/lib/cn';

export function WatchPage() {
  const dash = useQuery({
    queryKey: ['watch-dashboard'],
    queryFn: api.watchDashboard,
    refetchInterval: 30_000,
  });
  const code = useQuery({
    queryKey: ['watch-code-feed'],
    queryFn: () => api.watchCodeFeed(50),
    refetchInterval: 30_000,
  });

  const [filter, setFilter] = useState('');
  const [edit, setEdit] = useState<Protocol | null>(null);
  const [creating, setCreating] = useState(false);
  const [showRemoved, setShowRemoved] = useState(false);

  if (dash.isLoading) return <Loader />;
  if (dash.error) return <ErrorBlock error={dash.error} onRetry={() => dash.refetch()} />;
  const data = dash.data!;

  const filtered = data.cards
    .filter(showRemoved ? () => true : (c) => c.protocol.watched)
    .filter((c) => {
      if (!filter) return true;
      const f = filter.toLowerCase();
      return (
        c.protocol.name.toLowerCase().includes(f) ||
        c.protocol.id.toLowerCase().includes(f) ||
        c.protocol.category.toLowerCase().includes(f) ||
        c.protocol.tags.some((t) => t.toLowerCase().includes(f))
      );
    });

  return (
    <div className="space-y-6">
      <header className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Watched protocols</h1>
          <p className="text-sm text-fg-muted mt-0.5">
            Curated list of priority Sui protocols. Code upgrades are auto-detected
            from the indexer; activity is recomputed every 30s.
          </p>
        </div>
        <button
          className="inline-flex items-center gap-1.5 text-sm text-accent border border-accent/40 rounded-md px-3 py-1.5 hover:bg-accent/10"
          onClick={() => {
            setEdit(null);
            setCreating(true);
          }}
        >
          <Plus size={14} /> Add protocol
        </button>
      </header>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <Stat label="Watched" value={data.totals.watched} hint="protocols" />
        <Stat label="TVL" value={formatUsd(data.totals.tvl_usd)} />
        <Stat label="Activity 24h" value={formatNumber(data.totals.activity_24h)} hint="events" />
        <Stat
          label="Code updates 24h"
          value={formatNumber(data.totals.code_events_24h)}
          accent={data.totals.code_events_24h > 0}
        />
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <div className="relative flex-1 max-w-sm">
          <SearchIcon
            size={14}
            className="absolute left-3 top-1/2 -translate-y-1/2 text-fg-subtle"
          />
          <input
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            placeholder="Filter by name, slug, tag…"
            className="w-full bg-bg-subtle border border-border-subtle rounded-md pl-8 pr-3 py-1.5 text-sm focus:outline-none focus:border-accent"
          />
        </div>
        <label className="flex items-center gap-2 text-xs text-fg-subtle">
          <input
            type="checkbox"
            checked={showRemoved}
            onChange={(e) => setShowRemoved(e.target.checked)}
          />
          show inactive (watched=false)
        </label>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
        {filtered.map((c) => (
          <ProtocolCardView key={c.protocol.id} card={c} onEdit={() => setEdit(c.protocol)} />
        ))}
        {filtered.length === 0 && (
          <Empty
            label="No protocols match"
            hint="Add new ones with the button above, or via `trace watch add`."
          />
        )}
      </div>

      <CodeFeedCard
        events={code.data?.events ?? []}
        loading={code.isLoading}
        cards={data.cards}
      />

      {(creating || edit) && (
        <UpsertDialog
          initial={edit ?? null}
          onClose={() => {
            setCreating(false);
            setEdit(null);
          }}
        />
      )}
    </div>
  );
}

/* -------------------------- protocol card -------------------------- */

function ProtocolCardView({ card, onEdit }: { card: ProtocolCard; onEdit: () => void }) {
  const p = card.protocol;
  const sev = card.last_code_event?.severity;
  return (
    <Card noPadding className="overflow-hidden flex flex-col">
      <div className="p-4 flex items-start gap-3">
        <ProtocolLogo p={p} />
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <Link
              to="/watch/$id"
              params={{ id: p.id }}
              className="font-medium hover:text-accent truncate"
            >
              {p.name}
            </Link>
            {!p.watched && <Badge variant="default">inactive</Badge>}
            <RiskPill level={p.risk_level} />
          </div>
          <div className="text-xs text-fg-subtle mt-0.5 flex items-center gap-2 flex-wrap">
            <span className="mono">{p.id}</span>
            <span>·</span>
            <span>{p.category}</span>
            {p.website && (
              <>
                <span>·</span>
                <a
                  href={p.website}
                  target="_blank"
                  rel="noreferrer noopener"
                  className="hover:text-accent inline-flex items-center gap-0.5"
                >
                  <Globe size={10} /> site
                </a>
              </>
            )}
          </div>
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
        <button
          className="text-fg-subtle hover:text-fg p-1"
          title="Edit"
          onClick={onEdit}
        >
          <Pencil size={14} />
        </button>
      </div>

      <div className="grid grid-cols-3 border-t border-border-subtle text-xs">
        <Tile label="TVL" value={card.tvl_latest ? formatUsd(card.tvl_latest.tvl_usd) : '—'} />
        <Tile
          label="Activity 24h"
          value={formatNumber(card.activity_24h)}
          accent={card.activity_24h > 0}
        />
        <Tile
          label="Code 24h"
          value={formatNumber(card.code_events_24h)}
          accent={card.code_events_24h > 0}
          severity={sev}
        />
      </div>

      <div className="px-4 py-2.5 border-t border-border-subtle text-xs text-fg-subtle flex items-center justify-between">
        <span>
          {card.last_code_event ? (
            <>
              last <SeverityDot sev={card.last_code_event.severity} />{' '}
              {card.last_code_event.kind} v{card.last_code_event.version} —{' '}
              {formatRelative(card.last_code_event.happened_at)}
            </>
          ) : (
            <span>no code events yet</span>
          )}
        </span>
        <Link
          to="/watch/$id"
          params={{ id: p.id }}
          className="hover:text-accent inline-flex items-center gap-0.5"
        >
          open <ArrowUpRight size={11} />
        </Link>
      </div>
    </Card>
  );
}

function ProtocolLogo({ p }: { p: Protocol }) {
  if (p.logo_url) {
    return (
      <img
        src={p.logo_url}
        alt=""
        className="w-10 h-10 rounded-md object-contain bg-bg-elev border border-border-subtle p-1"
      />
    );
  }
  const letter = p.name.slice(0, 1).toUpperCase();
  return (
    <div className="w-10 h-10 rounded-md bg-accent/15 border border-accent/40 text-accent text-sm font-semibold flex items-center justify-center">
      {letter}
    </div>
  );
}

function Tile({
  label,
  value,
  accent,
  severity,
}: {
  label: string;
  value: React.ReactNode;
  accent?: boolean;
  severity?: string;
}) {
  return (
    <div className="px-3 py-2 border-r border-border-subtle last:border-r-0">
      <div className="text-[10px] uppercase tracking-wider text-fg-subtle">{label}</div>
      <div
        className={cn(
          'mono text-sm mt-0.5 flex items-center gap-1',
          accent && 'text-fg',
          severity === 'critical' && 'text-danger',
          severity === 'warning' && 'text-warn',
        )}
      >
        {value}
      </div>
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

function SeverityDot({ sev }: { sev: string }) {
  const map: Record<string, string> = {
    info: 'bg-info',
    warning: 'bg-warn',
    critical: 'bg-danger',
  };
  return (
    <span
      className={cn('inline-block w-1.5 h-1.5 rounded-full mr-1', map[sev] ?? 'bg-fg-subtle')}
    />
  );
}

function Stat({
  label,
  value,
  hint,
  accent,
}: {
  label: string;
  value: React.ReactNode;
  hint?: string;
  accent?: boolean;
}) {
  return (
    <div className="border border-border-subtle rounded-lg px-4 py-3 bg-bg-subtle/40">
      <div className="text-[10px] uppercase tracking-wider text-fg-subtle">{label}</div>
      <div
        className={cn('mt-1 text-xl font-semibold mono', accent && 'text-accent')}
      >
        {value}
      </div>
      {hint && <div className="text-xs text-fg-subtle">{hint}</div>}
    </div>
  );
}

/* -------------------------- code feed -------------------------- */

function CodeFeedCard({
  events,
  loading,
  cards,
}: {
  events: CodeEvent[];
  loading: boolean;
  cards: ProtocolCard[];
}) {
  const nameOf = (id: string) =>
    cards.find((c) => c.protocol.id === id)?.protocol.name ?? id;
  return (
    <Card
      title={
        <span className="flex items-center gap-2">
          <GitBranch size={14} className="text-accent" />
          Cross-protocol code update feed
        </span>
      }
      noPadding
    >
      <div className="divide-y divide-border-subtle">
        {loading && (
          <div className="p-6">
            <Loader />
          </div>
        )}
        {!loading && events.length === 0 && (
          <Empty label="No code events yet — looks quiet on chain." />
        )}
        {!loading &&
          events.map((e) => <CodeFeedRow key={e.id} e={e} protocolName={nameOf(e.protocol_id)} />)}
      </div>
    </Card>
  );
}

function CodeFeedRow({ e, protocolName }: { e: CodeEvent; protocolName: string }) {
  const sevIcon =
    e.severity === 'critical' ? (
      <AlertOctagon size={14} className="text-danger" />
    ) : e.severity === 'warning' ? (
      <AlertTriangle size={14} className="text-warn" />
    ) : (
      <CheckCircle2 size={14} className="text-info" />
    );
  const added = e.summary.modules_added?.length ?? 0;
  const removed = e.summary.modules_removed?.length ?? 0;
  const changed = e.summary.modules_changed?.length ?? 0;
  return (
    <div className="px-4 py-3 flex items-start gap-3">
      <div className="mt-1">{sevIcon}</div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 flex-wrap">
          <Link
            to="/watch/$id"
            params={{ id: e.protocol_id }}
            className="font-medium hover:text-accent"
          >
            {protocolName}
          </Link>
          <Badge variant={e.kind === 'publish' ? 'accent' : 'default'}>
            {e.kind} v{e.version}
          </Badge>
          <RiskPill level={e.severity} />
          <span className="text-xs text-fg-subtle">{formatRelative(e.happened_at)}</span>
        </div>
        <div className="text-xs text-fg-subtle mt-1 flex items-center gap-2 flex-wrap">
          pkg <Hash value={e.package_id} kind="package" copy={false} />
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
        </div>
        {(added || removed || changed) > 0 && (
          <div className="text-xs mt-1 flex items-center gap-3">
            {added > 0 && <span className="text-ok">+{added} modules</span>}
            {removed > 0 && <span className="text-danger">−{removed} modules</span>}
            {changed > 0 && <span className="text-warn">~{changed} changed</span>}
          </div>
        )}
      </div>
    </div>
  );
}

/* -------------------------- create/edit dialog -------------------------- */

function UpsertDialog({
  initial,
  onClose,
}: {
  initial: Protocol | null;
  onClose: () => void;
}) {
  const editing = !!initial;
  const qc = useQueryClient();
  const { push } = useToast();
  const [ingestKey, setIngestKey] = useState(() => localStorage.getItem('trace_ingest_key') ?? '');
  const [form, setForm] = useState({
    id: initial?.id ?? '',
    name: initial?.name ?? '',
    category: initial?.category ?? 'dex',
    website: initial?.website ?? '',
    defillama_slug: initial?.defillama_slug ?? '',
    description: initial?.description ?? '',
    logo_url: initial?.logo_url ?? '',
    contact: initial?.contact ?? '',
    notes: initial?.notes ?? '',
    risk_level: initial?.risk_level ?? 'unknown',
    priority: initial?.priority ?? 50,
    watched: initial?.watched ?? true,
    package_ids: (initial?.package_ids ?? []).join('\n'),
    treasury_addresses: (initial?.treasury_addresses ?? []).join('\n'),
    multisig_addresses: (initial?.multisig_addresses ?? []).join('\n'),
    tags: (initial?.tags ?? []).join(', '),
  });

  const save = useMutation({
    mutationFn: async () => {
      if (!ingestKey) throw new Error('Ingest API key required');
      const body = {
        id: form.id.trim(),
        name: form.name.trim(),
        category: form.category,
        website: form.website || null,
        defillama_slug: form.defillama_slug || null,
        description: form.description || null,
        logo_url: form.logo_url || null,
        contact: form.contact || null,
        notes: form.notes || null,
        risk_level: form.risk_level,
        priority: Number(form.priority),
        watched: form.watched,
        package_ids: splitLines(form.package_ids),
        treasury_addresses: splitLines(form.treasury_addresses),
        multisig_addresses: splitLines(form.multisig_addresses),
        tags: form.tags.split(',').map((t) => t.trim()).filter(Boolean),
      };
      localStorage.setItem('trace_ingest_key', ingestKey);
      if (editing) {
        const { id, ...rest } = body;
        return api.watchUpdate(id, rest, ingestKey);
      }
      return api.watchUpsert(body, ingestKey);
    },
    onSuccess: () => {
      push(editing ? 'Protocol updated' : 'Protocol added', 'success');
      qc.invalidateQueries({ queryKey: ['watch-dashboard'] });
      qc.invalidateQueries({ queryKey: ['watch-code-feed'] });
      onClose();
    },
    onError: (e: unknown) => push(`Save failed: ${(e as Error).message}`, 'danger'),
  });

  const remove = useMutation({
    mutationFn: () => api.watchRemove(initial!.id, ingestKey),
    onSuccess: () => {
      push('Protocol removed', 'success');
      qc.invalidateQueries({ queryKey: ['watch-dashboard'] });
      onClose();
    },
    onError: (e: unknown) => push(`Remove failed: ${(e as Error).message}`, 'danger'),
  });

  return (
    <div
      className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4"
      onClick={onClose}
    >
      <div
        className="bg-bg border border-border-subtle rounded-xl shadow-soft w-full max-w-2xl max-h-[85vh] overflow-y-auto"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="px-5 py-3 border-b border-border-subtle flex items-center justify-between">
          <h2 className="font-semibold flex items-center gap-2">
            {editing ? <Pencil size={14} /> : <Plus size={14} />}
            {editing ? `Edit ${initial!.name}` : 'Add protocol'}
          </h2>
          <button onClick={onClose} className="text-fg-subtle hover:text-fg text-sm">
            close
          </button>
        </div>
        <div className="px-5 py-4 grid grid-cols-1 md:grid-cols-2 gap-3 text-sm">
          <Field label="ID (slug)">
            <input
              disabled={editing}
              value={form.id}
              onChange={(e) => setForm({ ...form, id: e.target.value })}
              placeholder="cetus-amm"
              className="input-base"
            />
          </Field>
          <Field label="Display name">
            <input
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
              className="input-base"
            />
          </Field>
          <Field label="Category">
            <select
              value={form.category}
              onChange={(e) => setForm({ ...form, category: e.target.value })}
              className="input-base"
            >
              {['dex', 'lending', 'liquid_staking', 'derivatives', 'yield', 'bridge', 'launchpad', 'other'].map((c) => (
                <option key={c}>{c}</option>
              ))}
            </select>
          </Field>
          <Field label="Risk level">
            <select
              value={form.risk_level}
              onChange={(e) => setForm({ ...form, risk_level: e.target.value })}
              className="input-base"
            >
              {['low', 'medium', 'high', 'critical', 'unknown'].map((c) => (
                <option key={c}>{c}</option>
              ))}
            </select>
          </Field>
          <Field label="Priority (sort order)">
            <input
              type="number"
              value={form.priority}
              onChange={(e) => setForm({ ...form, priority: Number(e.target.value) })}
              className="input-base"
            />
          </Field>
          <Field label="Website">
            <input
              value={form.website}
              onChange={(e) => setForm({ ...form, website: e.target.value })}
              className="input-base"
            />
          </Field>
          <Field label="DefiLlama slug">
            <input
              value={form.defillama_slug}
              onChange={(e) => setForm({ ...form, defillama_slug: e.target.value })}
              className="input-base"
            />
          </Field>
          <Field label="Logo URL">
            <input
              value={form.logo_url}
              onChange={(e) => setForm({ ...form, logo_url: e.target.value })}
              className="input-base"
            />
          </Field>
          <Field label="Tags (comma separated)" className="md:col-span-2">
            <input
              value={form.tags}
              onChange={(e) => setForm({ ...form, tags: e.target.value })}
              className="input-base"
            />
          </Field>
          <Field label="Package original_ids (one per line)" className="md:col-span-2">
            <textarea
              rows={3}
              value={form.package_ids}
              onChange={(e) => setForm({ ...form, package_ids: e.target.value })}
              className="input-base mono text-xs"
            />
          </Field>
          <Field label="Treasury addresses" className="md:col-span-2">
            <textarea
              rows={2}
              value={form.treasury_addresses}
              onChange={(e) => setForm({ ...form, treasury_addresses: e.target.value })}
              className="input-base mono text-xs"
            />
          </Field>
          <Field label="Multisig / admin addresses" className="md:col-span-2">
            <textarea
              rows={2}
              value={form.multisig_addresses}
              onChange={(e) => setForm({ ...form, multisig_addresses: e.target.value })}
              className="input-base mono text-xs"
            />
          </Field>
          <Field label="Description" className="md:col-span-2">
            <textarea
              rows={2}
              value={form.description}
              onChange={(e) => setForm({ ...form, description: e.target.value })}
              className="input-base"
            />
          </Field>
          <Field label="Contact">
            <input
              value={form.contact}
              onChange={(e) => setForm({ ...form, contact: e.target.value })}
              className="input-base"
            />
          </Field>
          <Field label="Operator notes">
            <input
              value={form.notes}
              onChange={(e) => setForm({ ...form, notes: e.target.value })}
              className="input-base"
            />
          </Field>
          <Field label="Watched">
            <label className="inline-flex items-center gap-2">
              <input
                type="checkbox"
                checked={form.watched}
                onChange={(e) => setForm({ ...form, watched: e.target.checked })}
              />
              <span className="text-xs text-fg-subtle">show on dashboard</span>
            </label>
          </Field>
          <Field label="Ingest API key" className="md:col-span-2">
            <div className="relative">
              <KeyRound
                size={12}
                className="absolute left-2.5 top-1/2 -translate-y-1/2 text-fg-subtle"
              />
              <input
                type="password"
                value={ingestKey}
                onChange={(e) => setIngestKey(e.target.value)}
                placeholder="X-Trace-Ingest-Key"
                className="input-base pl-7"
              />
            </div>
            <p className="text-[11px] text-fg-subtle mt-1">
              Stored in <code className="mono">localStorage.trace_ingest_key</code> for this
              browser only. Configure on the server in{' '}
              <code className="mono">[auth].ingest_api_key</code>.
            </p>
          </Field>
        </div>
        <div className="px-5 py-3 border-t border-border-subtle flex items-center justify-between">
          {editing ? (
            <button
              className="text-xs text-danger hover:underline inline-flex items-center gap-1"
              onClick={() => {
                if (confirm(`Remove ${initial!.name} from the watchlist?`)) remove.mutate();
              }}
              disabled={remove.isPending}
            >
              <Trash2 size={12} /> remove
            </button>
          ) : (
            <span />
          )}
          <button
            className="text-sm bg-accent text-accent-fg rounded-md px-4 py-1.5 disabled:opacity-50"
            onClick={() => save.mutate()}
            disabled={save.isPending || !form.id.trim() || !form.name.trim()}
          >
            {save.isPending ? 'Saving…' : editing ? 'Save changes' : 'Create'}
          </button>
        </div>
      </div>
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
    <label className={cn('block', className)}>
      <span className="text-[10px] uppercase tracking-wider text-fg-subtle">{label}</span>
      <div className="mt-1">{children}</div>
    </label>
  );
}

function splitLines(s: string): string[] {
  return s
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter(Boolean);
}

// Re-export so other files can also use the icons we already imported here
export const _icons = { Activity, Wallet, ShieldCheck, Code2 };
