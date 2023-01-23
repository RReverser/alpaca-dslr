use crate::transaction::ASCOMRequest;
use crate::Devices;
use axum::extract::Path;
use axum::http::{Method, StatusCode};
use axum::routing::{on, MethodFilter};
use axum::{Form, Json, Router, TypedHeader};
use mediatype::MediaTypeList;

// A hack until TypedHeader supports Accept natively.
struct AcceptsImageBytes {
    accepts: bool,
}

impl axum::headers::Header for AcceptsImageBytes {
    fn name() -> &'static axum::headers::HeaderName {
        static ACCEPT: axum::headers::HeaderName = axum::headers::HeaderName::from_static("accept");
        &ACCEPT
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i axum::http::HeaderValue>,
    {
        let mut accepts = false;
        for value in values {
            for media_type in MediaTypeList::new(
                value
                    .to_str()
                    .map_err(|_err| axum::headers::Error::invalid())?,
            ) {
                let media_type = media_type.map_err(|_err| axum::headers::Error::invalid())?;
                if media_type.ty == mediatype::names::APPLICATION
                    && media_type.subty == "imagebytes"
                    && media_type.suffix.is_none()
                {
                    accepts = true;
                    break;
                }
            }
        }
        Ok(AcceptsImageBytes { accepts })
    }

    fn encode<E: Extend<axum::http::HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(axum::http::HeaderValue::from_static(
            if self.accepts {
                "application/imagebytes"
            } else {
                "*/*"
            },
        )));
    }
}

impl Devices {
    pub fn into_router(self) -> Router {
        Router::new().route(
            "/api/v1/:device_type/:device_number/:action",
            on(
                MethodFilter::GET | MethodFilter::PUT,
                move |method: Method,
                      Path((device_type, device_number, action)): Path<(String, usize, String)>,
                      TypedHeader(accepts_image_bytes): TypedHeader<AcceptsImageBytes>,
                      Form(request): Form<ASCOMRequest>| async move {
                        let mut device =
                            self.get(&device_type, device_number)
                            .ok_or((StatusCode::NOT_FOUND, "Device not found"))?
                            .lock()
                            .map_err(|_err| (StatusCode::INTERNAL_SERVER_ERROR, "This device can't be accessed anymore due to a previous fatal error"))?;

                        Ok::<_, axum::response::ErrorResponse>(Json(request.respond_with(move |params| {
                            device.handle_action(method == Method::PUT, &action, params)
                        })))
                },
            ),
        )
    }
}
