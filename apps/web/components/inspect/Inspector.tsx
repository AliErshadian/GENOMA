"use client";

import type { ChunkDna } from "@genoma/shared-types";

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
}: {
  fileName?: string;
  chunk?: ChunkDna | null;
}) {
  return (
    <div className="panel flex h-full flex-col rounded-2xl p-5">
      <p className="font-mono text-[10px] uppercase tracking-[0.22em] text-cyan/70">
        Selected object
      </p>
      <h2 className="mt-3 text-lg tracking-wide">{fileName ?? "No selection"}</h2>
      {chunk ? (
        <div className="mt-6 divide-y divide-white/5">
          <Row label="Block" value={`#${chunk.index}`} />
          <Row label="Offset" value={chunk.offset.toLocaleString()} />
          <Row label="Size" value={`${chunk.size.toLocaleString()} B`} />
          <Row label="Entropy" value={chunk.raw.entropy.toFixed(4)} />
          <Row label="Complexity" value={`${(chunk.raw.complexity * 100).toFixed(1)}%`} />
          <Row label="Repetition" value={`${(chunk.raw.repetition * 100).toFixed(2)}%`} />
          <Row label="π Offset" value={chunk.pi_derived.pi_offset.toLocaleString()} />
        </div>
      ) : (
        <p className="mt-6 font-mono text-xs leading-6 text-core/45">
          Click a particle cluster to inspect a block. Double-click the canvas to reset focus.
        </p>
      )}
    </div>
  );
}
