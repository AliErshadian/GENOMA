import { describe, expect, it } from "vitest";
import type { ChunkDna } from "@genoma/shared-types";
import { colorForChunk, particleBudget } from "./mappings";

function chunk(entropy: number, repetition: number): ChunkDna {
  return {
    index: 0,
    offset: 0,
    size: 1024,
    raw: {
      entropy,
      complexity: entropy,
      repetition,
      bit_transition: 0.4,
      compression: 0.5,
      diversity: 0.5,
      values: Array(16).fill(entropy),
    },
    pi_derived: {
      values: Array(16).fill(0.5),
      pi_offset: 0,
      pi_wrapped: false,
      pi_wrap_count: 0,
      generator_version: "dna-v1",
    },
    visual: {
      density: entropy,
      radius: 1,
      rotation: 0,
      branching: 0.5,
      particle_count: 400,
      particle_velocity: 0.1,
      cluster_strength: repetition,
      noise: 0.1,
      orbital_speed: 0.1,
      geometry_complexity: 0.5,
      hue_mix: entropy,
      repetition_tint: repetition,
    },
  };
}

describe("visual mappings", () => {
  it("uses cyan for low entropy and purple for high entropy", () => {
    const low = colorForChunk(chunk(0.05, 0));
    const high = colorForChunk(chunk(0.95, 0));
    expect(low).not.toEqual(high);
  });

  it("caps particle count for large files", () => {
    expect(particleBudget(chunk(0.9, 0.1).visual, 400)).toBeLessThanOrEqual(8000);
  });

  it("is a pure function of the fingerprint", () => {
    const a = colorForChunk(chunk(0.4, 0.2));
    const b = colorForChunk(chunk(0.4, 0.2));
    expect(a).toEqual(b);
  });
});
