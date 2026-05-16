fn main() {
    #[cfg(windows)]
    embed_resource::compile("src/icon/tracetui.rc", std::iter::empty::<&str>());
}
