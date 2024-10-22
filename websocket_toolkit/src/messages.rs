use serde::{Serialize, Deserialize};
use log::error;

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageFormat {
    Json,
    Cbor,
}

pub struct MessageHandler;

impl MessageHandler {
    pub fn serialize<T: Serialize>(data: &T, format: MessageFormat) -> Vec<u8> {
        match format {
            MessageFormat::Json => Self::private_serialize_json(data),
            MessageFormat::Cbor => Self::private_serialize_cbor(data),
        }
    }

    pub fn deserialize<'a, T: Deserialize<'a>>(data: &'a [u8], format: MessageFormat) -> Option<T> {
        match format {
            MessageFormat::Json => Self::private_deserialize_json(data),
            MessageFormat::Cbor => Self::private_deserialize_cbor(data),
        }
    }

    fn private_serialize_json<T: Serialize>(data: &T) -> Vec<u8> {
        serde_json::to_vec(data).unwrap_or_else(|e| {
            error!("Failed to serialize JSON: {}", e);
            vec![]
        })
    }

    fn private_serialize_cbor<T: Serialize>(data: &T) -> Vec<u8> {
        serde_cbor::to_vec(data).unwrap_or_else(|e| {
            error!("Failed to serialize CBOR: {}", e);
            vec![]
        })
    }

    fn private_deserialize_json<'a, T: Deserialize<'a>>(data: &'a [u8]) -> Option<T> {
        serde_json::from_slice(data).ok()
    }

    fn private_deserialize_cbor<'a, T: Deserialize<'a>>(data: &'a [u8]) -> Option<T> {
        serde_cbor::from_slice(data).ok()
    }
}
