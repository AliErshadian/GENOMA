"use client";

import { FormEvent, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { ApiError, login } from "@/lib/api";

export default function LoginPage() {
  const router = useRouter();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      await login(email, password);
      router.push("/analyze");
    } catch (err) {
      setError(err instanceof ApiError ? err.message : "Login failed");
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="flex min-h-screen items-center justify-center bg-void px-6 text-core">
      <div className="w-full max-w-md">
        <Link href="/" className="inline-block">
          <p className="text-[11px] tracking-[0.42em] text-cyan/80">GENOMA</p>
        </Link>
        <h1 className="mt-6 font-mono text-xl tracking-[0.12em]">Sign in</h1>
        <p className="mt-2 font-mono text-xs text-core/50">
          Local accounts with Bearer API tokens. Auth is optional unless the API sets{" "}
          <span className="text-core/70">GENOMA_AUTH_REQUIRED=true</span>.
        </p>
        <form onSubmit={onSubmit} className="mt-8 space-y-4">
          <label className="block">
            <span className="font-mono text-[10px] uppercase tracking-[0.18em] text-core/40">
              Email
            </span>
            <input
              type="email"
              required
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              className="mt-2 w-full rounded-xl border border-white/10 bg-[#080a10] px-3 py-2 font-mono text-sm text-core outline-none focus:border-cyan/40"
            />
          </label>
          <label className="block">
            <span className="font-mono text-[10px] uppercase tracking-[0.18em] text-core/40">
              Password
            </span>
            <input
              type="password"
              required
              minLength={8}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="mt-2 w-full rounded-xl border border-white/10 bg-[#080a10] px-3 py-2 font-mono text-sm text-core outline-none focus:border-cyan/40"
            />
          </label>
          {error ? <p className="font-mono text-xs text-anomaly">{error}</p> : null}
          <button
            type="submit"
            disabled={busy}
            className="rounded-full border border-cyan/30 px-5 py-2 font-mono text-[11px] tracking-[0.14em] text-cyan hover:bg-cyan/10 disabled:opacity-35"
          >
            {busy ? "Signing in…" : "Sign in"}
          </button>
        </form>
        <p className="mt-6 font-mono text-xs text-core/45">
          No account?{" "}
          <Link href="/register" className="text-cyan hover:underline">
            Register
          </Link>
        </p>
      </div>
    </main>
  );
}
