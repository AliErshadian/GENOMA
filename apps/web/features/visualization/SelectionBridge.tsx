"use client";

import { useThree } from "@react-three/fiber";
import { useEffect } from "react";
import type { FileDna } from "@genoma/shared-types";

export function SelectionBridge({
  dna,
  onSelect,
}: {
  dna: FileDna;
  onSelect?: (chunkIndex: number | null) => void;
}) {
  const { gl } = useThree();

  useEffect(() => {
    const el = gl.domElement;
    const onDblClick = () => onSelect?.(null);
    el.addEventListener("dblclick", onDblClick);
    return () => el.removeEventListener("dblclick", onDblClick);
  }, [gl, onSelect, dna.chunk_count]);

  return null;
}
