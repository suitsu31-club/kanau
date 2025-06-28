use thiserror::Error;

#[derive(Debug, Error)]
#[error("Failed to serialize message: {0}")]
/// Error when serializing message.
pub struct SerializeError(pub anyhow::Error);

impl From<anyhow::Error> for SerializeError {
    fn from(e: anyhow::Error) -> Self {
        SerializeError(e)
    }
}

#[cfg(feature = "rkyv")]
impl From<rkyv::rancor::Error> for SerializeError {
    fn from(e: rkyv::rancor::Error) -> Self {
        SerializeError(e.into())
    }
}

#[cfg(feature = "bincode")]
impl From<bincode::error::EncodeError> for SerializeError {
    fn from(e: bincode::error::EncodeError) -> Self {
        SerializeError(e.into())
    }
}

#[cfg(feature = "prost")]
impl From<prost::EncodeError> for SerializeError {
    fn from(e: prost::EncodeError) -> Self {
        SerializeError(e.into())
    }
}

#[cfg(feature = "serde_json")]
impl From<serde_json::Error> for SerializeError {
    fn from(e: serde_json::Error) -> Self {
        SerializeError(e.into())
    }
}

#[derive(Debug, Error)]
#[error("Failed to deserialize message: {0}")]
/// Error when deserializing message.
pub struct DeserializeError(pub anyhow::Error);

impl From<anyhow::Error> for DeserializeError {
    fn from(e: anyhow::Error) -> Self {
        DeserializeError(e)
    }
}

#[cfg(feature = "rkyv")]
impl From<rkyv::rancor::Error> for DeserializeError {
    fn from(e: rkyv::rancor::Error) -> Self {
        DeserializeError(e.into())
    }
}

#[cfg(feature = "bincode")]
impl From<bincode::error::DecodeError> for DeserializeError {
    fn from(e: bincode::error::DecodeError) -> Self {
        DeserializeError(e.into())
    }
}

#[cfg(feature = "prost")]
impl From<prost::DecodeError> for DeserializeError {
    fn from(e: prost::DecodeError) -> Self {
        DeserializeError(e.into())
    }
}

#[cfg(feature = "serde_json")]
impl From<serde_json::Error> for DeserializeError {
    fn from(e: serde_json::Error) -> Self {
        DeserializeError(e.into())
    }
}

/// Message serialization
pub trait MessageSer {
    /// Error type for serialization.
    type SerError: Into<SerializeError>;

    /// Serialize the message to bytes.
    fn to_bytes(self) -> Result<Box<[u8]>, Self::SerError>;
}

/// Message deserialization
pub trait MessageDe {
    /// Error type for deserialization.
    type DeError: Into<DeserializeError>;

    /// Deserialize the message from bytes.
    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::DeError>
    where
        Self: Sized;
}