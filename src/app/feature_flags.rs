use crate::app::types::AgentProvider;

pub const ENABLE_OLLAMA_PROVIDER: bool = true;
pub const ENABLE_OPENAI_PROVIDER: bool = false;
pub const ENABLE_ANTHROPIC_PROVIDER: bool = false;
pub const ENABLE_LLAMA_CPP_PROVIDER: bool = false;

pub fn agent_provider_enabled(provider: AgentProvider) -> bool {
    match provider {
        AgentProvider::Ollama => ENABLE_OLLAMA_PROVIDER,
        AgentProvider::OpenAI => ENABLE_OPENAI_PROVIDER,
        AgentProvider::Anthropic => ENABLE_ANTHROPIC_PROVIDER,
        AgentProvider::LlamaCpp => ENABLE_LLAMA_CPP_PROVIDER,
    }
}
