#[cfg(test)]
mod containers_tests {
    use crate::app::containers::{ContainerInfo, DockerHubSearchState};
    use crate::app::states::ContainerState;
    use crate::app::App;

    #[test]
    fn test_container_state_new() {
        let cs = ContainerState::new();
        assert!(cs.containers.is_empty());
        assert_eq!(cs.selected_container_index, 0);
        assert_eq!(cs.selected_container_action_index, 0);
        assert_eq!(cs.container_detail_scroll, 0);
        assert!(!cs.containers_loading);
        assert!(!cs.containers_loaded_once);
        assert!(cs.containers_error.is_none());
        assert!(cs.container_rx.is_none());
        assert!(cs.container_logs.is_empty());
        assert!(!cs.container_logs_loading);
        assert!(cs.container_logs_rx.is_none());
        assert!(!cs.show_container_logs_modal);
        assert!(!cs.show_container_console_modal);
        assert_eq!(cs.container_logs_scroll, 0);
        assert!(cs.container_console_input.is_empty());
        assert!(cs.container_console_output.is_empty());
        assert!(!cs.container_console_loading);
        assert_eq!(cs.container_console_scroll, 0);
        assert!(cs.container_console_rx.is_none());
        assert!(!cs.show_docker_hub_modal);
        assert!(cs.docker_hub_search_rx.is_none());
        assert!(cs.docker_hub_create_rx.is_none());
        assert!(cs.pending_container_action.is_none());
        assert!(cs.pending_docker_action.is_none());
    }

    #[test]
    fn test_container_info_defaults() {
        let info = ContainerInfo::default();
        assert!(info.id.is_empty());
        assert!(info.name.is_empty());
        assert!(info.image.is_empty());
        assert!(info.status.is_empty());
        assert!(info.state.is_empty());
        assert!(info.ports.is_empty());
        assert!(info.networks.is_empty());
        assert!(info.created.is_empty());
        assert!(info.running_for.is_empty());
        assert!(info.size.is_empty());
        assert!(info.cpu_percent.is_none());
        assert!(info.memory_usage.is_empty());
        assert!(info.memory_percent.is_none());
        assert!(info.net_io.is_empty());
        assert!(info.block_io.is_empty());
        assert!(info.pids.is_empty());
    }

    #[test]
    fn test_docker_hub_search_defaults() {
        let dhs = DockerHubSearchState::default();
        assert!(dhs.search_query.is_empty());
        assert!(dhs.results.is_empty());
        assert_eq!(dhs.selected_result_index, 0);
        assert!(dhs.container_name.is_empty());
        assert!(dhs.ports.is_empty());
        assert!(dhs.env_vars.is_empty());
        assert_eq!(dhs.focused_field, 0);
    }

    #[test]
    fn test_app_container_selection() {
        let app = App::new();
        assert_eq!(app.containers.selected_container_index, 0);
    }

    #[test]
    fn test_app_containers_loading_state() {
        let app = App::new();
        assert!(!app.containers.containers_loading);
        assert!(!app.containers.containers_loaded_once);
    }

    #[test]
    fn test_app_get_selected_container_none() {
        let app = App::new();
        assert!(app.get_selected_container().is_none());
    }

    #[test]
    fn test_app_containers_iteration() {
        let mut cs = ContainerState::new();
        cs.containers = vec![
            ContainerInfo {
                id: "abc123".to_string(),
                name: "nginx".to_string(),
                ..ContainerInfo::default()
            },
            ContainerInfo {
                id: "def456".to_string(),
                name: "redis".to_string(),
                ..ContainerInfo::default()
            },
            ContainerInfo {
                id: "ghi789".to_string(),
                name: "postgres".to_string(),
                ..ContainerInfo::default()
            },
        ];

        let mut names: Vec<&str> = Vec::new();
        for container in &cs.containers {
            names.push(container.name.as_str());
        }
        assert_eq!(names, vec!["nginx", "redis", "postgres"]);

        let ids: Vec<&str> = cs.containers.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, vec!["abc123", "def456", "ghi789"]);
    }
}
