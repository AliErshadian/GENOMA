import { describe, expect, it } from "vitest";
import { clusterColor } from "./galaxyColors";

describe("clusterColor", () => {
  it("is deterministic and cycles the palette", () => {
    expect(clusterColor(0)).toBe(clusterColor(8));
    expect(clusterColor(0)).not.toBe(clusterColor(1));
  });
});
