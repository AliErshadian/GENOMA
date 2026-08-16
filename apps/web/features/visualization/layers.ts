export type VizLayers = {
  particles: boolean;
  links: boolean;
  core: boolean;
  anomalies: boolean;
};

export const DEFAULT_LAYERS: VizLayers = {
  particles: true,
  links: true,
  core: true,
  anomalies: true,
};
