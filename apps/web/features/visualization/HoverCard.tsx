"use client";

import type { ChunkDna } from "@genoma/shared-types";

export function HoverCard({
  chunk,
  x,
  y,
}: {
  chunk: ChunkDna;
  x: number;
  y: number;
}) {
  return (
    <div
      className="pointer-events-none absolute z-30 w-56 rounded-xl border border-white/10 bg-[#080a10]/90 p-3 font-mono text-[10px] text-core/80 backdrop-blur"
      style={{ left: x + 16, top: y + 16 }}
    >
      <p className="tracking-[0.18em] text-cyan/80">BLOCK #{chunk.index}</p>
      <p className="mt-2">Offset {chunk.offset.toLocaleString()}</p>
      <p>Size {chunk.size.toLocaleString()} B</p>
      <p>Entropy {chunk.raw.entropy.toFixed(4)}</p>
      <p>Complexity {(chunk.raw.complexity * 100).toFixed(1)}%</p>
      <p>Repetition {(chunk.raw.repetition * 100).toFixed(2)}%</p>
      <p>π Offset {chunk.pi_derived.pi_offset.toLocaleString()}</p>
    </div>
  );
}
