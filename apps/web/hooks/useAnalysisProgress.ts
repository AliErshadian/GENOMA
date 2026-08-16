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

    const apply = (next: ProgressEvent) => {
      if (!cancelled) setEvent(next);
    };

    void getProgress(id).then(apply).catch(() => undefined);

    source = new EventSource(`/api/v1/analyses/${id}/progress`);
    source.onmessage = (message) => {
      try {
        apply(JSON.parse(message.data) as ProgressEvent);
      } catch {
        /* ignore malformed frames */
      }
    };

    timer = window.setInterval(() => {
      if (isTerminal(eventRef.current?.stage)) return;
      void getProgress(id).then(apply).catch(() => undefined);
    }, 1000);

    return () => {
      cancelled = true;
      source?.close();
      if (timer) window.clearInterval(timer);
    };
  }, [id]);

  return event;
}
