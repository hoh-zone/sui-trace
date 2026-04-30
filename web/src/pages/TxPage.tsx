import { useQuery } from '@tanstack/react-query';
import { Link, useParams } from '@tanstack/react-router';
import {
  AlertOctagon,
  ArrowDownRight,
  ArrowUpRight,
  Bookmark,
  Boxes,
  CheckCircle2,
  Coins,
  Cpu,
  Database,
  Fingerprint,
  Layers,
  ListTree,
  Receipt,
  ShieldAlert,
  Tag as TagIcon,
  Wallet,
  XCircle,
} from 'lucide-react';
import { Card } from '@/components/Card';
import { Loader, ErrorBlock, Empty } from '@/components/Loader';
import { Badge } from '@/components/Badge';
import { Hash } from '@/components/Hash';
import { Tabs } from '@/components/Tabs';
import { JsonView } from '@/components/JsonView';
import { CopyButton } from '@/components/CopyButton';
import {
  api,
  type AddressLabel,
  type SuiArg,
  type SuiBalanceChange,
  type SuiCallArg,
  type SuiCommand,
  type SuiObjectChange,
  type SuiOwner,
  type SuiRpcTransaction,
  type Tx,
  type TxFull,
} from '@/lib/api';
import {
  formatGas,
  formatNumber,
  formatRelative,
  formatSui,
  formatTime,
} from '@/lib/format';
import { cn } from '@/lib/cn';

export function TxPage() {
  const { digest } = useParams({ from: '/tx/$digest' });
  const q = useQuery({
    queryKey: ['tx-full', digest],
    queryFn: () => api.txFull(digest),
  });

  if (q.isLoading) return <Loader />;
  if (q.error) return <ErrorBlock error={q.error} onRetry={() => q.refetch()} />;
  const data = q.data!;
  const indexed = data.indexed;
  const rpc = data.rpc;
  if (!indexed && !rpc) return <Empty label="Transaction not found." />;

  const status: 'success' | 'failure' =
    rpc?.effects?.status?.status ?? indexed?.status ?? 'failure';
  const error = rpc?.effects?.status?.error;
  const sender = rpc?.transaction?.data?.sender ?? indexed?.sender ?? '0x0';
  const ts =
    indexed?.timestamp ??
    (rpc?.timestampMs ? new Date(Number(rpc.timestampMs)).toISOString() : new Date().toISOString());
  const checkpointSeq =
    indexed?.checkpoint_seq ?? (rpc?.checkpoint ? Number(rpc.checkpoint) : null);
  const gas = rpc?.effects?.gasUsed;
  const gasBudget = rpc?.transaction?.data?.gasData?.budget;
  const gasPrice = rpc?.transaction?.data?.gasData?.price ?? String(indexed?.gas_price ?? '');
  const txKind = txKindLabel(rpc, indexed);
  const totalGas = gas ? totalGasUsed(gas) : indexed?.gas_used ?? 0;

  return (
    <div className="space-y-6">
      <Header
        digest={digest}
        status={status}
        error={error}
        kind={txKind}
        timestamp={ts}
        checkpointSeq={checkpointSeq}
        sender={sender}
        senderLabels={data.labels[sender] ?? []}
        totalGas={totalGas}
      />

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <Stat label="Gas used (total)" value={formatGas(totalGas)} hint={formatSui(totalGas)} icon={<Cpu size={14} />} />
        <Stat
          label="Gas budget"
          value={gasBudget ? formatGas(Number(gasBudget)) : '—'}
          hint={gasBudget ? formatSui(Number(gasBudget)) : undefined}
          icon={<Coins size={14} />}
        />
        <Stat
          label="Gas price"
          value={gasPrice ? `${formatNumber(Number(gasPrice))} mist` : '—'}
          icon={<Receipt size={14} />}
        />
        <Stat
          label="Dependencies"
          value={formatNumber(rpc?.effects?.dependencies?.length ?? 0)}
          hint="upstream tx refs"
          icon={<Layers size={14} />}
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
            id: 'ptb',
            label: (
              <span className="flex items-center gap-1.5">
                <Cpu size={12} />
                PTB
              </span>
            ),
            badge: rpcCommandCount(rpc),
            content: <PtbTab rpc={rpc} packagesMap={data.packages} />,
          },
          {
            id: 'balance',
            label: (
              <span className="flex items-center gap-1.5">
                <Coins size={12} />
                Balance changes
              </span>
            ),
            badge: rpc?.balanceChanges?.length ?? 0,
            content: <BalanceTab changes={rpc?.balanceChanges ?? []} labels={data.labels} />,
          },
          {
            id: 'objects',
            label: (
              <span className="flex items-center gap-1.5">
                <Boxes size={12} />
                Object changes
              </span>
            ),
            badge: rpc?.objectChanges?.length ?? 0,
            content: <ObjectTab changes={rpc?.objectChanges ?? []} labels={data.labels} />,
          },
          {
            id: 'events',
            label: (
              <span className="flex items-center gap-1.5">
                <ListTree size={12} />
                Events
              </span>
            ),
            badge: data.events.length || rpc?.events?.length || 0,
            content: <EventsTab fullEvents={rpc?.events ?? []} indexedEvents={data.events} packagesMap={data.packages} />,
          },
          {
            id: 'gas',
            label: (
              <span className="flex items-center gap-1.5">
                <Database size={12} />
                Gas
              </span>
            ),
            content: <GasTab rpc={rpc} indexed={indexed} />,
          },
          {
            id: 'sigs',
            label: (
              <span className="flex items-center gap-1.5">
                <Fingerprint size={12} />
                Signatures
              </span>
            ),
            badge: rpc?.transaction?.txSignatures?.length ?? 0,
            content: <SignaturesTab sigs={rpc?.transaction?.txSignatures ?? []} />,
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

/* ------------------------------ Header ------------------------------ */

function Header({
  digest,
  status,
  error,
  kind,
  timestamp,
  checkpointSeq,
  sender,
  senderLabels,
  totalGas,
}: {
  digest: string;
  status: 'success' | 'failure';
  error?: string;
  kind: string;
  timestamp: string;
  checkpointSeq: number | null;
  sender: string;
  senderLabels: AddressLabel[];
  totalGas: number;
}) {
  return (
    <div className="border border-border-subtle rounded-xl bg-bg-subtle/40 p-4 lg:p-5 space-y-3">
      <div className="flex items-start justify-between gap-3 flex-wrap">
        <div className="min-w-0">
          <div className="text-[10px] uppercase tracking-widest text-fg-subtle">Transaction</div>
          <div className="mt-1 flex items-center gap-2 text-fg">
            <Receipt size={16} className="text-accent shrink-0" />
            <span className="mono break-all">{digest}</span>
            <CopyButton value={digest} silent />
          </div>
        </div>
        <div className="flex items-center gap-2">
          {status === 'success' ? (
            <Badge variant="success">
              <CheckCircle2 size={12} /> success
            </Badge>
          ) : (
            <Badge variant="danger">
              <XCircle size={12} /> failure
            </Badge>
          )}
          <Badge variant="accent">{kind}</Badge>
        </div>
      </div>

      {error && (
        <div className="flex items-start gap-2 text-xs text-danger bg-danger/10 border border-danger/30 rounded p-2">
          <AlertOctagon size={14} className="shrink-0 mt-0.5" />
          <span className="break-all">{error}</span>
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-3 gap-3 text-sm">
        <Field label="Sender">
          <Hash value={sender} kind="address" short={false} />
          {senderLabels.map((l) => (
            <span
              key={`${l.label}-${l.source}`}
              className="text-[10px] px-1.5 py-0.5 rounded border border-border-subtle text-fg-subtle ml-1"
              title={`${l.category} · ${l.source}`}
            >
              <TagIcon size={9} className="inline mr-0.5" /> {l.label}
            </span>
          ))}
        </Field>
        <Field label="Time">
          {formatTime(timestamp)}{' '}
          <span className="text-xs text-fg-subtle">({formatRelative(timestamp)})</span>
        </Field>
        <Field label="Checkpoint">
          {checkpointSeq != null ? (
            <Link
              to="/checkpoint/$seq"
              params={{ seq: String(checkpointSeq) }}
              className="text-accent hover:underline mono"
            >
              #{formatNumber(checkpointSeq)}
            </Link>
          ) : (
            <span className="text-fg-subtle">—</span>
          )}
          <span className="ml-2 text-xs text-fg-subtle">gas {formatGas(totalGas)}</span>
        </Field>
      </div>
    </div>
  );
}

function Stat({
  label,
  value,
  hint,
  icon,
}: {
  label: string;
  value: React.ReactNode;
  hint?: string;
  icon?: React.ReactNode;
}) {
  return (
    <div className="border border-border-subtle rounded-lg px-4 py-3 bg-bg-subtle/40">
      <div className="text-[10px] uppercase tracking-wider text-fg-subtle flex items-center gap-1">
        {icon}
        {label}
      </div>
      <div className="mt-1 text-lg font-semibold mono">{value}</div>
      {hint && <div className="text-xs text-fg-subtle">{hint}</div>}
    </div>
  );
}

/* ------------------------------ Overview ------------------------------ */

function Overview({ data }: { data: TxFull }) {
  const rpc = data.rpc;
  const inputs = ptbInputs(rpc);
  const cmds = ptbCommands(rpc);
  const balance = rpc?.balanceChanges ?? [];
  const obj = rpc?.objectChanges ?? [];
  return (
    <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
      <Card title="Effects summary" className="lg:col-span-2">
        <dl className="grid grid-cols-2 sm:grid-cols-3 gap-3 text-sm">
          <Mini icon={<Cpu size={12} />} label="Commands" value={cmds.length} />
          <Mini icon={<Layers size={12} />} label="Inputs" value={inputs.length} />
          <Mini icon={<Coins size={12} />} label="Balance changes" value={balance.length} />
          <Mini icon={<Boxes size={12} />} label="Object changes" value={obj.length} />
          <Mini
            icon={<ListTree size={12} />}
            label="Events"
            value={rpc?.events?.length ?? data.events.length}
          />
          <Mini
            icon={<Bookmark size={12} />}
            label="Shared objs"
            value={rpc?.effects?.sharedObjects?.length ?? 0}
          />
        </dl>
      </Card>
      <Card title="Risk signals">
        <RiskSignals data={data} />
      </Card>
    </div>
  );
}

function RiskSignals({ data }: { data: TxFull }) {
  const labels = Object.entries(data.labels);
  const risky = labels.filter(([, ls]) =>
    ls.some((l) =>
      ['hacker', 'scam', 'phishing', 'mixer', 'sanctioned', 'rug_pull'].includes(l.category),
    ),
  );
  const insecurePkgs = Object.entries(data.packages).filter(
    ([, p]) => p.max_severity === 'high' || p.max_severity === 'critical',
  );
  if (risky.length === 0 && insecurePkgs.length === 0) {
    return (
      <p className="text-sm text-fg-subtle">
        No high-risk addresses or packages flagged for this transaction.
      </p>
    );
  }
  return (
    <ul className="space-y-2 text-sm">
      {risky.map(([addr, ls]) => (
        <li key={addr} className="flex items-start gap-2">
          <ShieldAlert size={14} className="text-danger shrink-0 mt-0.5" />
          <div className="min-w-0">
            <Hash value={addr} kind="address" copy={false} />
            <div className="text-xs text-fg-subtle">
              {ls.map((l) => `${l.label} (${l.category})`).join(' · ')}
            </div>
          </div>
        </li>
      ))}
      {insecurePkgs.map(([pkg, p]) => (
        <li key={pkg} className="flex items-start gap-2">
          <ShieldAlert
            size={14}
            className={cn(
              'shrink-0 mt-0.5',
              p.max_severity === 'critical' ? 'text-danger' : 'text-warn',
            )}
          />
          <div className="min-w-0">
            <Hash value={pkg} kind="package" copy={false} />
            <div className="text-xs text-fg-subtle">
              {p.findings_count} findings · max severity {p.max_severity} · score{' '}
              {p.score.toFixed(1)}
            </div>
          </div>
        </li>
      ))}
    </ul>
  );
}

function Mini({
  icon,
  label,
  value,
}: {
  icon: React.ReactNode;
  label: string;
  value: number;
}) {
  return (
    <div className="border border-border-subtle rounded-md p-2 bg-bg/40">
      <dt className="text-[10px] uppercase tracking-wider text-fg-subtle flex items-center gap-1">
        {icon}
        {label}
      </dt>
      <dd className="mt-0.5 mono text-base">{formatNumber(value)}</dd>
    </div>
  );
}

/* ------------------------------ PTB tab ------------------------------ */

function PtbTab({
  rpc,
  packagesMap,
}: {
  rpc: SuiRpcTransaction | null;
  packagesMap: TxFull['packages'];
}) {
  const inputs = ptbInputs(rpc);
  const cmds = ptbCommands(rpc);
  if (!cmds.length && !inputs.length)
    return (
      <Empty
        label="No programmable transaction block"
        hint="This transaction is likely a system / consensus / genesis tx."
        icon={<Cpu size={16} />}
      />
    );
  return (
    <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
      <Card title="Inputs" className="lg:col-span-1" noPadding>
        {inputs.length ? (
          <ol className="text-xs">
            {inputs.map((inp, i) => (
              <li key={i} className="px-3 py-2 border-b border-border-subtle last:border-b-0">
                <div className="flex items-center justify-between">
                  <span className="font-medium text-fg">
                    [{i}] <span className="text-fg-subtle">{inp.type}</span>
                  </span>
                  <span className="text-[10px] uppercase text-fg-subtle">
                    {inp.type === 'pure' ? inp.valueType : 'object'}
                  </span>
                </div>
                {inp.type === 'pure' ? (
                  <code className="mono text-[11px] text-fg-muted break-all">
                    {JSON.stringify(inp.value)}
                  </code>
                ) : (
                  <div className="text-[11px] mt-0.5 space-y-0.5">
                    <div className="text-fg-subtle">{inp.objectType}</div>
                    <Hash value={inp.objectId} kind="address" />
                  </div>
                )}
              </li>
            ))}
          </ol>
        ) : (
          <p className="px-3 py-3 text-sm text-fg-subtle">no inputs</p>
        )}
      </Card>
      <Card title="Commands" className="lg:col-span-2" noPadding>
        <ol className="divide-y divide-border-subtle">
          {cmds.map((cmd, i) => (
            <CommandRow key={i} idx={i} cmd={cmd} packagesMap={packagesMap} />
          ))}
        </ol>
      </Card>
    </div>
  );
}

function CommandRow({
  idx,
  cmd,
  packagesMap,
}: {
  idx: number;
  cmd: SuiCommand;
  packagesMap: TxFull['packages'];
}) {
  const [name, body] = decodeCommand(cmd);
  const pkgInfo =
    name === 'MoveCall' && body && typeof body === 'object' && 'package' in body
      ? packagesMap[(body as { package: string }).package]
      : undefined;
  return (
    <li className="px-4 py-3">
      <div className="flex items-center justify-between gap-2 flex-wrap">
        <div className="flex items-center gap-2">
          <span className="text-xs text-fg-subtle">[{idx}]</span>
          <Badge variant="accent">{name}</Badge>
          {pkgInfo && (
            <span
              className={cn(
                'text-[10px] px-1.5 py-0.5 rounded border',
                pkgInfo.max_severity === 'critical'
                  ? 'border-danger/40 text-danger'
                  : pkgInfo.max_severity === 'high'
                    ? 'border-warn/40 text-warn'
                    : 'border-border-subtle text-fg-subtle',
              )}
              title={`security score ${pkgInfo.score.toFixed(2)}`}
            >
              <ShieldAlert size={9} className="inline mr-0.5" /> {pkgInfo.max_severity}
            </span>
          )}
        </div>
      </div>
      {name === 'MoveCall' ? (
        <MoveCallView call={body as MoveCallShape} />
      ) : (
        <pre className="mono text-[11px] text-fg-muted mt-2 overflow-x-auto bg-bg/40 border border-border-subtle rounded p-2">
          {JSON.stringify(body, null, 2)}
        </pre>
      )}
    </li>
  );
}

interface MoveCallShape {
  package: string;
  module: string;
  function: string;
  type_arguments?: string[];
  arguments?: SuiArg[];
}

function MoveCallView({ call }: { call: MoveCallShape }) {
  return (
    <div className="mt-2 text-xs space-y-1">
      <div className="mono text-fg break-all">
        <Hash value={call.package} kind="package" copy={false} />
        <span className="text-fg-subtle">::</span>
        <span className="text-info">{call.module}</span>
        <span className="text-fg-subtle">::</span>
        <span className="text-accent">{call.function}</span>
      </div>
      {call.type_arguments && call.type_arguments.length > 0 && (
        <div className="text-fg-subtle">
          types:{' '}
          {call.type_arguments.map((t, i) => (
            <span key={i} className="mono text-fg-muted mr-1">
              &lt;{t}&gt;
            </span>
          ))}
        </div>
      )}
      {call.arguments && call.arguments.length > 0 && (
        <div className="text-fg-subtle">
          args:{' '}
          {call.arguments.map((a, i) => (
            <span key={i} className="mono text-fg-muted mr-1.5">
              {argLabel(a)}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}

/* ------------------------------ Balance changes ------------------------------ */

function BalanceTab({
  changes,
  labels,
}: {
  changes: SuiBalanceChange[];
  labels: TxFull['labels'];
}) {
  if (!changes.length)
    return <Empty label="No balance changes" icon={<Coins size={16} />} />;
  return (
    <div className="border border-border-subtle rounded-lg overflow-hidden">
      <table className="w-full text-sm">
        <thead className="bg-bg-elev/40 text-[10px] uppercase tracking-wider text-fg-subtle">
          <tr>
            <th className="text-left px-3 py-2">Owner</th>
            <th className="text-left px-3 py-2">Coin type</th>
            <th className="text-right px-3 py-2">Amount</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-border-subtle">
          {changes.map((c, i) => {
            const owner = ownerAddress(c.owner);
            const amt = BigInt(c.amount);
            const positive = amt > 0n;
            const isSui = c.coinType === '0x2::sui::SUI';
            const decimals = isSui ? 9 : null;
            return (
              <tr key={i} className="hover:bg-bg-elev/30">
                <td className="px-3 py-2 align-top">
                  <div className="flex items-center gap-1.5">
                    {positive ? (
                      <ArrowDownRight size={12} className="text-ok" />
                    ) : (
                      <ArrowUpRight size={12} className="text-danger" />
                    )}
                    {owner ? (
                      <Hash value={owner} kind="address" copy={false} />
                    ) : (
                      <span className="text-fg-subtle text-xs">{ownerKindLabel(c.owner)}</span>
                    )}
                  </div>
                  {owner && labels[owner]?.length ? (
                    <div className="text-[10px] text-fg-subtle mt-0.5">
                      {labels[owner].map((l) => l.label).join(', ')}
                    </div>
                  ) : null}
                </td>
                <td className="px-3 py-2 align-top mono text-xs text-fg-muted break-all">
                  {c.coinType}
                </td>
                <td
                  className={cn(
                    'px-3 py-2 text-right mono align-top',
                    positive ? 'text-ok' : 'text-danger',
                  )}
                >
                  {decimals != null
                    ? formatDecimal(c.amount, decimals)
                    : c.amount}
                  {isSui && <span className="text-xs text-fg-subtle ml-1">SUI</span>}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

/* ------------------------------ Object changes ------------------------------ */

function ObjectTab({
  changes,
  labels,
}: {
  changes: SuiObjectChange[];
  labels: TxFull['labels'];
}) {
  if (!changes.length) return <Empty label="No object changes" icon={<Boxes size={16} />} />;
  const buckets: Record<string, SuiObjectChange[]> = {};
  for (const c of changes) {
    const k = c.type;
    (buckets[k] ??= []).push(c);
  }
  const order = ['published', 'created', 'mutated', 'transferred', 'unwrapped', 'wrapped', 'deleted'];
  const sortedKeys = order.filter((k) => buckets[k]).concat(
    Object.keys(buckets).filter((k) => !order.includes(k)),
  );
  return (
    <div className="space-y-4">
      {sortedKeys.map((k) => (
        <Card
          key={k}
          title={
            <span className="flex items-center gap-2 capitalize">
              <Boxes size={14} className="text-accent" />
              {k}
              <Badge variant="default">{buckets[k].length}</Badge>
            </span>
          }
          noPadding
        >
          <ul className="divide-y divide-border-subtle text-sm">
            {buckets[k].map((c, i) => (
              <ObjectRow key={i} c={c} labels={labels} />
            ))}
          </ul>
        </Card>
      ))}
    </div>
  );
}

function ObjectRow({
  c,
  labels,
}: {
  c: SuiObjectChange;
  labels: TxFull['labels'];
}) {
  switch (c.type) {
    case 'published':
      return (
        <li className="px-3 py-2 flex items-center gap-3 flex-wrap">
          <Badge variant="success">published</Badge>
          <Hash value={c.packageId} kind="package" />
          <span className="text-xs text-fg-subtle">v{c.version}</span>
          <span className="text-xs text-fg-subtle">{c.modules.length} modules</span>
        </li>
      );
    case 'deleted':
    case 'wrapped':
      return (
        <li className="px-3 py-2 flex items-center gap-3 flex-wrap">
          <Badge variant="default">{c.type}</Badge>
          <Hash value={c.objectId} kind="address" />
          <span className="mono text-xs text-fg-subtle break-all">{c.objectType}</span>
        </li>
      );
  }
  const owner = ownerAddress(c.owner);
  return (
    <li className="px-3 py-2 grid grid-cols-1 md:grid-cols-[1fr_auto] gap-2">
      <div className="min-w-0">
        <div className="flex items-center gap-2 flex-wrap">
          <Badge variant={c.type === 'created' ? 'success' : 'accent'}>{c.type}</Badge>
          <Hash value={c.objectId} kind="address" />
        </div>
        <div className="text-xs text-fg-subtle mt-0.5 break-all mono">{c.objectType}</div>
      </div>
      <div className="text-xs text-fg-subtle md:text-right space-y-0.5">
        <div>v{c.version}</div>
        <div className="flex items-center gap-1 md:justify-end">
          owner:{' '}
          {owner ? (
            <>
              <Hash value={owner} kind="address" copy={false} />
              {labels[owner]?.length ? (
                <span className="text-[10px] text-fg-subtle">
                  ({labels[owner].map((l) => l.label).join(',')})
                </span>
              ) : null}
            </>
          ) : (
            <span>{ownerKindLabel(c.owner)}</span>
          )}
        </div>
      </div>
    </li>
  );
}

/* ------------------------------ Events ------------------------------ */

function EventsTab({
  fullEvents,
  indexedEvents,
  packagesMap,
}: {
  fullEvents: NonNullable<SuiRpcTransaction['events']>;
  indexedEvents: TxFull['events'];
  packagesMap: TxFull['packages'];
}) {
  // Prefer the live RPC events (richer parsedJson). Fall back to indexed.
  const list = fullEvents.length
    ? fullEvents.map((e, i) => ({
        seq: i,
        package_id: e.packageId,
        module: e.transactionModule,
        event_type: e.type,
        sender: e.sender,
        payload: e.parsedJson ?? null,
      }))
    : indexedEvents.map((e) => ({
        seq: e.event_seq,
        package_id: e.package_id,
        module: e.module,
        event_type: e.event_type,
        sender: e.sender,
        payload: e.payload,
      }));
  if (!list.length)
    return <Empty label="No events emitted by this transaction" icon={<ListTree size={16} />} />;
  return (
    <ul className="space-y-3 text-sm">
      {list.map((e) => {
        const pkgInfo = packagesMap[e.package_id];
        return (
          <li key={e.seq} className="border border-border-subtle rounded-lg overflow-hidden">
            <div className="px-3 py-2 border-b border-border-subtle bg-bg-elev/40 flex items-center justify-between gap-2 flex-wrap">
              <span className="flex items-center gap-2 min-w-0">
                <Badge variant="accent">{e.event_type.split('::').slice(-1)[0]}</Badge>
                <span className="mono text-xs text-fg-muted truncate">{e.event_type}</span>
              </span>
              <span className="text-xs text-fg-subtle">seq #{e.seq}</span>
            </div>
            <div className="p-3 space-y-2">
              <div className="text-xs text-fg-muted flex items-center gap-2 flex-wrap">
                <span className="text-fg-subtle">module </span>
                <span className="mono">{e.module}</span>
                <span className="text-fg-subtle">· package</span>
                <Hash value={e.package_id} kind="package" copy={false} />
                {pkgInfo && (
                  <span className="text-[10px] text-fg-subtle">
                    sec {pkgInfo.max_severity}
                  </span>
                )}
              </div>
              {e.payload ? (
                <JsonView value={e.payload} maxHeight={240} />
              ) : (
                <p className="text-xs text-fg-subtle">no payload</p>
              )}
            </div>
          </li>
        );
      })}
    </ul>
  );
}

/* ------------------------------ Gas tab ------------------------------ */

function GasTab({
  rpc,
  indexed,
}: {
  rpc: SuiRpcTransaction | null;
  indexed: Tx | null;
}) {
  const gas = rpc?.effects?.gasUsed;
  const data = rpc?.transaction?.data?.gasData;
  const total = gas
    ? Number(gas.computationCost) + Number(gas.storageCost) - Number(gas.storageRebate)
    : indexed?.gas_used ?? 0;
  return (
    <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
      <Card title="Gas used breakdown" className="lg:col-span-2">
        {gas ? (
          <div className="grid grid-cols-2 sm:grid-cols-3 gap-3 text-sm">
            <GasMini label="Computation" value={gas.computationCost} />
            <GasMini label="Storage cost" value={gas.storageCost} />
            <GasMini label="Storage rebate" value={gas.storageRebate} negative />
            <GasMini label="Non-refundable" value={gas.nonRefundableStorageFee} />
            <GasMini label="Net total" value={String(total)} bold />
          </div>
        ) : (
          <p className="text-sm text-fg-subtle">No detailed gas breakdown available.</p>
        )}
      </Card>
      <Card title="Gas data">
        <dl className="text-sm space-y-2">
          <div className="flex items-center justify-between">
            <dt className="text-fg-subtle">Budget</dt>
            <dd className="mono">{data?.budget ?? '—'}</dd>
          </div>
          <div className="flex items-center justify-between">
            <dt className="text-fg-subtle">Price</dt>
            <dd className="mono">{data?.price ? `${data.price} mist` : '—'}</dd>
          </div>
          <div className="flex items-center justify-between">
            <dt className="text-fg-subtle">Owner</dt>
            <dd>{data?.owner ? <Hash value={data.owner} kind="address" copy={false} /> : '—'}</dd>
          </div>
          <div>
            <dt className="text-fg-subtle text-xs mb-1">Payment coins</dt>
            <dd className="space-y-1">
              {(data?.payment ?? []).map((p) => (
                <div key={p.objectId} className="text-xs flex items-center gap-2">
                  <Wallet size={10} className="text-fg-subtle" />
                  <Hash value={p.objectId} kind="address" copy={false} />
                  <span className="text-fg-subtle">v{p.version}</span>
                </div>
              ))}
              {(data?.payment ?? []).length === 0 && (
                <span className="text-fg-subtle text-xs">—</span>
              )}
            </dd>
          </div>
        </dl>
      </Card>
    </div>
  );
}

function GasMini({
  label,
  value,
  negative,
  bold,
}: {
  label: string;
  value: string;
  negative?: boolean;
  bold?: boolean;
}) {
  const num = Number(value);
  return (
    <div className="border border-border-subtle rounded-md p-2 bg-bg/40">
      <div className="text-[10px] uppercase tracking-wider text-fg-subtle">{label}</div>
      <div
        className={cn(
          'mono mt-0.5',
          bold && 'text-base font-semibold',
          !bold && 'text-sm',
          negative && 'text-ok',
        )}
      >
        {negative ? '-' : ''}
        {formatNumber(Math.abs(num))}
      </div>
      <div className="text-[10px] text-fg-subtle">{formatSui(num)}</div>
    </div>
  );
}

/* ------------------------------ Signatures ------------------------------ */

function SignaturesTab({ sigs }: { sigs: string[] }) {
  if (!sigs.length) return <Empty label="No signatures available" icon={<Fingerprint size={16} />} />;
  return (
    <ul className="space-y-2 text-sm">
      {sigs.map((s, i) => (
        <li
          key={i}
          className="border border-border-subtle rounded-lg p-3 flex items-start gap-3 bg-bg-subtle/30"
        >
          <Fingerprint size={14} className="text-accent shrink-0 mt-0.5" />
          <div className="min-w-0 flex-1">
            <div className="text-[10px] uppercase tracking-wider text-fg-subtle mb-1">
              Signature #{i + 1}
            </div>
            <code className="mono text-xs break-all text-fg-muted">{s}</code>
          </div>
          <CopyButton value={s} silent />
        </li>
      ))}
    </ul>
  );
}

/* ------------------------------ helpers ------------------------------ */

function Field({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <div className="border-l-2 border-border-subtle pl-3">
      <div className="text-[10px] uppercase tracking-wider text-fg-subtle">{label}</div>
      <div className="mt-1 text-sm flex items-center gap-2 flex-wrap min-w-0">{children}</div>
    </div>
  );
}

function txKindLabel(rpc: SuiRpcTransaction | null, indexed: Tx | null): string {
  const kind = rpc?.transaction?.data?.transaction?.kind ?? indexed?.kind;
  if (!kind) return 'unknown';
  if (kind === 'ProgrammableTransaction') return 'PTB';
  return kind;
}

function totalGasUsed(g: NonNullable<SuiRpcTransaction['effects']>['gasUsed']): number {
  if (!g) return 0;
  return Number(g.computationCost) + Number(g.storageCost) - Number(g.storageRebate);
}

function rpcCommandCount(rpc: SuiRpcTransaction | null): number {
  return ptbCommands(rpc).length;
}

function ptbInputs(rpc: SuiRpcTransaction | null): SuiCallArg[] {
  const k = rpc?.transaction?.data?.transaction;
  if (k && (k as { kind: string }).kind === 'ProgrammableTransaction') {
    return (k as { inputs?: SuiCallArg[] }).inputs ?? [];
  }
  return [];
}

function ptbCommands(rpc: SuiRpcTransaction | null): SuiCommand[] {
  const k = rpc?.transaction?.data?.transaction;
  if (k && (k as { kind: string }).kind === 'ProgrammableTransaction') {
    return (k as { transactions?: SuiCommand[] }).transactions ?? [];
  }
  return [];
}

function decodeCommand(cmd: SuiCommand): [string, unknown] {
  const entries = Object.entries(cmd as Record<string, unknown>);
  if (entries.length === 1) return entries[0];
  return ['Unknown', cmd];
}

function argLabel(a: SuiArg): string {
  if (a === 'GasCoin') return 'GasCoin';
  if ('Input' in a) return `Input(${a.Input})`;
  if ('Result' in a) return `Result(${a.Result})`;
  if ('NestedResult' in a) return `Nested(${a.NestedResult[0]},${a.NestedResult[1]})`;
  return JSON.stringify(a);
}

function ownerAddress(o: SuiOwner): string | null {
  if (typeof o === 'string') return o === 'Immutable' ? null : o;
  if ('AddressOwner' in o) return o.AddressOwner;
  if ('ObjectOwner' in o) return o.ObjectOwner;
  return null;
}

function ownerKindLabel(o: SuiOwner): string {
  if (typeof o === 'string') return o;
  if ('Shared' in o) return `Shared (v${o.Shared.initial_shared_version})`;
  return 'Object-owned';
}

function formatDecimal(amountStr: string, decimals: number): string {
  const negative = amountStr.startsWith('-');
  const digits = negative ? amountStr.slice(1) : amountStr;
  const padded = digits.padStart(decimals + 1, '0');
  const intPart = padded.slice(0, padded.length - decimals);
  const fracPart = padded.slice(padded.length - decimals).replace(/0+$/, '');
  const sign = negative ? '-' : '';
  return fracPart ? `${sign}${intPart}.${fracPart}` : `${sign}${intPart}`;
}
