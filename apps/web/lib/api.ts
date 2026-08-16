import type {
  AnalysisSummary,
  Anomaly,
  CompareRequest,
  CompareResponse,
  FileDna,
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

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, {
    ...init,
    headers: {
      Accept: "application/json",
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

export function getProgress(id: string): Promise<ProgressEvent> {
  return request<ProgressEvent>(`/api/v1/analyses/${id}/progress/latest`);
}

export function listDemos(): Promise<string[]> {
  return request<string[]>("/api/v1/demos");
}

export async function loadDemoDna(): Promise<FileDna> {
  const response = await fetch("/demo-dna.json", { cache: "no-store" });
  if (!response.ok) {
    throw new Error("Demo DNA is not available");
  }
  return response.json() as Promise<FileDna>;
}
