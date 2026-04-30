import { useId, useMemo } from 'react';
import { CopyButton } from './CopyButton';
import { cn } from '@/lib/cn';

interface MoveSourceProps {
  source: string;
  format?: string;
  filename?: string;
  className?: string;
  maxHeight?: number;
}

const KEYWORDS = new Set([
  'module',
  'public',
  'entry',
  'fun',
  'struct',
  'has',
  'use',
  'as',
  'native',
  'let',
  'mut',
  'return',
  'if',
  'else',
  'while',
  'loop',
  'break',
  'continue',
  'abort',
  'assert',
  'move',
  'copy',
  'friend',
  'spec',
  'acquires',
  'const',
  'script',
  'address',
]);

const TYPES = new Set([
  'u8',
  'u16',
  'u32',
  'u64',
  'u128',
  'u256',
  'bool',
  'address',
  'vector',
  'signer',
  'String',
]);

const ABILITIES = new Set(['key', 'store', 'copy', 'drop']);

export function MoveSource({
  source,
  format,
  filename,
  className,
  maxHeight = 560,
}: MoveSourceProps) {
  const id = useId();
  const lines = useMemo(() => source.split('\n'), [source]);
  const isDisasm = (format ?? '').includes('disasm');

  return (
    <div className={cn('rounded-lg border border-border-subtle bg-bg/70 overflow-hidden', className)}>
      <div className="flex items-center justify-between px-3 py-1.5 border-b border-border-subtle text-xs text-fg-subtle">
        <span className="flex items-center gap-2">
          {filename && <span className="mono text-fg-muted">{filename}</span>}
          {format && (
            <span className="px-1.5 py-0.5 rounded text-[10px] uppercase tracking-wider border border-border-subtle bg-bg-elev">
              {format}
            </span>
          )}
        </span>
        <span className="flex items-center gap-2">
          <span className="text-[10px]">{lines.length} lines</span>
          <CopyButton value={source} silent />
        </span>
      </div>
      <div className="overflow-auto" style={{ maxHeight }}>
        <pre
          id={id}
          className="mono text-[12px] leading-[1.55] whitespace-pre"
          style={{ counterReset: 'src 0' }}
        >
          {lines.map((ln, i) => (
            <div key={i} className="flex items-start hover:bg-bg-elev/40">
              <span className="select-none text-fg-subtle text-right w-12 pr-3 pl-3 border-r border-border-subtle/60 shrink-0">
                {i + 1}
              </span>
              <code className="flex-1 px-3">
                {isDisasm ? colouriseDisasm(ln) : colouriseMove(ln)}
                {/* keep trailing newline for selection */}
                {ln === '' ? '\u00A0' : ''}
              </code>
            </div>
          ))}
        </pre>
      </div>
    </div>
  );
}

/* ----------------------- highlighters ----------------------- */

const TOKEN_RE =
  /(\/\/[^\n]*)|(\/\*[\s\S]*?\*\/)|("(?:\\.|[^"\\])*")|(0x[0-9a-fA-F]+|\b\d+(?:u(?:8|16|32|64|128|256))?\b)|(\b[A-Za-z_][A-Za-z0-9_]*\b)|([{}()\[\];,:&<>=+\-*\/!.?])/g;

function colouriseMove(line: string): React.ReactNode {
  const out: React.ReactNode[] = [];
  let last = 0;
  let i = 0;
  for (const m of line.matchAll(TOKEN_RE)) {
    if (m.index! > last) out.push(line.slice(last, m.index!));
    if (m[1]) out.push(<span key={i++} className="text-fg-subtle italic">{m[1]}</span>);
    else if (m[2]) out.push(<span key={i++} className="text-fg-subtle italic">{m[2]}</span>);
    else if (m[3]) out.push(<span key={i++} className="text-ok">{m[3]}</span>);
    else if (m[4]) out.push(<span key={i++} className="text-warn">{m[4]}</span>);
    else if (m[5]) {
      const tok = m[5];
      if (KEYWORDS.has(tok)) out.push(<span key={i++} className="text-accent font-medium">{tok}</span>);
      else if (TYPES.has(tok)) out.push(<span key={i++} className="text-info">{tok}</span>);
      else if (ABILITIES.has(tok)) out.push(<span key={i++} className="text-info italic">{tok}</span>);
      else if (/^[A-Z]/.test(tok)) out.push(<span key={i++} className="text-info">{tok}</span>);
      else out.push(<span key={i++} className="text-fg">{tok}</span>);
    } else if (m[6]) out.push(<span key={i++} className="text-fg-subtle">{m[6]}</span>);
    last = m.index! + m[0].length;
  }
  if (last < line.length) out.push(line.slice(last));
  return out.length ? out : line;
}

/** Cheap highlighter for Move bytecode disassembly: opcode | operand | comment */
function colouriseDisasm(line: string): React.ReactNode {
  const trimmed = line.trimStart();
  const lead = line.slice(0, line.length - trimmed.length);
  if (!trimmed) return line;
  if (trimmed.startsWith(';') || trimmed.startsWith('//')) {
    return <span className="text-fg-subtle italic">{line}</span>;
  }
  // Common header lines
  if (/^(module|fun|struct|public|entry|use|address|version|resolved)/i.test(trimmed)) {
    return colouriseMove(line);
  }
  // <leading ws><LABEL?><opcode> [operands…] [; comment]
  const labelMatch = trimmed.match(/^([A-Za-z_][A-Za-z0-9_]*:)\s*(.*)$/);
  let body = trimmed;
  let label: React.ReactNode = null;
  if (labelMatch) {
    label = <span className="text-warn">{labelMatch[1]} </span>;
    body = labelMatch[2];
  }
  const commentIdx = body.indexOf(';');
  let head = body;
  let comment: React.ReactNode = null;
  if (commentIdx >= 0) {
    head = body.slice(0, commentIdx);
    comment = <span className="text-fg-subtle italic">{body.slice(commentIdx)}</span>;
  }
  const parts = head.split(/(\s+)/);
  return (
    <>
      {lead}
      {label}
      {parts.map((part, i) =>
        i === 0 ? (
          <span key={i} className="text-accent font-medium">
            {part}
          </span>
        ) : /^\s+$/.test(part) ? (
          part
        ) : (
          <span key={i} className="text-info">
            {part}
          </span>
        ),
      )}
      {comment}
    </>
  );
}
