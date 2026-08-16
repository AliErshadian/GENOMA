"use client";

import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useEffect, useState } from "react";
import { WorkspaceChrome } from "@/components/layout/WorkspaceChrome";
import { DropZone } from "@/components/upload/DropZone";
import { listDemos, startDemo, uploadFile } from "@/lib/api";

const CHUNK_SIZES = [
  { label: "4 KB", value: 4 * 1024 },
  { label: "16 KB", value: 16 * 1024 },
  { label: "64 KB", value: 64 * 1024 },
  { label: "1 MB", value: 1024 * 1024 },
  { label: "4 MB", value: 4 * 1024 * 1024 },
];

function AnalyzeInner() {
  const router = useRouter();
  const params = useSearchParams();
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [demos, setDemos] = useState<string[]>([]);
  const [chunkSize, setChunkSize] = useState(1024 * 1024);
  const [level, setLevel] = useState("BALANCED");

  useEffect(() => {
    void listDemos()
      .then(setDemos)
      .catch(() => setDemos([]));
  }, []);

  useEffect(() => {
    const demo = params.get("demo");
    if (demo) {
      void run(() => startDemo(demo, { chunkSize, level }));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [params]);

  async function run(job: () => Promise<{ id: string }>) {
    try {
      setBusy(true);
      setError(null);
      const created = await job();
      router.push(`/analyze/${created.id}`);
    } catch (err) {
      setBusy(false);
      setError(err instanceof Error ? err.message : "Analysis failed");
    }
  }

  return (
    <WorkspaceChrome>
      <div className="flex h-full items-center justify-center px-6">
        <div className="w-full max-w-xl">
          <p className="mb-6 font-mono text-[11px] tracking-[0.28em] text-core/50">ANALYSIS</p>
          <DropZone onFile={(file) => void run(() => uploadFile(file, { chunkSize, level }))} />
          <div className="mt-4 flex flex-wrap gap-3 font-mono text-[11px] text-core/55">
            <label className="flex items-center gap-2">
              Chunk
              <select
                className="rounded-md bg-white/5 px-2 py-1 text-core"
                value={chunkSize}
                onChange={(event) => setChunkSize(Number(event.target.value))}
              >
                {CHUNK_SIZES.map((item) => (
                  <option key={item.value} value={item.value}>
                    {item.label}
                  </option>
                ))}
              </select>
            </label>
            <label className="flex items-center gap-2">
              Level
              <select
                className="rounded-md bg-white/5 px-2 py-1 text-core"
                value={level}
                onChange={(event) => setLevel(event.target.value)}
              >
                <option value="FAST">FAST</option>
                <option value="BALANCED">BALANCED</option>
                <option value="DEEP">DEEP</option>
              </select>
            </label>
          </div>
          {demos.length > 0 ? (
            <div className="mt-6">
              <p className="mb-2 font-mono text-[10px] tracking-[0.18em] text-core/35">DEMOS</p>
              <div className="flex flex-wrap gap-2">
                {demos.map((name) => (
                  <button
                    key={name}
                    type="button"
                    disabled={busy}
                    onClick={() => void run(() => startDemo(name, { chunkSize, level }))}
                    className="rounded-full border border-white/10 px-3 py-1 font-mono text-[11px] text-core/70 hover:text-core"
                  >
                    {name}
                  </button>
                ))}
              </div>
            </div>
          ) : null}
          {error ? <p className="mt-4 font-mono text-xs text-anomaly">{error}</p> : null}
          {busy ? <p className="mt-4 font-mono text-xs text-cyan">Starting analysis…</p> : null}
        </div>
      </div>
    </WorkspaceChrome>
  );
}

export default function AnalyzePage() {
  return (
    <Suspense
      fallback={
        <WorkspaceChrome>
          <div />
        </WorkspaceChrome>
      }
    >
      <AnalyzeInner />
    </Suspense>
  );
}
