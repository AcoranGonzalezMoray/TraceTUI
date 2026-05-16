#[cfg(test)]
mod icon_extractor_tests {
    use crate::utils::icon_extractor::IconCache;

    #[cfg(windows)]
    #[test]
    fn test_icon_cache_new() {
        let mut cache = IconCache::new();
        let icon = cache.get_icon("", "test");
        assert_eq!(icon, "tes");
    }

    #[cfg(windows)]
    #[test]
    fn test_icon_cache_get_unknown() {
        let mut cache = IconCache::new();
        let icon = cache.get_icon("/nonexistent/path.exe", "test_proc");
        assert_eq!(icon, "tes");
    }

    #[test]
    fn test_icon_cache_insert_and_retrieve() {
        let mut cache = IconCache::new();
        cache.insert_icon("/path/to/app.exe", "cached_icon".to_string());
        let icon = cache.get_icon("/path/to/app.exe", "app");
        assert_eq!(icon, "cached_icon");
    }

    #[test]
    fn test_icon_cache_overwrite() {
        let mut cache = IconCache::new();
        cache.insert_icon("/app.exe", "first".to_string());
        cache.insert_icon("/app.exe", "second".to_string());
        let icon = cache.get_icon("/app.exe", "app");
        assert_eq!(icon, "second");
    }

    #[cfg(windows)]
    #[test]
    fn test_icon_cache_lru_eviction() {
        use crate::config;
        let mut cache = IconCache::new();
        for i in 0..config::LRU_CACHE_SIZE + 5 {
            cache.insert_icon(&format!("/path{}.exe", i), format!("icon{}", i));
        }
        let evicted = cache.get_icon("/path0.exe", "app");
        assert_eq!(evicted, "app");
    }

    #[test]
    fn test_fallback_icon_empty_name() {
        let mut cache = IconCache::new();
        let icon = cache.get_icon("", "");
        assert_eq!(icon, "");
    }

    #[cfg(windows)]
    #[test]
    fn test_fallback_icon_short_name() {
        let mut cache = IconCache::new();
        let icon = cache.get_icon("", "ab");
        assert_eq!(icon, "ab");
    }

    #[test]
    fn test_icon_cache_debug() {
        let cache = IconCache::new();
        let _debug = format!("{:?}", cache);
    }
}
