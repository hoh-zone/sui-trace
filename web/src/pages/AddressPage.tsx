import { useQuery } from '@tanstack/react-query';
import { useParams } from '@tanstack/react-router';
import { useState } from 'react';
import { Activity, Tag, User } from 'lucide-react';
import { Card } from '@/components/Card';
import { Loader, ErrorBlock, Empty } from '@/components/Loader';
import { Badge } from '@/components/Badge';
import { Hash } from '@/components/Hash';
import { Tabs } from '@/components/Tabs';
import { Pagination } from '@/components/Pagination';
import { CopyButton } from '@/components/CopyButton';
import { JsonView } from '@/components/JsonView';
import { api } from '@/lib/api';
import { categoryColor, formatGas, formatRelative, formatTime } from '@/lib/format';

const PAGE_SIZE = 25;

export function AddressPage() {
  const { addr } = useParams({ from: '/address/$addr' });
  const [page, setPage] = useState(0);

  const detail = useQuery({ queryKey: ['address', addr], queryFn: () => api.address(addr) });
  const txs = useQuery({
    queryKey: ['address-tx', addr, page],
    queryFn: () => api.addressTxs(addr, PAGE_SIZE, page * PAGE_SIZE),
  });
  const events = useQuery({
    queryKey: ['address-evt', addr],
    queryFn: () => api.addressEvents(addr, 50),
  });

  if (detail.isLoading) return <Loader />;
  if (detail.error) return <ErrorBlock error={detail.error} onRetry={() => detail.refetch()} />;
  const data = detail.data!;

  return (
    <div className="space-y-6">
      <div className="flex items-end gap-4 justify-between flex-wrap">
        <div className="min-w-0">
          <div className="text-xs text-fg-subtle uppercase tracking-wider">Address</div>
          <div className="mt-1 flex items-center gap-2">
            <User size={16} className="text-accent" />
            <span className="mono break-all">{data.address}</span>
            <CopyButton value={data.address} />
          </div>
          <div className="mt-2 flex flex-wrap gap-1.5">
            {data.labels.length === 0 ? (
              <span className="text-xs text-fg-subtle">No labels yet · contribute one in Labels</span>
            ) : (
              data.labels.map((l) => (
                <span
                  key={`${l.label}-${l.source}`}
                  className={`px-2 py-0.5 text-[11px] font-medium uppercase tracking-wider border rounded ${categoryColor(
                    l.category,
                  )}`}
                  title={`source: ${l.source} · confidence ${(l.confidence * 100).toFixed(0)}%`}
                >
                  {l.label}
                  {l.verified ? ' ✓' : ''}
                </span>
              ))
            )}
          </div>
        </div>
        <div className="text-xs text-fg-subtle text-right">
          <div>{txs.data?.transactions.length ?? '—'} transactions on this page</div>
          <div>{events.data?.events.length ?? '—'} recent events as sender</div>
        </div>
      </div>

      <Tabs
        tabs={[
          {
            id: 'tx',
            label: <span className="flex items-center gap-1.5"><Activity size={12} /> Transactions</span>,
            badge: txs.data?.transactions.length,
            content: (
              <Card noPadding>
                {txs.isLoading ? (
                  <div className="p-4"><Loader /></div>
                ) : txs.error ? (
                  <div className="p-4"><ErrorBlock error={txs.error} /></div>
                ) : txs.data?.transactions.length ? (
                  <>
                    <table className="w-full text-sm">
                      <thead className="text-left text-fg-subtle text-[10px] uppercase tracking-wider bg-bg/40">
                        <tr>
                          <th className="px-4 py-2 font-medium">Digest</th>
                          <th className="font-medium">Status</th>
                          <th className="font-medium text-right">Gas</th>
                          <th className="px-4 font-medium text-right">Time</th>
                        </tr>
                      </thead>
                      <tbody>
                        {txs.data.transactions.map((t) => (
                          <tr key={t.digest} className="border-t border-border-subtle">
                            <td className="px-4 py-2 max-w-[18rem]">
                              <Hash value={t.digest} kind="tx" copy={false} />
                            </td>
                            <td>
                              <Badge variant={t.status === 'success' ? 'success' : 'danger'}>{t.status}</Badge>
                            </td>
                            <td className="mono text-xs text-fg-muted text-right">{formatGas(t.gas_used)}</td>
                            <td
                              className="px-4 text-xs text-fg-subtle text-right whitespace-nowrap"
                              title={formatTime(t.timestamp)}
                            >
                              {formatRelative(t.timestamp)}
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                    <div className="px-4 py-2 border-t border-border-subtle">
                      <Pagination
                        page={page}
                        pageSize={PAGE_SIZE}
                        hasNext={(txs.data.transactions.length ?? 0) === PAGE_SIZE}
                        onChange={setPage}
                      />
                    </div>
                  </>
                ) : (
                  <div className="p-4"><Empty /></div>
                )}
              </Card>
            ),
          },
          {
            id: 'events',
            label: <span className="flex items-center gap-1.5"><Tag size={12} /> Events</span>,
            badge: events.data?.events.length,
            content: events.isLoading ? (
              <Loader />
            ) : events.data?.events.length ? (
              <ul className="space-y-2 text-sm">
                {events.data.events.map((e) => (
                  <li key={`${e.tx_digest}-${e.event_seq}`} className="border border-border-subtle rounded p-3">
                    <div className="flex items-center justify-between gap-2 mb-1">
                      <Badge variant="accent">{e.event_type.split('::').slice(-1)[0]}</Badge>
                      <span className="text-xs text-fg-subtle">{formatRelative(e.timestamp)}</span>
                    </div>
                    <div className="text-xs text-fg-muted truncate mono">{e.event_type}</div>
                    <div className="text-xs text-fg-subtle mt-1">
                      tx <Hash value={e.tx_digest} kind="tx" copy={false} /> · package <Hash value={e.package_id} kind="package" copy={false} />
                    </div>
                  </li>
                ))}
              </ul>
            ) : (
              <Empty label="This address hasn't emitted events as a sender yet." />
            ),
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
