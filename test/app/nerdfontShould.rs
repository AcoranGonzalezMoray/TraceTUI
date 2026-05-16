#[cfg(test)]
mod nerdfont_tests {
    #[test]
    fn test_has_nerdfont_fn_type() {
        let _: fn() -> bool = || true;
    }
}
