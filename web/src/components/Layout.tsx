import { Link, useNavigate, useRouterState } from '@tanstack/react-router';
import {
  Activity,
  AlertTriangle,
  BarChart3,
  Bell,
  Boxes,
  Clock,
  Code2,
  Compass,
  Layers,
  Search as SearchIcon,
  ShieldAlert,
  Star,
  Tag,
  X,
} from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { cn } from '@/lib/cn';
import { getUser, logout } from '@/lib/auth';
import { useQuery } from '@tanstack/react-query';
import { api } from '@/lib/api';
import { CommandPalette } from './CommandPalette';

interface LayoutProps {
  children: React.ReactNode;
}

const navGroups: Array<{ label: string; items: Array<{ to: string; label: string; icon: typeof Compass; exact?: boolean }> }> = [
  {
    label: 'Explore',
    items: [
      { to: '/', label: 'Overview', icon: Compass, exact: true },
      { to: '/checkpoints', label: 'Checkpoints', icon: Clock },
      { to: '/packages', label: 'Packages', icon: Boxes },
      { to: '/security', label: 'Security feed', icon: ShieldAlert },
      { to: '/network', label: 'Network', icon: Layers },
    ],
  },
  {
    label: 'Operations',
    items: [
      { to: '/watch', label: 'Watched protocols', icon: Star },
    ],
  },
  {
    label: 'Analytics',
    items: [
      { to: '/analytics/deployments', label: 'Deployments', icon: Code2 },
      { to: '/analytics/active', label: 'Active', icon: Activity },
      { to: '/analytics/tvl', label: 'TVL', icon: BarChart3 },
    ],
  },
  {
    label: 'Personal',
    items: [
      { to: '/labels', label: 'Labels', icon: Tag },
      { to: '/watchlist', label: 'Watchlist', icon: Bell },
      { to: '/alerts', label: 'Alerts', icon: AlertTriangle },
    ],
  },
];

export function Layout({ children }: LayoutProps) {
  const navigate = useNavigate();
  const state = useRouterState();
  const [q, setQ] = useState('');
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [mobileNav, setMobileNav] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const user = getUser();

  // Live network status pill
  const overview = useQuery({
    queryKey: ['network-pill'],
    queryFn: api.networkOverview,
    refetchInterval: 10_000,
    retry: false,
  });

  useEffect(() => {
    function handler(e: KeyboardEvent) {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setPaletteOpen((o) => !o);
      }
      if (e.key === '/' && document.activeElement?.tagName !== 'INPUT' && document.activeElement?.tagName !== 'TEXTAREA') {
        e.preventDefault();
        inputRef.current?.focus();
      }
    }
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  // close mobile nav on navigation
  useEffect(() => {
    setMobileNav(false);
  }, [state.location.pathname]);

  const cp = overview.data?.checkpoint;
  const stale = cp ? Date.now() - cp.timestamp_ms > 30_000 : false;

  return (
    <div className="min-h-screen flex">
      <CommandPalette open={paletteOpen} onClose={() => setPaletteOpen(false)} />

      {/* Sidebar */}
      <aside
        className={cn(
          'border-r border-border-subtle bg-bg-subtle/60 backdrop-blur w-60 shrink-0 hidden lg:flex flex-col gap-4 px-3 py-4 sticky top-0 h-screen',
        )}
      >
        <SidebarBrand />
        <nav className="flex-1 overflow-y-auto -mx-1 px-1">
          {navGroups.map((g) => (
            <div key={g.label} className="mb-4">
              <div className="text-[10px] uppercase tracking-widest text-fg-subtle px-2 mb-1">
                {g.label}
              </div>
              <ul className="space-y-0.5">
                {g.items.map((it) => {
                  const isActive = it.exact
                    ? state.location.pathname === it.to
                    : state.location.pathname.startsWith(it.to);
                  return (
                    <li key={it.to}>
                      <Link
                        to={it.to}
                        className={cn(
                          'flex items-center gap-2 px-2 py-1.5 rounded-md text-sm',
                          isActive
                            ? 'bg-accent/15 text-accent border border-accent/30'
                            : 'text-fg-muted hover:text-fg hover:bg-bg-elev/60 border border-transparent',
                        )}
                      >
                        <it.icon size={14} />
                        {it.label}
                      </Link>
                    </li>
                  );
                })}
              </ul>
            </div>
          ))}
        </nav>
        <NetworkPill cp={cp} stale={stale} />
      </aside>

      {/* Mobile nav drawer */}
      {mobileNav && (
        <div className="fixed inset-0 z-40 lg:hidden bg-black/60" onClick={() => setMobileNav(false)}>
          <aside
            className="bg-bg-subtle border-r border-border-subtle w-64 h-full p-3 overflow-y-auto"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between mb-3">
              <SidebarBrand />
              <button onClick={() => setMobileNav(false)} className="text-fg-subtle p-1">
                <X size={16} />
              </button>
            </div>
            {navGroups.map((g) => (
              <div key={g.label} className="mb-4">
                <div className="text-[10px] uppercase tracking-widest text-fg-subtle px-2 mb-1">{g.label}</div>
                <ul className="space-y-0.5">
                  {g.items.map((it) => (
                    <li key={it.to}>
                      <Link
                        to={it.to}
                        className="flex items-center gap-2 px-2 py-1.5 rounded-md text-sm text-fg-muted hover:text-fg"
                      >
                        <it.icon size={14} /> {it.label}
                      </Link>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
            <NetworkPill cp={cp} stale={stale} />
          </aside>
        </div>
      )}

      {/* Main area */}
      <div className="flex-1 min-w-0 flex flex-col">
        <header className="sticky top-0 z-20 border-b border-border-subtle bg-bg/80 backdrop-blur">
          <div className="px-4 lg:px-6 py-3 flex items-center gap-3">
            <button
              className="lg:hidden p-1.5 rounded border border-border-subtle"
              onClick={() => setMobileNav(true)}
              aria-label="Open navigation"
            >
              <Layers size={14} />
            </button>
            <form
              className="flex-1 max-w-2xl"
              onSubmit={(e) => {
                e.preventDefault();
                if (q.trim()) navigate({ to: '/search', search: { q: q.trim() } });
              }}
            >
              <div className="relative">
                <SearchIcon
                  size={14}
                  className="absolute left-3 top-1/2 -translate-y-1/2 text-fg-subtle"
                />
                <input
                  ref={inputRef}
                  type="search"
                  placeholder="Search digest, address, package, label, protocol …"
                  className="w-full bg-bg-subtle border border-border-subtle rounded-md pl-9 pr-20 py-2 text-sm focus:outline-none focus:border-accent placeholder:text-fg-subtle"
                  value={q}
                  onChange={(e) => setQ(e.target.value)}
                />
                <span className="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1 text-[10px] text-fg-subtle">
                  <kbd className="border border-border-subtle rounded px-1 mono">⌘K</kbd>
                  palette
                </span>
              </div>
            </form>
            {user ? (
              <div className="flex items-center gap-2 text-xs text-fg-muted">
                <span className="mono px-2 py-0.5 rounded bg-bg-elev border border-border-subtle">
                  {user.address.slice(0, 6)}…{user.address.slice(-4)}
                </span>
                <button
                  className="text-fg-subtle hover:text-fg text-xs"
                  onClick={() => {
                    logout();
                    window.location.href = '/';
                  }}
                >
                  logout
                </button>
              </div>
            ) : (
              <Link
                to="/login"
                className="text-xs text-accent hover:text-accent-fg border border-accent/40 rounded-md px-2.5 py-1.5"
              >
                Sign in
              </Link>
            )}
          </div>
        </header>
        <main className="flex-1 px-4 lg:px-6 py-6 max-w-7xl w-full mx-auto">{children}</main>
        <footer className="border-t border-border-subtle text-xs text-fg-subtle">
          <div className="px-4 lg:px-6 py-3 flex items-center justify-between max-w-7xl mx-auto">
            <span>
              sui-trace · developer-first explorer for Sui · API <code className="mono">/api/v1</code>
            </span>
            <span className="mono">v0.1.0</span>
          </div>
        </footer>
      </div>
    </div>
  );
}

function SidebarBrand() {
  return (
    <Link to="/" className="flex items-center gap-2 px-2 font-semibold tracking-tight">
      <span className="inline-flex w-8 h-8 rounded-lg bg-accent/15 border border-accent/40 items-center justify-center text-accent">
        <Compass size={16} />
      </span>
      <span>
        sui-<span className="text-accent">trace</span>
      </span>
    </Link>
  );
}

function NetworkPill({ cp, stale }: { cp: { sequence_number: number; epoch: number; timestamp_ms: number } | null | undefined; stale: boolean }) {
  return (
    <div className="border border-border-subtle rounded-lg p-2.5 text-xs space-y-1">
      <div className="flex items-center justify-between">
        <span className="flex items-center gap-1.5 text-fg-subtle">
          <span className={cn('live-dot', stale && 'warn')} /> network
        </span>
        <span className="text-fg-subtle text-[10px] uppercase">mainnet</span>
      </div>
      <div className="flex items-center justify-between">
        <span className="text-fg-subtle">epoch</span>
        <span className="mono text-fg">{cp ? cp.epoch : '—'}</span>
      </div>
      <div className="flex items-center justify-between">
        <span className="text-fg-subtle">checkpoint</span>
        <span className="mono text-fg">{cp ? `#${cp.sequence_number}` : '—'}</span>
      </div>
    </div>
  );
}
