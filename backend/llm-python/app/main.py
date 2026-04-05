from __future__ import annotations

import asyncio
import json
import re
from typing import Optional

from fastapi import FastAPI
from fastapi.responses import StreamingResponse
from pydantic import BaseModel, Field


class StyleConfig(BaseModel):
    documentTitle: Optional[str] = Field(
        default=None,
        min_length=1,
        max_length=120,
    )
    accentColor: Optional[str] = Field(
        default=None,
        pattern=r"^#[0-9A-Fa-f]{6}$",
    )
    fontFamily: Optional[str] = Field(
        default=None,
        min_length=2,
        max_length=40,
    )


class GenerateMarkdownRequest(BaseModel):
    prompt: str = Field(min_length=10, max_length=8000)
    style: Optional[StyleConfig] = None


class GenerateMarkdownResponse(BaseModel):
    markdown: str


app = FastAPI(title="Velium LLM Service", version="0.1.0")


@app.get("/health")
def health() -> dict[str, str]:
    return {"status": "ok"}


@app.post(
    "/internal/generate-markdown",
    response_model=GenerateMarkdownResponse,
)
async def generate_markdown(
    payload: GenerateMarkdownRequest,
) -> GenerateMarkdownResponse:
    title = payload.style.documentTitle if payload.style else None
    markdown = build_markdown(payload.prompt, title)
    return GenerateMarkdownResponse(markdown=markdown)


@app.post("/internal/generate-markdown/stream")
async def generate_markdown_stream(
    payload: GenerateMarkdownRequest,
) -> StreamingResponse:
    title = payload.style.documentTitle if payload.style else None
    markdown = build_markdown(payload.prompt, title)

    async def stream() -> asyncio.StreamReader:
        for line in markdown.splitlines():
            event = json.dumps({"chunk": f"{line}\n"})
            yield f"data: {event}\n\n"
            await asyncio.sleep(0)
        yield "event: done\ndata: {\"status\":\"complete\"}\n\n"

    return StreamingResponse(stream(), media_type="text/event-stream")


def build_markdown(prompt: str, explicit_title: Optional[str] = None) -> str:
    title = explicit_title or derive_title(prompt)
    normalized_prompt = normalize_whitespace(prompt)

    sections = [
        "## Executive Summary",
        (
            "This document was generated from the provided request "
            "and focuses on "
            f"the core objective: {normalized_prompt}."
        ),
        "## Key Insights",
        "- Define the desired business outcome and measurable success "
        "criteria.",
        "- Identify dependencies, risks, and mitigation strategies "
        "before execution.",
        "- Align implementation work with a phased rollout and ownership "
        "model.",
        "## Recommended Next Actions",
        "1. Validate assumptions with key stakeholders and update "
        "acceptance criteria.",
        "2. Implement a thin vertical slice and collect operational feedback.",
        "3. Expand scope iteratively with quality gates for security and "
        "reliability.",
    ]

    return "\n\n".join([f"# {title}", *sections]).strip() + "\n"


def derive_title(prompt: str) -> str:
    words = re.findall(r"[A-Za-z0-9']+", prompt)[:8]
    if not words:
        return "Generated Document"
    candidate = " ".join(words).strip()
    return candidate.title()


def normalize_whitespace(value: str) -> str:
    return re.sub(r"\s+", " ", value).strip()
