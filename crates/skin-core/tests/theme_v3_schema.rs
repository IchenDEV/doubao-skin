use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_json(path: &Path) -> Value {
    let bytes =
        fs::read(path).unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("invalid JSON in {}: {error}", path.display()))
}

fn set_pointer(document: &mut Value, pointer: &str, value: Value) {
    let segments = pointer
        .strip_prefix('/')
        .expect("fixture pointers must be absolute")
        .split('/')
        .map(|segment| segment.replace("~1", "/").replace("~0", "~"))
        .collect::<Vec<_>>();
    let (leaf, parents) = segments
        .split_last()
        .expect("fixture pointer cannot be empty");
    let mut cursor = document;
    for segment in parents {
        let object = cursor
            .as_object_mut()
            .expect("fixture mutation parent must be an object");
        cursor = object
            .entry(segment.clone())
            .or_insert_with(|| Value::Object(Default::default()));
    }
    cursor
        .as_object_mut()
        .expect("fixture mutation target must be an object")
        .insert(leaf.clone(), value);
}

#[test]
fn schema_is_valid_draft_2020_12_and_accepts_normative_examples() {
    let root = repo_root();
    let schema_path = root.join("design/theme-standard/theme-v3.schema.json");
    let fixtures = root.join("design/theme-standard/fixtures/v3");
    let schema = read_json(&schema_path);
    assert!(
        jsonschema::draft202012::meta::is_valid(&schema),
        "theme-v3.schema.json must itself satisfy the Draft 2020-12 meta-schema"
    );
    let validator = jsonschema::draft202012::new(&schema).expect("v3 schema must compile");

    for name in [
        "valid-workbuddy.json",
        "valid-doubao-family.json",
        "valid-all-targets.json",
        "valid-full.json",
    ] {
        let manifest = read_json(&fixtures.join(name));
        let errors = validator
            .iter_errors(&manifest)
            .map(|error| error.to_string())
            .collect::<Vec<_>>();
        assert!(errors.is_empty(), "{name} should be valid: {errors:#?}");
    }
}

#[test]
fn mutation_fixtures_lock_strict_v3_shape() {
    let root = repo_root();
    let schema = read_json(&root.join("design/theme-standard/theme-v3.schema.json"));
    let validator = jsonschema::draft202012::new(&schema).expect("v3 schema must compile");
    let fixtures = root.join("design/theme-standard/fixtures/v3");
    let cases = read_json(&fixtures.join("manifest-cases.json"));
    let base_name = cases["base"].as_str().expect("fixture base must be a path");
    let base = read_json(&fixtures.join(base_name));

    for case in cases["cases"]
        .as_array()
        .expect("fixture cases must be an array")
    {
        let name = case["name"].as_str().expect("case name must be a string");
        let mut manifest = base.clone();
        set_pointer(
            &mut manifest,
            case["pointer"]
                .as_str()
                .expect("case pointer must be a string"),
            case["value"].clone(),
        );
        assert_eq!(
            validator.is_valid(&manifest),
            case["valid"]
                .as_bool()
                .expect("case validity must be a bool"),
            "schema result differed for {name}"
        );
    }
}
