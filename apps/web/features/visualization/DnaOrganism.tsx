"use client";

import { useLayoutEffect, useMemo, useRef } from "react";
import { useFrame } from "@react-three/fiber";
import { Float, Line } from "@react-three/drei";
import type { InstancedMesh, Mesh } from "three";
import { Color, Object3D } from "three";
import type { Anomaly, FileDna } from "@genoma/shared-types";
import { SEMANTIC_COLORS } from "@genoma/shared-types";
import { buildOrganism } from "./buildOrganism";

const dummy = new Object3D();
const color = new Color();

export function DnaOrganism({
  dna,
  anomalies = [],
  highlighted,
  onSelect,
}: {
  dna: FileDna;
  anomalies?: Anomaly[];
  highlighted?: number | null;
  onSelect?: (chunkIndex: number | null) => void;
}) {
  const model = useMemo(() => buildOrganism(dna, anomalies), [dna, anomalies]);
  const mesh = useRef<InstancedMesh>(null);
  const core = useRef<Mesh>(null);

  useLayoutEffect(() => {
    if (!mesh.current) return;
    model.particles.forEach((particle, index) => {
      color.set(particle.color);
      mesh.current!.setColorAt(index, color);
    });
    if (mesh.current.instanceColor) {
      mesh.current.instanceColor.needsUpdate = true;
    }
  }, [model]);

  useFrame(({ clock }) => {
    const t = clock.getElapsedTime();
    const breathe = 1 + Math.sin(t * (0.35 + model.breathe)) * 0.018;
    if (core.current) {
      core.current.rotation.y = t * 0.05 + model.rotation * 0.05;
      core.current.scale.setScalar(breathe);
    }
    if (!mesh.current) return;
    model.particles.forEach((particle, index) => {
      const dim = highlighted != null && particle.chunkIndex !== highlighted ? 0.28 : 1;
      const pulse =
        highlighted === particle.chunkIndex ? 1 + Math.sin(t * 3.2) * 0.25 : 1;
      dummy.position.set(
        particle.position[0] * breathe,
        particle.position[1] * breathe,
        particle.position[2] * breathe,
      );
      dummy.rotation.set(0, t * particle.speed * 0.4, 0);
      dummy.scale.setScalar(particle.radius * 18 * pulse);
      dummy.updateMatrix();
      mesh.current!.setMatrixAt(index, dummy.matrix);
      color.set(particle.color).multiplyScalar(dim);
      mesh.current!.setColorAt(index, color);
    });
    mesh.current.instanceMatrix.needsUpdate = true;
    if (mesh.current.instanceColor) {
      mesh.current.instanceColor.needsUpdate = true;
    }
    mesh.current.rotation.y = t * 0.04;
  });

  return (
    <group>
      <Float speed={0.6} rotationIntensity={0.08} floatIntensity={0.12}>
        <mesh ref={core}>
          <icosahedronGeometry args={[0.22 + dna.visual.geometry_complexity * 0.18, 2]} />
          <meshStandardMaterial
            color={SEMANTIC_COLORS.white}
            emissive={SEMANTIC_COLORS.white}
            emissiveIntensity={0.18}
            roughness={0.35}
            metalness={0.2}
            transparent
            opacity={0.85}
          />
        </mesh>
        {Array.from({ length: model.rings }).map((_, index) => (
          <mesh key={index} rotation={[Math.PI / 2.4, index * 0.37, index * 0.21]}>
            <torusGeometry args={[0.42 + index * 0.11, 0.004, 8, 96]} />
            <meshBasicMaterial
              color={index % 2 === 0 ? SEMANTIC_COLORS.cyan : SEMANTIC_COLORS.purple}
              transparent
              opacity={0.22}
            />
          </mesh>
        ))}
      </Float>
      {model.links.map((link, index) => (
        <Line
          key={index}
          points={[link[0], link[1]]}
          color={SEMANTIC_COLORS.white}
          transparent
          opacity={0.08 + link[2] * 0.18}
          lineWidth={1}
        />
      ))}
      <instancedMesh
        ref={mesh}
        args={[undefined, undefined, Math.max(1, model.particles.length)]}
        frustumCulled={false}
        onClick={(event) => {
          event.stopPropagation();
          const id = event.instanceId;
          if (id == null) {
            onSelect?.(null);
            return;
          }
          onSelect?.(model.particles[id]?.chunkIndex ?? null);
        }}
      >
        <sphereGeometry args={[1, 8, 8]} />
        <meshStandardMaterial
          vertexColors
          roughness={0.28}
          metalness={0.12}
          emissive={SEMANTIC_COLORS.cyan}
          emissiveIntensity={0.08}
        />
      </instancedMesh>
    </group>
  );
}
