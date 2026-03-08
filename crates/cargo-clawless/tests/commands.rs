#[test]
fn commands() {
    trycmd::TestCases::new()
        .case("tests/commands/*.toml")
        .case("../../README.md");
}
