import { useState } from 'react';
import { Link } from '@tanstack/react-router';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Bell, Trash2 } from 'lucide-react';
import { Card } from '@/components/Card';
import { Loader, ErrorBlock, Empty } from '@/components/Loader';
import { Badge } from '@/components/Badge';
import { Hash } from '@/components/Hash';
import { useToast } from '@/components/Toast';
import { api, type Watchlist, type WatchlistBody } from '@/lib/api';
import { getToken } from '@/lib/auth';
import { formatRelative } from '@/lib/format';

const RULE_PRESETS: Record<string, Record<string, unknown>> = {
  address_activity: { window_secs: 300, min_amount_usd: 50000 },
  large_outflow: { window_secs: 600, min_amount_usd: 100000 },
  package_upgrade: {},
  high_severity_package: {},
  tvl_drop: { window_secs: 3600, threshold_pct: 10 },
  suspicious_recipient: {},
};

const CHANNEL_PRESETS = [
  { kind: 'telegram', chat_id: 'YOUR_CHAT_ID' },
  { kind: 'webhook', url: 'https://example.com/hook', secret: 'optional' },
  { kind: 'discord', webhook: 'https://discord.com/api/webhooks/…' },
  { kind: 'email', to: 'you@example.com' },
];

export function WatchlistPage() {
  const token = getToken();
  const qc = useQueryClient();
  const toast = useToast();
  const list = useQuery({
    queryKey: ['watchlists'],
    queryFn: () => api.watchlists(token!),
    enabled: !!token,
  });

  const empty: WatchlistBody = {
    name: '',
    target_type: 'address',
    target_id: '',
    rules: { window_secs: 300 },
    channels: [],
  };
  const [body, setBody] = useState<WatchlistBody>(empty);
  const [presetRule, setPresetRule] = useState('address_activity');

  const create = useMutation({
    mutationFn: () => api.createWatchlist(token!, body),
    onSuccess: () => {
      setBody(empty);
      toast.push('Watchlist created', 'success');
      qc.invalidateQueries({ queryKey: ['watchlists'] });
    },
  });

  const remove = useMutation({
    mutationFn: (id: string) => api.deleteWatchlist(token!, id),
    onSuccess: () => {
      toast.push('Watchlist removed', 'success');
      qc.invalidateQueries({ queryKey: ['watchlists'] });
    },
  });

  if (!token) {
    return (
      <Empty
        label="Sign in required"
        hint={
          <span>
            <Link to="/login" className="text-accent hover:underline">
              Sign in
            </Link>{' '}
            to manage watchlists.
          </span>
        }
        icon={<Bell size={20} />}
      />
    );
  }

  return (
    <div className="space-y-6">
      <header>
        <h1 className="text-xl font-semibold tracking-tight flex items-center gap-2">
          <Bell size={18} className="text-accent" /> Watchlists
        </h1>
        <p className="text-sm text-fg-muted mt-1">
          Define what to watch, pick alert channels, and Sui-trace will route every match through the
          dispatch engine with deduplication and retries.
        </p>
      </header>

      <Card title="Create watchlist">
        <form
          className="grid grid-cols-1 md:grid-cols-2 gap-3 text-sm"
          onSubmit={(e) => {
            e.preventDefault();
            create.mutate();
          }}
        >
          <Field label="Name">
            <input
              required
              placeholder="e.g. Treasury monitor"
              value={body.name}
              onChange={(e) => setBody({ ...body, name: e.target.value })}
              className="w-full bg-bg-elev border border-border-subtle rounded px-3 py-2"
            />
          </Field>

          <Field label="Target type">
            <select
              value={body.target_type}
              onChange={(e) => setBody({ ...body, target_type: e.target.value as WatchlistBody['target_type'] })}
              className="w-full bg-bg-elev border border-border-subtle rounded px-3 py-2"
            >
              <option value="address">Address</option>
              <option value="package">Package</option>
              <option value="protocol">Protocol</option>
            </select>
          </Field>

          <Field label="Target id" full>
            <input
              required
              placeholder={body.target_type === 'protocol' ? 'protocol slug e.g. cetus-amm' : '0x…'}
              value={body.target_id}
              onChange={(e) => setBody({ ...body, target_id: e.target.value })}
              className="w-full bg-bg-elev border border-border-subtle rounded px-3 py-2 mono text-xs"
            />
          </Field>

          <Field label="Rule preset">
            <select
              value={presetRule}
              onChange={(e) => {
                setPresetRule(e.target.value);
                setBody({ ...body, rules: RULE_PRESETS[e.target.value] ?? {} });
              }}
              className="w-full bg-bg-elev border border-border-subtle rounded px-3 py-2"
            >
              {Object.keys(RULE_PRESETS).map((k) => (
                <option key={k} value={k}>
                  {k}
                </option>
              ))}
            </select>
          </Field>

          <Field label="Add channel">
            <select
              defaultValue=""
              onChange={(e) => {
                const idx = Number(e.target.value);
                if (Number.isFinite(idx) && CHANNEL_PRESETS[idx]) {
                  setBody({ ...body, channels: [...(body.channels ?? []), CHANNEL_PRESETS[idx]] });
                }
                e.currentTarget.value = '';
              }}
              className="w-full bg-bg-elev border border-border-subtle rounded px-3 py-2"
            >
              <option value="" disabled>
                pick a channel preset…
              </option>
              {CHANNEL_PRESETS.map((c, i) => (
                <option key={i} value={i}>
                  {c.kind}
                </option>
              ))}
            </select>
          </Field>

          <Field label="Rules JSON" full>
            <textarea
              value={JSON.stringify(body.rules ?? {}, null, 2)}
              onChange={(e) => {
                try {
                  setBody({ ...body, rules: JSON.parse(e.target.value || '{}') });
                } catch {
                  /* ignore parse errors while typing */
                }
              }}
              className="w-full bg-bg-elev border border-border-subtle rounded px-3 py-2 mono text-xs h-28"
            />
          </Field>

          <Field label="Channels JSON" full>
            <textarea
              value={JSON.stringify(body.channels ?? [], null, 2)}
              onChange={(e) => {
                try {
                  setBody({ ...body, channels: JSON.parse(e.target.value || '[]') });
                } catch {
                  /* ignore parse errors while typing */
                }
              }}
              className="w-full bg-bg-elev border border-border-subtle rounded px-3 py-2 mono text-xs h-28"
            />
          </Field>

          <div className="md:col-span-2 flex items-center justify-end gap-3">
            {create.error && (
              <span className="text-xs text-danger">{(create.error as Error).message}</span>
            )}
            <button
              type="submit"
              disabled={create.isPending || !body.name || !body.target_id}
              className="px-4 py-2 rounded bg-accent text-accent-fg text-sm disabled:opacity-50 disabled:cursor-not-allowed hover:opacity-90"
            >
              {create.isPending ? 'Creating…' : 'Create watchlist'}
            </button>
          </div>
        </form>
      </Card>

      <Card title={`Your watchlists · ${list.data?.watchlists.length ?? 0}`} noPadding>
        {list.isLoading ? (
          <div className="p-4"><Loader /></div>
        ) : list.error ? (
          <div className="p-4"><ErrorBlock error={list.error} /></div>
        ) : list.data?.watchlists.length ? (
          <ul className="divide-y divide-border-subtle">
            {list.data.watchlists.map((w) => (
              <WatchlistRow key={w.id} w={w} onDelete={(id) => remove.mutate(id)} />
            ))}
          </ul>
        ) : (
          <div className="p-4">
            <Empty label="You don't have any watchlists yet." hint="Create one above to start receiving alerts." />
          </div>
        )}
      </Card>
    </div>
  );
}

function WatchlistRow({ w, onDelete }: { w: Watchlist; onDelete: (id: string) => void }) {
  const channels = (w.channels as Array<{ kind: string }>) ?? [];
  return (
    <li className="px-4 py-3 flex items-start justify-between gap-3">
      <div className="min-w-0">
        <div className="flex items-center gap-2 flex-wrap">
          <strong>{w.name}</strong>
          <Badge variant="accent">{w.target_type}</Badge>
          {channels.map((c, i) => (
            <Badge key={i} variant="outline">
              {c.kind}
            </Badge>
          ))}
        </div>
        <div className="text-xs text-fg-muted mt-1 flex items-center gap-2">
          {w.target_type === 'address' || w.target_type === 'package' ? (
            <Hash
              value={w.target_id}
              kind={w.target_type === 'address' ? 'address' : 'package'}
              short={false}
              copy
            />
          ) : (
            <span className="mono">{w.target_id}</span>
          )}
        </div>
        <div className="text-xs text-fg-subtle mt-0.5">created {formatRelative(w.created_at)}</div>
      </div>
      <button
        onClick={() => {
          if (window.confirm('Delete this watchlist?')) onDelete(w.id);
        }}
        className="text-xs text-danger border border-danger/40 rounded px-2 py-1 inline-flex items-center gap-1 hover:bg-danger/10"
      >
        <Trash2 size={12} /> delete
      </button>
    </li>
  );
}

function Field({
  label,
  full,
  children,
}: {
  label: string;
  full?: boolean;
  children: React.ReactNode;
}) {
  return (
    <div className={full ? 'md:col-span-2' : ''}>
      <div className="text-[10px] uppercase tracking-wider text-fg-subtle mb-1">{label}</div>
      {children}
    </div>
  );
}
