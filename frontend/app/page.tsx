"use client";

import { FormEvent, useMemo, useState } from "react";

type OutputFormat = "docx" | "markdown";

interface GenerateResponse {
  generationId: string;
  markdown: string;
  outputs: Partial<Record<OutputFormat, string>>;
  wordCount: number;
  createdAt: string;
}

export default function HomePage() {
  const [prompt, setPrompt] = useState(
    "Create a technical architecture proposal for a multi-tenant SaaS analytics platform.",
  );
  const [documentTitle, setDocumentTitle] = useState("Architecture Proposal");
  const [accentColor, setAccentColor] = useState("#1F4E79");
  const [fontFamily, setFontFamily] = useState("Calibri");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [response, setResponse] = useState<GenerateResponse | null>(null);

  const gatewayUrl = useMemo(
    () => process.env.NEXT_PUBLIC_GATEWAY_URL ?? "http://localhost:8080",
    [],
  );

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setIsLoading(true);

    try {
      const result = await fetch(`${gatewayUrl}/api/v1/generate`, {
        method: "POST",
        headers: {
          "content-type": "application/json",
          "x-api-key": "dgk_dev_local_key",
        },
        body: JSON.stringify({
          prompt,
          outputFormats: ["docx", "markdown"],
          style: {
            documentTitle,
            accentColor,
            fontFamily,
          },
        }),
      });

      if (!result.ok) {
        const body = (await result.json()) as { message?: string };
        throw new Error(body.message ?? "Generation failed");
      }

      const body = (await result.json()) as GenerateResponse;
      setResponse(body);
    } catch (submissionError) {
      const message = submissionError instanceof Error ? submissionError.message : "Request failed";
      setError(message);
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <main className="shell">
      <section className="hero">
        <h1>Velium DocGen Studio</h1>
        <p>
          Generate publication-ready output in Word and Markdown from one prompt. This starter UI
          is wired to the live Rust gateway and backend microservices.
        </p>
      </section>

      <section className="workspace">
        <form className="card" onSubmit={onSubmit}>
          <div className="field">
            <label htmlFor="prompt">Prompt</label>
            <textarea
              id="prompt"
              value={prompt}
              minLength={10}
              maxLength={8000}
              rows={8}
              onChange={(event) => setPrompt(event.target.value)}
              required
            />
          </div>

          <div className="row" style={{ marginTop: 12 }}>
            <div className="field">
              <label htmlFor="title">Document title</label>
              <input
                id="title"
                value={documentTitle}
                onChange={(event) => setDocumentTitle(event.target.value)}
                required
              />
            </div>
            <div className="field">
              <label htmlFor="accent">Accent color</label>
              <input
                id="accent"
                value={accentColor}
                pattern="^#[0-9A-Fa-f]{6}$"
                onChange={(event) => setAccentColor(event.target.value)}
                required
              />
            </div>
            <div className="field">
              <label htmlFor="font">Font family</label>
              <input
                id="font"
                value={fontFamily}
                onChange={(event) => setFontFamily(event.target.value)}
                required
              />
            </div>
          </div>

          <div className="actions" style={{ marginTop: 16 }}>
            <button type="submit" disabled={isLoading}>
              {isLoading ? "Generating..." : "Generate document"}
            </button>
          </div>
          {error ? <p className="error">{error}</p> : null}
        </form>

        {response ? (
          <article className="card">
            <h2 style={{ marginTop: 0 }}>Latest generation</h2>
            <p>
              Generation ID: {response.generationId}
              <br />
              Word count: {response.wordCount}
              <br />
              Created at: {new Date(response.createdAt).toLocaleString()}
            </p>
            <h3>Markdown preview</h3>
            <div className="result">{response.markdown}</div>
          </article>
        ) : null}
      </section>
    </main>
  );
}
