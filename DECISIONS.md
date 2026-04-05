# Implementation Decisions and Assumptions

## Assumptions

1. No existing production traffic must be preserved.
   - Chosen approach: Start with a clean greenfield vertical slice and strict contracts.
   - Alternative: Big-bang full feature implementation in one pass.
   - Why rejected: Increases defect risk and blocks fast feedback.

2. LLM provider credentials may not exist in local development.
   - Chosen approach: Deterministic markdown fallback generator in LLM service.
   - Alternative: Hard fail when Anthropic key is absent.
   - Why rejected: Prevents local development and testing.

3. Runtime polyglot architecture is required now.
   - Chosen approach: Rust gateway + Python LLM + Node renderer from day one.
   - Alternative: Temporary single-runtime simplification.
   - Why rejected: Would delay architectural integration risk discovery.

## Technology Decisions

| Decision | Alternatives considered | Rationale | Migration path |
| --- | --- | --- | --- |
| Rust Axum gateway | Go Fiber, Node Fastify | Strong type safety, high concurrency, explicit boundary control | Keep API contracts stable and add persistent repositories |
| Python FastAPI LLM service | Rust or Node LLM client | Mature AI ecosystem and straightforward async interfaces | Wrap providers behind internal strategy interface |
| Node renderer with `docx` | Rust docx libraries, Pandoc wrapper | Best practical quality for direct DOCX generation | Can switch to Pandoc pipeline via adapter |
| Docker Compose local orchestration | Manual local startup | Deterministic service networking and reproducibility | Promote same contracts into Kubernetes manifests |

## Known Technical Debt

1. High: In-memory rate limit store in gateway.
   - Impact: Limits are reset on restart and not shared across instances.
   - Planned resolution: Move counters to Redis with TTL windows.

2. Medium: Generation rows currently store raw API keys for scoping.
   - Impact: Increases sensitivity of at-rest data in case of database disclosure.
   - Planned resolution: Store key fingerprints (or key IDs) and keep raw keys out of persistence.

3. Medium: No OAuth/JWT user session flow in this initial slice.
   - Impact: API-key-only auth for now.
   - Planned resolution: Introduce auth bounded context and refresh token rotation.
