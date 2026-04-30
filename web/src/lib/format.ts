export function shortAddr(a: string | undefined | null, n = 6) {
  if (!a) return '';
  if (a.length <= n * 2 + 4) return a;
  return `${a.slice(0, n + 2)}…${a.slice(-n)}`;
}

export function formatTime(input: string | number | Date) {
  const date = input instanceof Date ? input : new Date(input);
  return date.toLocaleString();
}

export function formatRelative(input: string | number | Date) {
  const ms = (input instanceof Date ? input : new Date(input)).getTime() - Date.now();
  const secs = Math.round(ms / 1000);
  const abs = Math.abs(secs);
  const sign = secs < 0 ? -1 : 1;
  let value: number;
  let unit: Intl.RelativeTimeFormatUnit;
  if (abs < 60) {
    value = secs;
    unit = 'second';
  } else if (abs < 3600) {
    value = Math.round(secs / 60);
    unit = 'minute';
  } else if (abs < 86_400) {
    value = Math.round(secs / 3600);
    unit = 'hour';
  } else {
    value = Math.round(secs / 86_400);
    unit = 'day';
  }
  // Force the unit display to mimic "x ago"
  return new Intl.RelativeTimeFormat('en-US', { numeric: 'auto' }).format(value || sign, unit);
}

export function formatNumber(n: number | bigint, opts?: Intl.NumberFormatOptions) {
  return Intl.NumberFormat('en-US', opts).format(n as number);
}

export function formatGas(gas: number) {
  if (!gas && gas !== 0) return '—';
  if (gas < 1_000) return `${gas}`;
  if (gas < 1_000_000) return `${(gas / 1_000).toFixed(2)} K`;
  if (gas < 1_000_000_000) return `${(gas / 1_000_000).toFixed(2)} M`;
  return `${(gas / 1_000_000_000).toFixed(2)} B`;
}

/** SUI is 9 decimals */
export function formatSui(mist: number) {
  if (!mist && mist !== 0) return '—';
  const sui = mist / 1e9;
  if (sui === 0) return '0 SUI';
  if (Math.abs(sui) < 0.0001) return `${(sui * 1e6).toFixed(2)} μSUI`;
  if (Math.abs(sui) < 1) return `${sui.toFixed(6)} SUI`;
  if (Math.abs(sui) < 1000) return `${sui.toFixed(4)} SUI`;
  return `${sui.toLocaleString('en-US', { maximumFractionDigits: 2 })} SUI`;
}

export function formatUsd(v: number) {
  if (!Number.isFinite(v)) return '—';
  return `$${formatNumber(v, { notation: 'compact', maximumFractionDigits: 2 })}`;
}

export function severityColor(sev: string) {
  switch (sev) {
    case 'critical':
      return 'text-danger bg-danger/15 border-danger/40';
    case 'high':
      return 'text-warn bg-warn/15 border-warn/40';
    case 'medium':
      return 'text-warn bg-warn/10 border-warn/30';
    case 'low':
      return 'text-fg-muted bg-bg-elev border-border';
    case 'info':
      return 'text-info bg-info/10 border-info/30';
    default:
      return 'text-fg-subtle bg-bg-elev border-border-subtle';
  }
}

export function categoryColor(cat: string) {
  if (['hacker', 'scam', 'phishing', 'mixer', 'sanctioned', 'rug_pull'].includes(cat)) {
    return 'text-danger bg-danger/10 border-danger/40';
  }
  if (['exchange', 'cex_hotwallet', 'cex_coldwallet', 'bridge', 'validator'].includes(cat)) {
    return 'text-accent bg-accent/10 border-accent/40';
  }
  if (['protocol_treasury', 'team_multisig', 'vesting'].includes(cat)) {
    return 'text-info bg-info/10 border-info/30';
  }
  return 'text-fg-muted bg-bg-elev border-border';
}

/** Heuristic classifier matching the API search route. */
export function classifyTerm(q: string): 'tx' | 'address' | 'unknown' {
  const t = q.trim();
  if (!t.startsWith('0x')) return 'unknown';
  if (t.length === 66) return 'address';
  if (t.length === 64 || t.length === 44 || t.length === 46) return 'tx';
  return 'unknown';
}
