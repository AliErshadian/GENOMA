import type { Anomaly, ChunkDna, FileDna, Mutation } from "@genoma/shared-types";
import { colorForChunk, particleBudget } from "@/lib/mappings";

export interface Particle {
  position: [number, number, number];
  color: string;
  chunkIndex: number;
  speed: number;
  radius: number;
  orbitAxis: [number, number, number];
  orbitU: [number, number, number];
  orbitV: [number, number, number];
  orbitRadius: number;
  phase: number;
  anomaly: number;
  mutation: number;
}

export interface Cluster {
  chunk: ChunkDna;
  center: [number, number, number];
  color: string;
  anomaly: number;
  mutation: number;
}

export interface OrganismModel {
  particles: Particle[];
  clusters: Cluster[];
  links: Array<{
    from: number;
    to: number;
    a: [number, number, number];
    b: [number, number, number];
    strength: number;
  }>;
  rings: number;
  breathe: number;
  rotation: number;
  fileFocus: [number, number, number];
}

export const FILE_CAMERA_POSITION: [number, number, number] = [0, 0.4, 4.6];
export const FILE_CAMERA_TARGET: [number, number, number] = [0, 0, 0];

export function unitFromPi(values: number[], index: number): number {
  return values[index % values.length] ?? 0.5;
}

export function fibonacciDirection(i: number, n: number): [number, number, number] {
  const phi = Math.acos(1 - (2 * (i + 0.5)) / n);
  const theta = Math.PI * (1 + Math.sqrt(5)) * i;
  return [Math.sin(phi) * Math.cos(theta), Math.cos(phi), Math.sin(phi) * Math.sin(theta)];
}

export function hash01(a: number, b: number): number {
  let x = Math.imul(a + 1, 374761393) + Math.imul(b + 1, 668265263);
  x = Math.imul(x ^ (x >>> 13), 1274126177);
  return ((x ^ (x >>> 16)) >>> 0) / 4294967296;
}

function normalize(vector: [number, number, number]): [number, number, number] {
  const length = Math.hypot(vector[0], vector[1], vector[2]) || 1;
  return [vector[0] / length, vector[1] / length, vector[2] / length];
}

export function clusterCenter(
  model: OrganismModel,
  chunkIndex: number | null,
): [number, number, number] {
  if (chunkIndex == null) return FILE_CAMERA_TARGET;
  return (
    model.clusters.find((cluster) => cluster.chunk.index === chunkIndex)?.center ??
    FILE_CAMERA_TARGET
  );
}

export function buildOrganism(
  dna: FileDna,
  anomalies: Anomaly[] = [],
  mutations: Mutation[] = [],
): OrganismModel {
  const anomalyByChunk = new Map(anomalies.map((item) => [item.chunk_index, item.score]));
  const mutationByChunk = new Map(mutations.map((item) => [item.chunk_index, item.impact]));
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
      fileFocus: FILE_CAMERA_TARGET,
    };
  }

  const perCluster = Math.max(8, Math.floor(totalParticles / source.length));

  source.forEach((chunk, index) => {
    const anomaly = anomalyByChunk.get(chunk.index) ?? 0;
    const mutation = mutationByChunk.get(chunk.index) ?? 0;
    const dir = fibonacciDirection(index, source.length);
    const radius =
      0.35 +
      chunk.visual.radius * 0.85 +
      unitFromPi(chunk.pi_derived.values, 0) * 0.35 +
      anomaly * 0.22;
    const center: [number, number, number] = [
      dir[0] * radius,
      dir[1] * radius * 0.82,
      dir[2] * radius,
    ];
    const color = colorForChunk(chunk, anomaly);
    clusters.push({ chunk, center, color, anomaly, mutation });

    const count = Math.max(12, Math.round(perCluster * (0.45 + chunk.visual.density * 0.8)));
    for (let i = 0; i < count; i += 1) {
      const local = fibonacciDirection(i, count);
      const spread =
        (1.15 - chunk.visual.cluster_strength) * (0.18 + chunk.visual.noise * 0.35) *
        (1 + anomaly * 0.45);
      const jitter = (hash01(chunk.index, i) - 0.5) * chunk.visual.noise * 0.12;
      const distort = anomaly * (hash01(chunk.index, i + 17) - 0.5) * 0.16;
      const axis = normalize([
        hash01(chunk.index, i + 3) - 0.5,
        hash01(chunk.index, i + 5) - 0.5,
        unitFromPi(chunk.pi_derived.values, i) - 0.5,
      ]);
      const helper: [number, number, number] =
        Math.abs(axis[1]) > 0.9 ? [1, 0, 0] : [0, 1, 0];
      const orbitU = normalize(cross(axis, helper));
      const orbitV = normalize(cross(axis, orbitU));
      particles.push({
        position: [
          center[0] + local[0] * spread + jitter + distort,
          center[1] + local[1] * spread + distort * 0.4,
          center[2] + local[2] * spread - jitter,
        ],
        color,
        chunkIndex: chunk.index,
        speed: chunk.visual.particle_velocity,
        radius: 0.012 + chunk.visual.density * 0.018,
        orbitAxis: axis,
        orbitU,
        orbitV,
        orbitRadius: 0.012 + chunk.visual.particle_velocity * 0.045,
        phase: hash01(chunk.index, i + 9) * Math.PI * 2,
        anomaly,
        mutation,
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
        links.push({
          from: current.chunk.index,
          to: other.chunk.index,
          a: current.center,
          b: other.center,
          strength: 1 - bestDistance,
        });
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
    fileFocus: FILE_CAMERA_TARGET,
  };
}

function cross(a: [number, number, number], b: [number, number, number]): [number, number, number] {
  return [
    a[1] * b[2] - a[2] * b[1],
    a[2] * b[0] - a[0] * b[2],
    a[0] * b[1] - a[1] * b[0],
  ];
}

function l1(a: number[], b: number[]): number {
  const n = Math.min(a.length, b.length) || 1;
  let acc = 0;
  for (let i = 0; i < n; i += 1) {
    acc += Math.abs((a[i] ?? 0) - (b[i] ?? 0));
  }
  return acc / n;
}
