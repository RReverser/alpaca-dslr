use actix_web::{web::Json, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::sync::atomic::AtomicU32;

#[derive(Deserialize)]
struct TransactionRequest {
    #[serde(rename = "ClientID")]
    #[allow(dead_code)]
    pub client_id: Option<u32>,
    #[serde(rename = "ClientTransactionID")]
    pub client_transaction_id: Option<u32>,
}

#[derive(Serialize)]
struct TransactionResponse {
    #[serde(rename = "ClientTransactionID")]
    pub client_transaction_id: Option<u32>,
    #[serde(rename = "ServerTransactionID")]
    pub server_transaction_id: u32,
}

impl TransactionRequest {
    pub fn respond(&self) -> TransactionResponse {
        static SERVER_TRANSACTION_ID: AtomicU32 = AtomicU32::new(0);

        TransactionResponse {
            client_transaction_id: self.client_transaction_id,
            server_transaction_id: SERVER_TRANSACTION_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }
}

#[derive(Deserialize)]
pub struct ASCOMRequest<T> {
    #[serde(flatten)]
    transaction: TransactionRequest,
    #[serde(flatten)]
    pub request: T,
}

impl<T> ASCOMRequest<T> {
    pub fn respond_with<U>(self, f: impl FnOnce(T) -> Result<U, ASCOMError>) -> ASCOMResponse<U> {
        ASCOMResponse {
            transaction: self.transaction.respond(),
            result: f(self.request),
        }
    }
}

#[derive(Serialize)]
pub struct ASCOMErrorCode(u16);

impl ASCOMErrorCode {
    /// Generate a driver-specific error code.
    pub const fn new_for_driver(code: u16) -> Self {
        /// The starting value for driver-specific error numbers.
        const DRIVER_BASE: u16 = 0x500;
        /// The maximum value for driver-specific error numbers.
        const DRIVER_MAX: u16 = 0xFFF;

        assert!(code <= DRIVER_MAX - DRIVER_BASE, "Driver error code out of range");
        Self(DRIVER_BASE + code)
    }
}

#[derive(Serialize)]
pub struct ASCOMError {
    #[serde(rename = "ErrorNumber")]
    pub code: ASCOMErrorCode,
    #[serde(rename = "ErrorMessage")]
    pub message: Cow<'static, str>,
}

macro_rules! ascom_error_codes {
  ($(#[doc = $doc:literal] $name:ident = $value:literal,)*) => {
    impl ASCOMErrorCode {
      $(
        #[doc = $doc]
        pub const $name: Self = Self($value);
      )*
    }

    impl ASCOMError {
      $(
        #[doc = $doc]
        pub const $name: Self = Self {
          code: ASCOMErrorCode::$name,
          message: Cow::Borrowed(stringify!($name)),
        };
      )*
    }
  };
}

ascom_error_codes! {
  #[doc = "The requested action is not implemented in this driver."]
  ACTION_NOT_IMPLEMENTED = 0x40C,
  #[doc = "The requested operation can not be undertaken at this time."]
  INVALID_OPERATION = 0x40B,
  #[doc = "Invalid value."]
  INVALID_VALUE = 0x401,
  #[doc = "The attempted operation is invalid because the mount is currently in a Parked state."]
  INVALID_WHILE_PARKED = 0x408,
  #[doc = "The attempted operation is invalid because the mount is currently in a Slaved state."]
  INVALID_WHILE_SLAVED = 0x409,
  #[doc = "The communications channel is not connected."]
  NOT_CONNECTED = 0x407,
  #[doc = "Property or method not implemented."]
  NOT_IMPLEMENTED = 0x400,
  #[doc = "The requested item is not present in the ASCOM cache."]
  NOT_IN_CACHE = 0x40D,
  #[doc = "Settings error."]
  SETTINGS = 0x40A,
  #[doc = "'catch-all' error code used when nothing else was specified."]
  UNSPECIFIED = 0x4FF,
  #[doc = "A value has not been set."]
  VALUE_NOT_SET = 0x402,
}

#[derive(Serialize)]
pub struct ASCOMResponse<T> {
    #[serde(flatten)]
    transaction: TransactionResponse,
    #[serde(flatten, serialize_with = "serialize_result", bound = "T: Serialize")]
    pub result: Result<T, ASCOMError>,
}

fn serialize_result<T: Serialize, S: serde::Serializer>(value: &Result<T, ASCOMError>, serializer: S) -> Result<S::Ok, S::Error> {
    match value {
        Ok(value) => value.serialize(serializer),
        Err(error) => error.serialize(serializer),
    }
}

impl<T: Serialize> Responder for ASCOMResponse<T> {
    type Body = <Json<Self> as Responder>::Body;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        Json(self).respond_to(req)
    }
}
