# Velium DocGen Studio

API-first AI document generation platform that converts prompts into publication-ready Word and Markdown outputs.

## MVP slice in this repository

- Rust Axum gateway with API key auth, request validation, and rate limiting.
- Python FastAPI LLM service that transforms prompts into structured markdown.
- Node TypeScript renderer service that converts markdown into `.docx` and `.md` artifacts.
- PostgreSQL-backed generation persistence with API-key-scoped history retrieval.
- Docker Compose stack with PostgreSQL, Redis, NATS, and Nginx reverse proxy.
- Initial ADRs, API contract docs, and CI pipeline.

## Architecture (current implementation)

- `backend/gateway-rust`: edge API gateway and orchestration entrypoint.
- `backend/llm-python`: markdown generation service.
- `backend/renderer-node`: markdown-to-docx renderer.
- `infra/nginx`: reverse proxy for `/api` traffic.
- `docs/adr`: architecture decision records.

## Quick start

```bash
cp .env.example .env
docker compose up --build
```

When the stack is healthy, call the generation endpoint:

```bash
curl -X POST http://localhost:8080/api/v1/generate \
	-H "content-type: application/json" \
	-H "x-api-key: dgk_dev_local_key" \
	-d '{
		"prompt": "Create a technical architecture report for an event-driven ecommerce platform",
		"outputFormats": ["docx", "markdown"],
		"style": {
			"documentTitle": "Architecture Report",
			"accentColor": "#1F4E79",
			"fontFamily": "Calibri"
		}
	}'
```

Then fetch persisted history for the same API key:

```bash
curl -X GET 'http://localhost:8080/api/v1/generations?limit=5' \
	-H 'x-api-key: dgk_dev_local_key'
```

## Testing

```bash
# Rust gateway
cd backend/gateway-rust && cargo test

# Python service
cd ../llm-python && python -m pytest

# Node renderer
cd ../renderer-node && npm test
```

## Security posture (initial)

- API key boundary validation at gateway.
- Fixed-window rate limiting per API key.
- Strict JSON request validation in every service.
- Structured JSON errors to avoid stack trace leakage.

## Persistence and history

- Every successful generation is persisted in PostgreSQL by the gateway.
- History list endpoint: `GET /api/v1/generations?limit=20`
- History detail endpoint: `GET /api/v1/generations/:generationId`

## Current scope boundary

This implementation establishes the production-style vertical slice and runtime wiring. Full auth token lifecycle, billing, and frontend dashboard are tracked for upcoming phases.
