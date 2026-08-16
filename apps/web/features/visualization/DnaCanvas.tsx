"use client";

import { Canvas } from "@react-three/fiber";
import { OrbitControls, PerspectiveCamera } from "@react-three/drei";
import type { Anomaly, FileDna } from "@genoma/shared-types";
import { DnaOrganism } from "./DnaOrganism";
import { SelectionBridge } from "./SelectionBridge";

export function DnaCanvas({
  dna,
  anomalies = [],
  highlighted,
  onSelect,
  className,
}: {
  dna: FileDna;
  anomalies?: Anomaly[];
  highlighted?: number | null;
  onSelect?: (chunkIndex: number | null) => void;
  className?: string;
}) {
  return (
    <div className={className ?? "h-full w-full"}>
      <Canvas gl={{ antialias: true, alpha: true }} dpr={[1, 2]}>
        <color attach="background" args={["#05060a"]} />
        <fog attach="fog" args={["#05060a", 6, 16]} />
        <PerspectiveCamera makeDefault position={[0, 0.4, 4.6]} fov={42} />
        <ambientLight intensity={0.35} />
        <pointLight position={[3, 4, 2]} intensity={1.1} color="#7ee0f2" />
        <pointLight position={[-4, -2, -3]} intensity={0.7} color="#b48cff" />
        <DnaOrganism
          dna={dna}
          anomalies={anomalies}
          highlighted={highlighted}
          onSelect={onSelect}
        />
        <SelectionBridge dna={dna} onSelect={onSelect} />
        <OrbitControls
          enableDamping
          dampingFactor={0.08}
          minDistance={1.4}
          maxDistance={10}
          enablePan
        />
      </Canvas>
    </div>
  );
}
