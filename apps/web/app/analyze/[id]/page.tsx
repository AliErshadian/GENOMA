"use client";

import { useEffect, useMemo, useState } from "react";
import { useParams } from "next/navigation";
import type { AnalysisSummary, Anomaly, FileDna, Mutation } from "@genoma/shared-types";
import { WorkspaceChrome } from "@/components/layout/WorkspaceChrome";
import { AnomalyPanel } from "@/components/inspect/AnomalyPanel";
import { Inspector } from "@/components/inspect/Inspector";
import { StatsStrip } from "@/components/inspect/StatsStrip";
import { CompleteCard } from "@/components/inspect/CompleteCard";
import { ProgressPanel } from "@/components/upload/ProgressPanel";
import { DnaCanvas } from "@/features/visualization/DnaCanvas";
import { DEFAULT_LAYERS, type VizLayers } from "@/features/visualization/layers";
import { detectMutations, getAnalysis, getAnomalies, getDna, listAnalyses } from "@/lib/api";
import { useAnalysisProgress } from "@/hooks/useAnalysisProgress";

export default function AnalysisWorkspacePage() {
  const params = useParams<{ id: string }>();
  const id = params.id;
  const progress = useAnalysisProgress(id);
  const [summary, setSummary] = useState<AnalysisSummary | null>(null);
  const [dna, setDna] = useState<FileDna | null>(null);
  const [anomalies, setAnomalies] = useState<Anomaly[]>([]);
  const [mutations, setMutations] = useState<Mutation[]>([]);
  const [baselines, setBaselines] = useState<AnalysisSummary[]>([]);
  const [baselineId, setBaselineId] = useState("");
  const [mutationBusy, setMutationBusy] = useState(false);
  const [selected, setSelected] = useState<number | null>(null);
  const [showComplete, setShowComplete] = useState(true);
  const [layers, setLayers] = useState<VizLayers>(DEFAULT_LAYERS);

  useEffect(() => {
    void getAnalysis(id).then(setSummary).catch(() => undefined);
  }, [id]);

  useEffect(() => {
    if (progress?.stage !== "COMPLETE" && summary?.status !== "COMPLETE") return;
    void (async () => {
      const [next, fingerprint, found] = await Promise.all([
        getAnalysis(id),
        getDna(id),
        getAnomalies(id),
      ]);
      setSummary(next);
      setDna(fingerprint);
      setAnomalies(found);
    })();
  }, [id, progress?.stage, summary?.status]);

  useEffect(() => {
    void listAnalyses()
      .then((items) =>
        setBaselines(items.filter((item) => item.status === "COMPLETE" && item.id !== id)),
      )
      .catch(() => setBaselines([]));
  }, [id, summary?.status]);

  useEffect(() => {
    if (!baselineId) {
      setMutations([]);
      return;
    }
    let cancelled = false;
    setMutationBusy(true);
    void detectMutations(baselineId, id)
      .then((result) => {
        if (!cancelled) setMutations(result.mutations);
      })
      .catch(() => {
        if (!cancelled) setMutations([]);
      })
      .finally(() => {
        if (!cancelled) setMutationBusy(false);
      });
    return () => {
      cancelled = true;
    };
  }, [baselineId, id]);

  const chunk = useMemo(
    () => dna?.chunks.find((item) => item.index === selected) ?? null,
    [dna, selected],
  );

  const complete = summary?.status === "COMPLETE" && dna;
  const mutationCount = baselineId ? mutations.length : null;

  return (
    <WorkspaceChrome
      inspector={
        <div className="flex h-full flex-col gap-3">
          <div className="min-h-0 flex-1">
            <Inspector
              fileName={summary?.original_name}
              chunk={chunk}
              baselines={baselines}
              baselineId={baselineId}
              onBaselineChange={setBaselineId}
              mutations={mutations}
              mutationBusy={mutationBusy}
            />
          </div>
          {complete ? (
            <AnomalyPanel
              anomalies={anomalies}
              selected={selected}
              onSelect={setSelected}
              enabled={layers.anomalies}
            />
          ) : null}
        </div>
      }
      stats={
        <StatsStrip
          entropy={dna?.raw.entropy}
          complexity={dna?.raw.complexity}
          repetition={dna?.raw.repetition}
          mutations={mutationCount}
          anomalies={anomalies.length}
          piOffset={dna?.pi_base_offset}
        />
      }
    >
      {complete && showComplete && summary ? (
        <div className="flex h-full items-center justify-center px-6">
          <CompleteCard
            summary={summary}
            onContinue={() => {
              setShowComplete(false);
              const top = anomalies[0];
              if (top) setSelected(top.chunk_index);
            }}
          />
        </div>
      ) : complete && dna ? (
        <DnaCanvas
          dna={dna}
          anomalies={anomalies}
          mutations={mutations}
          highlighted={selected}
          onSelect={setSelected}
          showControls
          layers={layers}
          onLayersChange={setLayers}
        />
      ) : (
        <div className="flex h-full items-center justify-center">
          <ProgressPanel event={progress ?? summary?.progress ?? null} />
        </div>
      )}
    </WorkspaceChrome>
  );
}
