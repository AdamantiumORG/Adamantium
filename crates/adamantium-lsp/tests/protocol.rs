#[test]
fn initialize_contract_is_valid_json_with_formatting_support() {
    let result = adamantium_lsp::initialize_result();
    assert_eq!(result["serverInfo"]["name"], "adamantium-lsp");
    assert_eq!(
        result["capabilities"]["documentFormattingProvider"],
        true
    );
}
