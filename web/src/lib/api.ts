const BASE = '';

export type Json = unknown;

async function http<T = Json>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { 'content-type': 'application/json', ...(init?.headers ?? {}) },
    ...init,
  });
  if (!res.ok) {
    const body = await res.text().catch(() => '');
    throw new Error(`${res.status} ${res.statusText}${body ? ` – ${body.slice(0, 200)}` : ''}`);
  }
  if (res.status === 204) return undefined as T;
  return (await res.json()) as T;
}

export const api = {
  health: () => http<{ status: string; db: boolean }>('/health'),

  // Network + protocols
  networkOverview: () => http<NetworkOverview>('/api/v1/network/overview'),
  protocols: () => http<{ protocols: Protocol[] }>('/api/v1/protocols'),

  // Transactions
  latestTxs: (limit = 25) => http<{ transactions: Tx[] }>(`/api/v1/tx/latest?limit=${limit}`),
  tx: (digest: string) => http<{ transaction: Tx; events: TxEvent[] }>(`/api/v1/tx/${digest}`),
  txFull: (digest: string) => http<TxFull>(`/api/v1/tx/${digest}/full`),

  // Address
  address: (addr: string) => http<AddressDetail>(`/api/v1/address/${addr}`),
  addressTxs: (addr: string, limit = 50, offset = 0) =>
    http<{ transactions: Tx[] }>(
      `/api/v1/address/${addr}/transactions?limit=${limit}&offset=${offset}`,
    ),
  addressEvents: (addr: string, limit = 50) =>
    http<{ events: TxEvent[] }>(`/api/v1/address/${addr}/events?limit=${limit}`),

  // Packages
  pkg: (id: string) => http<PackageDetail>(`/api/v1/package/${id}`),
  pkgSecurity: (id: string) => http<{ report: SecurityReport }>(`/api/v1/package/${id}/security`),
  pkgEvents: (id: string, limit = 50) =>
    http<{ events: TxEvent[] }>(`/api/v1/package/${id}/events?limit=${limit}`),
  recentPackages: (limit = 25) => http<{ packages: Package[] }>(`/api/v1/package/recent?limit=${limit}`),
  pkgVersions: (id: string) =>
    http<{ package_id: string; versions: PackageVersion[] }>(`/api/v1/package/${id}/versions`),
  pkgSources: (id: string) =>
    http<{ package_id: string; modules: ModuleSourceSummary[] }>(`/api/v1/package/${id}/source`),
  pkgSource: (id: string, module: string, format?: string) =>
    http<{ source: ModuleSource }>(
      `/api/v1/package/${id}/source/${encodeURIComponent(module)}${format ? `?format=${encodeURIComponent(format)}` : ''}`,
    ),

  // Checkpoints
  checkpoint: (seq: number) => http<{ checkpoint: Checkpoint }>(`/api/v1/checkpoint/${seq}`),
  latestCheckpoint: () => http<{ checkpoint: Checkpoint | null }>('/api/v1/checkpoint/latest'),
  recentCheckpoints: (limit = 25) =>
    http<{ checkpoints: Checkpoint[] }>(`/api/v1/checkpoint/recent?limit=${limit}`),
  checkpointTxs: (seq: number, limit = 50, offset = 0) =>
    http<{ transactions: Tx[] }>(
      `/api/v1/checkpoint/${seq}/transactions?limit=${limit}&offset=${offset}`,
    ),

  // Search
  search: (q: string) => http<SearchResult>(`/api/v1/search?q=${encodeURIComponent(q)}`),

  // Labels
  labelsForAddress: (addr: string) => http<{ labels: Label[] }>(`/api/v1/labels/${addr}`),
  searchLabels: (q: string, limit = 25) =>
    http<{ labels: Label[] }>(`/api/v1/labels/search?q=${encodeURIComponent(q)}&limit=${limit}`),
  submitLabel: (token: string, body: SubmitLabelBody) =>
    http<{ label: Label }>(`/api/v1/labels`, {
      method: 'POST',
      headers: { authorization: `Bearer ${token}` },
      body: JSON.stringify(body),
    }),

  // Stats
  deploymentStats: (days = 30) =>
    http<{ stats: DailyStat[]; from: string; to: string }>(`/api/v1/stats/deployments?days=${days}`),
  activeProjects: (hours = 24, limit = 50) =>
    http<{ rankings: ProjectRanking[]; since: string }>(
      `/api/v1/stats/active?hours=${hours}&limit=${limit}`,
    ),
  tvlHistory: (protocol: string, hours = 24) =>
    http<{ history: TvlPoint[] }>(`/api/v1/stats/tvl/${protocol}?hours=${hours}`),
  throughput: (minutes = 60) =>
    http<{ minutes: number; points: ThroughputPoint[] }>(`/api/v1/stats/throughput?minutes=${minutes}`),

  // Security
  recentSecurityFindings: (limit = 50) =>
    http<{ findings: RecentFinding[] }>(`/api/v1/security/recent?limit=${limit}`),
  securityScoreboard: (days = 30) =>
    http<{ days: number; severity_counts: SeverityCount[]; rule_rankings: RuleRanking[] }>(
      `/api/v1/security/scoreboard?days=${days}`,
    ),

  // Auth + watchlists + alerts
  siwsLogin: (body: { address: string; message: string; signature: string }) =>
    http<{ token: string; user: { id: string; address: string; role: string } }>(
      `/api/v1/auth/siws`,
      { method: 'POST', body: JSON.stringify(body) },
    ),
  watchlists: (token: string) =>
    http<{ watchlists: Watchlist[] }>(`/api/v1/watchlists`, {
      headers: { authorization: `Bearer ${token}` },
    }),
  createWatchlist: (token: string, body: WatchlistBody) =>
    http<{ watchlist: Watchlist }>(`/api/v1/watchlists`, {
      method: 'POST',
      headers: { authorization: `Bearer ${token}` },
      body: JSON.stringify(body),
    }),
  deleteWatchlist: (token: string, id: string) =>
    http<void>(`/api/v1/watchlists/${id}`, {
      method: 'DELETE',
      headers: { authorization: `Bearer ${token}` },
    }),
  recentAlerts: (token: string, limit = 50) =>
    http<{ alerts: AlertRow[] }>(`/api/v1/alerts/recent?limit=${limit}`, {
      headers: { authorization: `Bearer ${token}` },
    }),
  alertsFeed: (limit = 50) => http<{ alerts: AlertRow[] }>(`/api/v1/alerts/feed?limit=${limit}`),

  // Curated protocol watchlist (operator-facing)
  watchDashboard: () => http<WatchDashboard>('/api/v1/watch/dashboard'),
  watchProtocols: (watchedOnly = false) =>
    http<{ protocols: Protocol[] }>(
      `/api/v1/watch/protocols${watchedOnly ? '?watched=true' : ''}`,
    ),
  watchProtocol: (id: string) =>
    http<WatchProtocolDetail>(`/api/v1/watch/protocols/${encodeURIComponent(id)}`),
  watchCodeFeed: (limit = 100) =>
    http<{ events: CodeEvent[] }>(`/api/v1/watch/feed/code?limit=${limit}`),
  watchActivity: (id: string, limit = 100) =>
    http<{ protocol_id: string; activity: ProtocolActivity[] }>(
      `/api/v1/watch/feed/activity?id=${encodeURIComponent(id)}&limit=${limit}`,
    ),
  watchUpsert: (body: ProtocolUpsertBody, ingestKey: string) =>
    http<{ protocol: Protocol }>('/api/v1/watch/protocols', {
      method: 'POST',
      headers: { 'x-trace-ingest-key': ingestKey },
      body: JSON.stringify(body),
    }),
  watchUpdate: (id: string, body: ProtocolUpdateBody, ingestKey: string) =>
    http<{ protocol: Protocol }>(`/api/v1/watch/protocols/${encodeURIComponent(id)}`, {
      method: 'PUT',
      headers: { 'x-trace-ingest-key': ingestKey },
      body: JSON.stringify(body),
    }),
  watchRemove: (id: string, ingestKey: string) =>
    http<void>(`/api/v1/watch/protocols/${encodeURIComponent(id)}`, {
      method: 'DELETE',
      headers: { 'x-trace-ingest-key': ingestKey },
    }),
};

export interface Tx {
  digest: string;
  checkpoint_seq: number;
  timestamp: string;
  sender: string;
  status: 'success' | 'failure';
  gas_used: number;
  gas_price: number;
  kind: string;
}

export interface TxEvent {
  tx_digest: string;
  event_seq: number;
  package_id: string;
  module: string;
  event_type: string;
  sender: string;
  timestamp: string;
  payload: unknown;
}

export interface AddressDetail {
  address: string;
  labels: Label[];
  recent_transactions: Tx[];
}

export interface Package {
  id: string;
  original_id: string;
  version: number;
  publisher: string;
  modules_count: number;
  source_verified: boolean;
  published_at: string;
}

export interface PackageDetail {
  package: Package;
  modules: { module_name: string; bytecode_hash: string; abi_json: unknown }[];
  security: SecurityReport | null;
}

export interface PackageVersion {
  package_id: string;
  original_id: string;
  version: number;
  previous_id: string | null;
  publisher: string;
  publish_tx: string | null;
  published_at: string;
  notes: string | null;
}

export interface ModuleSourceSummary {
  package_id: string;
  module_name: string;
  format: string;
  decompiler: string;
  decompiler_version: string | null;
  bytecode_hash: string | null;
  source_hash: string;
  bytes: number;
  decompiled_at: string;
}

export interface ModuleSource {
  package_id: string;
  module_name: string;
  format: string;
  source: string;
  decompiler: string;
  decompiler_version: string | null;
  bytecode_hash: string | null;
  source_hash: string;
  decompiled_at: string;
}

export type Severity = 'info' | 'low' | 'medium' | 'high' | 'critical';

export interface SecurityReport {
  package_id: string;
  version: number;
  score: number;
  max_severity: Severity;
  findings: Finding[];
  scanned_at: string;
}

export interface Finding {
  rule_id: string;
  rule_name: string;
  severity: Severity;
  confidence: number;
  module: string;
  function: string | null;
  location: string;
  message: string;
  suggestion: string;
}

export interface RecentFinding {
  package_id: string;
  version: number;
  finding: Finding;
  scanned_at: string;
}

export interface SeverityCount {
  severity: Severity;
  count: number;
}

export interface RuleRanking {
  rule_id: string;
  rule_name: string;
  severity: Severity;
  hits: number;
}

export interface Checkpoint {
  sequence_number: number;
  digest: string;
  timestamp_ms: number;
  previous_digest: string | null;
  network_total_transactions: number;
  epoch: number;
}

export interface SearchResult {
  query: string;
  kind: string;
  labels: Label[];
}

export interface Label {
  address: string;
  label: string;
  category: string;
  source: string;
  confidence: number;
  evidence_url: string | null;
  verified: boolean;
}

export interface SubmitLabelBody {
  address: string;
  label: string;
  category: string;
  evidence_url?: string;
}

export interface DailyStat {
  day: string;
  package_count: number;
  unique_publishers: number;
}

export interface ProjectRanking {
  package_id: string;
  calls: number;
  unique_callers: number;
  gas_total: number;
}

export interface TvlPoint {
  protocol_id: string;
  timestamp: string;
  tvl_usd: number;
  breakdown: unknown;
}

export interface ThroughputPoint {
  bucket: string;
  tx_count: number;
}

/* ----------------------- SuiVision-style tx detail ----------------------- */

export interface TxFull {
  digest: string;
  /** May be null if the tx isn't indexed yet (RPC-only). */
  indexed: Tx | null;
  events: TxEvent[];
  rpc: SuiRpcTransaction | null;
  /** address -> labels[] */
  labels: Record<string, AddressLabel[]>;
  /** package_id -> security summary */
  packages: Record<
    string,
    { score: number; max_severity: string; findings_count: number; scanned_at: string }
  >;
}

export interface AddressLabel {
  address: string;
  label: string;
  category: string;
  source: string;
  confidence: number;
  evidence_url: string | null;
  verified: boolean;
}

/** Subset of `sui_getTransactionBlock` response we actually consume. */
export interface SuiRpcTransaction {
  digest: string;
  timestampMs?: string;
  checkpoint?: string;
  transaction?: {
    data?: SuiTxData;
    txSignatures?: string[];
  };
  effects?: SuiEffects;
  events?: SuiRpcEvent[];
  balanceChanges?: SuiBalanceChange[];
  objectChanges?: SuiObjectChange[];
}

export interface SuiTxData {
  messageVersion?: string;
  sender: string;
  gasData?: SuiGasData;
  transaction?: SuiTxKind;
}

export interface SuiGasData {
  payment?: { objectId: string; version: number; digest: string }[];
  owner?: string;
  price?: string;
  budget?: string;
}

export type SuiTxKind = ProgrammableTxKind | OtherTxKind;
export interface ProgrammableTxKind {
  kind: 'ProgrammableTransaction';
  inputs: SuiCallArg[];
  transactions: SuiCommand[];
}
export interface OtherTxKind {
  kind: string;
  [k: string]: unknown;
}

/** PTB inputs (from showInput) */
export type SuiCallArg =
  | { type: 'pure'; valueType: string; value: unknown }
  | { type: 'object'; objectType: string; objectId: string; version?: string; digest?: string };

/** PTB commands (from showInput). The keys are tagged unions in JSON. */
export type SuiCommand =
  | { MoveCall: SuiMoveCall }
  | { TransferObjects: [SuiArg[], SuiArg] }
  | { SplitCoins: [SuiArg, SuiArg[]] }
  | { MergeCoins: [SuiArg, SuiArg[]] }
  | { Publish: [string[], string[]] }
  | { Upgrade: [string[], string[], string, SuiArg] }
  | { MakeMoveVec: [string | null, SuiArg[]] }
  | Record<string, unknown>;

export interface SuiMoveCall {
  package: string;
  module: string;
  function: string;
  type_arguments?: string[];
  arguments?: SuiArg[];
}

export type SuiArg =
  | 'GasCoin'
  | { Input: number }
  | { Result: number }
  | { NestedResult: [number, number] };

export interface SuiEffects {
  status?: { status: 'success' | 'failure'; error?: string };
  executedEpoch?: string;
  gasUsed?: SuiGasUsed;
  transactionDigest?: string;
  dependencies?: string[];
  gasObject?: { reference: { objectId: string; version: number; digest: string }; owner: SuiOwner };
  created?: SuiOwnedRef[];
  mutated?: SuiOwnedRef[];
  deleted?: { objectId: string; version: number; digest: string }[];
  unwrapped?: SuiOwnedRef[];
  wrapped?: { objectId: string; version: number; digest: string }[];
  sharedObjects?: { objectId: string; version: number; digest: string }[];
  modifiedAtVersions?: { objectId: string; sequenceNumber: string }[];
}

export interface SuiOwnedRef {
  owner: SuiOwner;
  reference: { objectId: string; version: number; digest: string };
}

export type SuiOwner =
  | string
  | { AddressOwner: string }
  | { ObjectOwner: string }
  | { Shared: { initial_shared_version: number } }
  | 'Immutable';

export interface SuiGasUsed {
  computationCost: string;
  storageCost: string;
  storageRebate: string;
  nonRefundableStorageFee: string;
}

export interface SuiRpcEvent {
  id?: { txDigest: string; eventSeq: string };
  packageId: string;
  transactionModule: string;
  sender: string;
  type: string;
  parsedJson?: unknown;
  bcs?: string;
}

export interface SuiBalanceChange {
  owner: SuiOwner;
  coinType: string;
  amount: string;
}

export type SuiObjectChange =
  | {
      type: 'created' | 'mutated' | 'transferred';
      sender: string;
      owner: SuiOwner;
      objectType: string;
      objectId: string;
      version: string;
      digest: string;
      previousVersion?: string;
    }
  | {
      type: 'deleted' | 'wrapped';
      sender: string;
      objectType: string;
      objectId: string;
      version: string;
    }
  | {
      type: 'published';
      packageId: string;
      version: string;
      digest: string;
      modules: string[];
    };

export interface NetworkOverview {
  checkpoint: Checkpoint | null;
  tx_24h: number;
  tx_total: number;
  packages_24h: number;
  packages_total: number;
}

export interface Protocol {
  id: string;
  name: string;
  package_ids: string[];
  category: string;
  website: string | null;
  defillama_slug: string | null;
  watched: boolean;
  priority: number;
  risk_level: string;
  description: string | null;
  logo_url: string | null;
  tags: string[];
  treasury_addresses: string[];
  multisig_addresses: string[];
  contact: string | null;
  notes: string | null;
  added_by: string | null;
  added_at: string;
  updated_at: string;
}

export interface ProtocolUpsertBody {
  id: string;
  name: string;
  package_ids?: string[];
  category?: string;
  website?: string | null;
  defillama_slug?: string | null;
  watched?: boolean;
  priority?: number;
  risk_level?: string;
  description?: string | null;
  logo_url?: string | null;
  tags?: string[];
  treasury_addresses?: string[];
  multisig_addresses?: string[];
  contact?: string | null;
  notes?: string | null;
}

export interface ProtocolUpdateBody extends Partial<Omit<ProtocolUpsertBody, 'id'>> {}

export interface CodeEvent {
  id: number;
  protocol_id: string;
  package_id: string;
  original_id: string;
  version: number;
  previous_id: string | null;
  publish_tx: string | null;
  publisher: string;
  kind: 'publish' | 'upgrade';
  summary: {
    modules_total?: number;
    modules_added?: string[];
    modules_removed?: string[];
    modules_changed?: { module: string; prev_hash: string; new_hash: string }[];
  };
  severity: 'info' | 'warning' | 'critical';
  detected_at: string;
  happened_at: string;
}

export interface ProtocolActivity {
  tx_digest: string;
  event_seq: number;
  package_id: string;
  module: string;
  event_type: string;
  sender: string;
  timestamp: string;
}

export interface ProtocolCard {
  protocol: Protocol;
  tvl_latest: TvlPoint | null;
  activity_24h: number;
  code_events_24h: number;
  last_code_event: CodeEvent | null;
}

export interface WatchDashboard {
  totals: {
    watched: number;
    tvl_usd: number;
    activity_24h: number;
    code_events_24h: number;
  };
  cards: ProtocolCard[];
}

export interface WatchProtocolDetail {
  protocol: Protocol;
  tvl_latest: TvlPoint | null;
  activity_24h: number;
  code_events: CodeEvent[];
  activity: ProtocolActivity[];
}

export interface WatchlistBody {
  name: string;
  target_type: 'address' | 'package' | 'protocol';
  target_id: string;
  rules?: Record<string, unknown>;
  channels?: unknown[];
}

export interface Watchlist extends WatchlistBody {
  id: string;
  created_at: string;
}

export interface AlertRow {
  id: string;
  rule_id: string;
  fired_at: string;
  payload: { title?: string; body?: string; [k: string]: unknown };
  delivered: boolean;
}
