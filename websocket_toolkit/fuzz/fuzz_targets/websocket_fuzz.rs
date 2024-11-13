#![no_main]
#![allow(unused_imports)]
use libfuzzer_sys::fuzz_target;
use websocket_toolkit::messages::{MessageHandler, MessageFormat};
use arbitrary::{Arbitrary, Unstructured};
use log::{info, error};

// Define a structure that holds random data for fuzzing
#[derive(Arbitrary, Debug)]
struct FuzzData {
    content: Vec<u8>,
    format: MessageFormat,
}

// Define a fuzz target that will fuzz the deserialization process
fuzz_target!(|data: &[u8]| {
    let mut unstructured = Unstructured::new(data);

    if let Ok(fuzz_data) = FuzzData::arbitrary(&mut unstructured) {
        // Attempt to deserialize the data in the specified format
        match MessageHandler::deserialize::<String>(&fuzz_data.content, fuzz_data.format) {
            Some(result) => info!("Successfully deserialized: {:?}", result),
            None => error!("Deserialization failed for format {:?}", fuzz_data.format),
        }
    }
});
