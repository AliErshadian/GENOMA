import type {
  AnalysisSummary,
  Anomaly,
  AuthResponse,
  AuthUser,
  CompareRequest,
  CompareResponse,
  CreateEvolutionRequest,
  EvolutionGitRequest,
  EvolutionSeries,
  EvolutionSnapshotInput,
  ExperimentResult,
  FileDna,
  GalaxyRequest,
  GalaxyResponse,
  MutationsRequest,
  MutationsResponse,
  ProgressEvent,
} from "@genoma/shared-types";

export class ApiError extends Error {
  constructor(
    message: string,
    public readonly code?: string,
    public readonly status?: number,
  ) {
    super(message);
  }
}

const TOKEN_KEY = "genoma_token";

export function getAuthToken(): string | null {
  if (typeof window === "undefined") return null;
  return window.localStorage.getItem(TOKEN_KEY);
}

export function setAuthToken(token: string | null): void {
  if (typeof window === "undefined") return;
  if (token) window.localStorage.setItem(TOKEN_KEY, token);
  else window.localStorage.removeItem(TOKEN_KEY);
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const token = getAuthToken();
  const response = await fetch(path, {
    ...init,
    headers: {
      Accept: "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...init?.headers,
    },
  });
  if (!response.ok) {
    let message = response.statusText;
    let code: string | undefined;
    try {
      const body = (await response.json()) as { error?: string; code?: string };
      message = body.error || message;
      code = body.code;
    } catch {
      /* non-JSON error */
    }
    throw new ApiError(message, code, response.status);
  }
  return response.json() as Promise<T>;
}

export type UploadOptions = {
  chunkSize?: number;
  level?: string;
  piOffset?: number;
};

export function uploadFile(file: File, options: UploadOptions = {}): Promise<AnalysisSummary> {
  const data = new FormData();
  data.append("file", file);
  const query = new URLSearchParams();
  if (options.chunkSize) query.set("chunk_size", String(options.chunkSize));
  if (options.level) query.set("level", options.level);
  if (options.piOffset != null) query.set("pi_offset", String(options.piOffset));
  const suffix = query.toString();
  return request<AnalysisSummary>(`/api/v1/analyses${suffix ? `?${suffix}` : ""}`, {
    method: "POST",
    body: data,
  });
}

export function startDemo(
  file = "sample.txt",
  options: UploadOptions = {},
): Promise<AnalysisSummary> {
  const query = new URLSearchParams({ file });
  if (options.chunkSize) query.set("chunk_size", String(options.chunkSize));
  if (options.level) query.set("level", options.level);
  return request<AnalysisSummary>(`/api/v1/analyses/demo?${query.toString()}`, {
    method: "POST",
  });
}

export function getAnalysis(id: string): Promise<AnalysisSummary> {
  return request<AnalysisSummary>(`/api/v1/analyses/${id}`);
}

export function listAnalyses(): Promise<AnalysisSummary[]> {
  return request<AnalysisSummary[]>("/api/v1/analyses");
}

export function getDna(id: string): Promise<FileDna> {
  return request<FileDna>(`/api/v1/dna/${id}`);
}

export function getAnomalies(id: string): Promise<Anomaly[]> {
  return request<Anomaly[]>(`/api/v1/anomalies/${id}`);
}

export function compareAnalyses(leftId: string, rightId: string): Promise<CompareResponse> {
  return request<CompareResponse>("/api/v1/compare", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ left_id: leftId, right_id: rightId } satisfies CompareRequest),
  });
}

export function detectMutations(
  baselineId: string,
  currentId: string,
): Promise<MutationsResponse> {
  return request<MutationsResponse>("/api/v1/mutations", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      baseline_id: baselineId,
      current_id: currentId,
    } satisfies MutationsRequest),
  });
}

export function fetchGalaxy(analysisIds: string[]): Promise<GalaxyResponse> {
  return request<GalaxyResponse>("/api/v1/galaxy", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ analysis_ids: analysisIds } satisfies GalaxyRequest),
  });
}

export function createEvolution(
  snapshots: EvolutionSnapshotInput[],
  name?: string,
): Promise<EvolutionSeries> {
  return request<EvolutionSeries>("/api/v1/evolution", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name, snapshots } satisfies CreateEvolutionRequest),
  });
}

export function getEvolution(id: string): Promise<EvolutionSeries> {
  return request<EvolutionSeries>(`/api/v1/evolution/${id}`);
}

export function listEvolution(): Promise<EvolutionSeries[]> {
  return request<EvolutionSeries[]>("/api/v1/evolution");
}

export function importEvolutionFromGit(
  repo: string,
  path: string,
  maxCommits = 8,
): Promise<EvolutionSeries> {
  return request<EvolutionSeries>("/api/v1/evolution/git", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      repo,
      path,
      max_commits: maxCommits,
    } satisfies EvolutionGitRequest),
  });
}

export function runIsolationExperiment(analysisId: string): Promise<ExperimentResult> {
  return request<ExperimentResult>("/api/v1/experiments/isolation", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ analysis_id: analysisId }),
  });
}

export function runKnnDensityExperiment(
  analysisIds: string[],
  k?: number,
): Promise<ExperimentResult> {
  return request<ExperimentResult>("/api/v1/experiments/knn-density", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ analysis_ids: analysisIds, k }),
  });
}

export function getProgress(id: string): Promise<ProgressEvent> {
  return request<ProgressEvent>(`/api/v1/analyses/${id}/progress/latest`);
}

export function listDemos(): Promise<string[]> {
  return request<string[]>("/api/v1/demos");
}

export async function register(email: string, password: string): Promise<AuthResponse> {
  const result = await request<AuthResponse>("/api/v1/auth/register", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, password }),
  });
  setAuthToken(result.token);
  return result;
}

export async function login(email: string, password: string): Promise<AuthResponse> {
  const result = await request<AuthResponse>("/api/v1/auth/login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, password }),
  });
  setAuthToken(result.token);
  return result;
}

export async function logout(): Promise<void> {
  try {
    await request<{ ok: boolean }>("/api/v1/auth/logout", { method: "POST" });
  } finally {
    setAuthToken(null);
  }
}

export function authMe(): Promise<AuthUser> {
  return request<AuthUser>("/api/v1/auth/me");
}

export async function loadDemoDna(): Promise<FileDna> {
  const response = await fetch("/demo-dna.json", { cache: "no-store" });
  if (!response.ok) {
    throw new Error("Demo DNA is not available");
  }
  return response.json() as Promise<FileDna>;
}
