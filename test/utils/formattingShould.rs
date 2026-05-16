#[cfg(test)]
mod formatting_tests {
    use crate::utils::formatting::format_bytes;

    #[test]
    fn test_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1), "1 B");
        assert_eq!(format_bytes(1023), "1023 B");
    }

    #[test]
    fn test_kilobytes() {
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1024 * 1024 - 1), "1024.00 KB");
    }

    #[test]
    fn test_megabytes() {
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(2 * 1024 * 1024), "2.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 - 1), "1024.00 MB");
    }

    #[test]
    fn test_gigabytes() {
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_bytes(2 * 1024 * 1024 * 1024), "2.00 GB");
    }

    #[test]
    fn test_terabytes() {
        let tb = 1024u64 * 1024 * 1024 * 1024;
        assert_eq!(format_bytes(tb), "1.00 TB");
        assert_eq!(format_bytes(2 * tb), "2.00 TB");
    }
}
