#[cfg(test)]
mod trend_tests {
    use crate::app::states::TrendState;

    #[test]
    fn test_trend_state_new() {
        let state = TrendState::new();
        assert!(state.cpu_history.is_empty());
        assert!(state.conn_count_history.is_empty());
    }

    #[test]
    fn test_trend_state_cpu_history() {
        let mut state = TrendState::new();
        state.cpu_history.push(12.5);
        state.cpu_history.push(45.8);
        state.cpu_history.push(99.1);
        assert_eq!(state.cpu_history.len(), 3);
        assert_eq!(state.cpu_history[0], 12.5);
        assert_eq!(state.cpu_history[1], 45.8);
        assert_eq!(state.cpu_history[2], 99.1);
    }

    #[test]
    fn test_trend_state_conn_count_history() {
        let mut state = TrendState::new();
        state.conn_count_history.push(10);
        state.conn_count_history.push(25);
        state.conn_count_history.push(100);
        assert_eq!(state.conn_count_history.len(), 3);
        assert_eq!(state.conn_count_history[0], 10);
        assert_eq!(state.conn_count_history[1], 25);
        assert_eq!(state.conn_count_history[2], 100);
    }

    #[test]
    fn test_trend_state_max_conn() {
        let mut state = TrendState::new();
        state.conn_count_history.push(5);
        state.conn_count_history.push(42);
        state.conn_count_history.push(18);
        state.conn_count_history.push(99);
        state.conn_count_history.push(7);
        let max = state.conn_count_history.iter().max().copied().unwrap_or(0);
        assert_eq!(max, 99);
    }

    #[test]
    fn test_trend_cpu_history_capacity() {
        let mut state = TrendState::new();
        for i in 0..1000 {
            state.cpu_history.push(i as f64);
        }
        assert_eq!(state.cpu_history.len(), 1000);
        assert_eq!(state.cpu_history[0], 0.0);
        assert_eq!(state.cpu_history[999], 999.0);
    }
}
