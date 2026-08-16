"use client";

import { useEffect, useRef } from "react";
import { useFrame, useThree } from "@react-three/fiber";
import { OrbitControls } from "@react-three/drei";
import { Vector3 } from "three";
import { FILE_CAMERA_POSITION, FILE_CAMERA_TARGET } from "./buildOrganism";

const FOCUS_SECONDS = 0.6;

export function CameraRig({
  target,
  resetToken,
  interactive = true,
}: {
  target: [number, number, number];
  resetToken: number;
  interactive?: boolean;
}) {
  const controls = useRef<{ target: Vector3; update: () => void } | null>(null);
  const { camera } = useThree();
  const goalTarget = useRef(new Vector3(...FILE_CAMERA_TARGET));
  const goalPosition = useRef(new Vector3(...FILE_CAMERA_POSITION));

  useEffect(() => {
    goalTarget.current.set(...FILE_CAMERA_TARGET);
    goalPosition.current.set(...FILE_CAMERA_POSITION);
  }, [resetToken]);

  useEffect(() => {
    const isFile = target[0] === 0 && target[1] === 0 && target[2] === 0;
    goalTarget.current.set(...target);
    if (isFile) {
      goalPosition.current.set(...FILE_CAMERA_POSITION);
      return;
    }
    const offset = new Vector3(...FILE_CAMERA_POSITION).normalize().multiplyScalar(2.15);
    goalPosition.current.copy(new Vector3(...target)).add(offset);
  }, [target]);

  useFrame((_, delta) => {
    if (!controls.current) return;
    const alpha = 1 - Math.exp((-delta * 4) / FOCUS_SECONDS);
    camera.position.lerp(goalPosition.current, alpha);
    controls.current.target.lerp(goalTarget.current, alpha);
    controls.current.update();
  });

  return (
    <OrbitControls
      ref={(node) => {
        controls.current = node;
      }}
      enabled={interactive}
      enableDamping
      dampingFactor={0.08}
      minDistance={1.2}
      maxDistance={12}
      enablePan
    />
  );
}
