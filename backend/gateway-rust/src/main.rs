use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{
    collections::{HashMap, HashSet},
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
    llm_service_url: String,
    renderer_service_url: String,
    valid_api_keys: HashSet<String>,
    limiter: RateLimiter,
    db_pool: PgPool,
}

#[derive(Clone)]
struct RateLimiter {
    state: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    fn check(&self, key: &str) -> Result<(), AppError> {
        let mut guard = self.state.lock().map_err(|_| {
            AppError::internal("rate_limit_store_poisoned", "Rate limiter unavailable")
        })?;

        let now = Instant::now();
        let entries = guard.entry(key.to_owned()).or_default();
        entries.retain(|time| now.duration_since(*time) < self.window);

        if entries.len() >= self.max_requests {
            return Err(AppError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Rate limit exceeded",
            ));
        }

        entries.push(now);
        Ok(())
    }
}

#[derive(Debug)]
struct AppError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl AppError {
    fn new(status: StatusCode, code: &'static str, message: &str) -> Self {
        Self {
            status,
            code,
            message: message.to_owned(),
        }
    }

    fn bad_request(code: &'static str, message: &str) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message)
    }

    fn unauthorized(message: &str) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "unauthorized", message)
    }

    fn not_found(message: &str) -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", message)
    }

    fn bad_gateway(code: &'static str, message: &str) -> Self {
        Self::new(StatusCode::BAD_GATEWAY, code, message)
    }

    fn internal(code: &'static str, message: &str) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, code, message)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            error: self.code,
            message: self.message,
        });
        (self.status, body).into_response()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    error: &'static str,
    message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenerateRequest {
    prompt: String,
    output_formats: Vec<OutputFormat>,
    style: Option<StyleConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum OutputFormat {
    Docx,
    Markdown,
}

impl OutputFormat {
    fn as_str(&self) -> &'static str {
        match self {
            OutputFormat::Docx => "docx",
            OutputFormat::Markdown => "markdown",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct StyleConfig {
    document_title: Option<String>,
    accent_color: Option<String>,
    font_family: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerateResponse {
    generation_id: String,
    markdown: String,
    outputs: HashMap<String, String>,
    word_count: usize,
    created_at: String,
}

#[derive(Deserialize)]
struct GenerationHistoryQuery {
    limit: Option<usize>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationHistoryResponse {
    items: Vec<GenerationHistoryItem>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationHistoryItem {
    generation_id: String,
    prompt: String,
    output_formats: Vec<String>,
    word_count: usize,
    created_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationDetailResponse {
    generation_id: String,
    prompt: String,
    markdown: String,
    outputs: HashMap<String, String>,
    output_formats: Vec<String>,
    style: Option<StyleConfig>,
    word_count: usize,
    created_at: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LlmGenerateRequest {
    prompt: String,
    style: Option<StyleConfig>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LlmGenerateResponse {
    markdown: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RenderRequest {
    markdown: String,
    output_formats: Vec<String>,
    style: Option<StyleConfig>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenderResponse {
    outputs: HashMap<String, String>,
    word_count: usize,
}

#[derive(sqlx::FromRow)]
struct GenerationListRow {
    id: Uuid,
    prompt: String,
    output_formats: serde_json::Value,
    word_count: i32,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct GenerationDetailRow {
    id: Uuid,
    prompt: String,
    markdown: String,
    outputs: serde_json::Value,
    output_formats: serde_json::Value,
    style: Option<serde_json::Value>,
    word_count: i32,
    created_at: DateTime<Utc>,
}

#[derive(Serialize)]
struct DependencyState {
    llm: &'static str,
    renderer: &'static str,
    database: &'static str,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    dependencies: DependencyState,
}

#[tokio::main]
async fn main() {
    init_logger();

    let port = env::var("GATEWAY_PORT")
        .unwrap_or_else(|_| "8080".to_owned())
        .parse::<u16>()
        .expect("GATEWAY_PORT must be a valid u16");

    let llm_service_url =
        env::var("LLM_SERVICE_URL").unwrap_or_else(|_| "http://localhost:8000".to_owned());
    let renderer_service_url =
        env::var("RENDERER_SERVICE_URL").unwrap_or_else(|_| "http://localhost:3001".to_owned());
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://velium:velium@localhost:5432/velium".to_owned());
    let valid_api_keys = read_api_keys();
    let db_pool = initialize_database(&database_url)
        .await
        .unwrap_or_else(|error| panic!("database initialization failed: {error}"));

    let state = AppState {
        client: reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .expect("HTTP client must initialize"),
        llm_service_url,
        renderer_service_url,
        valid_api_keys,
        limiter: RateLimiter::new(100, Duration::from_secs(60)),
        db_pool,
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/generate", post(generate_document))
        .route("/api/v1/generations", get(list_generations))
        .route("/api/v1/generations/:generation_id", get(get_generation))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let address = SocketAddr::from(([0, 0, 0, 0], port));
    info!("gateway listening on {}", address);

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("gateway listener must bind");

    axum::serve(listener, app)
        .await
        .expect("gateway server must run");
}

fn init_logger() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();
}

fn read_api_keys() -> HashSet<String> {
    env::var("MASTER_API_KEYS")
        .unwrap_or_else(|_| "dgk_dev_local_key".to_owned())
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

async fn initialize_database(database_url: &str) -> Result<PgPool, String> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(|error| error.to_string())?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|error| error.to_string())?;

    Ok(pool)
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let llm_state = dependency_state(&state.client, &state.llm_service_url).await;
    let renderer_state = dependency_state(&state.client, &state.renderer_service_url).await;
    let database_state = database_state(&state.db_pool).await;

    Json(HealthResponse {
        status: "ok",
        dependencies: DependencyState {
            llm: llm_state,
            renderer: renderer_state,
            database: database_state,
        },
    })
}

async fn dependency_state(client: &reqwest::Client, base_url: &str) -> &'static str {
    let url = format!("{}/health", base_url.trim_end_matches('/'));
    let response = client.get(url).send().await;
    if matches!(response, Ok(result) if result.status().is_success()) {
        "up"
    } else {
        "down"
    }
}

async fn database_state(pool: &PgPool) -> &'static str {
    let result = sqlx::query("SELECT 1").execute(pool).await;
    if result.is_ok() {
        "up"
    } else {
        "down"
    }
}

async fn generate_document(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, AppError> {
    validate_request(&payload)?;

    let api_key = authorize_and_rate_limit(&state, &headers)?;

    let llm_response = call_llm_service(&state, &payload).await?;
    let render_response = call_renderer_service(&state, &payload, &llm_response.markdown).await?;
    let generation_id = Uuid::new_v4();
    let created_at = Utc::now();
    let markdown = llm_response.markdown;
    let outputs = render_response.outputs;
    let word_count = render_response.word_count;

    persist_generation(
        &state,
        generation_id,
        &api_key,
        &payload,
        &markdown,
        &outputs,
        word_count,
        created_at,
    )
    .await?;

    Ok(Json(GenerateResponse {
        generation_id: generation_id.to_string(),
        markdown,
        outputs,
        word_count,
        created_at: created_at.to_rfc3339(),
    }))
}

async fn list_generations(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<GenerationHistoryQuery>,
) -> Result<Json<GenerationHistoryResponse>, AppError> {
    let api_key = authorize_and_rate_limit(&state, &headers)?;
    let limit = resolve_history_limit(query.limit)?;

    let rows = sqlx::query_as::<_, GenerationListRow>(
        "SELECT id, prompt, output_formats, word_count, created_at
         FROM generations
         WHERE api_key = $1
         ORDER BY created_at DESC
         LIMIT $2",
    )
    .bind(&api_key)
    .bind(limit)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| AppError::internal("db_query_failed", "Failed to load generation history"))?;

    let items = rows
        .into_iter()
        .map(|row| {
            let output_formats = decode_output_formats(row.output_formats)?;
            let word_count = usize::try_from(row.word_count).map_err(|_| {
                AppError::internal("db_decode_failed", "Stored word count is invalid")
            })?;

            Ok(GenerationHistoryItem {
                generation_id: row.id.to_string(),
                prompt: row.prompt,
                output_formats,
                word_count,
                created_at: row.created_at.to_rfc3339(),
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    Ok(Json(GenerationHistoryResponse { items }))
}

async fn get_generation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(generation_id): Path<String>,
) -> Result<Json<GenerationDetailResponse>, AppError> {
    let api_key = authorize_and_rate_limit(&state, &headers)?;
    let parsed_generation_id = Uuid::parse_str(&generation_id).map_err(|_| {
        AppError::bad_request(
            "invalid_generation_id",
            "Generation id must be a valid UUID",
        )
    })?;

    let row = sqlx::query_as::<_, GenerationDetailRow>(
        "SELECT id, prompt, markdown, outputs, output_formats, style, word_count, created_at
         FROM generations
         WHERE id = $1 AND api_key = $2",
    )
    .bind(parsed_generation_id)
    .bind(&api_key)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| AppError::internal("db_query_failed", "Failed to load generation record"))?
    .ok_or_else(|| AppError::not_found("Generation not found"))?;

    let outputs = decode_outputs(row.outputs)?;
    let output_formats = decode_output_formats(row.output_formats)?;
    let style = decode_style(row.style)?;
    let word_count = usize::try_from(row.word_count)
        .map_err(|_| AppError::internal("db_decode_failed", "Stored word count is invalid"))?;

    Ok(Json(GenerationDetailResponse {
        generation_id: row.id.to_string(),
        prompt: row.prompt,
        markdown: row.markdown,
        outputs,
        output_formats,
        style,
        word_count,
        created_at: row.created_at.to_rfc3339(),
    }))
}

fn decode_output_formats(value: serde_json::Value) -> Result<Vec<String>, AppError> {
    serde_json::from_value(value)
        .map_err(|_| AppError::internal("db_decode_failed", "Stored output formats are invalid"))
}

fn decode_outputs(value: serde_json::Value) -> Result<HashMap<String, String>, AppError> {
    serde_json::from_value(value)
        .map_err(|_| AppError::internal("db_decode_failed", "Stored outputs are invalid"))
}

fn decode_style(value: Option<serde_json::Value>) -> Result<Option<StyleConfig>, AppError> {
    value
        .map(|style| {
            serde_json::from_value(style).map_err(|_| {
                AppError::internal("db_decode_failed", "Stored style payload is invalid")
            })
        })
        .transpose()
}

fn resolve_history_limit(limit: Option<usize>) -> Result<i64, AppError> {
    let limit = limit.unwrap_or(20);
    if limit == 0 || limit > 100 {
        return Err(AppError::bad_request(
            "invalid_limit",
            "Query parameter 'limit' must be between 1 and 100",
        ));
    }

    i64::try_from(limit).map_err(|_| {
        AppError::bad_request(
            "invalid_limit",
            "Query parameter 'limit' must be between 1 and 100",
        )
    })
}

async fn persist_generation(
    state: &AppState,
    generation_id: Uuid,
    api_key: &str,
    payload: &GenerateRequest,
    markdown: &str,
    outputs: &HashMap<String, String>,
    word_count: usize,
    created_at: DateTime<Utc>,
) -> Result<(), AppError> {
    let output_formats = payload
        .output_formats
        .iter()
        .map(|format| format.as_str().to_owned())
        .collect::<Vec<_>>();

    let output_formats_json = serde_json::to_value(output_formats).map_err(|_| {
        AppError::internal("serialization_failed", "Failed to serialize output formats")
    })?;
    let outputs_json = serde_json::to_value(outputs)
        .map_err(|_| AppError::internal("serialization_failed", "Failed to serialize outputs"))?;
    let style_json = payload
        .style
        .as_ref()
        .map(serde_json::to_value)
        .transpose()
        .map_err(|_| {
            AppError::internal("serialization_failed", "Failed to serialize style payload")
        })?;
    let word_count = i32::try_from(word_count).map_err(|_| {
        AppError::internal("invalid_word_count", "Word count exceeded supported range")
    })?;

    sqlx::query(
        "INSERT INTO generations (
            id,
            api_key,
            prompt,
            markdown,
            output_formats,
            outputs,
            style,
            word_count,
            created_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(generation_id)
    .bind(api_key)
    .bind(&payload.prompt)
    .bind(markdown)
    .bind(output_formats_json)
    .bind(outputs_json)
    .bind(style_json)
    .bind(word_count)
    .bind(created_at)
    .execute(&state.db_pool)
    .await
    .map_err(|_| AppError::internal("db_write_failed", "Failed to persist generation record"))?;

    Ok(())
}

fn validate_request(payload: &GenerateRequest) -> Result<(), AppError> {
    let trimmed_prompt = payload.prompt.trim();
    if trimmed_prompt.len() < 10 || trimmed_prompt.len() > 8000 {
        return Err(AppError::bad_request(
            "invalid_prompt",
            "Prompt length must be between 10 and 8000 characters",
        ));
    }

    if payload.output_formats.is_empty() {
        return Err(AppError::bad_request(
            "invalid_output_formats",
            "At least one output format must be provided",
        ));
    }

    if payload.output_formats.len() > 2 {
        return Err(AppError::bad_request(
            "invalid_output_formats",
            "No more than two output formats are allowed",
        ));
    }

    if let Some(style) = &payload.style {
        if let Some(color) = &style.accent_color {
            if !is_hex_color(color) {
                return Err(AppError::bad_request(
                    "invalid_accent_color",
                    "Accent color must be a valid hex value like #1F4E79",
                ));
            }
        }
    }

    Ok(())
}

fn is_hex_color(value: &str) -> bool {
    if value.len() != 7 || !value.starts_with('#') {
        return false;
    }

    value
        .chars()
        .skip(1)
        .all(|character| character.is_ascii_hexdigit())
}

fn read_api_key(headers: &HeaderMap) -> Result<String, AppError> {
    headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
        .ok_or_else(|| AppError::unauthorized("Missing API key"))
}

fn authorize_and_rate_limit(state: &AppState, headers: &HeaderMap) -> Result<String, AppError> {
    let api_key = read_api_key(headers)?;
    authorize_api_key(&state.valid_api_keys, &api_key)?;
    state.limiter.check(&api_key)?;
    Ok(api_key)
}

fn authorize_api_key(allowed_keys: &HashSet<String>, provided_key: &str) -> Result<(), AppError> {
    if allowed_keys.contains(provided_key) {
        Ok(())
    } else {
        Err(AppError::unauthorized("Invalid API key"))
    }
}

async fn call_llm_service(
    state: &AppState,
    payload: &GenerateRequest,
) -> Result<LlmGenerateResponse, AppError> {
    let response = state
        .client
        .post(format!(
            "{}/internal/generate-markdown",
            state.llm_service_url.trim_end_matches('/')
        ))
        .json(&LlmGenerateRequest {
            prompt: payload.prompt.clone(),
            style: payload.style.clone(),
        })
        .send()
        .await
        .map_err(|_| AppError::bad_gateway("llm_unreachable", "LLM service unavailable"))?;

    if !response.status().is_success() {
        return Err(AppError::bad_gateway(
            "llm_failure",
            "LLM service returned a non-success status",
        ));
    }

    response.json::<LlmGenerateResponse>().await.map_err(|_| {
        AppError::bad_gateway("llm_invalid_response", "LLM response payload was invalid")
    })
}

async fn call_renderer_service(
    state: &AppState,
    payload: &GenerateRequest,
    markdown: &str,
) -> Result<RenderResponse, AppError> {
    let output_formats = payload
        .output_formats
        .iter()
        .map(|format| format.as_str().to_owned())
        .collect::<Vec<_>>();

    let response = state
        .client
        .post(format!(
            "{}/internal/render",
            state.renderer_service_url.trim_end_matches('/')
        ))
        .json(&RenderRequest {
            markdown: markdown.to_owned(),
            output_formats,
            style: payload.style.clone(),
        })
        .send()
        .await
        .map_err(|_| {
            AppError::bad_gateway("renderer_unreachable", "Renderer service unavailable")
        })?;

    if !response.status().is_success() {
        return Err(AppError::bad_gateway(
            "renderer_failure",
            "Renderer service returned a non-success status",
        ));
    }

    response.json::<RenderResponse>().await.map_err(|_| {
        AppError::bad_gateway(
            "renderer_invalid_response",
            "Renderer response payload was invalid",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{is_hex_color, resolve_history_limit, RateLimiter};
    use std::time::Duration;

    #[test]
    fn hex_color_validation_accepts_valid_value() {
        assert!(is_hex_color("#1F4E79"));
    }

    #[test]
    fn hex_color_validation_rejects_invalid_value() {
        assert!(!is_hex_color("1F4E79"));
        assert!(!is_hex_color("#XYZ123"));
    }

    #[test]
    fn rate_limiter_blocks_after_limit() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));
        assert!(limiter.check("key").is_ok());
        assert!(limiter.check("key").is_ok());
        assert!(limiter.check("key").is_err());
    }

    #[test]
    fn history_limit_enforces_bounds() {
        assert_eq!(resolve_history_limit(None).unwrap(), 20);
        assert_eq!(resolve_history_limit(Some(1)).unwrap(), 1);
        assert_eq!(resolve_history_limit(Some(100)).unwrap(), 100);
        assert!(resolve_history_limit(Some(0)).is_err());
        assert!(resolve_history_limit(Some(101)).is_err());
    }
}
