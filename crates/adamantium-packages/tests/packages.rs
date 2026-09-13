#[test]
fn formats_release_asset() {
    assert_eq!(
        adamantium_packages::release_asset("1.2.0").unwrap(),
        "adamantium_packet_1_2_0.wasm"
    );
}
