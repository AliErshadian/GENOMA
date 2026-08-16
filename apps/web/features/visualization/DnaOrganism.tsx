"use client";

import { useLayoutEffect, useMemo, useRef } from "react";
import { useFrame, type ThreeEvent } from "@react-three/fiber";
import { Float, Line } from "@react-three/drei";
import type { InstancedMesh, Mesh } from "three";
import { Color, Object3D } from "three";
import type { Anomaly, FileDna, Mutation } from "@genoma/shared-types";
import { SEMANTIC_COLORS } from "@genoma/shared-types";
import { buildOrganism } from "./buildOrganism";
import { DEFAULT_LAYERS, type VizLayers } from "./layers";

const dummy = new Object3D();
const color = new Color();
const pulse = new Color();

export function DnaOrganism({
  dna,
  anomalies = [],
  mutations = [],
  highlighted,
  layers = DEFAULT_LAYERS,
  onSelect,
  onHover,
}: {
  dna: FileDna;
  anomalies?: Anomaly[];
  mutations?: Mutation[];
  highlighted?: number | null;
  layers?: VizLayers;
  onSelect?: (chunkIndex: number | null) => void;
  onHover?: (chunkIndex: number | null, pointer?: { x: number; y: number }) => void;
}) {
  const model = useMemo(
    () => buildOrganism(dna, anomalies, mutations),
    [dna, anomalies, mutations],
  );
  const mesh = useRef<InstancedMesh>(null);
  const core = useRef<Mesh>(null);
  const colorKey = `${highlighted ?? ""}:${layers.anomalies}:${layers.mutations}`;
  const pulseFrame = useRef(0);

  const neighborChunks = useMemo(() => {
    const set = new Set<number>();
    if (highlighted == null) return set;
    set.add(highlighted);
    model.links.forEach((link) => {
      if (link.from === highlighted) set.add(link.to);
      if (link.to === highlighted) set.add(link.from);
    });
    return set;
  }, [highlighted, model]);

  const paintColors = (time = 0) => {
    if (!mesh.current) return;
    model.particles.forEach((particle, index) => {
      const isolated = highlighted != null && particle.chunkIndex !== highlighted;
      color.set(particle.color).multiplyScalar(isolated ? 0.28 : 1);
      if (particle.mutation > 0 && layers.mutations) {
        pulse.set(SEMANTIC_COLORS.orange);
        color.lerp(pulse, 0.22 + Math.sin(time * 1.8 + particle.phase) * 0.14);
      }
      if (particle.anomaly >= 0.35 && layers.anomalies) {
        pulse.set(SEMANTIC_COLORS.red);
        color.lerp(pulse, 0.25 + Math.sin(time * 1.4 + particle.phase) * 0.12);
      }
      mesh.current!.setColorAt(index, color);
    });
    if (mesh.current.instanceColor) {
      mesh.current.instanceColor.needsUpdate = true;
    }
  };

  useLayoutEffect(() => {
    paintColors(0);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [model, colorKey]);

  useFrame(({ clock }) => {
    const t = clock.getElapsedTime();
    const breathe = 1 + Math.sin(t * (0.35 + model.breathe)) * 0.018;
    if (core.current) {
      core.current.rotation.y = t * 0.05 + model.rotation * 0.05;
      core.current.scale.setScalar(breathe);
    }
    if (!mesh.current) return;
    model.particles.forEach((particle, index) => {
      const hideAnomaly = !layers.anomalies && particle.anomaly >= 0.35;
      const theta = particle.phase + t * particle.speed * 0.55;
      const cos = Math.cos(theta) * particle.orbitRadius;
      const sin = Math.sin(theta) * particle.orbitRadius;
      dummy.position.set(
        (particle.position[0] + particle.orbitU[0] * cos + particle.orbitV[0] * sin) * breathe,
        (particle.position[1] + particle.orbitU[1] * cos + particle.orbitV[1] * sin) * breathe,
        (particle.position[2] + particle.orbitU[2] * cos + particle.orbitV[2] * sin) * breathe,
      );
      dummy.scale.setScalar(hideAnomaly ? 0 : particle.radius * 18);
      dummy.updateMatrix();
      mesh.current!.setMatrixAt(index, dummy.matrix);
    });
    mesh.current.instanceMatrix.needsUpdate = true;
    mesh.current.rotation.y = t * 0.04;
    pulseFrame.current += 1;
    const needsPulse =
      model.particles.some((item) => item.anomaly >= 0.35) ||
      model.particles.some((item) => item.mutation > 0);
    if (pulseFrame.current % 8 === 0 && needsPulse) {
      paintColors(t);
    }
  });

  const pick = (event: ThreeEvent<MouseEvent>, kind: "click" | "hover") => {
    event.stopPropagation();
    const id = event.instanceId;
    const chunkIndex = id == null ? null : (model.particles[id]?.chunkIndex ?? null);
    if (kind === "click") onSelect?.(chunkIndex);
    else {
      onHover?.(chunkIndex, { x: event.nativeEvent.clientX, y: event.nativeEvent.clientY });
    }
  };

  return (
    <group>
      {layers.core ? (
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
      ) : null}
      {layers.links
        ? model.links.map((link) => {
            if (
              highlighted != null &&
              !neighborChunks.has(link.from) &&
              !neighborChunks.has(link.to)
            ) {
              return null;
            }
            return (
              <Line
                key={`${link.from}-${link.to}`}
                points={[link.a, link.b]}
                color={SEMANTIC_COLORS.white}
                transparent
                opacity={0.08 + link.strength * 0.18}
                lineWidth={1}
              />
            );
          })
        : null}
      {layers.particles ? (
        <instancedMesh
          ref={mesh}
          frustumCulled
          args={[undefined, undefined, Math.max(1, model.particles.length)]}
          onClick={(event) => pick(event, "click")}
          onPointerMove={(event) => pick(event, "hover")}
          onPointerOut={() => onHover?.(null)}
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
      ) : null}
    </group>
  );
}
