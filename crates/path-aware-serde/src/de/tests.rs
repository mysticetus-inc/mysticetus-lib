use serde::Deserialize;

use super::Deserializer;

#[test]
#[allow(dead_code)]
fn test_de() {
    #[derive(Debug, Deserialize)]
    struct Outer {
        nested: Middle,
    }

    #[derive(Debug, Deserialize)]
    struct Middle {
        even_more_nested: Inner,
    }

    #[derive(Debug, Deserialize)]
    struct Inner {
        seq: Vec<SeqItem>,
    }

    #[derive(Debug, Deserialize)]
    struct SeqItem {
        nested_in_seq: usize,
    }

    const RAW_INVALID_JSON: &str = r#"{
        "nested": {
            "even_more_nested": {
                "seq": [
                    { "nested_in_seq": 0 },
                    { "nested_in_seq": 0 },
                    { "nested_in_seq": null }
                ]
            }
        }
    }"#;

    const EXPECTED_ERROR_PATH: &str = "nested.even_more_nested.seq[2].nested_in_seq";

    let mut inner_de = serde_json::Deserializer::from_str(RAW_INVALID_JSON);
    let de = Deserializer::new(&mut inner_de);

    let parsed_path = EXPECTED_ERROR_PATH
        .parse::<crate::Path>()
        .expect("could not parse expected path");

    let err = Outer::deserialize(de).expect_err("invalid 'null' field will cause this to fail");

    let path = err.path().expect("we should have a path");

    assert_eq!(&parsed_path, path);

    let path_str = path.to_string();
    assert_eq!(&path_str, EXPECTED_ERROR_PATH);
}
