#![allow(unused_imports)]
use serde::{Serialize, Deserialize};
use log::{error, info};
use arbitrary::Arbitrary;

/// Implementation of the `Arbitrary` trait for `MessageFormat`.
///
/// This allows `MessageFormat` to be used in fuzz testing by generating random values.
impl<'a> Arbitrary<'a> for MessageFormat {
    /// Generates a random `MessageFormat`.
    ///
    /// # Arguments
    ///
    /// * `u` - The `Unstructured` data used to generate random values.
    ///
    /// # Returns
    ///
    /// A random instance of `MessageFormat`.
    ///
    /// # Errors
    ///
    /// Returns an error if random generation fails.
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let choice = u.int_in_range(0..=1)?;
        match choice {
            0 => Ok(MessageFormat::Json),
            1 => Ok(MessageFormat::Cbor),
            _ => unreachable!(),
        }
    }
}

/// Enum representing the supported message formats for serialization and deserialization.
///
/// This enum is used to specify whether messages should be serialized or deserialized
/// in JSON or CBOR formats.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum MessageFormat {
    /// JSON format.
    Json,
    /// CBOR format.
    Cbor,
}

/// A handler for serializing and deserializing messages.
///
/// Provides utility functions to handle message encoding and decoding in JSON and CBOR formats.
pub struct MessageHandler;

impl MessageHandler {
    /// Serializes the given data into the specified format.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to serialize.
    /// * `format` - The format to serialize the data into (`MessageFormat::Json` or `MessageFormat::Cbor`).
    ///
    /// # Returns
    ///
    /// A `Result` containing the serialized data as a `Vec<u8>` on success, or an error message as a `String` on failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use websocket_toolkit::messages::{MessageHandler, MessageFormat};
    ///
    /// let message = "Hello, WebSocket!";
    /// let serialized = MessageHandler::serialize(&message, MessageFormat::Json).unwrap();
    /// assert!(!serialized.is_empty());
    /// ```
    pub fn serialize<T: Serialize>(data: &T, format: MessageFormat) -> Result<Vec<u8>, String> {
        match format {
            MessageFormat::Json => Self::private_serialize_json(data),
            MessageFormat::Cbor => Self::private_serialize_cbor(data),
        }
    }

    /// Deserializes the given byte slice into the specified type.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte slice containing the serialized data.
    /// * `format` - The format of the serialized data (`MessageFormat::Json` or `MessageFormat::Cbor`).
    ///
    /// # Returns
    ///
    /// A `Result` containing the deserialized data as an `Option<T>` on success, or an error message as a `String` on failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use websocket_toolkit::messages::{MessageHandler, MessageFormat};
    ///
    /// let serialized = b"\"Hello, WebSocket!\"";
    /// let deserialized: Option<String> = MessageHandler::deserialize(serialized, MessageFormat::Json).unwrap();
    /// assert_eq!(deserialized, Some("Hello, WebSocket!".to_string()));
    /// ```
    pub fn deserialize<'a, T: Deserialize<'a>>(data: &'a [u8], format: MessageFormat) -> Result<Option<T>, String> {
        match format {
            MessageFormat::Json => Self::private_deserialize_json(data),
            MessageFormat::Cbor => Self::private_deserialize_cbor(data),
        }
    }

    /// Serializes the data to JSON format.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to serialize.
    ///
    /// # Returns
    ///
    /// A `Result` containing the serialized JSON as a `Vec<u8>` on success, or an error message on failure.
    fn private_serialize_json<T: Serialize>(data: &T) -> Result<Vec<u8>, String> {
        serde_json::to_vec(data).map_err(|e| {
            error!("Failed to serialize JSON: {}", e);
            format!("Failed to serialize JSON: {}", e)
        })
    }

    /// Serializes the data to CBOR format.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to serialize.
    ///
    /// # Returns
    ///
    /// A `Result` containing the serialized CBOR as a `Vec<u8>` on success, or an error message on failure.
    fn private_serialize_cbor<T: Serialize>(data: &T) -> Result<Vec<u8>, String> {
        serde_cbor::to_vec(data).map_err(|e| {
            error!("Failed to serialize CBOR: {}", e);
            format!("Failed to serialize CBOR: {}", e)
        })
    }

    /// Deserializes data from JSON format.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte slice containing the serialized JSON data.
    ///
    /// # Returns
    ///
    /// A `Result` containing the deserialized data as an `Option<T>` on success, or an error message on failure.
    fn private_deserialize_json<'a, T: Deserialize<'a>>(data: &'a [u8]) -> Result<Option<T>, String> {
        serde_json::from_slice(data).map(|v| Some(v)).map_err(|e| {
            error!("Failed to deserialize JSON: {}", e);
            format!("Failed to deserialize JSON: {}", e)
        })
    }

    /// Deserializes data from CBOR format.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte slice containing the serialized CBOR data.
    ///
    /// # Returns
    ///
    /// A `Result` containing the deserialized data as an `Option<T>` on success, or an error message on failure.
    fn private_deserialize_cbor<'a, T: Deserialize<'a>>(data: &'a [u8]) -> Result<Option<T>, String> {
        serde_cbor::from_slice(data).map(|v| Some(v)).map_err(|e| {
            error!("Failed to deserialize CBOR: {}", e);
            format!("Failed to deserialize CBOR: {}", e)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests JSON serialization and deserialization.
    #[test]
    fn test_json_serialization() {
        let message = "Hello, WebSocket!";
        let serialized = MessageHandler::serialize(&message, MessageFormat::Json);
        assert!(serialized.is_ok(), "Expected successful JSON serialization");
        let serialized = serialized.unwrap();
        assert!(!serialized.is_empty(), "Expected non-empty JSON serialized data");

        let deserialized: Result<Option<String>, String> = MessageHandler::deserialize(&serialized, MessageFormat::Json);
        assert!(deserialized.is_ok(), "Expected successful JSON deserialization");
        assert_eq!(deserialized.unwrap(), Some(message.to_string()), "Expected deserialized JSON to match original message");
    }

    /// Tests CBOR serialization and deserialization.
    #[test]
    fn test_cbor_serialization() {
        let message = "Hello, WebSocket!";
        let serialized = MessageHandler::serialize(&message, MessageFormat::Cbor);
        assert!(serialized.is_ok(), "Expected successful CBOR serialization");
        let serialized = serialized.unwrap();
        assert!(!serialized.is_empty(), "Expected non-empty CBOR serialized data");

        let deserialized: Result<Option<String>, String> = MessageHandler::deserialize(&serialized, MessageFormat::Cbor);
        assert!(deserialized.is_ok(), "Expected successful CBOR deserialization");
        assert_eq!(deserialized.unwrap(), Some(message.to_string()), "Expected deserialized CBOR to match original message");
    }
}
