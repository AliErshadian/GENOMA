"use client";

import { useEffect, useMemo, useState } from "react";
import type { AnalysisSummary, CompareResponse } from "@genoma/shared-types";
import { WorkspaceChrome } from "@/components/layout/WorkspaceChrome";
import { compareAnalyses, listAnalyses } from "@/lib/api";

function ScoreBar({ label, value }: { label: string; value: number }) {
  const pct = Math.round(value * 1000) / 10;
  return (
    <div className="space-y-2">
      <div className="flex items-baseline justify-between gap-4">
        <span className="font-mono text-[10px] uppercase tracking-[0.16em] text-core/45">
          {label}
        </span>
        <span className="font-mono text-sm text-core/90">{pct.toFixed(1)}%</span>
      </div>
      <div className="h-1.5 overflow-hidden rounded-full bg-white/5">
        <div
          className="h-full rounded-full bg-gradient-to-r from-cyan/80 to-purple/70"
          style={{ width: `${Math.max(2, Math.min(100, pct))}%` }}
        />
      </div>
    </div>
  );
}

function AnalysisSelect({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: AnalysisSummary[];
  onChange: (id: string) => void;
}) {
  return (
    <label className="block space-y-2">
      <span className="font-mono text-[10px] uppercase tracking-[0.18em] text-core/40">{label}</span>
      <select
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className="w-full rounded-xl border border-white/10 bg-[#080a10] px-3 py-3 font-mono text-xs text-core outline-none focus:border-cyan/40"
      >
        <option value="">Select analysis…</option>
        {options.map((item) => (
          <option key={item.id} value={item.id}>
            {item.original_name} · {item.id.slice(0, 8)}
          </option>
        ))}
      </select>
    </label>
  );
}

export default function ComparePage() {
  const [analyses, setAnalyses] = useState<AnalysisSummary[]>([]);
  const [leftId, setLeftId] = useState("");
  const [rightId, setRightId] = useState("");
  const [result, setResult] = useState<CompareResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void listAnalyses()
      .then((items) => setAnalyses(items.filter((item) => item.status === "COMPLETE")))
      .catch(() => setAnalyses([]));
  }, []);

  const ready = leftId.length > 0 && rightId.length > 0;

  const overall = useMemo(() => result?.similarity.overall ?? null, [result]);

  const runCompare = async () => {
    if (!ready) return;
    setBusy(true);
    setError(null);
    try {
      const next = await compareAnalyses(leftId, rightId);
      setResult(next);
    } catch (err) {
      setResult(null);
      setError(err instanceof Error ? err.message : "Compare failed");
    } finally {
      setBusy(false);
    }
  };

  return (
    <WorkspaceChrome>
      <div className="flex h-full items-center justify-center px-6">
        <div className="panel w-full max-w-2xl rounded-3xl p-8 md:p-10">
          <p className="font-mono text-[11px] tracking-[0.32em] text-cyan">COMPARE</p>
          <h1 className="mt-3 text-2xl tracking-[0.1em]">Structural similarity</h1>
          <p className="mt-3 max-w-lg font-mono text-xs leading-relaxed text-core/50">
            Score two completed Digital DNA fingerprints with the weighted entropy, distribution,
            pattern, complexity, and π-vector blend.
          </p>

          <div className="mt-8 grid gap-4 md:grid-cols-2">
            <AnalysisSelect
              label="Left"
              value={leftId}
              options={analyses}
              onChange={setLeftId}
            />
            <AnalysisSelect
              label="Right"
              value={rightId}
              options={analyses}
              onChange={setRightId}
            />
          </div>

          <button
            type="button"
            disabled={!ready || busy}
            onClick={() => void runCompare()}
            className="mt-6 w-full rounded-full bg-core px-6 py-3 text-sm tracking-[0.16em] text-void disabled:cursor-not-allowed disabled:opacity-35"
          >
            {busy ? "Comparing…" : "Compare"}
          </button>

          {error ? <p className="mt-4 font-mono text-xs text-anomaly">{error}</p> : null}

          {analyses.length === 0 ? (
            <p className="mt-6 font-mono text-xs text-core/40">
              No completed analyses yet. Run an upload or demo from Analysis first.
            </p>
          ) : null}

          {result && overall != null ? (
            <div className="mt-8 space-y-5 border-t border-white/5 pt-8">
              <div className="flex flex-wrap items-end justify-between gap-4">
                <div>
                  <p className="font-mono text-[10px] uppercase tracking-[0.18em] text-core/40">
                    Pair
                  </p>
                  <p className="mt-1 font-mono text-xs text-core/80">
                    {result.left_name} ↔ {result.right_name}
                  </p>
                </div>
                <div className="text-right">
                  <p className="font-mono text-[10px] uppercase tracking-[0.18em] text-core/40">
                    Overall
                  </p>
                  <p className="mt-1 font-mono text-3xl text-cyan">
                    {(overall * 100).toFixed(1)}%
                  </p>
                </div>
              </div>
              <ScoreBar label="Entropy" value={result.similarity.entropy} />
              <ScoreBar label="Distribution" value={result.similarity.distribution} />
              <ScoreBar label="Pattern" value={result.similarity.pattern} />
              <ScoreBar label="Complexity" value={result.similarity.complexity} />
            </div>
          ) : null}
        </div>
      </div>
    </WorkspaceChrome>
  );
}
