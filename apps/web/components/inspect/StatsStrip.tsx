"use client";

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="min-w-[110px]">
      <p className="font-mono text-[9px] uppercase tracking-[0.18em] text-core/35">{label}</p>
      <p className="mt-1 font-mono text-sm text-core/90">{value}</p>
    </div>
  );
}

export function StatsStrip({
  entropy,
  complexity,
  repetition,
  mutations,
  anomalies,
  piOffset,
}: {
  entropy?: number;
  complexity?: number;
  repetition?: number;
  mutations?: number | null;
  anomalies?: number;
  piOffset?: number;
}) {
  return (
    <div className="panel mx-auto flex max-w-4xl flex-wrap items-center justify-between gap-4 rounded-2xl px-6 py-4">
      <Stat label="Entropy" value={entropy == null ? "—" : entropy.toFixed(4)} />
      <Stat
        label="Complexity"
        value={complexity == null ? "—" : `${(complexity * 100).toFixed(1)}%`}
      />
      <Stat
        label="Patterns"
        value={repetition == null ? "—" : `${(repetition * 100).toFixed(2)}%`}
      />
      <Stat label="Mutations" value={mutations == null ? "—" : String(mutations)} />
      <Stat label="Anomalies" value={anomalies == null ? "—" : String(anomalies)} />
      <Stat label="π Offset" value={piOffset == null ? "—" : piOffset.toLocaleString()} />
    </div>
  );
}
