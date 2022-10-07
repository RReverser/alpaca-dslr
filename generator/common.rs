use actix_web::{
    dev::Payload,
    error::ErrorBadRequest,
    http::Method,
    web::{Bytes, Json, Query},
    FromRequest, HttpMessage, HttpRequest, HttpResponse, Responder,
};
use pin_project::pin_project;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::borrow::Cow;
use std::future::Future;
use std::pin::Pin;
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

// #[derive(Deserialize)]
pub struct ASCOMRequest<T> {
    // #[serde(flatten)]
    transaction: TransactionRequest,
    // #[serde(flatten)]
    pub request: T,
}

impl<T: DeserializeOwned> ASCOMRequest<T> {
    /// This awkward machinery is to accomodate for the fact that the serde(flatten)
    /// breaks all deserialization because it collects data into an internal representation
    /// first and then can't recover other types from string values stored from the query string.
    ///
    /// See https://github.com/nox/serde_urlencoded/issues/33.
    fn from_encoded_params(encoded_params: &[u8]) -> Result<Self, actix_web::Error> {
        let mut transaction_params = form_urlencoded::Serializer::new(String::new());
        let mut request_params = form_urlencoded::Serializer::new(String::new());

        for (key, value) in form_urlencoded::parse(encoded_params) {
            match key.as_ref() {
                "ClientID" | "ClientTransactionID" => {
                    transaction_params.append_pair(&key, &value);
                }
                _ => {
                    request_params.append_pair(&key, &value);
                }
            }
        }

        Ok(ASCOMRequest {
            transaction: Query::<TransactionRequest>::from_query(&transaction_params.finish())?.into_inner(),
            request: Query::<T>::from_query(&request_params.finish())?.into_inner(),
        })
    }
}

#[pin_project]
pub struct BodyParamsFuture<T> {
    #[pin]
    inner: <Bytes as FromRequest>::Future,
    _phantom: std::marker::PhantomData<fn() -> ASCOMRequest<T>>,
}

impl<T> BodyParamsFuture<T> {
    fn new(fut: <Bytes as FromRequest>::Future) -> Self {
        BodyParamsFuture {
            inner: fut,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: DeserializeOwned> Future for BodyParamsFuture<T> {
    type Output = Result<ASCOMRequest<T>, actix_web::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        self.project().inner.poll(cx).map(|encoded_params| ASCOMRequest::from_encoded_params(encoded_params?.as_ref()))
    }
}

impl<T: 'static + DeserializeOwned> FromRequest for ASCOMRequest<T> {
    type Error = actix_web::Error;
    type Future = actix_utils::future::Either<actix_utils::future::Ready<Result<Self, actix_web::Error>>, BodyParamsFuture<T>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        if req.method() == Method::GET {
            actix_utils::future::Either::left(actix_utils::future::ready(ASCOMRequest::from_encoded_params(req.query_string().as_bytes())))
        } else {
            if let Err(err) = req.mime_type().map_err(actix_web::Error::from).and_then(|mime| {
                if mime != Some(mime::APPLICATION_WWW_FORM_URLENCODED) {
                    Err(ErrorBadRequest("Invalid content type"))
                } else {
                    Ok(())
                }
            }) {
                return actix_utils::future::Either::left(actix_utils::future::err(err));
            }
            actix_utils::future::Either::right(BodyParamsFuture::new(Bytes::from_request(req, payload)))
        }
    }
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
