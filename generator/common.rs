use actix_web::{
    dev::{Payload, ServiceRequest, ServiceResponse},
    error::{BlockingError, ErrorBadRequest},
    http::Method,
    web::{Bytes, Json, Query},
    FromRequest, HttpMessage, HttpRequest, HttpResponse, Responder,
};

use pin_project::pin_project;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::{fmt, pin::Pin};
use std::{future::Future, sync::Mutex};
use tracing::Span;
use tracing_actix_web::RootSpan;

#[derive(Serialize, Deserialize)]
struct TransactionIds {
    #[serde(rename = "ClientID")]
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub client_id: Option<u32>,
    #[serde(rename = "ClientTransactionID")]
    pub client_transaction_id: Option<u32>,
    #[serde(rename = "ServerTransactionID")]
    #[serde(skip_deserializing)]
    #[serde(default = "generate_server_transaction_id")]
    pub server_transaction_id: u32,
}

impl TransactionIds {
    pub fn record(&self, root_span: RootSpan) {
        if let Some(client_id) = self.client_id {
            root_span.record("client_id", client_id);
        }

        if let Some(client_transaction_id) = self.client_transaction_id {
            root_span.record("client_transaction_id", client_transaction_id);
        }

        root_span.record("server_transaction_id", self.server_transaction_id);
    }
}

fn generate_server_transaction_id() -> u32 {
    static SERVER_TRANSACTION_ID: AtomicU32 = AtomicU32::new(0);
    SERVER_TRANSACTION_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

// #[derive(Deserialize)]
pub(crate) struct ASCOMRequest {
    // #[serde(flatten)]
    transaction: TransactionIds,
    // #[serde(flatten)]
    request_encoded_params: String,
}

impl ASCOMRequest {
    /// This awkward machinery is to accomodate for the fact that the serde(flatten)
    /// breaks all deserialization because it collects data into an internal representation
    /// first and then can't recover other types from string values stored from the query string.
    ///
    /// See [nox/serde_urlencoded#33](https://github.com/nox/serde_urlencoded/issues/33).
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
            transaction: Query::<TransactionIds>::from_query(&transaction_params.finish())?.into_inner(),
            request_encoded_params: request_params.finish(),
        })
    }
}

#[pin_project]
pub(crate) struct BodyParamsFuture {
    #[pin]
    inner: <Bytes as FromRequest>::Future,
}

impl BodyParamsFuture {
    fn new(inner: <Bytes as FromRequest>::Future) -> Self {
        BodyParamsFuture { inner }
    }
}

impl Future for BodyParamsFuture {
    type Output = Result<ASCOMRequest, actix_web::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        self.project().inner.poll(cx).map(|encoded_params| ASCOMRequest::from_encoded_params(encoded_params?.as_ref()))
    }
}

impl FromRequest for ASCOMRequest {
    type Error = actix_web::Error;
    type Future = actix_utils::future::Either<actix_utils::future::Ready<Result<Self, actix_web::Error>>, BodyParamsFuture>;

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

impl ASCOMRequest {
    pub fn respond_with(self, root_span: RootSpan, f: impl FnOnce(&str) -> Result<serde_json::Value, ASCOMError> + Send + 'static) -> impl Future<Output = Result<ASCOMResponse, BlockingError>> {
        self.transaction.record(root_span);

        actix_web::web::block(move || ASCOMResponse {
            transaction: self.transaction,
            result: f(&self.request_encoded_params),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct ASCOMError {
    #[serde(rename = "ErrorNumber")]
    pub code: ASCOMErrorCode,
    #[serde(rename = "ErrorMessage")]
    pub message: Cow<'static, str>,
}

impl ASCOMError {
    pub fn new(code: ASCOMErrorCode, message: impl Into<Cow<'static, str>>) -> Self {
        Self { code, message: message.into() }
    }
}

pub type ASCOMResult<T = ()> = Result<T, ASCOMError>;

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
pub(crate) struct ASCOMResponse {
    #[serde(flatten)]
    transaction: TransactionIds,
    #[serde(flatten, serialize_with = "serialize_result")]
    pub result: ASCOMResult<serde_json::Value>,
}

pub(crate) trait ToResponse {
    type Response: Serialize;

    fn to_response(self) -> Self::Response;
}

impl ToResponse for () {
    type Response = ();

    fn to_response(self) {}
}

#[derive(Serialize)]
pub(crate) struct ValueResponse<T> {
    /// Returned value.
    value: T,
}

impl<T> From<T> for ValueResponse<T> {
    fn from(value: T) -> Self {
        Self { value }
    }
}

fn serialize_result<R: Serialize, S: serde::Serializer>(value: &ASCOMResult<R>, serializer: S) -> Result<S::Ok, S::Error> {
    match value {
        Ok(value) => value.serialize(serializer),
        Err(error) => error.serialize(serializer),
    }
}

impl Responder for ASCOMResponse {
    type Body = <Json<Self> as Responder>::Body;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        Json(self).respond_to(req)
    }
}

pub(crate) struct DomainRootSpanBuilder;

impl tracing_actix_web::RootSpanBuilder for DomainRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        use tracing::field::Empty;

        tracing_actix_web::root_span!(request, client_id = Empty, client_transaction_id = Empty, server_transaction_id = Empty)
    }

    fn on_request_end<B>(span: Span, outcome: &Result<ServiceResponse<B>, actix_web::Error>) {
        tracing_actix_web::DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}

type DevicesStorage = HashMap<(&'static str, usize), Box<Mutex<dyn super::Device>>>;

impl fmt::Debug for dyn super::Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(self.ty()).field("name", &self.name()).field("description", &self.description()).finish()
    }
}

#[derive(Debug, Default)]
pub struct DevicesBuilder {
    devices: DevicesStorage,
    counters: HashMap<&'static str, usize>,
}

impl DevicesBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with<T: super::Device + 'static>(mut self, device: T) -> Self {
        let index_ref = self.counters.entry(device.ty()).or_insert(0);
        let index = *index_ref;
        self.devices.insert((device.ty(), index), Box::new(Mutex::new(device)));
        *index_ref += 1;
        self
    }

    pub fn finish(self) -> Devices {
        Devices { devices: Arc::new(self.devices) }
    }
}

#[derive(Debug, Clone)]
pub struct Devices {
    devices: Arc<DevicesStorage>,
}

impl Devices {
    pub fn handle_action(&self, is_mut: bool, device_type: &str, device_number: usize, action: &str, params: &str) -> ASCOMResult<serde_json::Value> {
        self.devices
            .get(&(device_type, device_number))
            .ok_or(ASCOMError::NOT_CONNECTED)?
            .lock()
            .unwrap()
            .handle_action(is_mut, action, params)
    }
}

impl actix_web::dev::HttpServiceFactory for Devices {
    fn register(self, config: &mut actix_web::dev::AppService) {
        let resource = actix_web::web::resource("/api/v1/{device_type}/{device_number}/{action}")
            .app_data(self)
            .guard(actix_web::guard::Any(actix_web::guard::Get()).or(actix_web::guard::Put()))
            .to(
                move |request: HttpRequest, root_span: tracing_actix_web::RootSpan, path: actix_web::web::Path<(String, usize, String)>, ascom: crate::api::common::ASCOMRequest| {
                    let devices = request.app_data::<Self>().unwrap().clone();
                    let is_mut = request.method() == actix_web::http::Method::PUT;
                    let (device_type, device_number, action) = path.into_inner();
                    ascom.respond_with(root_span, move |params| devices.handle_action(is_mut, &device_type, device_number, &action, params))
                },
            );

        actix_web::dev::HttpServiceFactory::register(resource, config);
    }
}

macro_rules! rpc {
    (@http_method mut self) => {
        actix_web::web::put
    };

    (@http_method self) => {
        actix_web::web::get
    };

    (@if_parent $parent_trait_name:ident { $($then:tt)* } { $($else:tt)* }) => {
        $($then)*
    };

    (@if_parent { $($then:tt)* } { $($else:tt)* }) => {
        $($else)*
    };

    (@is_mut mut self) => (true);

    (@is_mut self) => (false);

    ($(
        $(#[doc = $doc:literal])*
        #[http($path:literal)]
        pub trait $trait_name:ident $(: $parent_trait_name:ident)? {
            $(
                $(#[doc = $method_doc:literal])*
                #[http($method_path:literal)]
                fn $method_name:ident(& $($mut_self:ident)* $(, $params:ident: $params_ty:ty)?) $(-> $return_type:ty)?;
            )*
        }
    )*) => {
        $(
            #[allow(unused_variables)]
            $(#[doc = $doc])*
            pub trait $trait_name: $($parent_trait_name +)? Send + Sync {
                rpc!(@if_parent $($parent_trait_name)? {
                    const TYPE: &'static str = $path;
                } {
                    fn ty(&self) -> &'static str;

                    fn handle_action(&mut self, is_mut: bool, action: &str, params: &str) -> ASCOMResult<serde_json::Value>;
                });

                $(
                    $(#[doc = $method_doc])*
                    fn $method_name(& $($mut_self)* $(, $params: $params_ty)?) -> ASCOMResult$(<$return_type>)? {
                        Err(ASCOMError::ACTION_NOT_IMPLEMENTED)
                    }
                )*

                fn handle_action_impl(&mut self, is_mut: bool, action: &str, params: &str) -> ASCOMResult<serde_json::Value> {
                    match (is_mut, action) {
                        $((rpc!(@is_mut $($mut_self)*), $method_path) => {
                            $(
                                let $params =
                                    serde_urlencoded::from_str::<$params_ty>(params)
                                    .map_err(|err| ASCOMError::new(ASCOMErrorCode::INVALID_VALUE, err.to_string()))?;
                            )?
                            let result = self.$method_name($($params)?)?;
                            serde_json::to_value(result).map_err(|err| ASCOMError::new(ASCOMErrorCode::UNSPECIFIED, err.to_string()))
                        })*
                        _ => {
                            rpc!(@if_parent $($parent_trait_name)? {
                                $($parent_trait_name)?::handle_action_impl(self, is_mut, action, params)
                            } {
                                Err(ASCOMError::ACTION_NOT_IMPLEMENTED)
                            })
                        }
                    }
                }
            }
        )*
    };
}

pub(crate) use rpc;
