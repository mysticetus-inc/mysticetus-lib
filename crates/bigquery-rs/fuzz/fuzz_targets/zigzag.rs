#![no_main]
use libfuzzer_sys::fuzz_target;

use bigquery_rs::storage::write::proto::zigzag;

fuzz_target!(|pair: (usize, isize)| {
    let (unsigned, signed) = pair;

    assert_eq!(unsigned, zigzag::encode(zigzag::decode(unsigned)), "decode -> encode");
    assert_eq!(signed, zigzag::decode(zigzag::encode(signed)), "encode -> decode");
});
