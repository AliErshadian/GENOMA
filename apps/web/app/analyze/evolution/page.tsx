"use client";

import { useEffect, useMemo, useState } from "react";
import Link from "next/link";
import type {
  AnalysisSummary,
  EvolutionSeries,
  EvolutionSnapshot,
  SimilarityBreakdown,
} from "@genoma/shared-types";
import { WorkspaceChrome } from "@/components/layout/WorkspaceChrome";
import {
  compareAnalyses,
  createEvolution,
  detectMutations,
  getEvolution,
  importEvolutionFromGit,
  listAnalyses,
  listEvolution,
} from "@/lib/api";

export default function EvolutionPage() {
  const [analyses, setAnalyses] = useState<AnalysisSummary[]>([]);
  const [seriesList, setSeriesList] = useState<EvolutionSeries[]>([]);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [series, setSeries] = useState<EvolutionSeries | null>(null);
  const [leftIdx, setLeftIdx] = useState(0);
  const [rightIdx, setRightIdx] = useState(1);
  const [similarity, setSimilarity] = useState<SimilarityBreakdown | null>(null);
  const [mutationCount, setMutationCount] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [seriesName, setSeriesName] = useState("");

  const completed = useMemo(
    () => analyses.filter((item) => item.status === "COMPLETE"),
    [analyses],
  );

  useEffect(() => {
    void listAnalyses()
      .then(setAnalyses)
      .catch(() => setAnalyses([]));
    void listEvolution()
      .then(setSeriesList)
      .catch(() => setSeriesList([]));
  }, []);

  useEffect(() => {
    if (!series || series.snapshots.length < 2) {
      setSimilarity(null);
      setMutationCount(null);
      return;
    }
    const left = series.snapshots[Math.min(leftIdx, series.snapshots.length - 1)];
    const right = series.snapshots[Math.min(rightIdx, series.snapshots.length - 1)];
    if (!left || !right || left.analysis_id === right.analysis_id) {
      setSimilarity(null);
      setMutationCount(null);
      return;
    }
    let cancelled = false;
    void (async () => {
      try {
        const [cmp, mutations] = await Promise.all([
          compareAnalyses(left.analysis_id, right.analysis_id),
          detectMutations(left.analysis_id, right.analysis_id),
        ]);
        if (!cancelled) {
          setSimilarity(cmp.similarity);
          setMutationCount(mutations.mutations.length);
        }
      } catch {
        if (!cancelled) {
          setSimilarity(null);
          setMutationCount(null);
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [series, leftIdx, rightIdx]);

  const toggleId = (id: string) => {
    setSelectedIds((prev) =>
      prev.includes(id) ? prev.filter((value) => value !== id) : [...prev, id],
    );
  };

  const createSeries = async () => {
    if (selectedIds.length === 0) return;
    setBusy(true);
    setError(null);
    try {
      const snapshots = selectedIds.map((id, index) => ({
        analysis_id: id,
        version_label: `v${index + 1}`,
      }));
      const created = await createEvolution(snapshots, seriesName || undefined);
      setSeries(created);
      setLeftIdx(0);
      setRightIdx(Math.min(1, created.snapshots.length - 1));
      setSeriesList((prev) => [created, ...prev.filter((item) => item.id !== created.id)]);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create series");
    } finally {
      setBusy(false);
    }
  };

  const openSeries = async (id: string) => {
    setBusy(true);
    setError(null);
    try {
      const next = await getEvolution(id);
      setSeries(next);
      setLeftIdx(0);
      setRightIdx(Math.min(1, next.snapshots.length - 1));
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load series");
    } finally {
      setBusy(false);
    }
  };

  const importGitDemo = async () => {
    setBusy(true);
    setError(null);
    try {
      const created = await importEvolutionFromGit("demo-evolve", "sample.txt", 8);
      setSeries(created);
      setLeftIdx(0);
      setRightIdx(Math.min(1, created.snapshots.length - 1));
      setSeriesList((prev) => [created, ...prev.filter((item) => item.id !== created.id)]);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Git import failed");
    } finally {
      setBusy(false);
    }
  };

  const selectAdjacent = (index: number) => {
    if (!series || series.snapshots.length < 2) return;
    setLeftIdx(index);
    setRightIdx(Math.min(index + 1, series.snapshots.length - 1));
  };

  const snapshotAt = (index: number): EvolutionSnapshot | null =>
    series?.snapshots[index] ?? null;

  return (
    <WorkspaceChrome>
      <div className="flex h-full justify-center overflow-y-auto px-6 py-24 md:pl-56">
        <div className="panel w-full max-w-4xl rounded-3xl p-8 md:p-10">
          <p className="font-mono text-[11px] tracking-[0.32em] text-cyan">EVOLUTION</p>
          <h1 className="mt-3 text-2xl tracking-[0.1em]">Version timeline</h1>
          <p className="mt-3 max-w-2xl font-mono text-xs leading-relaxed text-core/50">
            Build a series from completed analyses, scrub the timeline, and compare adjacent
            versions with structural similarity and mutation counts. Or import the allowlisted
            Git demo repo under data/repos.
          </p>

          <div className="mt-6">
            <button
              type="button"
              disabled={busy}
              onClick={() => void importGitDemo()}
              className="rounded-full border border-cyan/30 px-4 py-2 font-mono text-[11px] tracking-[0.14em] text-cyan hover:bg-cyan/10 disabled:opacity-35"
            >
              Import Git demo
            </button>
          </div>

          <div className="mt-8 grid gap-8 lg:grid-cols-2">
            <div>
              <p className="font-mono text-[10px] uppercase tracking-[0.18em] text-core/40">
                New series
              </p>
              <input
                value={seriesName}
                onChange={(event) => setSeriesName(event.target.value)}
                placeholder="Series name (optional)"
                className="mt-3 w-full rounded-xl border border-white/10 bg-[#080a10] px-3 py-2 font-mono text-xs text-core outline-none focus:border-cyan/40"
              />
              {completed.length === 0 ? (
                <p className="mt-4 font-mono text-xs text-core/40">
                  No completed analyses.{" "}
                  <Link href="/analyze" className="text-cyan hover:underline">
                    Analyze files
                  </Link>{" "}
                  first.
                </p>
              ) : (
                <ul className="mt-3 max-h-40 space-y-1 overflow-y-auto">
                  {completed.map((item) => (
                    <li key={item.id}>
                      <label className="flex cursor-pointer items-center gap-3 rounded-lg px-2 py-1.5 hover:bg-white/5">
                        <input
                          type="checkbox"
                          checked={selectedIds.includes(item.id)}
                          onChange={() => toggleId(item.id)}
                          className="accent-cyan"
                        />
                        <span className="truncate font-mono text-xs text-core/80">
                          {item.original_name}
                        </span>
                      </label>
                    </li>
                  ))}
                </ul>
              )}
              <button
                type="button"
                disabled={busy || selectedIds.length === 0}
                onClick={() => void createSeries()}
                className="mt-4 w-full rounded-full bg-core px-6 py-3 text-sm tracking-[0.16em] text-void disabled:cursor-not-allowed disabled:opacity-35"
              >
                Create series
              </button>
            </div>

            <div>
              <p className="font-mono text-[10px] uppercase tracking-[0.18em] text-core/40">
                Existing series
              </p>
              {seriesList.length === 0 ? (
                <p className="mt-4 font-mono text-xs text-core/40">No series yet.</p>
              ) : (
                <ul className="mt-3 max-h-52 space-y-1 overflow-y-auto">
                  {seriesList.map((item) => (
                    <li key={item.id}>
                      <button
                        type="button"
                        onClick={() => void openSeries(item.id)}
                        className={`flex w-full items-center justify-between rounded-lg px-2 py-1.5 text-left font-mono text-xs hover:bg-white/5 ${
                          series?.id === item.id ? "bg-white/5 text-cyan" : "text-core/75"
                        }`}
                      >
                        <span className="truncate">{item.name}</span>
                        <span className="text-core/35">{item.snapshots.length}</span>
                      </button>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          </div>

          {error ? <p className="mt-4 font-mono text-xs text-anomaly">{error}</p> : null}
          {busy ? <p className="mt-4 font-mono text-xs text-cyan">Working…</p> : null}

          {series && series.snapshots.length > 0 ? (
            <div className="mt-10 border-t border-white/5 pt-8">
              <div className="flex flex-wrap items-baseline justify-between gap-3">
                <p className="font-mono text-[10px] uppercase tracking-[0.18em] text-core/40">
                  Timeline · {series.name}
                </p>
                <p className="font-mono text-[10px] text-core/35">
                  {series.snapshots.length} snapshots
                </p>
              </div>

              <div className="mt-6 flex gap-2 overflow-x-auto pb-2">
                {series.snapshots.map((snapshot, index) => {
                  const active = index === leftIdx || index === rightIdx;
                  const role =
                    index === leftIdx && index === rightIdx
                      ? "both"
                      : index === leftIdx
                        ? "left"
                        : index === rightIdx
                          ? "right"
                          : null;
                  return (
                    <button
                      key={snapshot.id}
                      type="button"
                      onClick={() => selectAdjacent(index)}
                      className={`min-w-[120px] rounded-2xl border px-3 py-3 text-left transition ${
                        active
                          ? "border-cyan/40 bg-cyan/10"
                          : "border-white/10 bg-white/[0.02] hover:border-white/20"
                      }`}
                    >
                      <p className="font-mono text-[10px] uppercase tracking-[0.14em] text-cyan/80">
                        {snapshot.version_label}
                        {role ? ` · ${role}` : ""}
                      </p>
                      <p className="mt-2 truncate font-mono text-xs text-core/80">
                        {snapshot.file_name}
                      </p>
                      <Link
                        href={`/analyze/${snapshot.analysis_id}`}
                        onClick={(event) => event.stopPropagation()}
                        className="mt-2 inline-block font-mono text-[10px] text-core/40 hover:text-cyan"
                      >
                        Open organism
                      </Link>
                    </button>
                  );
                })}
              </div>

              <div className="mt-6 flex flex-wrap gap-2">
                <button
                  type="button"
                  onClick={() => {
                    setLeftIdx((value) => {
                      const nextLeft = Math.max(0, value - 1);
                      setRightIdx(Math.min(series.snapshots.length - 1, nextLeft + 1));
                      return nextLeft;
                    });
                  }}
                  className="rounded-full border border-white/10 px-3 py-1 font-mono text-[11px] text-core/70 hover:text-core"
                >
                  ← Step
                </button>
                <button
                  type="button"
                  onClick={() => {
                    setLeftIdx((value) => {
                      const nextLeft = Math.min(series.snapshots.length - 2, value + 1);
                      setRightIdx(Math.min(series.snapshots.length - 1, nextLeft + 1));
                      return Math.max(0, nextLeft);
                    });
                  }}
                  className="rounded-full border border-white/10 px-3 py-1 font-mono text-[11px] text-core/70 hover:text-core"
                >
                  Step →
                </button>
              </div>

              <div className="mt-8 grid gap-4 md:grid-cols-2">
                <div className="rounded-2xl border border-white/10 p-4">
                  <p className="font-mono text-[10px] uppercase tracking-[0.16em] text-core/40">
                    Left
                  </p>
                  <p className="mt-2 font-mono text-sm text-core">
                    {snapshotAt(leftIdx)?.version_label ?? "—"} ·{" "}
                    {snapshotAt(leftIdx)?.file_name ?? "—"}
                  </p>
                </div>
                <div className="rounded-2xl border border-white/10 p-4">
                  <p className="font-mono text-[10px] uppercase tracking-[0.16em] text-core/40">
                    Right
                  </p>
                  <p className="mt-2 font-mono text-sm text-core">
                    {snapshotAt(rightIdx)?.version_label ?? "—"} ·{" "}
                    {snapshotAt(rightIdx)?.file_name ?? "—"}
                  </p>
                </div>
              </div>

              {similarity ? (
                <div className="mt-6 rounded-2xl border border-white/10 p-5">
                  <div className="flex flex-wrap items-end justify-between gap-4">
                    <div>
                      <p className="font-mono text-[10px] uppercase tracking-[0.16em] text-core/40">
                        Adjacent similarity
                      </p>
                      <p className="mt-2 font-mono text-3xl text-cyan">
                        {(similarity.overall * 100).toFixed(1)}%
                      </p>
                    </div>
                    <div className="text-right">
                      <p className="font-mono text-[10px] uppercase tracking-[0.16em] text-core/40">
                        Mutations
                      </p>
                      <p className="mt-2 font-mono text-2xl text-mutation">
                        {mutationCount == null ? "—" : mutationCount}
                      </p>
                    </div>
                  </div>
                  <div className="mt-4 grid gap-2 font-mono text-[11px] text-core/60 sm:grid-cols-4">
                    <span>Entropy {(similarity.entropy * 100).toFixed(1)}%</span>
                    <span>Distribution {(similarity.distribution * 100).toFixed(1)}%</span>
                    <span>Pattern {(similarity.pattern * 100).toFixed(1)}%</span>
                    <span>Complexity {(similarity.complexity * 100).toFixed(1)}%</span>
                  </div>
                </div>
              ) : (
                <p className="mt-6 font-mono text-xs text-core/40">
                  Select two different timeline points to compare.
                </p>
              )}
            </div>
          ) : null}
        </div>
      </div>
    </WorkspaceChrome>
  );
}
