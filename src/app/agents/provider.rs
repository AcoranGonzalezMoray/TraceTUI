use crate::app::agents::constants::{
    ANTHROPIC_MESSAGES_PATH, ANTHROPIC_VERSION, DEFAULT_MAX_TOKENS, DEFAULT_NUM_PREDICT,
    DEFAULT_TEMPERATURE, HEADER_ANTHROPIC_VERSION, HEADER_X_API_KEY, OLLAMA_CHAT_PATH,
    OPENAI_CHAT_PATH,
};
use crate::app::types::AgentProvider;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};
use reqwest::{Client, RequestBuilder};
use serde_json::Value;

pub enum Endpoint {
    OllamaChat,
    OpenAIChat,
    AnthropicMessages,
}

pub enum Auth {
    Bearer,
    AnthropicApiKey,
}

pub struct ProviderSpec {
    pub endpoint: Endpoint,
    pub auth: Auth,
}

impl ProviderSpec {
    pub fn for_provider(provider: AgentProvider) -> Self {
        match provider {
            AgentProvider::Ollama => Self {
                endpoint: Endpoint::OllamaChat,
                auth: Auth::Bearer,
            },
            AgentProvider::OpenAI | AgentProvider::LlamaCpp => Self {
                endpoint: Endpoint::OpenAIChat,
                auth: Auth::Bearer,
            },
            AgentProvider::Anthropic => Self {
                endpoint: Endpoint::AnthropicMessages,
                auth: Auth::AnthropicApiKey,
            },
        }
    }

    pub fn url_path(&self) -> &'static str {
        match self.endpoint {
            Endpoint::OllamaChat => OLLAMA_CHAT_PATH,
            Endpoint::OpenAIChat => OPENAI_CHAT_PATH,
            Endpoint::AnthropicMessages => ANTHROPIC_MESSAGES_PATH,
        }
    }

    pub fn build_url(&self, base: &str) -> String {
        let base = base.trim_end_matches('/');
        let path = self.url_path().trim_start_matches('/');
        if base.ends_with(&path) {
            base.to_string()
        } else {
            format!("{}/{}", base, path)
        }
    }

    pub fn build_body(&self, model: &str, prompt: &str, stream: bool) -> Value {
        match self.endpoint {
            Endpoint::OllamaChat => ollama_chat_payload(model, prompt, stream),
            Endpoint::OpenAIChat => chat_completions_payload(model, prompt, stream),
            Endpoint::AnthropicMessages => anthropic_payload(model, prompt, stream),
        }
    }

    pub fn apply_auth(
        &self,
        request: RequestBuilder,
        api_key: &str,
    ) -> RequestBuilder {
        if api_key.is_empty() {
            return request;
        }
        match self.auth {
            Auth::Bearer => request.header(AUTHORIZATION, format!("Bearer {api_key}")),
            Auth::AnthropicApiKey => {
                let mut headers = HeaderMap::new();
                if let Ok(v) = HeaderValue::from_str(api_key) {
                    headers.insert(HeaderName::from_static(HEADER_X_API_KEY), v);
                }
                headers.insert(
                    HeaderName::from_static(HEADER_ANTHROPIC_VERSION),
                    HeaderValue::from_static(ANTHROPIC_VERSION),
                );
                request.headers(headers)
            }
        }
    }

    pub fn extract_delta<'a>(&self, json: &'a Value) -> Option<&'a str> {
        match self.endpoint {
            Endpoint::OllamaChat => json["message"]["content"].as_str(),
            Endpoint::OpenAIChat => json["choices"][0]["delta"]["content"].as_str(),
            Endpoint::AnthropicMessages => json["delta"]["text"]
                .as_str()
                .or_else(|| json["content_block"]["text"].as_str()),
        }
    }
}

pub fn ollama_chat_payload(model: &str, prompt: &str, stream: bool) -> Value {
    serde_json::json!({
        "model": model,
        "stream": stream,
        "messages": [
            { "role": "user", "content": prompt }
        ],
        "options": { "temperature": DEFAULT_TEMPERATURE, "num_predict": DEFAULT_NUM_PREDICT }
    })
}

pub fn chat_completions_payload(model: &str, prompt: &str, stream: bool) -> Value {
    serde_json::json!({
        "model": model,
        "stream": stream,
        "messages": [
            { "role": "system", "content": "You are a precise security analysis agent." },
            { "role": "user", "content": prompt }
        ],
        "temperature": DEFAULT_TEMPERATURE,
        "max_tokens": DEFAULT_MAX_TOKENS
    })
}

pub fn anthropic_payload(model: &str, prompt: &str, stream: bool) -> Value {
    serde_json::json!({
        "model": model,
        "stream": stream,
        "max_tokens": DEFAULT_MAX_TOKENS,
        "temperature": DEFAULT_TEMPERATURE,
        "messages": [{ "role": "user", "content": prompt }]
    })
}

pub fn build_http_client() -> Result<Client, String> {
    use crate::app::agents::constants::{AGENT_REQUEST_TIMEOUT, HTTP_CLIENT_ERROR_PREFIX};
    Client::builder()
        .timeout(AGENT_REQUEST_TIMEOUT)
        .build()
        .map_err(|e| format!("{}: {}", HTTP_CLIENT_ERROR_PREFIX, e))
}

pub fn provider_label_from_string(label: &str) -> AgentProvider {
    match label {
        "OpenAI" => AgentProvider::OpenAI,
        "Anthropic" => AgentProvider::Anthropic,
        "llama.cpp" => AgentProvider::LlamaCpp,
        _ => AgentProvider::Ollama,
    }
}
