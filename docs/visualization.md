# Visualization mappings

Every visual property is a deterministic function of Digital DNA. The renderer does not call `Math.random()`.

| Structural property | Visual channel | Semantic color |
| --- | --- | --- |
| Entropy | Particle density, cyan → purple mix | Cyan = stable/low entropy, purple = high entropy |
| Complexity | Geometry complexity, branching, radius | — |
| Repetition | Cluster tightness | Blue |
| Byte diversity | Color variation through the entropy mix | — |
| Bit transition rate | Particle velocity, orbital speed | — |
| π-derived vector | Cluster orientation and global rotation | — |
| Mutation (library) | Orange pulse | Orange |
| Anomaly score | Red highlight / distortion | Red |
| Structural core | Central icosahedron + rings | White |

Particle positions: Fibonacci sphere per chunk, radius from visual DNA, pull-to-centroid from repetition (`cluster_strength`). Links connect nearest neighbors in raw feature space.

Hierarchical zoom has two levels (byte-region descent is later):

- **FILE:** default camera pose `[0, 0.4, 4.6]` looking at the origin; all clusters visible.
- **BLOCK:** camera lerps (~0.6s) to the selected cluster center; other particles dim; non-neighbor links hide. Selecting another block interpolates to that cluster. Reset (button or double-click empty space) returns to FILE.

Particles orbit on deterministic axes seeded from `hash01(chunk, i)` and `particle_velocity`. Anomaly chunks add radial distortion and a slow red pulse. Mutation chunks (when a baseline comparison is loaded) add an orange pulse. Instance colors update on highlight/hover/anomaly/mutation pulse, not every frame. Particle budget is capped at 8000 with frustum culling.

Workspace overlays: hover inspect (block #, offset, size, entropy, complexity, repetition, π offset), mutation baseline picker, anomaly list (score / chunk / offset / entropy z — click selects BLOCK zoom), and layer toggles (Particles / Links / Core / Anomalies / Mutations). Turning Anomalies off hides red-distorted particles and dims the anomaly list. Mutations layer toggles the orange pulse only.

## Galaxy mode

`/analyze/galaxy` embeds up to 50 completed analyses in a shared 3D space:

- Distance: `1 - compare_dna.overall`
- Clusters: average-linkage agglomerative cut at 0.35
- Layout: classical MDS → 3D (deterministic Jacobi eigendecomposition; no `Math.random()`)
- Links: pairs with similarity ≥ 0.65
- Interaction: orbit camera; click a node to open that analysis organism

Node colors follow a fixed cluster palette derived from semantic colors.

## Evolution timeline

`/analyze/evolution` builds a linear series of completed analyses (version labels) and compares adjacent snapshots with existing similarity / mutation endpoints. Selecting a timeline node pairs it with the next version; “Open organism” jumps to that analysis’s 3D workspace.

Git import (`POST /api/v1/evolution/git`) only reads repositories under `data/repos/` (default `GENOMA_GIT_REPOS_DIR`). Seed the bundled demo with `scripts/seed-demo-evolve.sh`, then use **Import Git demo** to analyze `demo-evolve`’s `sample.txt` history into a timeline series.
