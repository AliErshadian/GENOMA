import type { Anomaly, ChunkDna, FileDna } from "@genoma/shared-types";
import { colorForChunk, particleBudget } from "@/lib/mappings";

export interface Particle {
  position: [number, number, number];
  color: string;
  chunkIndex: number;
  speed: number;
  radius: number;
}

export interface Cluster {
  chunk: ChunkDna;
  center: [number, number, number];
  color: string;
  anomaly: number;
}

export interface OrganismModel {
  particles: Particle[];
  clusters: Cluster[];
  links: Array<[[number, number, number], [number, number, number], number]>;
  rings: number;
  breathe: number;
  rotation: number;
}

function unitFromPi(values: number[], index: number): number {
  return values[index % values.length] ?? 0.5;
}

function fibonacciDirection(i: number, n: number): [number, number, number] {
  const phi = Math.acos(1 - (2 * (i + 0.5)) / n);
  const theta = Math.PI * (1 + Math.sqrt(5)) * i;
  return [Math.sin(phi) * Math.cos(theta), Math.cos(phi), Math.sin(phi) * Math.sin(theta)];
}

function hash01(a: number, b: number): number {
  let x = Math.imul(a + 1, 374761393) + Math.imul(b + 1, 668265263);
  x = Math.imul(x ^ (x >>> 13), 1274126177);
  return ((x ^ (x >>> 16)) >>> 0) / 4294967296;
}

export function buildOrganism(dna: FileDna, anomalies: Anomaly[] = []): OrganismModel {
  const anomalyByChunk = new Map(anomalies.map((item) => [item.chunk_index, item.score]));
  const chunks = dna.chunks.length > 0 ? dna.chunks : [];
  const source = chunks.length > 0 ? chunks : null;
  const totalParticles = particleBudget(dna.visual, Math.max(1, chunks.length));
  const clusters: Cluster[] = [];
  const particles: Particle[] = [];

  if (!source) {
    return {
      particles: [],
      clusters: [],
      links: [],
      rings: 3,
      breathe: dna.visual.orbital_speed,
      rotation: dna.visual.rotation,
    };
  }

  const perCluster = Math.max(8, Math.floor(totalParticles / source.length));

  source.forEach((chunk, index) => {
    const anomaly = anomalyByChunk.get(chunk.index) ?? 0;
    const dir = fibonacciDirection(index, source.length);
    const radius =
      0.35 +
      chunk.visual.radius * 0.85 +
      unitFromPi(chunk.pi_derived.values, 0) * 0.35;
    const center: [number, number, number] = [
      dir[0] * radius,
      dir[1] * radius * 0.82,
      dir[2] * radius,
    ];
    const color = colorForChunk(chunk, anomaly);
    clusters.push({ chunk, center, color, anomaly });

    const count = Math.max(12, Math.round(perCluster * (0.45 + chunk.visual.density * 0.8)));
    for (let i = 0; i < count; i += 1) {
      const local = fibonacciDirection(i, count);
      const spread = (1.15 - chunk.visual.cluster_strength) * (0.18 + chunk.visual.noise * 0.35);
      const jitter = (hash01(chunk.index, i) - 0.5) * chunk.visual.noise * 0.12;
      particles.push({
        position: [
          center[0] + local[0] * spread + jitter,
          center[1] + local[1] * spread,
          center[2] + local[2] * spread - jitter,
        ],
        color,
        chunkIndex: chunk.index,
        speed: chunk.visual.particle_velocity,
        radius: 0.012 + chunk.visual.density * 0.018,
      });
    }
  });

  const links: OrganismModel["links"] = [];
  for (let i = 0; i < clusters.length; i += 1) {
    const current = clusters[i];
    if (!current) continue;
    let best = -1;
    let bestDistance = Number.POSITIVE_INFINITY;
    for (let j = 0; j < clusters.length; j += 1) {
      if (i === j) continue;
      const other = clusters[j];
      if (!other) continue;
      const similarity = l1(current.chunk.raw.values, other.chunk.raw.values);
      if (similarity < bestDistance) {
        bestDistance = similarity;
        best = j;
      }
    }
    if (best >= 0 && bestDistance < 0.35) {
      const other = clusters[best];
      if (other) {
        links.push([current.center, other.center, 1 - bestDistance]);
      }
    }
  }

  return {
    particles: particles.slice(0, 8000),
    clusters,
    links,
    rings: Math.round(3 + dna.visual.branching * 6),
    breathe: dna.visual.orbital_speed,
    rotation: dna.visual.rotation,
  };
}

function l1(a: number[], b: number[]): number {
  const n = Math.min(a.length, b.length) || 1;
  let acc = 0;
  for (let i = 0; i < n; i += 1) {
    acc += Math.abs((a[i] ?? 0) - (b[i] ?? 0));
  }
  return acc / n;
}
