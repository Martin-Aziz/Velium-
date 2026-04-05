export type OutputFormat = "docx" | "markdown";

export interface StyleConfig {
  documentTitle?: string;
  accentColor?: string;
  fontFamily?: string;
}

export interface GenerateRequest {
  prompt: string;
  outputFormats: OutputFormat[];
  style?: StyleConfig;
}

export interface GenerateResponse {
  generationId: string;
  markdown: string;
  outputs: Partial<Record<OutputFormat, string>>;
  wordCount: number;
  createdAt: string;
}
