import type { AnalysisSummary, FileDna, ProgressEvent } from "@genoma/shared-types";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, {
    ...init,
    headers: {
      Accept: "application/json",
      ...init?.headers,
    },
  });
  if (!response.ok) {
    const body = await response.text();
    throw new Error(body || response.statusText);
  }
  return response.json() as Promise<T>;
}

export function uploadFile(file: File): Promise<AnalysisSummary> {
  const data = new FormData();
  data.append("file", file);
  return request<AnalysisSummary>("/api/v1/analyses", { method: "POST", body: data });
}

export function startDemo(file = "sample.txt"): Promise<AnalysisSummary> {
  return request<AnalysisSummary>(`/api/v1/analyses/demo?file=${encodeURIComponent(file)}`, {
    method: "POST",
  });
}

export function getAnalysis(id: string): Promise<AnalysisSummary> {
  return request<AnalysisSummary>(`/api/v1/analyses/${id}`);
}

export function getDna(id: string): Promise<FileDna> {
  return request<FileDna>(`/api/v1/dna/${id}`);
}

export function listDemos(): Promise<string[]> {
  return request<string[]>("/api/v1/demos");
}

export function subscribeProgress(
  id: string,
  onEvent: (event: ProgressEvent) => void,
): () => void {
  const source = new EventSource(`/api/v1/analyses/${id}/progress`);
  source.onmessage = (message) => {
    const event = JSON.parse(message.data) as ProgressEvent;
    onEvent(event);
    if (event.stage === "COMPLETE" || event.stage === "FAILED") {
      source.close();
    }
  };
  return () => source.close();
}

export async function loadDemoDna(): Promise<FileDna> {
  const response = await fetch("/demo-dna.json", { cache: "no-store" });
  if (!response.ok) {
    throw new Error("Demo DNA is not available");
  }
  return response.json() as Promise<FileDna>;
}
