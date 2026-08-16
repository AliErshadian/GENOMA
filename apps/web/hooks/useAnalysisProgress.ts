"use client";

import { useEffect, useState } from "react";
import type { ProgressEvent } from "@genoma/shared-types";
import { subscribeProgress } from "@/lib/api";

export function useAnalysisProgress(id: string | null) {
  const [event, setEvent] = useState<ProgressEvent | null>(null);

  useEffect(() => {
    if (!id) return;
    return subscribeProgress(id, setEvent);
  }, [id]);

  return event;
}
