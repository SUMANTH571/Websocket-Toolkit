#![allow(unused_imports)]

//! Fuzz tests for the `websocket_toolkit` crate.
//!
//! This module contains tests to verify the robustness of the deserialization logic
//! by using random, arbitrary input data. These tests help ensure that the
//! `MessageHandler` can handle unexpected or malformed data gracefully.

use websocket_toolkit::messages::{MessageHandler, MessageFormat};
use arbitrary::{Arbitrary, Unstructured};
use log::{info, error};

/// A struct representing fuzz test data for deserialization.
/// 
/// This struct includes random binary data and a randomly selected message format (JSON or CBOR),
/// which are used to test the deserialization logic.
///
/// # Fields
/// * `content` - Random binary data used as the input for deserialization.
/// * `format` - Randomly selected message format, either `MessageFormat::Json` or `MessageFormat::Cbor`.
#[derive(Debug)]
struct FuzzTestData {
    /// Random binary data.
    content: Vec<u8>,
    /// Random message format, either `MessageFormat::Json` or `MessageFormat::Cbor`.
    format: MessageFormat,
}

impl<'a> Arbitrary<'a> for FuzzTestData {
    /// Generates random test data for fuzzing purposes.
    ///
    /// # Arguments
    /// * `u` - An instance of `Unstructured`, which is used to generate random data.
    ///
    /// # Returns
    /// * `Ok(FuzzTestData)` - A struct containing random binary data and a message format.
    /// * `Err(arbitrary::Error)` - An error if the generation fails.
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let content = u.arbitrary::<Vec<u8>>()?; // Generate random binary data
        let format = if u.arbitrary::<bool>()? {
            MessageFormat::Json
        } else {
            MessageFormat::Cbor
        }; // Randomly pick between JSON and CBOR formats
        Ok(FuzzTestData { content, format })
    }
}

/// Fuzzes the deserialization process by testing with arbitrary input data.
///
/// This function attempts to deserialize the given binary data into a `String`
/// using the specified format (`JSON` or `CBOR`) and logs the results.
///
/// # Arguments
/// * `data` - A byte slice containing the input data to deserialize.
/// * `format` - The format of the input data (`MessageFormat::Json` or `MessageFormat::Cbor`).
///
/// # Logs
/// * Logs an informational message if deserialization succeeds.
/// * Logs an error message if deserialization fails or returns `None`.
fn fuzz_deserialization(data: &[u8], format: MessageFormat) {
    match MessageHandler::deserialize::<String>(data, format) {
        Ok(Some(result)) => info!("Valid deserialization for {:?}: {:?}", format, result),
        Ok(None) => error!("Deserialization returned None for {:?}", format),
        Err(e) => error!("Deserialization failed for {:?}: {:?}", format, e),
    }
}

/// A fuzz test to validate the deserialization logic with random data.
///
/// This test generates random data using the `Arbitrary` trait and attempts
/// to deserialize it using the `fuzz_deserialization` function. It verifies
/// the ability of the deserialization logic to handle unexpected or malformed input.
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
