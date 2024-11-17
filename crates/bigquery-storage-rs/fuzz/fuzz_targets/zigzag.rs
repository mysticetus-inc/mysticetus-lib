#![no_main]
use bigquery_storage_rs::write::proto::zigzag;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|pair: (usize, isize)| {
    let (unsigned, signed) = pair;

    assert_eq!(
        unsigned,
        zigzag::encode(zigzag::decode(unsigned)),
        "decode -> encode"
    );
    assert_eq!(
        signed,
        zigzag::decode(zigzag::encode(signed)),
        "encode -> decode"
    );
});
