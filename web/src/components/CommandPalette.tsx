import { useNavigate } from '@tanstack/react-router';
import {
  Activity,
  AlertTriangle,
  BarChart3,
  Bell,
  Boxes,
  Clock,
  Code2,
  Command,
  Compass,
  Layers,
  Search as SearchIcon,
  ShieldAlert,
  Tag,
} from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';

interface CommandPaletteProps {
  open: boolean;
  onClose: () => void;
}

interface Item {
  id: string;
  label: string;
  hint: string;
  icon: React.ComponentType<{ size?: number }>;
  action: () => void;
}

export function CommandPalette({ open, onClose }: CommandPaletteProps) {
  const navigate = useNavigate();
  const [q, setQ] = useState('');
  const [active, setActive] = useState(0);

  useEffect(() => {
    if (!open) {
      setQ('');
      setActive(0);
    }
  }, [open]);

  const items: Item[] = useMemo(
    () => [
      { id: 'home', label: 'Overview', hint: 'Network home dashboard', icon: Compass, action: () => navigate({ to: '/' }) },
      { id: 'cps', label: 'Checkpoints', hint: 'Browse latest checkpoints', icon: Clock, action: () => navigate({ to: '/checkpoints' }) },
      { id: 'pkgs', label: 'Packages', hint: 'Browse newly published packages', icon: Boxes, action: () => navigate({ to: '/packages' }) },
      { id: 'sec', label: 'Security feed', hint: 'Recent findings + scoreboard', icon: ShieldAlert, action: () => navigate({ to: '/security' }) },
      { id: 'depl', label: 'Deployments', hint: 'Daily deployments', icon: Code2, action: () => navigate({ to: '/analytics/deployments' }) },
      { id: 'act', label: 'Active packages', hint: 'Top callers', icon: Activity, action: () => navigate({ to: '/analytics/active' }) },
      { id: 'tvl', label: 'TVL', hint: 'Per-protocol TVL history', icon: BarChart3, action: () => navigate({ to: '/analytics/tvl' }) },
      { id: 'lab', label: 'Labels', hint: 'Address label library', icon: Tag, action: () => navigate({ to: '/labels' }) },
      { id: 'wl', label: 'Watchlists', hint: 'Manage watchlists', icon: Bell, action: () => navigate({ to: '/watchlist' }) },
      { id: 'al', label: 'Alerts', hint: 'Personal alert history', icon: AlertTriangle, action: () => navigate({ to: '/alerts' }) },
      { id: 'net', label: 'Network', hint: 'Epoch + validators', icon: Layers, action: () => navigate({ to: '/network' }) },
    ],
    [navigate],
  );

  const filtered = q
    ? items.filter((i) => `${i.label} ${i.hint}`.toLowerCase().includes(q.toLowerCase()))
    : items;

  if (!open) return null;

  // Treat a non-empty query that looks like a hash/address as a quick search
  const looksHash = q.startsWith('0x') && q.length >= 8;

  return (
    <div
      className="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm flex items-start justify-center pt-24 px-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-xl bg-bg-elev border border-border rounded-xl shadow-soft overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-2 px-3 border-b border-border-subtle">
          <SearchIcon size={14} className="text-fg-subtle" />
          <input
            autoFocus
            value={q}
            onChange={(e) => {
              setQ(e.target.value);
              setActive(0);
            }}
            onKeyDown={(e) => {
              if (e.key === 'Escape') onClose();
              if (e.key === 'ArrowDown') {
                e.preventDefault();
                setActive((a) => Math.min(filtered.length - 1, a + 1));
              }
              if (e.key === 'ArrowUp') {
                e.preventDefault();
                setActive((a) => Math.max(0, a - 1));
              }
              if (e.key === 'Enter') {
                e.preventDefault();
                if (looksHash) {
                  navigate({ to: '/search', search: { q } });
                  onClose();
                  return;
                }
                const item = filtered[active];
                if (item) {
                  item.action();
                  onClose();
                }
              }
            }}
            placeholder="Jump to a page or paste a hash / address / package id…"
            className="flex-1 bg-transparent border-0 py-3 text-sm focus:outline-none placeholder:text-fg-subtle"
          />
          <kbd className="text-[10px] text-fg-subtle border border-border-subtle rounded px-1 py-0.5 mono">
            ESC
          </kbd>
        </div>
        <ul className="max-h-80 overflow-auto">
          {looksHash && (
            <li
              className="px-3 py-2 flex items-center gap-2 text-sm bg-bg-elev-2 border-b border-border-subtle"
              onClick={() => {
                navigate({ to: '/search', search: { q } });
                onClose();
              }}
              role="button"
            >
              <SearchIcon size={14} className="text-accent" />
              <span>
                Search <span className="mono text-accent">{q.slice(0, 14)}…</span>
              </span>
            </li>
          )}
          {filtered.map((it, i) => {
            const Icon = it.icon;
            return (
              <li
                key={it.id}
                onMouseEnter={() => setActive(i)}
                onClick={() => {
                  it.action();
                  onClose();
                }}
                className={`flex items-center gap-3 px-3 py-2 text-sm cursor-pointer ${
                  i === active ? 'bg-bg-elev-2' : ''
                }`}
              >
                <Icon size={14} />
                <span className="text-fg">{it.label}</span>
                <span className="text-xs text-fg-subtle ml-auto">{it.hint}</span>
              </li>
            );
          })}
          {filtered.length === 0 && !looksHash && (
            <li className="text-sm text-fg-subtle px-3 py-4 text-center">No matches</li>
          )}
        </ul>
        <div className="px-3 py-1.5 border-t border-border-subtle text-[10px] text-fg-subtle flex items-center justify-between">
          <span className="flex items-center gap-2">
            <Command size={10} /> palette · ↑↓ navigate · ↵ open
          </span>
          <span>sui-trace</span>
        </div>
      </div>
    </div>
  );
}
