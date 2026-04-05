# API Documentation

## Authentication

Generation and history endpoints require `x-api-key` header. Health endpoint is unauthenticated.

## Endpoints

### POST /api/v1/generate

Generates markdown and requested output formats.

Request body:

```json
{
  "prompt": "Create an executive summary about Q2 growth",
  "outputFormats": ["docx", "markdown"],
  "style": {
    "documentTitle": "Q2 Executive Summary",
    "accentColor": "#1F4E79",
    "fontFamily": "Calibri"
  }
}
```

Response `200`:

```json
{
  "generationId": "8f652bb6-f94c-4f7e-a693-ddf5119bc5ec",
  "markdown": "# Q2 Executive Summary\\n...",
  "outputs": {
    "docx": "<base64>",
    "markdown": "<base64>"
  },
  "wordCount": 123,
  "createdAt": "2026-04-05T00:00:00Z"
}
```

Response `401`:

```json
{
  "error": "unauthorized",
  "message": "Invalid API key"
}
```

Response `429`:

```json
{
  "error": "rate_limited",
  "message": "Rate limit exceeded"
}
```

### GET /health

Gateway health endpoint with dependency checks.

Response `200`:

```json
{
  "status": "ok",
  "dependencies": {
    "llm": "up",
    "renderer": "up",
    "database": "up"
  }
}
```

### GET /api/v1/generations

Returns persisted generation history scoped to the API key.

Query params:

- `limit` optional integer between `1` and `100` (defaults to `20`).

Response `200`:

```json
{
  "items": [
    {
      "generationId": "8f652bb6-f94c-4f7e-a693-ddf5119bc5ec",
      "prompt": "Create an executive summary about Q2 growth",
      "outputFormats": ["docx", "markdown"],
      "wordCount": 123,
      "createdAt": "2026-04-05T00:00:00Z"
    }
  ]
}
```

Response `400`:

```json
{
  "error": "invalid_limit",
  "message": "Query parameter 'limit' must be between 1 and 100"
}
```

### GET /api/v1/generations/:generationId

Returns one persisted generation record for the authenticated API key.

Response `200`:

```json
{
  "generationId": "8f652bb6-f94c-4f7e-a693-ddf5119bc5ec",
  "prompt": "Create an executive summary about Q2 growth",
  "markdown": "# Q2 Executive Summary\n...",
  "outputs": {
    "docx": "<base64>",
    "markdown": "<base64>"
  },
  "outputFormats": ["docx", "markdown"],
  "style": {
    "documentTitle": "Q2 Executive Summary",
    "accentColor": "#1F4E79",
    "fontFamily": "Calibri"
  },
  "wordCount": 123,
  "createdAt": "2026-04-05T00:00:00Z"
}
```

Response `404`:

```json
{
  "error": "not_found",
  "message": "Generation not found"
}
```

## Internal service routes

- `POST /internal/generate-markdown` (LLM service)
- `POST /internal/render` (Renderer service)
