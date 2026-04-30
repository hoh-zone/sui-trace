import { useQuery } from '@tanstack/react-query';
import { useParams } from '@tanstack/react-router';
import { useState } from 'react';
import {
  Box,
  Code2,
  FileCode2,
  GitBranch,
  ListTree,
  ShieldCheck,
} from 'lucide-react';
import { Card } from '@/components/Card';
import { Loader, ErrorBlock, Empty } from '@/components/Loader';
import { Badge } from '@/components/Badge';
import { Hash } from '@/components/Hash';
import { Tabs } from '@/components/Tabs';
import { JsonView } from '@/components/JsonView';
import { CopyButton } from '@/components/CopyButton';
import { SeverityPill, SeverityScore } from '@/components/SeverityPill';
import { MoveSource } from '@/components/MoveSource';
import {
  api,
  type Finding,
  type ModuleSourceSummary,
  type PackageVersion,
  type SecurityReport,
} from '@/lib/api';
import { formatRelative, formatTime } from '@/lib/format';

export function PackagePage() {
  const { id } = useParams({ from: '/package/$id' });
  const q = useQuery({ queryKey: ['package', id], queryFn: () => api.pkg(id) });
  const events = useQuery({
    queryKey: ['package-events', id],
    queryFn: () => api.pkgEvents(id, 30),
  });
  const versions = useQuery({
    queryKey: ['package-versions', id],
    queryFn: () => api.pkgVersions(id),
  });
  const sources = useQuery({
    queryKey: ['package-sources', id],
    queryFn: () => api.pkgSources(id),
  });

  if (q.isLoading) return <Loader />;
  if (q.error) return <ErrorBlock error={q.error} onRetry={() => q.refetch()} />;
  const data = q.data!;
  const sec = data.security;

  return (
    <div className="space-y-6">
      <div className="flex items-end gap-4 justify-between flex-wrap">
        <div className="min-w-0">
          <div className="text-xs text-fg-subtle uppercase tracking-wider">Package</div>
          <div className="mt-1 flex items-center gap-2">
            <Box size={16} className="text-accent" />
            <span className="mono break-all">{data.package.id}</span>
            <CopyButton value={data.package.id} />
          </div>
          <div className="mt-1 flex items-center gap-2 text-xs text-fg-muted">
            <span>v{data.package.version}</span>
            <span>·</span>
            <span>{data.package.modules_count} modules</span>
            <span>·</span>
            <span title={formatTime(data.package.published_at)}>
              published {formatRelative(data.package.published_at)}
            </span>
          </div>
        </div>
        {sec && <SeverityScore score={sec.score} />}
      </div>

      <Tabs
        tabs={[
          { id: 'overview', label: 'Overview', content: <Overview pkg={data} /> },
          {
            id: 'modules',
            label: 'Modules',
            badge: data.modules.length,
            content: <ModulesTab modules={data.modules} />,
          },
          {
            id: 'source',
            label: (
              <span className="flex items-center gap-1.5">
                <Code2 size={12} />
                Source
              </span>
            ),
            badge: sources.data?.modules.length ?? 0,
            content: (
              <SourceTab
                packageId={id}
                modules={sources.data?.modules ?? []}
                loading={sources.isLoading}
              />
            ),
          },
          {
            id: 'versions',
            label: (
              <span className="flex items-center gap-1.5">
                <GitBranch size={12} />
                Versions
              </span>
            ),
            badge: versions.data?.versions.length ?? 0,
            content: (
              <VersionsTab
                current={data.package.id}
                lineage={versions.data?.versions ?? []}
                loading={versions.isLoading}
              />
            ),
          },
          {
            id: 'security',
            label: (
              <span className="flex items-center gap-1.5">
                <ShieldCheck size={12} />
                Security
              </span>
            ),
            badge: sec?.findings.length ?? 0,
            content: <SecurityTab report={sec} packageId={data.package.id} />,
          },
          {
            id: 'events',
            label: 'Events',
            badge: events.data?.events.length,
            content: events.isLoading ? (
              <Loader />
            ) : events.data?.events.length ? (
              <ul className="space-y-2 text-sm">
                {events.data.events.map((e) => (
                  <li
                    key={`${e.tx_digest}-${e.event_seq}`}
                    className="border border-border-subtle rounded p-3"
                  >
                    <div className="flex items-center justify-between gap-2 mb-1">
                      <Badge variant="accent">{e.event_type.split('::').slice(-1)[0]}</Badge>
                      <span className="text-xs text-fg-subtle">{formatRelative(e.timestamp)}</span>
                    </div>
                    <div className="text-xs text-fg-muted truncate mono">{e.event_type}</div>
                    <div className="text-xs text-fg-subtle mt-1">
                      tx <Hash value={e.tx_digest} kind="tx" copy={false} /> · sender{' '}
                      <Hash value={e.sender} kind="address" copy={false} />
                    </div>
                  </li>
                ))}
              </ul>
            ) : (
              <Empty label="No events emitted by this package yet." icon={<ListTree size={16} />} />
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

function Overview({ pkg }: { pkg: Awaited<ReturnType<typeof api.pkg>> }) {
  return (
    <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <Card title="Identity" className="lg:col-span-2">
        <dl className="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-3 text-sm">
          <Field label="Original ID">
            <Hash value={pkg.package.original_id} kind="package" short={false} />
          </Field>
          <Field label="Publisher">
            <Hash value={pkg.package.publisher} kind="address" short={false} />
          </Field>
          <Field label="Version">
            <span className="mono">v{pkg.package.version}</span>
          </Field>
          <Field label="Modules">
            <span className="mono">{pkg.package.modules_count}</span>
          </Field>
          <Field label="Published">
            {formatTime(pkg.package.published_at)}
            <span className="text-xs text-fg-subtle ml-1">({formatRelative(pkg.package.published_at)})</span>
          </Field>
          <Field label="Source">
            <Badge variant={pkg.package.source_verified ? 'success' : 'default'}>
              {pkg.package.source_verified ? 'verified' : 'unverified'}
            </Badge>
          </Field>
        </dl>
      </Card>
      <Card title="Security at a glance">
        {pkg.security ? (
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-xs text-fg-subtle uppercase tracking-wider">Max severity</span>
              <SeverityPill value={pkg.security.max_severity} />
            </div>
            <div className="flex items-center justify-between">
              <span className="text-xs text-fg-subtle uppercase tracking-wider">Findings</span>
              <span className="mono">{pkg.security.findings.length}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-xs text-fg-subtle uppercase tracking-wider">Last scan</span>
              <span className="text-xs">{formatRelative(pkg.security.scanned_at)}</span>
            </div>
            <div className="pt-2">
              <SeverityScore score={pkg.security.score} />
            </div>
          </div>
        ) : (
          <p className="text-sm text-fg-subtle">
            Not scanned yet — the security worker scans every newly published package and stores
            the report. You can re-enqueue manually with{' '}
            <code className="mono">redis-cli LPUSH trace:packages:to_scan</code>.
          </p>
        )}
      </Card>
    </div>
  );
}

function ModulesTab({
  modules,
}: {
  modules: Awaited<ReturnType<typeof api.pkg>>['modules'];
}) {
  if (!modules.length) return <Empty label="No modules indexed yet." icon={<FileCode2 size={16} />} />;
  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-3">
      {modules.map((m) => {
        const fns = extractFunctions(m.abi_json);
        return (
          <Card
            key={m.module_name}
            title={
              <span className="flex items-center gap-2">
                <FileCode2 size={14} className="text-accent" />
                <span className="mono">{m.module_name}</span>
              </span>
            }
            action={
              <span className="mono text-fg-subtle text-xs">
                {m.bytecode_hash.slice(0, 12)}…
                <CopyButton value={m.bytecode_hash} silent />
              </span>
            }
          >
            {fns.length ? (
              <ul className="text-xs space-y-1">
                {fns.map((f) => (
                  <li key={f.name} className="flex items-center justify-between">
                    <span className="mono text-fg">{f.name}</span>
                    <span className="text-fg-subtle">
                      {f.kind} {f.entry ? '· entry' : ''}
                    </span>
                  </li>
                ))}
              </ul>
            ) : (
              <details className="text-xs">
                <summary className="cursor-pointer text-fg-subtle">show ABI JSON</summary>
                <JsonView value={m.abi_json} maxHeight={240} className="mt-2" />
              </details>
            )}
          </Card>
        );
      })}
    </div>
  );
}

function SecurityTab({ report, packageId }: { report: SecurityReport | null; packageId: string }) {
  if (!report) {
    return (
      <Empty
        label="No security report yet."
        hint={
          <>
            The scanner picks up <span className="mono">{packageId.slice(0, 12)}…</span> automatically;
            check back in a minute or push to the scan queue manually.
          </>
        }
        icon={<ShieldCheck size={18} />}
      />
    );
  }
  return (
    <div className="space-y-4">
      <Card noPadding>
        <div className="flex items-center justify-between p-3">
          <div className="flex items-center gap-3">
            <SeverityPill value={report.max_severity} />
            <span className="text-sm text-fg-muted">{report.findings.length} findings</span>
            <span className="text-xs text-fg-subtle">scanned {formatRelative(report.scanned_at)}</span>
          </div>
          <SeverityScore score={report.score} />
        </div>
      </Card>
      {report.findings.length ? (
        <ul className="space-y-3 text-sm">
          {report.findings.map((f, i) => (
            <FindingRow key={i} f={f} />
          ))}
        </ul>
      ) : (
        <Empty label="No findings produced — package looks clean." />
      )}
    </div>
  );
}

function FindingRow({ f }: { f: Finding }) {
  return (
    <li className="border border-border-subtle rounded-lg p-4 bg-bg-subtle/30">
      <div className="flex items-center justify-between gap-2 mb-2">
        <div className="flex items-center gap-2">
          <SeverityPill value={f.severity} />
          <span className="mono text-xs text-fg-muted">{f.rule_id}</span>
          <span className="font-medium text-fg">{f.rule_name}</span>
        </div>
        <span className="text-[11px] text-fg-subtle uppercase tracking-wide">
          confidence {(f.confidence * 100).toFixed(0)}%
        </span>
      </div>
      <div className="text-xs text-fg-muted mono">
        {f.location}
        {f.function ? ` · ${f.function}` : ''}
      </div>
      <p className="mt-2 text-fg">{f.message}</p>
      {f.suggestion && <p className="mt-1 text-fg-muted text-xs">→ {f.suggestion}</p>}
    </li>
  );
}

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
      <div className="mt-1 text-sm flex items-center gap-1.5 flex-wrap min-w-0">{children}</div>
    </div>
  );
}

interface FunctionInfo {
  name: string;
  kind: string;
  entry: boolean;
}

function VersionsTab({
  current,
  lineage,
  loading,
}: {
  current: string;
  lineage: PackageVersion[];
  loading: boolean;
}) {
  if (loading) return <Loader />;
  if (!lineage.length) {
    return (
      <Empty
        label="No version lineage recorded yet."
        hint={
          <>
            New publishes & upgrades land in <code className="mono">package_versions</code> as the
            indexer ingests them. Existing rows were back-filled by migration{' '}
            <span className="mono">20260101_03</span>.
          </>
        }
        icon={<GitBranch size={18} />}
      />
    );
  }
  // newest first for display
  const items = [...lineage].sort((a, b) => b.version - a.version);
  return (
    <ol className="relative border-l border-border-subtle ml-2 space-y-4">
      {items.map((v) => {
        const isCurrent = v.package_id === current;
        return (
          <li key={v.package_id} className="pl-5 relative">
            <span
              className={`absolute -left-1.5 top-1 w-3 h-3 rounded-full border-2 ${
                isCurrent
                  ? 'bg-accent border-accent shadow-[0_0_0_4px_rgba(56,178,255,0.18)]'
                  : 'bg-bg border-border-subtle'
              }`}
            />
            <div className="flex flex-wrap items-center gap-2">
              <Badge variant={isCurrent ? 'accent' : 'default'}>v{v.version}</Badge>
              <Hash value={v.package_id} kind="package" />
              {isCurrent && (
                <span className="text-[10px] uppercase tracking-wider text-accent">
                  current
                </span>
              )}
            </div>
            <div className="mt-1 text-xs text-fg-subtle flex flex-wrap items-center gap-x-3 gap-y-1">
              <span title={formatTime(v.published_at)}>
                published {formatRelative(v.published_at)}
              </span>
              <span>
                by <Hash value={v.publisher} kind="address" copy={false} />
              </span>
              {v.publish_tx && (
                <span>
                  tx <Hash value={v.publish_tx} kind="tx" copy={false} />
                </span>
              )}
              {v.previous_id && (
                <span>
                  upgrades <Hash value={v.previous_id} kind="package" copy={false} />
                </span>
              )}
            </div>
          </li>
        );
      })}
    </ol>
  );
}

function SourceTab({
  packageId,
  modules,
  loading,
}: {
  packageId: string;
  modules: ModuleSourceSummary[];
  loading: boolean;
}) {
  if (loading) return <Loader />;
  if (!modules.length) {
    return (
      <Empty
        label="No decompiled source pushed yet."
        hint={
          <>
            Run the external decompiler and push results with{' '}
            <code className="mono">trace push-source --package {packageId.slice(0, 10)}…</code> or{' '}
            <code className="mono">POST /api/v1/package/{'{id}'}/source</code>.
          </>
        }
        icon={<Code2 size={18} />}
      />
    );
  }
  // Group rows by module name; each module may have multiple format rows.
  const grouped = new Map<string, ModuleSourceSummary[]>();
  for (const m of modules) {
    const arr = grouped.get(m.module_name) ?? [];
    arr.push(m);
    grouped.set(m.module_name, arr);
  }
  return (
    <div className="space-y-4">
      {Array.from(grouped.entries()).map(([name, rows]) => (
        <ModuleSourceCard key={name} packageId={packageId} moduleName={name} rows={rows} />
      ))}
    </div>
  );
}

function ModuleSourceCard({
  packageId,
  moduleName,
  rows,
}: {
  packageId: string;
  moduleName: string;
  rows: ModuleSourceSummary[];
}) {
  // Pick the highest-fidelity format by default.
  const ranked = [...rows].sort((a, b) => fmtRank(a.format) - fmtRank(b.format));
  const [format, setFormat] = useState(ranked[0]!.format);
  const summary = ranked.find((r) => r.format === format) ?? ranked[0]!;

  const body = useQuery({
    queryKey: ['package-source', packageId, moduleName, format],
    queryFn: () => api.pkgSource(packageId, moduleName, format),
  });

  return (
    <Card
      title={
        <span className="flex items-center gap-2">
          <FileCode2 size={14} className="text-accent" />
          <span className="mono">{moduleName}</span>
        </span>
      }
      action={
        <span className="flex items-center gap-2 text-[11px] text-fg-subtle">
          <span className="hidden sm:inline">
            {summary.decompiler}
            {summary.decompiler_version ? ` · ${summary.decompiler_version}` : ''}
          </span>
          <span className="hidden sm:inline">·</span>
          <span title={formatTime(summary.decompiled_at)}>
            {formatRelative(summary.decompiled_at)}
          </span>
          {rows.length > 1 && (
            <select
              value={format}
              onChange={(e) => setFormat(e.target.value)}
              className="ml-2 bg-bg-elev border border-border-subtle rounded px-2 py-0.5 text-[11px]"
            >
              {ranked.map((r) => (
                <option key={r.format} value={r.format}>
                  {r.format}
                </option>
              ))}
            </select>
          )}
        </span>
      }
      noPadding
    >
      <div className="p-3">
        {body.isLoading ? (
          <Loader />
        ) : body.error ? (
          <ErrorBlock error={body.error} onRetry={() => body.refetch()} />
        ) : body.data ? (
          <MoveSource
            source={body.data.source.source}
            format={body.data.source.format}
            filename={`${moduleName}.${formatExt(body.data.source.format)}`}
          />
        ) : null}
      </div>
    </Card>
  );
}

function fmtRank(f: string): number {
  if (f === 'move-source') return 0;
  if (f === 'pseudo') return 1;
  if (f === 'move-disasm') return 2;
  return 3;
}
function formatExt(f: string): string {
  if (f === 'move-source') return 'move';
  if (f === 'pseudo') return 'pseudo';
  return 'mvasm';
}

function extractFunctions(abi: unknown): FunctionInfo[] {
  if (!abi || typeof abi !== 'object') return [];
  // Sui's normalized module shape is { exposed_functions: { name: { visibility, is_entry, … } } }
  const node = abi as { exposed_functions?: Record<string, { visibility?: string; is_entry?: boolean }> };
  const map = node.exposed_functions;
  if (!map || typeof map !== 'object') return [];
  return Object.entries(map)
    .map(([name, info]) => ({
      name,
      kind: info?.visibility ?? 'unknown',
      entry: !!info?.is_entry,
    }))
    .sort((a, b) => a.name.localeCompare(b.name));
}
