import { SEMANTIC_COLORS } from "@genoma/shared-types";

const PALETTE = [
  SEMANTIC_COLORS.cyan,
  SEMANTIC_COLORS.purple,
  SEMANTIC_COLORS.blue,
  SEMANTIC_COLORS.orange,
  SEMANTIC_COLORS.red,
  "#7ef2c2",
  "#f2e07e",
  "#f27eb8",
] as const;

export function clusterColor(clusterId: number): string {
  const index = ((clusterId % PALETTE.length) + PALETTE.length) % PALETTE.length;
  return PALETTE[index] ?? SEMANTIC_COLORS.white;
}
