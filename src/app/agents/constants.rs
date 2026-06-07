use std::time::Duration;

pub const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";
pub const DEFAULT_OLLAMA_MODEL: &str = "llama3.2:latest";
pub const DEFAULT_API_KEY: &str = "";
pub const DEFAULT_TEMPERATURE: f32 = 0.3;
pub const DEFAULT_NUM_PREDICT: u32 = 4096;
pub const DEFAULT_MAX_TOKENS: u32 = 4096;
pub const AGENT_REQUEST_TIMEOUT: Duration = Duration::from_secs(180);
pub const OLLAMA_FETCH_TIMEOUT: Duration = Duration::from_secs(10);

pub const HISTORY_DIR_NAME: &str = "agent_history";
pub const HISTORY_FILE_EXT: &str = "md";
pub const HISTORY_TIMESTAMP_FORMAT: &str = "%Y%m%d_%H%M%S";
pub const HISTORY_META_PREFIX: &str = "agent:";
pub const HISTORY_META_SEPARATOR: &str = "|";
pub const HISTORY_META_NEWLINE: &str = "\n";
pub const HISTORY_NAME_PARTS_OFFSET: usize = 2;

pub const OLLAMA_TAGS_PATH: &str = "/api/tags";
pub const OLLAMA_CHAT_PATH: &str = "/api/chat";
pub const OPENAI_CHAT_PATH: &str = "/v1/chat/completions";
pub const ANTHROPIC_MESSAGES_PATH: &str = "/v1/messages";
pub const ANTHROPIC_VERSION: &str = "2023-06-01";

pub const SSE_DATA_PREFIX: &str = "data:";
pub const SSE_DONE_SENTINEL: &str = "[DONE]";
pub const STREAM_NEWLINE: char = '\n';

pub const NO_PROCESS_DATA_ERROR: &str = "No process data available";
pub const NO_NETWORK_DATA_ERROR: &str = "No network data available";
pub const NO_RESPONSE_ERROR: &str = "No response from model";
pub const CANCELLED_ERROR: &str = "Cancelled";
pub const HTTP_CLIENT_ERROR_PREFIX: &str = "HTTP client error";
pub const PROVIDER_REQUEST_ERROR_PREFIX: &str = "Provider request failed";
pub const PROVIDER_HTTP_ERROR_PREFIX: &str = "Provider HTTP";
pub const STREAM_ERROR_PREFIX: &str = "Stream error";
pub const OLLAMA_FETCH_ERROR_PREFIX: &str = "Failed to reach Ollama";
pub const PARSE_RESPONSE_ERROR_PREFIX: &str = "Failed to parse response";

pub const NO_PROCESS_NAME_PLACEHOLDER: &str = "N/A";
pub const NO_DEPENDENCY_CONTEXT_PLACEHOLDER: &str = "None";

pub const HEADER_X_API_KEY: &str = "x-api-key";
pub const HEADER_ANTHROPIC_VERSION: &str = "anthropic-version";
