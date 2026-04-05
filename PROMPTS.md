# Prompt Engineering Strategy

## System prompt architecture

- Role: technical document author with consistent professional structure.
- Constraints: produce valid markdown with heading hierarchy and concise sections.
- Safety: no untrusted HTML output and no executable code snippets by default.

## Context composition

1. Style context (title, accent, tone).
2. Domain intent from user prompt.
3. Structural requirements (summary, analysis, recommendations).

## Iteration log

1. Initial fallback prompt generator added for local deterministic runs.
2. Structured section-first markdown format added to stabilize renderer output.
3. Next iteration planned: provider abstraction for Claude and fallback chain policies.
