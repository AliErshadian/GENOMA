"use client";

import { useEffect, useRef, useState } from "react";
import type { ProgressEvent, Stage } from "@genoma/shared-types";
import { getProgress } from "@/lib/api";

function isTerminal(stage: Stage | undefined): boolean {
  return stage === "COMPLETE" || stage === "FAILED";
}

export function useAnalysisProgress(id: string | null) {
  const [event, setEvent] = useState<ProgressEvent | null>(null);
  const eventRef = useRef(event);
  eventRef.current = event;

  useEffect(() => {
    if (!id) return;
    let cancelled = false;
    let source: EventSource | null = null;
    let timer: number | undefined;
    let sseAlive = false;
    let stopped = false;

    setEvent(null);

    const stop = () => {
      if (stopped) return;
      stopped = true;
      source?.close();
      source = null;
      if (timer !== undefined) {
        window.clearInterval(timer);
        timer = undefined;
      }
    };

    const apply = (next: ProgressEvent) => {
      if (cancelled) return;
      setEvent(next);
      if (isTerminal(next.stage)) stop();
    };

    source = new EventSource(`/api/v1/analyses/${id}/progress`);
    source.onmessage = (message) => {
      sseAlive = true;
      try {
        apply(JSON.parse(message.data) as ProgressEvent);
      } catch {
        /* ignore malformed frames */
      }
    };
    source.onerror = () => {
      sseAlive = false;
    };

    void getProgress(id).then(apply).catch(() => undefined);

    timer = window.setInterval(() => {
      if (stopped || isTerminal(eventRef.current?.stage) || sseAlive) return;
      void getProgress(id).then(apply).catch(() => undefined);
    }, 4000);

    return () => {
      cancelled = true;
      stop();
    };
  }, [id]);

  return event;
}
