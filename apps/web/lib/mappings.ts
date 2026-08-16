import { SEMANTIC_COLORS, type ChunkDna, type VisualDna } from "@genoma/shared-types";

export function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * Math.min(1, Math.max(0, t));
}

export function mixHex(a: string, b: string, t: number): string {
  const pa = hexToRgb(a);
  const pb = hexToRgb(b);
  const m = pa.map((value, i) => Math.round(lerp(value, pb[i] ?? value, t)));
  return `rgb(${m[0]}, ${m[1]}, ${m[2]})`;
}

function hexToRgb(hex: string): number[] {
  const value = hex.replace("#", "");
  return [
    parseInt(value.slice(0, 2), 16),
    parseInt(value.slice(2, 4), 16),
    parseInt(value.slice(4, 6), 16),
  ];
}

export function colorForChunk(chunk: ChunkDna, anomalyScore = 0): string {
  if (anomalyScore >= 0.7) {
    return SEMANTIC_COLORS.red;
  }
  const entropyColor = mixHex(SEMANTIC_COLORS.cyan, SEMANTIC_COLORS.purple, chunk.raw.entropy);
  if (chunk.raw.repetition > 0.25) {
    return mixHex(entropyColor, SEMANTIC_COLORS.blue, chunk.raw.repetition);
  }
  return entropyColor;
}

export function particleBudget(visual: VisualDna, chunkCount: number): number {
  const requested = visual.particle_count * Math.max(1, Math.min(chunkCount, 24) / 4);
  return Math.round(Math.min(8000, Math.max(240, requested)));
}

export const VISUAL_MAPPINGS = [
  { property: "Entropy", visual: "Particle density, cyan → purple" },
  { property: "Complexity", visual: "Geometry complexity and branching" },
  { property: "Repetition", visual: "Clustering strength, blue tint" },
  { property: "Byte diversity", visual: "Color variation" },
  { property: "Bit transition rate", visual: "Particle motion / orbital speed" },
  { property: "π position", visual: "Rotation / orientation" },
  { property: "Mutation", visual: "Orange pulse" },
  { property: "Anomaly score", visual: "Red distortion" },
] as const;
