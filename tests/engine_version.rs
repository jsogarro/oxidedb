use oxidedb::engine_version;

#[test]
fn test_engine_version_is_non_empty() {
    let version = engine_version();
    assert!(!version.is_empty());
}
