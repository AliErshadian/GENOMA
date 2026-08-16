"use client";

import type { ProgressEvent, Stage } from "@genoma/shared-types";

const STAGES: { id: Stage; label: string }[] = [
  { id: "READING_FILE", label: "Reading file" },
  { id: "EXTRACTING_FEATURES", label: "Extracting features" },
  { id: "GENERATING_PIDNA", label: "Generating πDNA" },
  { id: "DETECTING_ANOMALIES", label: "Detecting anomalies" },
  { id: "BUILDING_VISUALIZATION", label: "Building visualization" },
];

export function ProgressPanel({ event }: { event: ProgressEvent | null }) {
  const progress = event?.progress ?? 0;
  return (
    <div className="panel w-full max-w-lg rounded-3xl p-8">
      <p className="font-mono text-[10px] tracking-[0.28em] text-cyan/70">PREPARING ANALYSIS</p>
      <p className="mt-3 text-xl">{event?.message ?? "Queued"}</p>
      <div className="mt-6 h-[3px] overflow-hidden rounded-full bg-white/10">
        <div
          className="h-full bg-gradient-to-r from-cyan to-purple transition-[width] duration-300"
          style={{ width: `${Math.round(progress * 100)}%` }}
        />
      </div>
      <ul className="mt-6 space-y-2 font-mono text-xs text-core/55">
        {STAGES.map((stage) => {
          const active = event?.stage === stage.id;
          const done = progress > 0 && STAGES.findIndex((item) => item.id === event?.stage) > STAGES.findIndex((item) => item.id === stage.id);
          return (
            <li key={stage.id} className={active ? "text-cyan" : done ? "text-core/80" : ""}>
              {stage.label}
            </li>
          );
        })}
      </ul>
    </div>
  );
}
