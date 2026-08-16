import { describe, expect, it } from "vitest";
import type { FileDna } from "@genoma/shared-types";
import { buildOrganism } from "./buildOrganism";

function sampleDna(): FileDna {
  const values = Array.from({ length: 16 }, (_, i) => 0.1 + i * 0.04);
  const chunk = {
    index: 0,
    offset: 0,
    size: 1024,
    raw: {
      entropy: 0.4,
      complexity: 0.5,
      repetition: 0.2,
      bit_transition: 0.33,
      compression: 0.6,
      diversity: 0.4,
      values,
    },
    pi_derived: {
      values: values.map((value) => 1 - value),
      pi_offset: 64,
      pi_wrapped: false,
      pi_wrap_count: 0,
      generator_version: "dna-v1",
    },
    visual: {
      density: 0.4,
      radius: 1.1,
      rotation: 0.2,
      branching: 0.4,
      particle_count: 400,
      particle_velocity: 0.2,
      cluster_strength: 0.3,
      noise: 0.15,
      orbital_speed: 0.1,
      geometry_complexity: 0.4,
      hue_mix: 0.4,
      repetition_tint: 0.2,
    },
  };
  return {
    generator_version: "dna-v1",
    pi_base_offset: 0,
    chunk_count: 1,
    total_bytes: 1024,
    raw: chunk.raw,
    pi_derived: chunk.pi_derived,
    visual: chunk.visual,
    chunks: [chunk, { ...chunk, index: 1, offset: 1024 }],
  };
}

describe("buildOrganism", () => {
  it("is deterministic for the same DNA", () => {
    const dna = sampleDna();
    const a = buildOrganism(dna, [{ chunk_index: 1, offset: 1024, score: 0.8, entropy_z: 2, neighbor_distance: 0.4 }]);
    const b = buildOrganism(dna, [{ chunk_index: 1, offset: 1024, score: 0.8, entropy_z: 2, neighbor_distance: 0.4 }]);
    expect(a.particles).toEqual(b.particles);
    expect(a.clusters.map((cluster) => cluster.center)).toEqual(
      b.clusters.map((cluster) => cluster.center),
    );
  });

  it("assigns orbit seeds and anomaly distortion from data", () => {
    const dna = sampleDna();
    const model = buildOrganism(dna, [
      { chunk_index: 1, offset: 1024, score: 0.9, entropy_z: 3, neighbor_distance: 0.5 },
    ]);
    expect(model.particles.length).toBeGreaterThan(0);
    expect(model.particles.every((particle) => particle.orbitRadius > 0)).toBe(true);
    const anomalous = model.particles.filter((particle) => particle.chunkIndex === 1);
    expect(anomalous.some((particle) => particle.anomaly >= 0.35)).toBe(true);
    expect(model.fileFocus).toEqual([0, 0, 0]);
  });
});
