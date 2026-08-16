"use client";

import { useMemo, useState } from "react";
import { Canvas } from "@react-three/fiber";
import { Line, OrbitControls } from "@react-three/drei";
import type { GalaxyLink, GalaxyNode } from "@genoma/shared-types";
import { SEMANTIC_COLORS } from "@genoma/shared-types";
import { clusterColor } from "./galaxyColors";

function GalaxyScene({
  nodes,
  links,
  onHover,
  onSelect,
}: {
  nodes: GalaxyNode[];
  links: GalaxyLink[];
  onHover: (node: GalaxyNode | null) => void;
  onSelect: (id: string) => void;
}) {
  const byId = useMemo(() => new Map(nodes.map((node) => [node.id, node])), [nodes]);

  return (
    <>
      <ambientLight intensity={0.4} />
      <pointLight position={[4, 5, 3]} intensity={1.0} color={SEMANTIC_COLORS.cyan} />
      <pointLight position={[-5, -2, -4]} intensity={0.55} color={SEMANTIC_COLORS.purple} />
      {links.map((link) => {
        const from = byId.get(link.from);
        const to = byId.get(link.to);
        if (!from || !to) return null;
        return (
          <Line
            key={`${link.from}-${link.to}`}
            points={[from.position, to.position]}
            color={SEMANTIC_COLORS.white}
            transparent
            opacity={0.08 + link.strength * 0.22}
            lineWidth={1}
          />
        );
      })}
      {nodes.map((node) => (
        <mesh
          key={node.id}
          position={node.position}
          onClick={(event) => {
            event.stopPropagation();
            onSelect(node.id);
          }}
          onPointerOver={(event) => {
            event.stopPropagation();
            onHover(node);
            document.body.style.cursor = "pointer";
          }}
          onPointerOut={() => {
            onHover(null);
            document.body.style.cursor = "auto";
          }}
        >
          <sphereGeometry args={[0.08 + Math.min(0.12, node.entropy * 0.1), 16, 16]} />
          <meshStandardMaterial
            color={clusterColor(node.cluster_id)}
            emissive={clusterColor(node.cluster_id)}
            emissiveIntensity={0.22}
            roughness={0.35}
            metalness={0.15}
          />
        </mesh>
      ))}
      <OrbitControls enablePan enableZoom enableRotate makeDefault />
    </>
  );
}

export function GalaxyCanvas({
  nodes,
  links,
  onSelect,
  className,
}: {
  nodes: GalaxyNode[];
  links: GalaxyLink[];
  onSelect: (id: string) => void;
  className?: string;
}) {
  const [hover, setHover] = useState<GalaxyNode | null>(null);

  return (
    <div className={`relative ${className ?? "h-80 w-full"}`}>
      <Canvas camera={{ position: [0, 0.6, 4.2], fov: 45 }} dpr={[1, 2]}>
        <color attach="background" args={["#05060a"]} />
        <fog attach="fog" args={["#05060a", 8, 18]} />
        <GalaxyScene nodes={nodes} links={links} onHover={setHover} onSelect={onSelect} />
      </Canvas>
      {hover ? (
        <div className="pointer-events-none absolute left-4 top-4 rounded-xl border border-white/10 bg-[#080a10]/85 px-3 py-2 font-mono text-[11px] text-core/80 backdrop-blur">
          <p className="text-core">{hover.name}</p>
          <p className="mt-1 text-core/45">
            Cluster C{hover.cluster_id} · entropy {hover.entropy.toFixed(3)}
          </p>
        </div>
      ) : (
        <div className="pointer-events-none absolute left-4 top-4 font-mono text-[10px] uppercase tracking-[0.16em] text-core/35">
          Drag to orbit · click a node
        </div>
      )}
    </div>
  );
}
