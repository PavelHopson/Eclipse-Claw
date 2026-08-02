use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
struct Manifest {
    schema_version: u8,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    id: String,
    source_url: String,
    captured_at: String,
    fixture: String,
    sha256: String,
    expected_title: String,
    expected_contains: Vec<String>,
    expected_excludes: Vec<String>,
}

fn fixture_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/public-pages")
}

#[test]
fn fixed_public_pages_meet_quality_contract() {
    let directory = fixture_directory();
    let manifest: Manifest = serde_json::from_str(
        &fs::read_to_string(directory.join("manifest.json")).expect("read benchmark manifest"),
    )
    .expect("parse benchmark manifest");
    assert_eq!(manifest.schema_version, 1);
    assert!(manifest.cases.len() >= 4);

    let mut passed_signals = 0_usize;
    let mut total_signals = 0_usize;
    for case in manifest.cases {
        assert_eq!(case.captured_at, "2026-08-02");
        assert_eq!(case.sha256.len(), 64, "{} must pin SHA-256", case.id);
        let html = fs::read_to_string(directory.join(&case.fixture)).expect("read fixture");
        let result = eclipse_claw_core::extract(&html, Some(&case.source_url))
            .unwrap_or_else(|error| panic!("{} extraction failed: {error}", case.id));
        assert_eq!(
            result.metadata.title.as_deref(),
            Some(case.expected_title.as_str()),
            "{} title",
            case.id
        );

        for expected in &case.expected_contains {
            total_signals += 1;
            if result.content.markdown.contains(expected) {
                passed_signals += 1;
            } else {
                panic!("{} omitted expected signal: {expected}", case.id);
            }
        }
        for noise in &case.expected_excludes {
            total_signals += 1;
            if !result.content.markdown.contains(noise) {
                passed_signals += 1;
            } else {
                panic!("{} retained known noise: {noise}", case.id);
            }
        }

        if case.id == "product-structured-data" {
            total_signals += 1;
            assert!(
                !result.structured_data.is_empty(),
                "product JSON-LD missing"
            );
            passed_signals += 1;
        }
        if case.id == "spa-data-island" {
            total_signals += 1;
            let serialized = serde_json::to_string(&result.structured_data).unwrap();
            assert!(
                serialized.contains("operational"),
                "SPA data island missing"
            );
            passed_signals += 1;
        }
    }

    assert_eq!(passed_signals, total_signals);
    eprintln!(
        "fixed-public-page benchmark: {passed_signals}/{total_signals} quality signals passed"
    );
}
