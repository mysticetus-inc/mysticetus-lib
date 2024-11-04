use enc_rs::Iso8211Reader;
use enc_rs::iso8211::Iso8211File;
use intern::local::LocalInterner;

const TEST_FILE: &str = "./tests/ENC_ROOT/US4MI2SH/US4MI2SH.000";

#[ignore]
#[test]
fn test_read() -> Result<(), enc_rs::iso8211::Iso8211Error> {
    let file = Iso8211Reader::new(std::fs::File::open(TEST_FILE)?);
    let mut interner = LocalInterner::new();

    let _iso_file = Iso8211File::read(file, &mut interner)?;

    Ok(())
}
