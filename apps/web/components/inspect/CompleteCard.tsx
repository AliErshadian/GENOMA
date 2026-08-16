"use client";

import type { AnalysisSummary } from "@genoma/shared-types";

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-baseline justify-between gap-6 py-2">
      <span className="font-mono text-[10px] uppercase tracking-[0.16em] text-core/40">{label}</span>
      <span className="font-mono text-sm text-core/90">{value}</span>
    </div>
  );
}

export function CompleteCard({
  summary,
  onContinue,
}: {
  summary: AnalysisSummary;
  onContinue: () => void;
}) {
  const dna = summary.dna;
  return (
    <div className="panel w-full max-w-lg rounded-3xl p-10">
      <p className="font-mono text-[11px] tracking-[0.32em] text-cyan">GENOMA COMPLETE</p>
      <h2 className="mt-4 text-2xl tracking-[0.12em]">{summary.original_name}</h2>
      <div className="mt-6 divide-y divide-white/5">
        <Row label="File" value={summary.original_name} />
        <Row label="Size" value={`${summary.size_bytes.toLocaleString()} bytes`} />
        <Row label="Entropy" value={dna ? dna.entropy.toFixed(4) : "—"} />
        <Row
          label="Complexity"
          value={dna ? `${(dna.complexity * 100).toFixed(1)}%` : "—"}
        />
        <Row
          label="Repetition"
          value={dna ? `${(dna.repetition * 100).toFixed(2)}%` : "—"}
        />
        <Row label="Anomalies" value={String(summary.anomalies)} />
        <Row label="Mutations" value="—" />
        <Row
          label="π Offset"
          value={dna ? dna.pi_offset.toLocaleString() : "—"}
        />
      </div>
      <button
        type="button"
        onClick={onContinue}
        className="mt-8 w-full rounded-full bg-core px-6 py-3 text-sm tracking-[0.16em] text-void"
      >
        Enter visualization
        {summary.anomalies > 0 ? " · focus top anomaly" : ""}
      </button>
    </div>
  );
}
