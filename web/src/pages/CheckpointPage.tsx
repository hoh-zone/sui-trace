import { useQuery } from '@tanstack/react-query';
import { Link, useParams } from '@tanstack/react-router';
import { useState } from 'react';
import { ArrowLeft, ArrowRight, Clock } from 'lucide-react';
import { Card } from '@/components/Card';
import { Loader, ErrorBlock, Empty } from '@/components/Loader';
import { Badge } from '@/components/Badge';
import { Hash } from '@/components/Hash';
import { Tabs } from '@/components/Tabs';
import { Pagination } from '@/components/Pagination';
import { CopyButton } from '@/components/CopyButton';
import { api } from '@/lib/api';
import { formatGas, formatNumber, formatRelative, formatTime } from '@/lib/format';

const PAGE_SIZE = 50;

export function CheckpointPage() {
  const { seq } = useParams({ from: '/checkpoint/$seq' });
  const seqN = Number(seq);
  const [page, setPage] = useState(0);
  const q = useQuery({ queryKey: ['cp', seq], queryFn: () => api.checkpoint(seqN) });
  const txs = useQuery({
    queryKey: ['cp-tx', seq, page],
    queryFn: () => api.checkpointTxs(seqN, PAGE_SIZE, page * PAGE_SIZE),
  });

  if (q.isLoading) return <Loader />;
  if (q.error) return <ErrorBlock error={q.error} onRetry={() => q.refetch()} />;
  const cp = q.data?.checkpoint;
  if (!cp) return <Empty label="Checkpoint not found" />;

  return (
    <div className="space-y-6">
      <div className="flex items-end justify-between gap-4 flex-wrap">
        <div>
          <div className="text-xs text-fg-subtle uppercase tracking-wider">Checkpoint</div>
          <div className="mt-1 flex items-center gap-2">
            <Clock size={16} className="text-accent" />
            <span className="text-2xl font-semibold mono">#{cp.sequence_number}</span>
            <Badge variant="outline">epoch {cp.epoch}</Badge>
          </div>
          <div className="mt-1 text-xs text-fg-subtle">
            {formatTime(cp.timestamp_ms)} ({formatRelative(cp.timestamp_ms)})
          </div>
        </div>
        <div className="flex items-center gap-2 text-xs">
          <Link
            to="/checkpoint/$seq"
            params={{ seq: String(seqN - 1) }}
            className="px-2 py-1 rounded border border-border-subtle hover:bg-bg-elev flex items-center gap-1"
          >
            <ArrowLeft size={12} /> prev
          </Link>
          <Link
            to="/checkpoint/$seq"
            params={{ seq: String(seqN + 1) }}
            className="px-2 py-1 rounded border border-border-subtle hover:bg-bg-elev flex items-center gap-1"
          >
            next <ArrowRight size={12} />
          </Link>
        </div>
      </div>

      <Tabs
        tabs={[
          {
            id: 'overview',
            label: 'Overview',
            content: (
              <Card>
                <dl className="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-3 text-sm">
                  <Field label="Digest">
                    <span className="mono break-all">{cp.digest}</span>
                    <CopyButton value={cp.digest} silent />
                  </Field>
                  <Field label="Previous">
                    {cp.previous_digest ? (
                      <span className="mono break-all">{cp.previous_digest}</span>
                    ) : (
                      <span className="text-fg-subtle">—</span>
                    )}
                  </Field>
                  <Field label="Network total tx">
                    <span className="mono">{formatNumber(cp.network_total_transactions)}</span>
                  </Field>
                  <Field label="Epoch">
                    <span className="mono">{cp.epoch}</span>
                  </Field>
                </dl>
              </Card>
            ),
          },
          {
            id: 'tx',
            label: 'Transactions',
            badge: txs.data?.transactions.length,
            content: (
              <Card noPadding>
                {txs.isLoading ? (
                  <div className="p-4"><Loader /></div>
                ) : txs.data?.transactions.length ? (
                  <>
                    <table className="w-full text-sm">
                      <thead className="text-left text-fg-subtle text-[10px] uppercase tracking-wider bg-bg/40">
                        <tr>
                          <th className="px-4 py-2 font-medium">Digest</th>
                          <th className="font-medium">Sender</th>
                          <th className="font-medium">Status</th>
                          <th className="font-medium text-right">Gas</th>
                        </tr>
                      </thead>
                      <tbody>
                        {txs.data.transactions.map((t) => (
                          <tr key={t.digest} className="border-t border-border-subtle">
                            <td className="px-4 py-2 max-w-[18rem]">
                              <Hash value={t.digest} kind="tx" copy={false} />
                            </td>
                            <td className="max-w-[14rem]">
                              <Hash value={t.sender} kind="address" copy={false} />
                            </td>
                            <td>
                              <Badge variant={t.status === 'success' ? 'success' : 'danger'}>{t.status}</Badge>
                            </td>
                            <td className="mono text-xs text-fg-muted text-right pr-4">{formatGas(t.gas_used)}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                    <div className="px-4 py-2 border-t border-border-subtle">
                      <Pagination
                        page={page}
                        pageSize={PAGE_SIZE}
                        hasNext={txs.data.transactions.length === PAGE_SIZE}
                        onChange={setPage}
                      />
                    </div>
                  </>
                ) : (
                  <div className="p-4"><Empty label="No transactions in this checkpoint." /></div>
                )}
              </Card>
            ),
          },
        ]}
      />
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="border-l-2 border-border-subtle pl-3">
      <div className="text-[10px] uppercase tracking-wider text-fg-subtle">{label}</div>
      <div className="mt-1 text-sm flex items-center gap-2 flex-wrap min-w-0">{children}</div>
    </div>
  );
}
