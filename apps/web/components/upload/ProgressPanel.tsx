"use client";

import type { ProgressEvent, Stage } from "@genoma/shared-types";

const STAGES: { id: Stage; label: string }[] = [
  { id: "READING_FILE", label: "Reading file" },
  { id: "EXTRACTING_FEATURES", label: "Extracting features" },
  { id: "GENERATING_PIDNA", label: "Generating πDNA" },
  { id: "DETECTING_ANOMALIES", label: "Detecting anomalies" },
  { id: "BUILDING_VISUALIZATION", label: "Building visualization" },
];

function stageFill(stage: Stage, current: Stage | undefined, progress: number): number {
  if (!current) return 0;
  if (current === "COMPLETE") return 1;
  const order = ["QUEUED", ...STAGES.map((item) => item.id), "COMPLETE", "FAILED"] as Stage[];
  const currentIndex = order.indexOf(current);
  const index = order.indexOf(stage);
  if (current === "FAILED") return index <= currentIndex ? 1 : 0;
  if (index < currentIndex) return 1;
  if (index > currentIndex) return 0;
  return Math.min(1, Math.max(0, progress));
}

export function ProgressPanel({ event }: { event: ProgressEvent | null }) {
  const progress = event?.progress ?? 0;
  const bytes = event?.processed_bytes;
  const total = event?.total_bytes;

  return (
    <div className="panel w-full max-w-lg rounded-3xl p-8">
      <p className="font-mono text-[10px] tracking-[0.28em] text-cyan/70">PREPARING ANALYSIS</p>
      <p className="mt-3 text-xl">{event?.message ?? "Queued"}</p>
      {bytes != null ? (
        <p className="mt-2 font-mono text-[11px] text-core/45">
          {bytes.toLocaleString()}
          {total ? ` / ${total.toLocaleString()}` : ""} bytes
        </p>
      ) : null}
      <ul className="mt-6 space-y-3">
        {STAGES.map((stage) => {
          const fill = stageFill(stage.id, event?.stage, progress);
          const active = event?.stage === stage.id;
          return (
            <li key={stage.id}>
              <div className="mb-1 flex justify-between font-mono text-[11px]">
                <span className={active ? "text-cyan" : fill >= 1 ? "text-core/80" : "text-core/40"}>
                  {stage.label}
                </span>
                <span className="text-core/35">{Math.round(fill * 100)}%</span>
              </div>
              <div className="h-[3px] overflow-hidden rounded-full bg-white/10">
                <div
                  className="h-full bg-gradient-to-r from-cyan to-purple transition-[width] duration-300"
                  style={{ width: `${Math.round(fill * 100)}%` }}
                />
              </div>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
