"use client";

import { useEffect, useMemo, useState } from "react";
import { useParams } from "next/navigation";
import type { Anomaly, FileDna } from "@genoma/shared-types";
import { WorkspaceChrome } from "@/components/layout/WorkspaceChrome";
import { Inspector } from "@/components/inspect/Inspector";
import { StatsStrip } from "@/components/inspect/StatsStrip";
import { ProgressPanel } from "@/components/upload/ProgressPanel";
import { DnaCanvas } from "@/features/visualization/DnaCanvas";
import { getAnalysis, getDna, subscribeProgress } from "@/lib/api";
import type { AnalysisSummary, ProgressEvent } from "@genoma/shared-types";

export default function AnalysisWorkspacePage() {
  const params = useParams<{ id: string }>();
  const id = params.id;
  const [summary, setSummary] = useState<AnalysisSummary | null>(null);
  const [dna, setDna] = useState<FileDna | null>(null);
  const [anomalies, setAnomalies] = useState<Anomaly[]>([]);
  const [progress, setProgress] = useState<ProgressEvent | null>(null);
  const [selected, setSelected] = useState<number | null>(null);
  const [completeOverlay, setCompleteOverlay] = useState(true);

  useEffect(() => {
    let stop: (() => void) | undefined;
    void (async () => {
      const current = await getAnalysis(id);
      setSummary(current);
      setProgress(current.progress);
      if (current.status === "COMPLETE") {
        const fingerprint = await getDna(id);
        setDna(fingerprint);
        const anomalyRes = await fetch(`/api/v1/anomalies/${id}`);
        if (anomalyRes.ok) {
          setAnomalies((await anomalyRes.json()) as Anomaly[]);
        }
      } else {
        stop = subscribeProgress(id, async (event) => {
          setProgress(event);
          if (event.stage === "COMPLETE") {
            const [next, fingerprint, anomalyRes] = await Promise.all([
              getAnalysis(id),
              getDna(id),
              fetch(`/api/v1/anomalies/${id}`),
            ]);
            setSummary(next);
            setDna(fingerprint);
            if (anomalyRes.ok) {
              setAnomalies((await anomalyRes.json()) as Anomaly[]);
            }
          }
        });
      }
    })();
    return () => stop?.();
  }, [id]);

  useEffect(() => {
    if (!dna) return;
    const timer = window.setTimeout(() => setCompleteOverlay(false), 2200);
    return () => window.clearTimeout(timer);
  }, [dna]);

  const chunk = useMemo(
    () => dna?.chunks.find((item) => item.index === selected) ?? null,
    [dna, selected],
  );

  return (
    <WorkspaceChrome
      inspector={<Inspector fileName={summary?.original_name} chunk={chunk} />}
      stats={
        <StatsStrip
          entropy={dna?.raw.entropy}
          complexity={dna?.raw.complexity}
          repetition={dna?.raw.repetition}
          anomalies={anomalies.length}
          piOffset={dna?.pi_base_offset}
        />
      }
    >
      {dna ? (
        <>
          <DnaCanvas
            dna={dna}
            anomalies={anomalies}
            highlighted={selected}
            onSelect={setSelected}
          />
          {completeOverlay ? (
            <div className="pointer-events-none absolute inset-0 z-10 flex items-center justify-center bg-void/55">
              <div className="panel rounded-3xl p-10 text-center">
                <p className="font-mono text-[11px] tracking-[0.32em] text-cyan">
                  GENOMA COMPLETE
                </p>
                <h2 className="mt-3 text-2xl tracking-[0.12em]">{summary?.original_name}</h2>
                <p className="mt-4 font-mono text-sm text-core/70">
                  {(summary?.size_bytes ?? 0).toLocaleString()} bytes · entropy{" "}
                  {dna.raw.entropy.toFixed(4)}
                </p>
              </div>
            </div>
          ) : null}
        </>
      ) : (
        <div className="flex h-full items-center justify-center">
          <ProgressPanel event={progress} />
        </div>
      )}
    </WorkspaceChrome>
  );
}
