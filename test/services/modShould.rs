#[cfg(test)]
mod services_mod_tests {
    #[test]
    fn test_services_modules_accessible() {
        let _geo_config = crate::config::GEOIP_RATE_LIMIT;
        let _http_config = crate::config::HTTP_TIMEOUT_SECS;
    }
}
