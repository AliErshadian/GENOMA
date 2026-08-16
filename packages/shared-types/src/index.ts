export const GENERATOR_VERSION = "dna-v1";
export const FEATURE_DIM = 16;

export type Stage =
  | "QUEUED"
  | "READING_FILE"
  | "EXTRACTING_FEATURES"
  | "GENERATING_PIDNA"
  | "DETECTING_ANOMALIES"
  | "BUILDING_VISUALIZATION"
  | "COMPLETE"
  | "FAILED";

export interface ProgressEvent {
  stage: Stage;
  progress: number;
  processed_bytes: number;
  total_bytes: number | null;
  message: string;
}

export interface RawFeatureVector {
  entropy: number;
  complexity: number;
  repetition: number;
  bit_transition: number;
  compression: number;
  diversity: number;
  values: number[];
}

export interface PiDerivedVector {
  values: number[];
  pi_offset: number;
  pi_wrapped: boolean;
  pi_wrap_count: number;
  generator_version: string;
}

export interface VisualDna {
  density: number;
  radius: number;
  rotation: number;
  branching: number;
  particle_count: number;
  particle_velocity: number;
  cluster_strength: number;
  noise: number;
  orbital_speed: number;
  geometry_complexity: number;
  hue_mix: number;
  repetition_tint: number;
}

export interface ChunkDna {
  index: number;
  offset: number;
  size: number;
  raw: RawFeatureVector;
  pi_derived: PiDerivedVector;
  visual: VisualDna;
}

export interface FileDna {
  generator_version: string;
  pi_base_offset: number;
  chunk_count: number;
  total_bytes: number;
  raw: RawFeatureVector;
  pi_derived: PiDerivedVector;
  visual: VisualDna;
  chunks: ChunkDna[];
}

export interface Anomaly {
  chunk_index: number;
  offset: number;
  score: number;
  entropy_z: number;
  neighbor_distance: number;
}

export interface Mutation {
  chunk_index: number;
  offset: number;
  impact: number;
  confidence: number;
  distance: number;
}

export interface SimilarityBreakdown {
  entropy: number;
  distribution: number;
  pattern: number;
  complexity: number;
  overall: number;
}

export interface CompareRequest {
  left_id: string;
  right_id: string;
}

export interface CompareResponse {
  left_id: string;
  right_id: string;
  left_name: string;
  right_name: string;
  similarity: SimilarityBreakdown;
}

export interface MutationsRequest {
  baseline_id: string;
  current_id: string;
}

export interface MutationsResponse {
  baseline_id: string;
  current_id: string;
  baseline_name: string;
  current_name: string;
  mutations: Mutation[];
}

export interface GalaxyRequest {
  analysis_ids: string[];
}

export interface GalaxyNode {
  id: string;
  name: string;
  size_bytes: number;
  entropy: number;
  complexity: number;
  repetition: number;
  chunk_count: number;
  generator_version: string;
  cluster_id: number;
  position: [number, number, number];
}

export interface GalaxyLink {
  from: string;
  to: string;
  strength: number;
}

export interface GalaxyResponse {
  nodes: GalaxyNode[];
  cluster_count: number;
  links: GalaxyLink[];
}

export interface EvolutionSnapshotInput {
  analysis_id: string;
  version_label: string;
}

export interface CreateEvolutionRequest {
  name?: string;
  snapshots: EvolutionSnapshotInput[];
}

export interface EvolutionSnapshot {
  id: string;
  analysis_id: string;
  version_label: string;
  file_name: string;
  created_at: string;
}

export interface EvolutionSeries {
  id: string;
  name: string;
  created_at: string;
  snapshots: EvolutionSnapshot[];
}

export interface FileSummary {
  entropy: number;
  complexity: number;
  repetition: number;
  anomalies: number;
  mutations: number;
  pi_offset: number;
  chunk_count: number;
  generator_version: string;
}

export interface AnalysisSummary {
  id: string;
  status: Stage;
  original_name: string;
  size_bytes: number;
  mime_type: string | null;
  created_at: string;
  completed_at: string | null;
  progress: ProgressEvent | null;
  dna: FileSummary | null;
  anomalies: number;
}

export const SEMANTIC_COLORS = {
  cyan: "#7ee0f2",
  purple: "#b48cff",
  blue: "#6b8cff",
  orange: "#ff9a4a",
  red: "#ff5d6c",
  white: "#f4f6fb",
} as const;
