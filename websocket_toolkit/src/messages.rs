#![allow(unused_imports)]
use serde::{Serialize, Deserialize};
use log::{error, info};

/// Enum representing the supported message formats.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum MessageFormat {
    Json,
    Cbor,
}

/// A handler for serializing and deserializing messages.
pub struct MessageHandler;

impl MessageHandler {
    /// Serializes the given data into the specified format.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to serialize.
    /// * `format` - The format to serialize the data into.
    ///
    /// # Returns
    ///
    /// A `Result<Vec<u8>, String>` containing the serialized data or an error message.
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
    /// * `format` - The format of the serialized data.
    ///
    /// # Returns
    ///
    /// A `Result<Option<T>, String>` containing the deserialized data or an error message.
    pub fn deserialize<'a, T: Deserialize<'a>>(data: &'a [u8], format: MessageFormat) -> Result<Option<T>, String> {
        match format {
            MessageFormat::Json => Self::private_deserialize_json(data),
            MessageFormat::Cbor => Self::private_deserialize_cbor(data),
        }
    }

    /// Serializes the data to JSON format.
    fn private_serialize_json<T: Serialize>(data: &T) -> Result<Vec<u8>, String> {
        serde_json::to_vec(data).map_err(|e| {
            error!("Failed to serialize JSON: {}", e);
            format!("Failed to serialize JSON: {}", e)
        })
    }

    /// Serializes the data to CBOR format.
    fn private_serialize_cbor<T: Serialize>(data: &T) -> Result<Vec<u8>, String> {
        serde_cbor::to_vec(data).map_err(|e| {
            error!("Failed to serialize CBOR: {}", e);
            format!("Failed to serialize CBOR: {}", e)
        })
    }

    /// Deserializes data from JSON format.
    fn private_deserialize_json<'a, T: Deserialize<'a>>(data: &'a [u8]) -> Result<Option<T>, String> {
        serde_json::from_slice(data).map(|v| Some(v)).map_err(|e| {
            error!("Failed to deserialize JSON: {}", e);
            format!("Failed to deserialize JSON: {}", e)
        })
    }

    /// Deserializes data from CBOR format.
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

    // #[ignore]
    // #[test]
    // fn test_failed_json_serialization() {
    //     let message = vec![0xFF, 0xFF]; // Invalid data for JSON serialization
    //     let serialized = MessageHandler::serialize(&message, MessageFormat::Json);
    //     assert!(serialized.is_err(), "Expected failure in JSON serialization");

    //     let deserialized: Result<Option<Vec<u8>>, String> = MessageHandler::deserialize(&[], MessageFormat::Json);
    //     assert!(deserialized.is_err(), "Expected failure in JSON deserialization");
    // }

    // #[ignore]
    // #[test]
    // fn test_failed_cbor_serialization() {
    //     let message = vec![0xFF, 0xFF]; // Invalid data for CBOR serialization
    //     let serialized = MessageHandler::serialize(&message, MessageFormat::Cbor);
    //     assert!(serialized.is_err(), "Expected failure in CBOR serialization");

    //     let deserialized: Result<Option<Vec<u8>>, String> = MessageHandler::deserialize(&[], MessageFormat::Cbor);
    //     assert!(deserialized.is_err(), "Expected failure in CBOR deserialization");
    // }
}
