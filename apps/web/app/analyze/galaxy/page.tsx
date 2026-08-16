"use client";

import { Suspense, useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import type { AnalysisSummary, GalaxyNode } from "@genoma/shared-types";
import { WorkspaceChrome } from "@/components/layout/WorkspaceChrome";
import { fetchGalaxy, listAnalyses } from "@/lib/api";

function GalaxyInner() {
  const params = useSearchParams();
  const [analyses, setAnalyses] = useState<AnalysisSummary[]>([]);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [nodes, setNodes] = useState<GalaxyNode[]>([]);
  const [clusterCount, setClusterCount] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const completed = useMemo(
    () => analyses.filter((item) => item.status === "COMPLETE"),
    [analyses],
  );

  useEffect(() => {
    void listAnalyses()
      .then((items) => {
        setAnalyses(items);
        const fromQuery = params.get("ids");
        if (fromQuery) {
          const ids = fromQuery
            .split(",")
            .map((value) => value.trim())
            .filter(Boolean);
          setSelected(new Set(ids));
        }
      })
      .catch(() => setAnalyses([]));
  }, [params]);

  useEffect(() => {
    const ids = Array.from(selected);
    if (ids.length === 0) {
      setNodes([]);
      setClusterCount(0);
      return;
    }
    let cancelled = false;
    setBusy(true);
    setError(null);
    void fetchGalaxy(ids)
      .then((result) => {
        if (!cancelled) {
          setNodes(result.nodes);
          setClusterCount(result.cluster_count);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setNodes([]);
          setClusterCount(0);
          setError(err instanceof Error ? err.message : "Galaxy request failed");
        }
      })
      .finally(() => {
        if (!cancelled) setBusy(false);
      });
    return () => {
      cancelled = true;
    };
  }, [selected]);

  const toggle = (id: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const selectAll = () => {
    setSelected(new Set(completed.map((item) => item.id)));
  };

  const clear = () => setSelected(new Set());

  return (
    <WorkspaceChrome>
      <div className="flex h-full justify-center overflow-y-auto px-6 py-24 md:pl-56">
        <div className="panel w-full max-w-3xl rounded-3xl p-8 md:p-10">
          <p className="font-mono text-[11px] tracking-[0.32em] text-cyan">GALAXY</p>
          <h1 className="mt-3 text-2xl tracking-[0.1em]">Multi-file set</h1>
          <p className="mt-3 max-w-xl font-mono text-xs leading-relaxed text-core/50">
            Select completed analyses to form a galaxy. Nodes are clustered by structural
            similarity (average-linkage cut at distance 0.35). 3D embedding lands next.
          </p>

          <div className="mt-6 flex flex-wrap gap-2">
            <button
              type="button"
              onClick={selectAll}
              className="rounded-full border border-white/10 px-3 py-1 font-mono text-[11px] text-core/70 hover:text-core"
            >
              Select all completed
            </button>
            <button
              type="button"
              onClick={clear}
              className="rounded-full border border-white/10 px-3 py-1 font-mono text-[11px] text-core/70 hover:text-core"
            >
              Clear
            </button>
          </div>

          {completed.length === 0 ? (
            <p className="mt-8 font-mono text-xs text-core/40">
              No completed analyses yet.{" "}
              <Link href="/analyze" className="text-cyan hover:underline">
                Analyze files
              </Link>{" "}
              first, or drop multiple files to queue a galaxy set.
            </p>
          ) : (
            <ul className="mt-6 max-h-48 space-y-1 overflow-y-auto border-t border-white/5 pt-4">
              {completed.map((item) => {
                const checked = selected.has(item.id);
                return (
                  <li key={item.id}>
                    <label className="flex cursor-pointer items-center gap-3 rounded-lg px-2 py-1.5 hover:bg-white/5">
                      <input
                        type="checkbox"
                        checked={checked}
                        onChange={() => toggle(item.id)}
                        className="accent-cyan"
                      />
                      <span className="truncate font-mono text-xs text-core/80">
                        {item.original_name}
                      </span>
                      <span className="ml-auto font-mono text-[10px] text-core/35">
                        {item.id.slice(0, 8)}
                      </span>
                    </label>
                  </li>
                );
              })}
            </ul>
          )}

          {error ? <p className="mt-4 font-mono text-xs text-anomaly">{error}</p> : null}
          {busy ? <p className="mt-4 font-mono text-xs text-cyan">Loading galaxy…</p> : null}

          {nodes.length > 0 ? (
            <div className="mt-8 overflow-x-auto border-t border-white/5 pt-6">
              <p className="mb-3 font-mono text-[10px] uppercase tracking-[0.18em] text-core/40">
                Nodes · {nodes.length} · Clusters · {clusterCount}
              </p>
              <table className="w-full min-w-[520px] text-left font-mono text-[11px]">
                <thead className="text-core/40">
                  <tr>
                    <th className="pb-2 font-normal">Name</th>
                    <th className="pb-2 font-normal">Cluster</th>
                    <th className="pb-2 font-normal">Entropy</th>
                    <th className="pb-2 font-normal">Complexity</th>
                    <th className="pb-2 font-normal">Size</th>
                    <th className="pb-2 font-normal">Chunks</th>
                  </tr>
                </thead>
                <tbody>
                  {[...nodes]
                    .sort((a, b) => a.cluster_id - b.cluster_id || a.name.localeCompare(b.name))
                    .map((node) => (
                      <tr key={node.id} className="border-t border-white/5 text-core/80">
                        <td className="py-2">
                          <Link href={`/analyze/${node.id}`} className="hover:text-cyan">
                            {node.name}
                          </Link>
                        </td>
                        <td className="py-2">
                          <span
                            className="inline-block rounded px-1.5 py-0.5 text-[10px]"
                            style={{
                              background: `hsla(${(node.cluster_id * 57) % 360} 55% 45% / 0.25)`,
                              color: `hsl(${(node.cluster_id * 57) % 360} 70% 72%)`,
                            }}
                          >
                            C{node.cluster_id}
                          </span>
                        </td>
                        <td className="py-2">{node.entropy.toFixed(4)}</td>
                        <td className="py-2">{(node.complexity * 100).toFixed(1)}%</td>
                        <td className="py-2">{node.size_bytes.toLocaleString()}</td>
                        <td className="py-2">{node.chunk_count}</td>
                      </tr>
                    ))}
                </tbody>
              </table>
            </div>
          ) : selected.size > 0 && !busy && !error ? (
            <p className="mt-6 font-mono text-xs text-core/40">Waiting for node summaries…</p>
          ) : null}
        </div>
      </div>
    </WorkspaceChrome>
  );
}

export default function GalaxyPage() {
  return (
    <Suspense
      fallback={
        <WorkspaceChrome>
          <div />
        </WorkspaceChrome>
      }
    >
      <GalaxyInner />
    </Suspense>
  );
}
