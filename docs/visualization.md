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

Particles orbit on deterministic axes seeded from `hash01(chunk, i)` and `particle_velocity`. Anomaly chunks add radial distortion and a slow red pulse. Instance colors update on highlight/hover/anomaly pulse, not every frame. Particle budget is capped at 8000 with frustum culling.

Workspace overlays: hover inspect (block #, offset, size, entropy, complexity, repetition, π offset) and layer toggles (Particles / Links / Core / Anomalies). Turning Anomalies off hides red-distorted particles.

Galaxy mode is not implemented in this phase and is not faked.
