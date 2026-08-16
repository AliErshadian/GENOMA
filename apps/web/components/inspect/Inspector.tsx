"use client";

import type { AnalysisSummary, ChunkDna, Mutation } from "@genoma/shared-types";

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-baseline justify-between gap-4 py-1.5">
      <span className="font-mono text-[10px] uppercase tracking-[0.16em] text-core/40">
        {label}
      </span>
      <span className="font-mono text-[12px] text-core/90">{value}</span>
    </div>
  );
}

export function Inspector({
  fileName,
  chunk,
  baselines = [],
  baselineId,
  onBaselineChange,
  mutations = [],
  mutationBusy = false,
}: {
  fileName?: string;
  chunk?: ChunkDna | null;
  baselines?: AnalysisSummary[];
  baselineId?: string;
  onBaselineChange?: (id: string) => void;
  mutations?: Mutation[];
  mutationBusy?: boolean;
}) {
  const selectedMutation =
    chunk == null ? null : mutations.find((item) => item.chunk_index === chunk.index) ?? null;
  const topMutations = mutations.slice(0, 6);

  return (
    <div className="panel flex h-full flex-col overflow-hidden rounded-2xl p-5">
      <p className="font-mono text-[10px] uppercase tracking-[0.22em] text-cyan/70">
        Selected object
      </p>
      <h2 className="mt-3 text-lg tracking-wide">{fileName ?? "No selection"}</h2>

      {onBaselineChange ? (
        <label className="mt-5 block space-y-2">
          <span className="font-mono text-[10px] uppercase tracking-[0.16em] text-core/40">
            Mutation baseline
          </span>
          <select
            value={baselineId ?? ""}
            onChange={(event) => onBaselineChange(event.target.value)}
            className="w-full rounded-xl border border-white/10 bg-[#080a10] px-3 py-2 font-mono text-[11px] text-core outline-none focus:border-cyan/40"
          >
            <option value="">None</option>
            {baselines.map((item) => (
              <option key={item.id} value={item.id}>
                {item.original_name}
              </option>
            ))}
          </select>
          {mutationBusy ? (
            <p className="font-mono text-[10px] text-core/40">Detecting mutations…</p>
          ) : null}
        </label>
      ) : null}

      {chunk ? (
        <div className="mt-6 divide-y divide-white/5">
          <Row label="Block" value={`#${chunk.index}`} />
          <Row label="Offset" value={chunk.offset.toLocaleString()} />
          <Row label="Size" value={`${chunk.size.toLocaleString()} B`} />
          <Row label="Entropy" value={chunk.raw.entropy.toFixed(4)} />
          <Row label="Complexity" value={`${(chunk.raw.complexity * 100).toFixed(1)}%`} />
          <Row label="Repetition" value={`${(chunk.raw.repetition * 100).toFixed(2)}%`} />
          <Row label="π Offset" value={chunk.pi_derived.pi_offset.toLocaleString()} />
          {selectedMutation ? (
            <>
              <Row label="Mutation" value={`${(selectedMutation.impact * 100).toFixed(1)}%`} />
              <Row
                label="Confidence"
                value={`${(selectedMutation.confidence * 100).toFixed(1)}%`}
              />
            </>
          ) : null}
        </div>
      ) : (
        <p className="mt-6 font-mono text-xs leading-6 text-core/45">
          Click a particle cluster to inspect a block. Double-click the canvas to reset focus.
        </p>
      )}

      {topMutations.length > 0 ? (
        <div className="mt-6 min-h-0 flex-1 overflow-y-auto border-t border-white/5 pt-4">
          <p className="font-mono text-[10px] uppercase tracking-[0.16em] text-mutation/80">
            Top mutations
          </p>
          <ul className="mt-3 space-y-2">
            {topMutations.map((item) => (
              <li
                key={`${item.chunk_index}-${item.offset}`}
                className="flex items-baseline justify-between gap-3 font-mono text-[11px] text-core/70"
              >
                <span>#{item.chunk_index}</span>
                <span className="text-mutation">{(item.impact * 100).toFixed(1)}%</span>
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </div>
  );
}
