"use client";

import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useEffect, useState } from "react";
import { WorkspaceChrome } from "@/components/layout/WorkspaceChrome";
import { DropZone } from "@/components/upload/DropZone";
import { ProgressPanel } from "@/components/upload/ProgressPanel";
import { startDemo, subscribeProgress, uploadFile } from "@/lib/api";
import type { ProgressEvent } from "@genoma/shared-types";

function AnalyzeInner() {
  const router = useRouter();
  const params = useSearchParams();
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState<ProgressEvent | null>(null);

  useEffect(() => {
    const demo = params.get("demo");
    if (demo) {
      void run(startDemo(demo));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [params]);

  async function run(job: Promise<{ id: string }>) {
    try {
      setError(null);
      const created = await job;
      const stop = subscribeProgress(created.id, (event) => {
        setProgress(event);
        if (event.stage === "COMPLETE") {
          router.push(`/analyze/${created.id}`);
        }
        if (event.stage === "FAILED") {
          setError(event.message);
        }
      });
      return () => stop();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Analysis failed");
    }
  }

  return (
    <WorkspaceChrome>
      <div className="flex h-full items-center justify-center px-6">
        <div className="w-full max-w-xl">
          {progress && progress.stage !== "FAILED" ? (
            <ProgressPanel event={progress} />
          ) : (
            <>
              <p className="mb-6 font-mono text-[11px] tracking-[0.28em] text-core/50">
                ANALYSIS
              </p>
              <DropZone onFile={(file) => void run(uploadFile(file))} />
              {error ? <p className="mt-4 font-mono text-xs text-anomaly">{error}</p> : null}
            </>
          )}
        </div>
      </div>
    </WorkspaceChrome>
  );
}

export default function AnalyzePage() {
  return (
    <Suspense fallback={<WorkspaceChrome><div /></WorkspaceChrome>}>
      <AnalyzeInner />
    </Suspense>
  );
}
