"use client";

import { useEffect, useMemo, useState } from "react";
import { useParams } from "next/navigation";
import type { Anomaly, FileDna } from "@genoma/shared-types";
import { WorkspaceChrome } from "@/components/layout/WorkspaceChrome";
import { Inspector } from "@/components/inspect/Inspector";
import { StatsStrip } from "@/components/inspect/StatsStrip";
import { CompleteCard } from "@/components/inspect/CompleteCard";
import { ProgressPanel } from "@/components/upload/ProgressPanel";
import { DnaCanvas } from "@/features/visualization/DnaCanvas";
import { getAnalysis, getAnomalies, getDna } from "@/lib/api";
import { useAnalysisProgress } from "@/hooks/useAnalysisProgress";
import type { AnalysisSummary } from "@genoma/shared-types";

export default function AnalysisWorkspacePage() {
  const params = useParams<{ id: string }>();
  const id = params.id;
  const progress = useAnalysisProgress(id);
  const [summary, setSummary] = useState<AnalysisSummary | null>(null);
  const [dna, setDna] = useState<FileDna | null>(null);
  const [anomalies, setAnomalies] = useState<Anomaly[]>([]);
  const [selected, setSelected] = useState<number | null>(null);
  const [showComplete, setShowComplete] = useState(true);

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

  const chunk = useMemo(
    () => dna?.chunks.find((item) => item.index === selected) ?? null,
    [dna, selected],
  );

  const complete = summary?.status === "COMPLETE" && dna;

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
      {complete && showComplete && summary ? (
        <div className="flex h-full items-center justify-center px-6">
          <CompleteCard summary={summary} onContinue={() => setShowComplete(false)} />
        </div>
      ) : complete && dna ? (
        <DnaCanvas
          dna={dna}
          anomalies={anomalies}
          highlighted={selected}
          onSelect={setSelected}
        />
      ) : (
        <div className="flex h-full items-center justify-center">
          <ProgressPanel event={progress ?? summary?.progress ?? null} />
        </div>
      )}
    </WorkspaceChrome>
  );
}
