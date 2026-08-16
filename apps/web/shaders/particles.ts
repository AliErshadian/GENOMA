// Optional GLSL hooks for a future GPU particle path.
// Phase 1 uses InstancedMesh; particle attributes still come from Digital DNA.

export const PARTICLE_VERTEX = /* glsl */ `
  varying vec3 vColor;
  void main() {
    vColor = instanceColor;
    vec4 mv = modelViewMatrix * instanceMatrix * vec4(position, 1.0);
    gl_Position = projectionMatrix * mv;
  }
`;

export const PARTICLE_FRAGMENT = /* glsl */ `
  varying vec3 vColor;
  void main() {
    gl_FragColor = vec4(vColor, 0.92);
  }
`;
