"use client";

import { useMemo, useRef, useState } from "react";
import { Canvas } from "@react-three/fiber";
import type { Anomaly, ChunkDna, FileDna } from "@genoma/shared-types";
import { DnaOrganism } from "./DnaOrganism";
import { CameraRig } from "./CameraRig";
import { HoverCard } from "./HoverCard";
import { VizControls } from "./VizControls";
import { SelectionBridge } from "./SelectionBridge";
import { buildOrganism, clusterCenter, FILE_CAMERA_POSITION, FILE_CAMERA_TARGET } from "./buildOrganism";
import { DEFAULT_LAYERS, type VizLayers } from "./layers";

export function DnaCanvas({
  dna,
  anomalies = [],
  highlighted,
  onSelect,
  className,
  showControls = false,
}: {
  dna: FileDna;
  anomalies?: Anomaly[];
  highlighted?: number | null;
  onSelect?: (chunkIndex: number | null) => void;
  className?: string;
  showControls?: boolean;
}) {
  const root = useRef<HTMLDivElement>(null);
  const [layers, setLayers] = useState<VizLayers>(DEFAULT_LAYERS);
  const [resetToken, setResetToken] = useState(0);
  const [hover, setHover] = useState<{ chunk: ChunkDna; x: number; y: number } | null>(null);
  const model = useMemo(() => buildOrganism(dna, anomalies), [dna, anomalies]);
  const cameraTarget =
    highlighted == null ? FILE_CAMERA_TARGET : clusterCenter(model, highlighted);
  const level = highlighted == null ? "FILE" : "BLOCK";

  const handleReset = () => {
    onSelect?.(null);
    setHover(null);
    setResetToken((value) => value + 1);
  };

  return (
    <div ref={root} className={`relative ${className ?? "h-full w-full"}`}>
      <Canvas
        camera={{ position: FILE_CAMERA_POSITION, fov: 45 }}
        gl={{ antialias: true, alpha: true }}
        dpr={[1, 2]}
        onPointerMissed={() => setHover(null)}
      >
        <color attach="background" args={["#05060a"]} />
        <fog attach="fog" args={["#05060a", 6, 16]} />
        <ambientLight intensity={0.35} />
        <pointLight position={[3, 4, 2]} intensity={1.1} color="#7ee0f2" />
        <pointLight position={[-4, -2, -3]} intensity={0.7} color="#b48cff" />
        <DnaOrganism
          dna={dna}
          anomalies={anomalies}
          highlighted={highlighted}
          layers={layers}
          onSelect={(chunkIndex) => onSelect?.(chunkIndex)}
          onHover={(chunkIndex, pointer) => {
            if (chunkIndex == null || !pointer || !root.current) {
              setHover(null);
              return;
            }
            const chunk = model.clusters.find((item) => item.chunk.index === chunkIndex)?.chunk;
            const rect = root.current.getBoundingClientRect();
            if (!chunk) return;
            setHover({ chunk, x: pointer.x - rect.left, y: pointer.y - rect.top });
          }}
        />
        <CameraRig target={cameraTarget} resetToken={resetToken} interactive />
        <SelectionBridge dna={dna} onSelect={handleReset} />
      </Canvas>
      {hover ? <HoverCard chunk={hover.chunk} x={hover.x} y={hover.y} /> : null}
      {showControls ? (
        <VizControls layers={layers} onChange={setLayers} onReset={handleReset} level={level} />
      ) : null}
    </div>
  );
}
