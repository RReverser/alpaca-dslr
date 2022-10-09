use actix_web::{
    dev::{Payload, ServiceRequest, ServiceResponse},
    error::{BlockingError, ErrorBadRequest},
    http::Method,
    web::{Bytes, Json, Query},
    FromRequest, HttpMessage, HttpRequest, HttpResponse, Responder,
};
use dashmap::DashMap;
use pin_project::pin_project;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::borrow::Cow;
use std::future::Future;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::atomic::AtomicU32;
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
pub struct ASCOMRequest<T> {
    // #[serde(flatten)]
    transaction: TransactionIds,
    // #[serde(flatten)]
    pub request: T,
}

impl<T: DeserializeOwned> ASCOMRequest<T> {
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

impl<T: 'static + Send> ASCOMRequest<T> {
    pub fn respond_with<U: Send + 'static>(self, root_span: RootSpan, f: impl FnOnce(T) -> Result<U, ASCOMError> + Send + 'static) -> impl Future<Output = Result<ASCOMResponse<U>, BlockingError>> {
        self.transaction.record(root_span);

        actix_web::web::block(move || ASCOMResponse {
            transaction: self.transaction,
            result: f(self.request),
        })
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

pub type ASCOMResult<T> = Result<T, ASCOMError>;

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
    transaction: TransactionIds,
    #[serde(flatten, serialize_with = "serialize_result", bound = "T: Serialize")]
    pub result: ASCOMResult<T>,
}

fn serialize_result<T: Serialize, S: serde::Serializer>(value: &ASCOMResult<T>, serializer: S) -> Result<S::Ok, S::Error> {
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

pub struct DomainRootSpanBuilder;

impl tracing_actix_web::RootSpanBuilder for DomainRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        use tracing::field::Empty;

        tracing_actix_web::root_span!(request, client_id = Empty, client_transaction_id = Empty, server_transaction_id = Empty)
    }

    fn on_request_end<B>(span: Span, outcome: &Result<ServiceResponse<B>, actix_web::Error>) {
        tracing_actix_web::DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}

pub struct RpcDevices<T: ?Sized> {
    devices: DashMap<u32, Box<T>>,
    counter: AtomicU32,
}

impl<T: ?Sized> Default for RpcDevices<T> {
    fn default() -> Self {
        Self {
            devices: DashMap::new(),
            counter: AtomicU32::new(0),
        }
    }
}

impl<T: ?Sized> RpcDevices<T> {
    pub fn register_dyn(&self, device: Box<T>) -> u32 {
        let id = self.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.devices.insert(id, device);
        id
    }

    pub fn unregister(&self, id: u32) {
        self.devices.remove(&id);
    }

    pub fn get(&self, id: u32) -> Option<impl '_ + Deref<Target = impl Deref<Target = T>>> {
        self.devices.get(&id)
    }

    pub fn get_mut(&self, id: u32) -> Option<impl '_ + DerefMut<Target = impl DerefMut<Target = T>>> {
        self.devices.get_mut(&id)
    }
}

pub struct RpcService<T: ?Sized>(PhantomData<T>);

impl<T: ?Sized> Default for RpcService<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

#[allow(unused_macros)]
macro_rules! rpc {
    (@dashmap_get $dashmap:expr, $index:expr, mut self) => {
        $dashmap.get_mut($index)
    };

    (@dashmap_get $dashmap:expr, $index:expr, self) => {
        $dashmap.get($index)
    };

    (@type_or_void $name:ident = $ty:ty) => {
        type $name = $ty;
    };

    (@type_or_void $name:ident) => {
        type $name = ();
    };

    (@http_method mut self) => {
        actix_web::web::put
    };

    (@http_method self) => {
        actix_web::web::get
    };

    ($(
        $(#[doc = $doc:literal])*
        #[http($path:literal)]
        pub trait $trait_name:ident {
            $(
                $(#[doc = $method_doc:literal])*
                #[http($method_path:literal)]
                fn $method_name:ident(& $($mut_self:ident)* $(, $params:ident: $params_ty:ty)?) -> $return_type:ty;
            )*
        }
    )*) => {
        $(
            $(#[doc = $doc])*
            pub trait $trait_name: Send + Sync {
                $(
                    $(#[doc = $method_doc])*
                    fn $method_name(& $($mut_self)* $(, $params: $params_ty)?) -> $return_type;
                )*
            }

            impl crate::api::common::RpcDevices<dyn $trait_name> {
                pub fn register<T: 'static + $trait_name>(&self, device: T) -> u32 {
                    self.register_dyn(Box::new(device))
                }
            }

            impl actix_web::dev::HttpServiceFactory for crate::api::common::RpcService<dyn $trait_name> {
                fn register(self, config: &mut actix_web::dev::AppService) {
                    fn missing_device_err(device_number: u32) -> ASCOMError {
                        ASCOMError {
                            code: crate::api::common::ASCOMErrorCode::NOT_CONNECTED,
                            message: format!("{} #{} not found", stringify!($trait_name), device_number).into(),
                        }
                    }

                    let scope =
                        actix_web::web::scope(concat!("/", $path, "/{device_number}"))
                        .app_data(actix_web::web::Data::new(crate::api::common::RpcDevices::<dyn $trait_name>::default()))
                        $(.route(concat!("/", $method_path), {
                            rpc!(@type_or_void Params $( = $params_ty)?);

                            fn $method_name(
                                devices: actix_web::web::Data<crate::api::common::RpcDevices<dyn $trait_name>>,
                                root_span: tracing_actix_web::RootSpan,
                                device_number: actix_web::web::Path<u32>,
                                ascom: crate::api::common::ASCOMRequest<Params>
                            ) -> impl std::future::Future<Output = Result<
                                crate::api::common::ASCOMResponse<impl serde::Serialize>,
                                actix_web::error::BlockingError>
                            > {
                                ascom.respond_with(root_span, move |params| {
                                    let device_number = device_number.into_inner();

                                    rpc!(@dashmap_get devices, device_number, $($mut_self)*)
                                    .ok_or_else(move || {
                                        missing_device_err(device_number)
                                    })?
                                    .$method_name($({
                                        let $params = params;
                                        $params
                                    })?)
                                })
                            }

                            rpc!(@http_method $($mut_self)*)().to($method_name)
                        }))*
                        ;

                    actix_web::dev::HttpServiceFactory::register(scope, config);
                }
            }
        )*
    };
}

pub(crate) use rpc;
