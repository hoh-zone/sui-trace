import { useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { KeyRound, Wallet } from 'lucide-react';
import { Card } from '@/components/Card';
import { useToast } from '@/components/Toast';
import { api } from '@/lib/api';
import { login } from '@/lib/auth';

export function LoginPage() {
  const [address, setAddress] = useState('');
  const [signature, setSignature] = useState('siws-stub');
  const [message, setMessage] = useState(`Sign in to sui-trace · ${new Date().toISOString()}`);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);
  const navigate = useNavigate();
  const toast = useToast();

  return (
    <div className="max-w-md mx-auto">
      <Card title={<span className="flex items-center gap-2"><KeyRound size={14} className="text-accent" /> Sign in with Sui</span>}>
        <p className="text-xs text-fg-subtle mb-4">
          Connect your Sui wallet, sign the personal-message challenge below and the API mints a JWT
          bound to your address. The first iteration accepts any well-formed Sui address; the full
          signature verification is wired up server-side once the SDK signs the canonical SIWS payload.
        </p>
        <form
          className="space-y-3 text-sm"
          onSubmit={async (e) => {
            e.preventDefault();
            setError(null);
            setPending(true);
            try {
              const r = await api.siwsLogin({ address, message, signature });
              login(r.token, r.user);
              toast.push('Signed in', 'success');
              navigate({ to: '/watchlist' });
            } catch (err) {
              setError((err as Error).message);
              toast.push('Sign-in failed', 'danger');
            } finally {
              setPending(false);
            }
          }}
        >
          <div>
            <div className="text-[10px] uppercase tracking-wider text-fg-subtle mb-1">Sui address</div>
            <input
              required
              placeholder="0x…"
              value={address}
              onChange={(e) => setAddress(e.target.value)}
              className="w-full bg-bg-elev border border-border-subtle rounded px-3 py-2 mono text-xs"
            />
          </div>
          <div>
            <div className="text-[10px] uppercase tracking-wider text-fg-subtle mb-1">Message to sign</div>
            <textarea
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              className="w-full bg-bg-elev border border-border-subtle rounded px-3 py-2 text-xs h-20"
            />
          </div>
          <div>
            <div className="text-[10px] uppercase tracking-wider text-fg-subtle mb-1">Signature</div>
            <input
              value={signature}
              onChange={(e) => setSignature(e.target.value)}
              className="w-full bg-bg-elev border border-border-subtle rounded px-3 py-2 mono text-xs"
            />
          </div>
          {error && <div className="text-xs text-danger">{error}</div>}
          <button
            disabled={pending || !address}
            className="w-full px-3 py-2 rounded bg-accent text-accent-fg flex items-center justify-center gap-2 disabled:opacity-50"
          >
            <Wallet size={14} /> {pending ? 'Signing in…' : 'Sign in'}
          </button>
        </form>
        <p className="text-[11px] text-fg-subtle mt-4">
          Tip: when running locally without a wallet, paste a fake but well-formed address (e.g.{' '}
          <code className="mono">0x{'1'.repeat(64)}</code>) and any signature.
        </p>
      </Card>
    </div>
  );
}
