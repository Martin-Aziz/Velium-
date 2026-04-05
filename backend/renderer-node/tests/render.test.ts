import { describe, expect, it } from "vitest";

import { renderOutputs } from "../src/render.js";

describe("renderOutputs", () => {
  it("returns markdown output as base64", async () => {
    const result = await renderOutputs("# Title\n\nHello world", ["markdown"]);
    expect(result.outputs.markdown).toBeDefined();
    expect(result.wordCount).toBeGreaterThan(0);
  });

  it("returns docx output as base64", async () => {
    const result = await renderOutputs("# Title\n\n- One\n- Two", ["docx"]);
    expect(result.outputs.docx).toBeDefined();
    expect((result.outputs.docx ?? "").length).toBeGreaterThan(100);
  });
});
