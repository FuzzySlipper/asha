use protocol_game_extension::{GameplayContractRef, GameplayEventSchemaDeclaration};
use std::any::{Any, TypeId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameplayCodecError {
    UnknownEvent { event: String },
    WrongPayloadType { event: String },
    Encode { event: String, message: String },
    Decode { event: String, message: String },
}

impl core::fmt::Display for GameplayCodecError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownEvent { event } => write!(f, "no codec for event `{event}`"),
            Self::WrongPayloadType { event } => {
                write!(f, "payload type does not match codec for `{event}`")
            }
            Self::Encode { event, message } => {
                write!(f, "codec for `{event}` could not encode: {message}")
            }
            Self::Decode { event, message } => {
                write!(f, "codec for `{event}` could not decode: {message}")
            }
        }
    }
}

impl std::error::Error for GameplayCodecError {}

pub struct TypedGameplayEventCodec<T: 'static> {
    declaration: GameplayEventSchemaDeclaration,
    encode: fn(&T) -> Result<Vec<u8>, String>,
    decode: fn(&[u8]) -> Result<T, String>,
}

/// Opaque, heterogeneous codec token for static provider composition. A
/// downstream provider constructs it from one concrete typed codec; only the
/// closed registry can erase and invoke the codec.
pub struct GameplayEventCodecRegistration {
    pub(crate) codec: RegisteredCodec,
}

impl GameplayEventCodecRegistration {
    pub fn typed<T: 'static>(codec: TypedGameplayEventCodec<T>) -> Self {
        Self {
            codec: codec.into(),
        }
    }
}

impl<T: 'static> TypedGameplayEventCodec<T> {
    pub fn new(
        declaration: GameplayEventSchemaDeclaration,
        encode: fn(&T) -> Result<Vec<u8>, String>,
        decode: fn(&[u8]) -> Result<T, String>,
    ) -> Self {
        Self {
            declaration,
            encode,
            decode,
        }
    }

    pub fn declaration(&self) -> &GameplayEventSchemaDeclaration {
        &self.declaration
    }
}

pub(crate) trait ErasedGameplayEventCodec {
    fn declaration(&self) -> &GameplayEventSchemaDeclaration;
    fn payload_type_id(&self) -> TypeId;
    fn encode_any(&self, payload: &dyn Any) -> Result<Vec<u8>, GameplayCodecError>;
    fn decode_any(&self, bytes: &[u8]) -> Result<Box<dyn Any>, GameplayCodecError>;
}

impl<T: 'static> ErasedGameplayEventCodec for TypedGameplayEventCodec<T> {
    fn declaration(&self) -> &GameplayEventSchemaDeclaration {
        &self.declaration
    }

    fn payload_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn encode_any(&self, payload: &dyn Any) -> Result<Vec<u8>, GameplayCodecError> {
        let event = self.declaration.event.key();
        let typed =
            payload
                .downcast_ref::<T>()
                .ok_or_else(|| GameplayCodecError::WrongPayloadType {
                    event: event.clone(),
                })?;
        (self.encode)(typed).map_err(|message| GameplayCodecError::Encode { event, message })
    }

    fn decode_any(&self, bytes: &[u8]) -> Result<Box<dyn Any>, GameplayCodecError> {
        let event = self.declaration.event.key();
        (self.decode)(bytes)
            .map(|payload| Box::new(payload) as Box<dyn Any>)
            .map_err(|message| GameplayCodecError::Decode { event, message })
    }
}

pub(crate) struct RegisteredCodec {
    pub event: GameplayContractRef,
    pub codec_id: String,
    pub codec: Box<dyn ErasedGameplayEventCodec>,
}

impl<T: 'static> From<TypedGameplayEventCodec<T>> for RegisteredCodec {
    fn from(codec: TypedGameplayEventCodec<T>) -> Self {
        Self {
            event: codec.declaration.event.clone(),
            codec_id: codec.declaration.codec_id.clone(),
            codec: Box::new(codec),
        }
    }
}
