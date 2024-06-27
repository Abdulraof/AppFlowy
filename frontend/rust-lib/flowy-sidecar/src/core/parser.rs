use crate::core::rpc_object::RpcObject;
use crate::core::rpc_peer::ResponsePayload;
use crate::error::{ReadError, RemoteError};
use serde_json::{json, Value as JsonValue};
use std::io::BufRead;

#[derive(Debug, Default)]
pub struct MessageReader(String);

impl MessageReader {
  /// Attempts to read the next line from the stream and parse it as
  /// an RPC object.
  ///
  /// # Errors
  ///
  /// This function will return an error if there is an underlying
  /// I/O error, if the stream is closed, or if the message is not
  /// a valid JSON object.
  pub fn next<R: BufRead>(&mut self, reader: &mut R) -> Result<RpcObject, ReadError> {
    self.0.clear();
    let _ = reader.read_line(&mut self.0)?;
    if self.0.is_empty() {
      Err(ReadError::Disconnect)
    } else {
      self.parse(&self.0)
    }
  }

  /// Attempts to parse a &str as an RPC Object.
  ///
  /// This should not be called directly unless you are writing tests.
  #[doc(hidden)]
  pub fn parse(&self, s: &str) -> Result<RpcObject, ReadError> {
    match serde_json::from_str::<JsonValue>(s) {
      Ok(val) => {
        if !val.is_object() {
          Err(ReadError::NotObject(s.to_string()))
        } else {
          Ok(val.into())
        }
      },
      Err(_) => Ok(RpcObject(json!({"message": s.to_string()}))),
    }
  }
}

pub type RequestId = u64;
#[derive(Debug, Clone)]
/// An RPC call, which may be either a notification or a request.
pub enum Call<R> {
  Message(JsonValue),
  /// An id and an RPC Request
  Request(RequestId, R),
  /// A malformed request: the request contained an id, but could
  /// not be parsed. The client will receive an error.
  InvalidRequest(RequestId, RemoteError),
}

pub trait ResponseParser {
  type ValueType;
  fn parse_response(payload: JsonValue) -> Result<Self::ValueType, RemoteError>;
}

pub struct ChatResponseParser;
impl ResponseParser for ChatResponseParser {
  type ValueType = String;

  fn parse_response(json: JsonValue) -> Result<Self::ValueType, RemoteError> {
    if json.is_object() {
      if let Some(data) = json.get("data") {
        if let Some(message) = data.as_str() {
          return Ok(message.to_string());
        }
      }
    }
    return Err(RemoteError::InvalidResponse(json));
  }
}

pub struct ChatRelatedQuestionsResponseParser;
impl ResponseParser for ChatRelatedQuestionsResponseParser {
  type ValueType = Vec<JsonValue>;

  fn parse_response(json: JsonValue) -> Result<Self::ValueType, RemoteError> {
    if json.is_object() {
      if let Some(data) = json.get("data") {
        if let Some(values) = data.as_array() {
          return Ok(values.clone());
        }
      }
    }
    return Err(RemoteError::InvalidResponse(json));
  }
}

pub struct SimilarityResponseParser;
impl ResponseParser for SimilarityResponseParser {
  type ValueType = f64;

  fn parse_response(json: JsonValue) -> Result<Self::ValueType, RemoteError> {
    if json.is_object() {
      if let Some(data) = json.get("data") {
        if let Some(score) = data.get("score").and_then(|v| v.as_f64()) {
          return Ok(score);
        }
      }
    }

    return Err(RemoteError::InvalidResponse(json));
  }
}
