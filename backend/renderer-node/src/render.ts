import {
  Document,
  HeadingLevel,
  Packer,
  Paragraph,
  TextRun,
} from "docx";

export type OutputFormat = "docx" | "markdown";

export interface StyleConfig {
  documentTitle?: string;
  accentColor?: string;
  fontFamily?: string;
}

export interface RenderResult {
  outputs: Partial<Record<OutputFormat, string>>;
  wordCount: number;
}

export async function renderOutputs(
  markdown: string,
  outputFormats: OutputFormat[],
  style: StyleConfig = {},
): Promise<RenderResult> {
  const outputs: Partial<Record<OutputFormat, string>> = {};

  if (outputFormats.includes("markdown")) {
    outputs.markdown = Buffer.from(markdown, "utf8").toString("base64");
  }

  if (outputFormats.includes("docx")) {
    const buffer = await buildDocx(markdown, style);
    outputs.docx = buffer.toString("base64");
  }

  return {
    outputs,
    wordCount: countWords(markdown),
  };
}

async function buildDocx(markdown: string, style: StyleConfig): Promise<Buffer> {
  const paragraphs = markdownToParagraphs(markdown, style);
  const document = new Document({
    sections: [
      {
        children: paragraphs,
      },
    ],
  });

  return Packer.toBuffer(document);
}

function markdownToParagraphs(markdown: string, style: StyleConfig): Paragraph[] {
  const font = style.fontFamily ?? "Calibri";
  const accentColor = (style.accentColor ?? "#1F4E79").replace("#", "");
  const lines = markdown.split("\n");
  const result: Paragraph[] = [];

  for (const rawLine of lines) {
    const line = rawLine.trim();

    if (!line) {
      result.push(new Paragraph({ text: "" }));
      continue;
    }

    if (line.startsWith("# ")) {
      result.push(
        new Paragraph({
          heading: HeadingLevel.HEADING_1,
          children: [
            new TextRun({
              text: line.slice(2),
              bold: true,
              color: accentColor,
              font,
            }),
          ],
        }),
      );
      continue;
    }

    if (line.startsWith("## ")) {
      result.push(
        new Paragraph({
          heading: HeadingLevel.HEADING_2,
          children: [
            new TextRun({
              text: line.slice(3),
              bold: true,
              color: accentColor,
              font,
            }),
          ],
        }),
      );
      continue;
    }

    if (line.startsWith("- ")) {
      result.push(
        new Paragraph({
          bullet: { level: 0 },
          children: [
            new TextRun({
              text: line.slice(2),
              font,
            }),
          ],
        }),
      );
      continue;
    }

    result.push(
      new Paragraph({
        children: [new TextRun({ text: line, font })],
      }),
    );
  }

  return result;
}

function countWords(markdown: string): number {
  const tokens = markdown.match(/[A-Za-z0-9']+/g);
  return tokens ? tokens.length : 0;
}
