#[cfg(test)]
mod utils_mod_tests {
    #[test]
    fn test_utils_modules_accessible() {
        let _fmt = crate::utils::formatting::format_bytes(0);
        let _builder = crate::utils::api_builder::ApiUrlBuilder::new("http://test.com");
        let _limiter = crate::utils::rate_limiter::RateLimiter::new(10, 60);
        let _cache = crate::utils::icon_extractor::IconCache::new();
        let _sig = crate::utils::signatures::SignatureVerifier::verify("");
        drop((_fmt, _builder, _limiter, _cache, _sig));
    }
}
