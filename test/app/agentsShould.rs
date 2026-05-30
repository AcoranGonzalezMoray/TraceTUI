#[cfg(test)]
mod agents_tests {
    use crate::app::states::AgentState;
    use crate::app::types::{AgentInstance, AgentMission, AgentProvider, AgentStatus, OllamaConfig};

    #[test]
    fn test_agent_state_new() {
        let state = AgentState::new();
        assert!(state.agents.is_empty());
        assert_eq!(state.provider, AgentProvider::Ollama);
        assert_eq!(state.ollama.api_url, "http://localhost:11434");
        assert!(!state.ollama.models.is_empty());
        assert!(!state.show_provider_modal);
        assert!(!state.show_process_selector);
        assert!(!state.show_network_selector);
        assert!(state.selected_mission.is_none());
        assert_eq!(state.selected_agent_index, 0);
        assert_eq!(state.agent_action_index, 0);
        assert_eq!(state.agent_detail_scroll, 0);
        assert!(state.selected_pids.is_empty());
    }

    #[test]
    fn test_ollama_config_default() {
        let config = OllamaConfig {
            api_url: "http://localhost:11434".to_string(),
            models: vec!["llama3.2:latest".to_string()],
        };
        assert_eq!(config.api_url, "http://localhost:11434");
        assert_eq!(config.models.len(), 1);
        assert_eq!(config.models[0], "llama3.2:latest");
    }

    #[test]
    fn test_agent_instance_creation() {
        let agent = AgentInstance {
            mission: AgentMission::ProcessAnalysis,
            provider: AgentProvider::Ollama,
            model: "llama3.2:latest".to_string(),
            status: AgentStatus::Idle,
            started_at_frame: 0,
            completed_at_frame: None,
            target_name: String::new(),
            target_path: None,
        };
        assert_eq!(agent.mission, AgentMission::ProcessAnalysis);
        assert_eq!(agent.model, "llama3.2:latest");
        assert!(matches!(agent.status, AgentStatus::Idle));
    }

    #[test]
    fn test_agent_status_transitions() {
        let statuses = vec![
            AgentStatus::Idle,
            AgentStatus::Running("Analyzing...".to_string()),
            AgentStatus::Completed("Process is clean".to_string()),
            AgentStatus::Failed("Connection error".to_string()),
        ];
        assert_eq!(statuses.len(), 4);
        assert!(matches!(statuses[0], AgentStatus::Idle));
        assert!(matches!(statuses[1], AgentStatus::Running(_)));
        assert!(matches!(statuses[2], AgentStatus::Completed(_)));
        assert!(matches!(statuses[3], AgentStatus::Failed(_)));
    }

    #[test]
    fn test_agent_state_add_remove_agent() {
        let mut state = AgentState::new();
        assert!(state.agents.is_empty());

        state.agents.push(AgentInstance {
            mission: AgentMission::NetworkAnalysis,
            provider: AgentProvider::Ollama,
            model: "llama3.2:latest".to_string(),
            status: AgentStatus::Running("Scanning...".to_string()),
            started_at_frame: 0,
            completed_at_frame: None,
            target_name: "test_process".to_string(),
            target_path: None,
        });
        assert_eq!(state.agents.len(), 1);

        state.agents.remove(0);
        assert!(state.agents.is_empty());
    }

    #[test]
    fn test_agent_mission_variants() {
        let missions = vec![
            AgentMission::ProcessAnalysis,
            AgentMission::NetworkAnalysis,
        ];
        assert_eq!(missions.len(), 2);
    }

    #[test]
    fn test_ollama_config_serde() {
        let config = OllamaConfig {
            api_url: "http://192.168.1.100:11434".to_string(),
            models: vec!["llama3.2:latest".to_string(), "mistral:latest".to_string()],
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: OllamaConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.api_url, "http://192.168.1.100:11434");
        assert_eq!(deserialized.models.len(), 2);
        assert_eq!(deserialized.models[1], "mistral:latest");
    }
}
