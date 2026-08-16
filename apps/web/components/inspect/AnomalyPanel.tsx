"use client";

import type { Anomaly } from "@genoma/shared-types";

export function AnomalyPanel({
  anomalies,
  selected,
  onSelect,
  enabled = true,
}: {
  anomalies: Anomaly[];
  selected?: number | null;
  onSelect?: (chunkIndex: number) => void;
  enabled?: boolean;
}) {
  return (
    <div
      className={`panel rounded-2xl p-4 ${enabled ? "" : "opacity-40"}`}
      aria-disabled={!enabled}
    >
      <div className="flex items-baseline justify-between gap-3">
        <p className="font-mono text-[10px] uppercase tracking-[0.22em] text-anomaly/80">
          Anomalies
        </p>
        <span className="font-mono text-[10px] text-core/40">{anomalies.length}</span>
      </div>
      {!enabled ? (
        <p className="mt-3 font-mono text-[10px] text-core/40">
          Anomalies layer is off
        </p>
      ) : anomalies.length === 0 ? (
        <p className="mt-3 font-mono text-[10px] text-core/40">No statistical outliers</p>
      ) : (
        <ul className="mt-3 max-h-48 space-y-1 overflow-y-auto">
          {anomalies.map((item) => {
            const active = selected === item.chunk_index;
            return (
              <li key={`${item.chunk_index}-${item.offset}`}>
                <button
                  type="button"
                  disabled={!enabled}
                  onClick={() => onSelect?.(item.chunk_index)}
                  className={`flex w-full items-baseline justify-between gap-3 rounded-lg px-2 py-1.5 text-left font-mono text-[11px] transition ${
                    active
                      ? "bg-anomaly/15 text-core"
                      : "text-core/65 hover:bg-white/5 hover:text-core"
                  }`}
                >
                  <span>
                    #{item.chunk_index}
                    <span className="ml-2 text-core/35">
                      @{item.offset.toLocaleString()}
                    </span>
                  </span>
                  <span className="text-anomaly">
                    {(item.score * 100).toFixed(0)}%
                    <span className="ml-2 text-core/35">z{item.entropy_z.toFixed(1)}</span>
                  </span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
