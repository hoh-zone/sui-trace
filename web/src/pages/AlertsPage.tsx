import { useQuery } from '@tanstack/react-query';
import { AlertTriangle, Bell } from 'lucide-react';
import { useState } from 'react';
import { Link } from '@tanstack/react-router';
import { Loader, ErrorBlock, Empty } from '@/components/Loader';
import { Badge } from '@/components/Badge';
import { JsonView } from '@/components/JsonView';
import { Tabs } from '@/components/Tabs';
import { api, type AlertRow } from '@/lib/api';
import { getToken } from '@/lib/auth';
import { formatRelative, formatTime } from '@/lib/format';

export function AlertsPage() {
  const token = getToken();
  const personal = useQuery({
    queryKey: ['alerts-personal'],
    queryFn: () => api.recentAlerts(token!, 100),
    enabled: !!token,
    refetchInterval: 30_000,
  });
  const feed = useQuery({
    queryKey: ['alerts-feed'],
    queryFn: () => api.alertsFeed(100),
    refetchInterval: 30_000,
  });

  return (
    <div className="space-y-4">
      <header className="flex items-end justify-between gap-4 flex-wrap">
        <div>
          <h1 className="text-xl font-semibold tracking-tight flex items-center gap-2">
            <AlertTriangle size={18} className="text-warn" /> Alerts
          </h1>
          <p className="text-sm text-fg-muted mt-1">
            Public alert stream and your personal watchlist alerts.
          </p>
        </div>
        {!token && (
          <Link
            to="/login"
            className="text-xs px-3 py-1.5 rounded border border-accent/40 text-accent hover:bg-accent/10"
          >
            Sign in for personal alerts
          </Link>
        )}
      </header>

      <Tabs
        tabs={[
          {
            id: 'feed',
            label: 'Public feed',
            badge: feed.data?.alerts.length,
            content: <AlertsList query={feed} emptyLabel="No alerts fired yet — quiet day on Sui." />,
          },
          {
            id: 'me',
            label: <span className="flex items-center gap-1.5"><Bell size={12} /> Your watchlists</span>,
            badge: personal.data?.alerts.length,
            content: token ? (
              <AlertsList query={personal} emptyLabel="No personal alerts yet." />
            ) : (
              <Empty
                label="Sign in to see alerts triggered by your watchlists."
                hint={
                  <Link to="/login" className="text-accent hover:underline">
                    Open sign-in →
                  </Link>
                }
              />
            ),
          },
        ]}
      />
    </div>
  );
}

interface AlertsQueryShape {
  isLoading: boolean;
  error: unknown;
  data?: { alerts: AlertRow[] };
  refetch?: () => void;
}

function AlertsList({ query, emptyLabel }: { query: AlertsQueryShape; emptyLabel: string }) {
  const [expanded, setExpanded] = useState<string | null>(null);

  if (query.isLoading) return <Loader />;
  if (query.error) return <ErrorBlock error={query.error} onRetry={query.refetch} />;
  if (!query.data?.alerts.length) return <Empty label={emptyLabel} />;

  return (
    <ul className="space-y-2 text-sm">
      {query.data.alerts.map((a) => (
        <li
          key={a.id}
          className="border border-border-subtle rounded-md p-3 cursor-pointer"
          onClick={() => setExpanded(expanded === a.id ? null : a.id)}
        >
          <div className="flex items-center justify-between gap-2">
            <span className="flex items-center gap-2">
              <Badge variant={a.delivered ? 'success' : 'warn'}>{a.delivered ? 'delivered' : 'pending'}</Badge>
              <Badge variant="outline">{a.rule_id}</Badge>
              <strong className="text-fg">{a.payload?.title ?? a.rule_id}</strong>
            </span>
            <span className="text-xs text-fg-subtle" title={formatTime(a.fired_at)}>
              {formatRelative(a.fired_at)}
            </span>
          </div>
          {a.payload?.body && <p className="mt-1 text-fg-muted text-xs whitespace-pre-wrap">{a.payload.body}</p>}
          {expanded === a.id && (
            <div className="mt-2">
              <JsonView value={a.payload} maxHeight={200} />
            </div>
          )}
        </li>
      ))}
    </ul>
  );
}
