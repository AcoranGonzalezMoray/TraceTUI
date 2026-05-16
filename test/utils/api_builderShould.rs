#[cfg(test)]
mod api_builder_tests {
    use crate::utils::api_builder::ApiUrlBuilder;

    #[test]
    fn test_basic_url() {
        let url = ApiUrlBuilder::new("http://example.com").build();
        assert_eq!(url, "http://example.com");
    }

    #[test]
    fn test_with_path() {
        let url = ApiUrlBuilder::new("http://example.com")
            .with_path("api/v1")
            .build();
        assert_eq!(url, "http://example.com/api/v1");
    }

    #[test]
    fn test_trailing_slash_stripped() {
        let url = ApiUrlBuilder::new("http://example.com/").build();
        assert_eq!(url, "http://example.com");
    }

    #[test]
    fn test_path_leading_slash_handled() {
        let url = ApiUrlBuilder::new("http://example.com")
            .with_path("/api/v1")
            .build();
        assert_eq!(url, "http://example.com/api/v1");
    }

    #[test]
    fn test_with_fields() {
        let url = ApiUrlBuilder::new("http://example.com")
            .with_fields(&["name", "age"])
            .build();
        assert!(url.contains("fields=name%2Cage"));
        assert!(url.contains('?'));
    }

    #[test]
    fn test_path_and_fields() {
        let url = ApiUrlBuilder::new("http://example.com")
            .with_path("data")
            .with_fields(&["x", "y"])
            .build();
        assert_eq!(url, "http://example.com/data?fields=x%2Cy");
    }

    #[test]
    fn test_geo_ip_builder() {
        use crate::utils::api_builder::GeoIpApiBuilder;
        let url = GeoIpApiBuilder::lookup_ip("8.8.8.8").build();
        assert!(url.starts_with("http://ip-api.com/json/8.8.8.8"));
        assert!(url.contains("fields="));
    }
}
