#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .env(
            "PATH",
            format!(
                "{}:{}",
                std::env::current_dir()
                    .expect("a current dir should exist")
                    .join("node_modules")
                    .join(".bin")
                    .display(),
                std::env!("PATH")
            ),
        )
        .case("tests/cmd/*.toml")
        .case("README.md");
}
