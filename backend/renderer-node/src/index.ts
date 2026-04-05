import cors from "cors";
import express, { type Request, type Response } from "express";
import helmet from "helmet";
import { z } from "zod";

import { renderOutputs } from "./render.js";

const app = express();
const port = Number(process.env.RENDERER_PORT ?? "3001");

const styleSchema = z
  .object({
    documentTitle: z.string().min(1).max(120).optional(),
    accentColor: z.string().regex(/^#[0-9A-Fa-f]{6}$/).optional(),
    fontFamily: z.string().min(2).max(40).optional(),
  })
  .strict();

const renderRequestSchema = z
  .object({
    markdown: z.string().min(1).max(50000),
    outputFormats: z.array(z.enum(["docx", "markdown"]))
      .min(1)
      .max(2),
    style: styleSchema.optional(),
  })
  .strict();

app.use(helmet());
app.use(cors());
app.use(express.json({ limit: "2mb" }));

app.get("/health", (_req: Request, res: Response) => {
  res.json({ status: "ok" });
});

app.post("/internal/render", async (req: Request, res: Response) => {
  const parsed = renderRequestSchema.safeParse(req.body);
  if (!parsed.success) {
    res.status(400).json({
      error: "invalid_request",
      message: "Request payload did not pass validation",
      details: parsed.error.flatten(),
    });
    return;
  }

  const result = await renderOutputs(
    parsed.data.markdown,
    parsed.data.outputFormats,
    parsed.data.style,
  );

  res.json(result);
});

app.listen(port, () => {
  process.stdout.write(`renderer listening on ${port}\n`);
});
