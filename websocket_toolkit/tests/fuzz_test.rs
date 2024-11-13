#![allow(unused_imports)]
use websocket_toolkit::messages::{MessageHandler, MessageFormat};
use arbitrary::{Arbitrary, Unstructured};
use log::{info, error};

/// Struct to hold arbitrary fuzz test data.
#[derive(Debug)]
struct FuzzTestData {
    content: Vec<u8>,           // Random binary data
    format: MessageFormat,      // Random message format (JSON or CBOR)
}

impl<'a> Arbitrary<'a> for FuzzTestData {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let content = u.arbitrary::<Vec<u8>>()?;  // Generate random binary content
        let format = if u.arbitrary::<bool>()? { MessageFormat::Json } else { MessageFormat::Cbor }; // Randomly pick format
        Ok(FuzzTestData { content, format })
    }
}

/// Fuzzes the deserialization process with random data.
fn fuzz_deserialization(data: &[u8], format: MessageFormat) {
    match MessageHandler::deserialize::<String>(data, format) {
        Ok(Some(result)) => info!("Valid deserialization for {:?}: {:?}", format, result),
        Ok(None) => error!("Deserialization returned None for {:?}", format),
        Err(e) => error!("Deserialization failed for {:?}: {:?}", format, e),
    }
}

#[test]
fn fuzz_test_deserialization() {
    // Sample raw data for fuzzing
    let raw_data = b"random_test_data_for_fuzzing";
    let mut u = Unstructured::new(raw_data);

    // Generate random test data using the Arbitrary trait
    if let Ok(fuzz_data) = FuzzTestData::arbitrary(&mut u) {
        fuzz_deserialization(&fuzz_data.content, fuzz_data.format);
    }
}
