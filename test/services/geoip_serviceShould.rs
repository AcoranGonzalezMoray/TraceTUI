#[cfg(test)]
mod geoip_service_tests {
    use crate::services::geoip_service::GeoIpService;

    #[test]
    fn test_is_private_ip_empty() {
        assert!(GeoIpService::is_private_ip(""));
    }

    #[test]
    fn test_is_private_ip_localhost() {
        assert!(GeoIpService::is_private_ip("127.0.0.1"));
    }

    #[test]
    fn test_is_private_ip_zero() {
        assert!(GeoIpService::is_private_ip("0.0.0.0"));
    }

    #[test]
    fn test_is_private_ip_ipv6_localhost() {
        assert!(GeoIpService::is_private_ip("::1"));
    }

    #[test]
    fn test_is_private_ip_192_168() {
        assert!(GeoIpService::is_private_ip("192.168.1.1"));
    }

    #[test]
    fn test_is_private_ip_10_dot() {
        assert!(GeoIpService::is_private_ip("10.0.0.1"));
    }

    #[test]
    fn test_is_private_ip_172_16() {
        assert!(GeoIpService::is_private_ip("172.16.0.1"));
    }

    #[test]
    fn test_is_private_ip_169_254() {
        assert!(GeoIpService::is_private_ip("169.254.1.1"));
    }

    #[test]
    fn test_is_private_ip_public() {
        assert!(!GeoIpService::is_private_ip("8.8.8.8"));
    }

    #[test]
    fn test_is_private_ip_public_1_1_1_1() {
        assert!(!GeoIpService::is_private_ip("1.1.1.1"));
    }

    #[test]
    fn test_get_flag_emoji_empty() {
        let flag = GeoIpService::get_flag_emoji("");
        assert_eq!(flag, "\u{f0320}");
    }

    #[test]
    fn test_get_flag_emoji_single_char() {
        let flag = GeoIpService::get_flag_emoji("U");
        assert_eq!(flag, "\u{f0320}");
    }

    #[test]
    fn test_get_flag_emoji_us() {
        let flag = GeoIpService::get_flag_emoji("US");
        assert!(!flag.is_empty());
        assert!(flag.len() > 2);
    }

    #[test]
    fn test_get_flag_emoji_jp() {
        let flag = GeoIpService::get_flag_emoji("JP");
        assert!(!flag.is_empty());
    }

    #[test]
    fn test_get_flag_emoji_lowercase() {
        let flag = GeoIpService::get_flag_emoji("us");
        assert!(!flag.is_empty());
    }

    fn make_test_geo_info(
        country: &str,
        city: &str,
        isp: &str,
        org: &str,
        lat: f64,
        lon: f64,
    ) -> crate::services::geoip_service::GeoInfo {
        crate::services::geoip_service::GeoInfo {
            status: "success".to_string(),
            country: country.to_string(),
            countryCode: country.to_string(),
            city: city.to_string(),
            regionName: None,
            zip: None,
            isp: isp.to_string(),
            org: org.to_string(),
            lat,
            lon,
            timezone: None,
            as_info: None,
            query: None,
            mobile: None,
            proxy: None,
            hosting: None,
        }
    }

    #[test]
    fn test_geo_info_debug() {
        let info = make_test_geo_info(
            "US",
            "Mountain View",
            "Google",
            "Google LLC",
            37.42,
            -122.08,
        );
        let debug = format!("{:?}", info);
        assert!(debug.contains("success"));
        assert!(debug.contains("Google"));
    }

    #[test]
    fn test_geo_info_clone() {
        let info = make_test_geo_info("US", "NYC", "ISP", "Org", 40.0, -74.0);
        let cloned = info.clone();
        assert_eq!(cloned.city, "NYC");
        assert_eq!(cloned.lat, 40.0);
    }

    #[test]
    fn test_geoip_service_new() {
        let service = GeoIpService::new();
        assert!(service.is_ok());
    }
}
