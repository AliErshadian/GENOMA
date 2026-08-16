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

Hierarchical zoom in Phase 1: selecting a cluster isolates it (others dim). Camera orbit/pan/zoom via OrbitControls. Deeper byte-region descent is later.

Galaxy mode is not implemented in this phase and is not faked.
