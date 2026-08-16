"use client";

import type { VizLayers } from "./layers";

export function VizControls({
  layers,
  onChange,
  onReset,
  level,
}: {
  layers: VizLayers;
  onChange: (layers: VizLayers) => void;
  onReset: () => void;
  level: "FILE" | "BLOCK";
}) {
  const toggle = (key: keyof VizLayers) => {
    onChange({ ...layers, [key]: !layers[key] });
  };

  return (
    <div className="pointer-events-auto absolute bottom-28 left-1/2 z-20 flex -translate-x-1/2 items-center gap-2 rounded-full border border-white/10 bg-[#080a10]/80 px-3 py-2 font-mono text-[10px] tracking-[0.14em] text-core/70 backdrop-blur">
      <span className="px-2 text-cyan/70">{level}</span>
      {(
        [
          ["particles", "Particles"],
          ["links", "Links"],
          ["core", "Core"],
          ["anomalies", "Anomalies"],
          ["mutations", "Mutations"],
        ] as const
      ).map(([key, label]) => (
        <button
          key={key}
          type="button"
          onClick={() => toggle(key)}
          className={`rounded-full px-2 py-1 ${layers[key] ? "text-core" : "text-core/30"}`}
        >
          {label}
        </button>
      ))}
      <button type="button" onClick={onReset} className="rounded-full px-2 py-1 text-cyan">
        Reset
      </button>
    </div>
  );
}
