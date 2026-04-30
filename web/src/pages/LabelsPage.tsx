import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Tag } from 'lucide-react';
import { Link } from '@tanstack/react-router';
import { Card } from '@/components/Card';
import { Loader, ErrorBlock, Empty } from '@/components/Loader';
import { Hash } from '@/components/Hash';
import { useToast } from '@/components/Toast';
import { api } from '@/lib/api';
import { categoryColor } from '@/lib/format';
import { getToken } from '@/lib/auth';

const CATEGORIES = [
  'exchange',
  'cex_hotwallet',
  'cex_coldwallet',
  'market_maker',
  'vc_fund',
  'protocol_treasury',
  'bridge',
  'validator',
  'hacker',
  'scam',
  'phishing',
  'mixer',
  'sanctioned',
  'rug_pull',
  'team_multisig',
  'vesting',
  'airdrop_distributor',
  'other',
];

export function LabelsPage() {
  const [term, setTerm] = useState('');
  const [submitOpen, setSubmitOpen] = useState(false);

  const search = useQuery({
    queryKey: ['labels-search', term],
    queryFn: () => api.searchLabels(term, 100),
    enabled: term.length >= 2,
  });

  return (
    <div className="space-y-6">
      <header className="flex items-end justify-between gap-4 flex-wrap">
        <div>
          <h1 className="text-xl font-semibold tracking-tight flex items-center gap-2">
            <Tag size={18} className="text-accent" /> Address labels
          </h1>
          <p className="text-sm text-fg-muted mt-1">
            Community-curated labels for Sui addresses. Search by category name or paste an address.
          </p>
        </div>
        <button
          className="text-xs text-accent hover:text-accent-fg border border-accent/40 rounded px-3 py-1.5"
          onClick={() => setSubmitOpen((s) => !s)}
        >
          {submitOpen ? 'Close form' : 'Submit a label'}
        </button>
      </header>

      <Card>
        <input
          type="search"
          value={term}
          onChange={(e) => setTerm(e.target.value)}
          placeholder="Search by address, label name, or category…"
          className="w-full bg-bg-elev border border-border-subtle rounded-md px-3 py-2 text-sm focus:outline-none focus:border-accent"
        />
        {submitOpen && <SubmitForm onDone={() => setSubmitOpen(false)} />}
      </Card>

      <Card title={search.data ? `Results · ${search.data.labels.length}` : 'Type at least 2 characters'} noPadding>
        {term.length < 2 ? (
          <div className="p-4">
            <Empty label="Try searching for a category like 'hacker' or paste an address." />
          </div>
        ) : search.isLoading ? (
          <div className="p-4"><Loader /></div>
        ) : search.error ? (
          <div className="p-4"><ErrorBlock error={search.error} /></div>
        ) : search.data?.labels.length ? (
          <ul className="divide-y divide-border-subtle">
            {search.data.labels.map((l) => (
              <li
                key={`${l.address}-${l.label}-${l.source}`}
                className="px-4 py-2.5 flex items-center justify-between gap-3"
              >
                <span className="flex items-center gap-2 min-w-0">
                  <Hash value={l.address} kind="address" copy={false} />
                  <span className="text-fg-muted truncate">{l.label}</span>
                </span>
                <span className="flex items-center gap-2 shrink-0">
                  <span
                    className={`px-2 py-0.5 text-[10px] uppercase tracking-wider border rounded ${categoryColor(l.category)}`}
                  >
                    {l.category}
                  </span>
                  <span className="text-[10px] text-fg-subtle uppercase">{l.source}</span>
                  {l.verified && <span className="text-ok text-[11px]">✓</span>}
                </span>
              </li>
            ))}
          </ul>
        ) : (
          <div className="p-4"><Empty /></div>
        )}
      </Card>
    </div>
  );
}

function SubmitForm({ onDone }: { onDone: () => void }) {
  const qc = useQueryClient();
  const toast = useToast();
  const [address, setAddress] = useState('');
  const [label, setLabel] = useState('');
  const [category, setCategory] = useState('other');
  const [evidenceUrl, setEvidenceUrl] = useState('');
  const token = getToken();

  const m = useMutation({
    mutationFn: () => {
      if (!token) throw new Error('login required');
      return api.submitLabel(token, {
        address,
        label,
        category,
        evidence_url: evidenceUrl || undefined,
      });
    },
    onSuccess: () => {
      toast.push('Submitted for review', 'success');
      qc.invalidateQueries();
      onDone();
    },
  });

  return (
    <form
      className="grid grid-cols-1 md:grid-cols-2 gap-3 mt-4 text-sm"
      onSubmit={(e) => {
        e.preventDefault();
        m.mutate();
      }}
    >
      <input
        required
        placeholder="0x… address"
        value={address}
        onChange={(e) => setAddress(e.target.value)}
        className="bg-bg-elev border border-border-subtle rounded px-3 py-2 mono text-xs"
      />
      <input
        required
        placeholder="Label e.g. Binance Hot 1"
        value={label}
        onChange={(e) => setLabel(e.target.value)}
        className="bg-bg-elev border border-border-subtle rounded px-3 py-2"
      />
      <select
        value={category}
        onChange={(e) => setCategory(e.target.value)}
        className="bg-bg-elev border border-border-subtle rounded px-3 py-2"
      >
        {CATEGORIES.map((c) => (
          <option key={c} value={c}>
            {c}
          </option>
        ))}
      </select>
      <input
        placeholder="Evidence URL (optional)"
        value={evidenceUrl}
        onChange={(e) => setEvidenceUrl(e.target.value)}
        className="bg-bg-elev border border-border-subtle rounded px-3 py-2"
      />
      <div className="md:col-span-2 flex items-center justify-end gap-3">
        {!token && (
          <span className="text-xs text-warn">
            <Link to="/login" className="underline">
              Sign in
            </Link>{' '}
            first to submit.
          </span>
        )}
        {m.error && <span className="text-xs text-danger">{(m.error as Error).message}</span>}
        <button
          type="submit"
          disabled={!token || m.isPending}
          className="px-4 py-2 rounded bg-accent text-accent-fg text-sm disabled:opacity-50"
        >
          {m.isPending ? 'Submitting…' : 'Submit for review'}
        </button>
      </div>
    </form>
  );
}
