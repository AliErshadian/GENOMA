"use client";

import { useEffect, useMemo, useState } from "react";
import type { AnalysisSummary, ExperimentResult } from "@genoma/shared-types";
import { WorkspaceChrome } from "@/components/layout/WorkspaceChrome";
import { listAnalyses, runIsolationExperiment, runKnnDensityExperiment } from "@/lib/api";

export default function ExperimentsPage() {
  const [analyses, setAnalyses] = useState<AnalysisSummary[]>([]);
  const [selected, setSelected] = useState<string[]>([]);
  const [isolationId, setIsolationId] = useState("");
  const [result, setResult] = useState<ExperimentResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const completed = useMemo(
    () => analyses.filter((item) => item.status === "COMPLETE"),
    [analyses],
  );

  useEffect(() => {
    void listAnalyses()
      .then(setAnalyses)
      .catch(() => setAnalyses([]));
  }, []);

  const toggle = (id: string) => {
    setSelected((prev) =>
      prev.includes(id) ? prev.filter((value) => value !== id) : [...prev, id],
    );
  };

  const runIsolation = async () => {
    if (!isolationId) return;
    setBusy(true);
    setError(null);
    try {
      setResult(await runIsolationExperiment(isolationId));
    } catch (err) {
      setError(err instanceof Error ? err.message : "Isolation experiment failed");
    } finally {
      setBusy(false);
    }
  };

  const runKnn = async () => {
    if (selected.length < 2) {
      setError("Select at least two completed analyses");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      setResult(await runKnnDensityExperiment(selected, 3));
    } catch (err) {
      setError(err instanceof Error ? err.message : "k-NN experiment failed");
    } finally {
      setBusy(false);
    }
  };

  return (
    <WorkspaceChrome>
      <div className="flex h-full justify-center overflow-y-auto px-6 py-24 md:pl-56">
        <div className="panel w-full max-w-3xl rounded-3xl p-8 md:p-10">
          <p className="font-mono text-[11px] tracking-[0.32em] text-cyan">EXPERIMENTS</p>
          <h1 className="mt-3 text-2xl tracking-[0.1em]">Classical ML lab</h1>
          <p className="mt-3 max-w-xl font-mono text-xs leading-relaxed text-core/50">
            Heuristic, deterministic experiments on Digital DNA — isolation path length and k-NN
            density. Not neural nets; scores are exploratory, not proven metrics.
          </p>

          <div className="mt-8 space-y-8">
            <section>
              <p className="font-mono text-[10px] uppercase tracking-[0.18em] text-core/40">
                Isolation score
              </p>
              <div className="mt-3 flex flex-wrap gap-3">
                <select
                  value={isolationId}
                  onChange={(event) => setIsolationId(event.target.value)}
                  className="min-w-[220px] rounded-xl border border-white/10 bg-[#080a10] px-3 py-2 font-mono text-xs text-core outline-none focus:border-cyan/40"
                >
                  <option value="">Select analysis…</option>
                  {completed.map((item) => (
                    <option key={item.id} value={item.id}>
                      {item.original_name}
                    </option>
                  ))}
                </select>
                <button
                  type="button"
                  disabled={busy || !isolationId}
                  onClick={() => void runIsolation()}
                  className="rounded-full bg-core px-5 py-2 text-sm tracking-[0.14em] text-void disabled:opacity-35"
                >
                  Run
                </button>
              </div>
            </section>

            <section>
              <p className="font-mono text-[10px] uppercase tracking-[0.18em] text-core/40">
                k-NN density (k=3)
              </p>
              <ul className="mt-3 max-h-40 space-y-1 overflow-y-auto">
                {completed.map((item) => (
                  <li key={item.id}>
                    <label className="flex cursor-pointer items-center gap-3 rounded-lg px-2 py-1.5 hover:bg-white/5">
                      <input
                        type="checkbox"
                        checked={selected.includes(item.id)}
                        onChange={() => toggle(item.id)}
                        className="accent-cyan"
                      />
                      <span className="truncate font-mono text-xs text-core/80">
                        {item.original_name}
                      </span>
                    </label>
                  </li>
                ))}
              </ul>
              <button
                type="button"
                disabled={busy || selected.length < 2}
                onClick={() => void runKnn()}
                className="mt-3 rounded-full border border-white/15 px-5 py-2 font-mono text-[11px] tracking-[0.14em] text-core/80 hover:text-core disabled:opacity-35"
              >
                Run k-NN density
              </button>
            </section>
          </div>

          {error ? <p className="mt-4 font-mono text-xs text-anomaly">{error}</p> : null}
          {busy ? <p className="mt-4 font-mono text-xs text-cyan">Running…</p> : null}

          {result ? (
            <div className="mt-8 border-t border-white/5 pt-6">
              <p className="font-mono text-[10px] uppercase tracking-[0.16em] text-cyan/80">
                {result.method}
              </p>
              <p className="mt-2 font-mono text-xs text-core/50">{result.description}</p>
              <ul className="mt-4 space-y-2">
                {result.scores.map((score) => (
                  <li
                    key={`${score.analysis_index}-${score.label}`}
                    className="flex items-baseline justify-between gap-4 font-mono text-xs text-core/80"
                  >
                    <span className="truncate">{score.label}</span>
                    <span className="text-cyan">{(score.score * 100).toFixed(1)}%</span>
                  </li>
                ))}
              </ul>
            </div>
          ) : null}
        </div>
      </div>
    </WorkspaceChrome>
  );
}
