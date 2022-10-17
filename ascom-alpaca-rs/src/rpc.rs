use crate::{ASCOMError, ASCOMErrorCode, ASCOMResult};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct OpaqueResponse(serde_json::Map<String, serde_json::Value>);

pub(crate) fn to_response<T: Serialize>(value: T) -> ASCOMResult<OpaqueResponse> {
    let json = serde_json::to_value(value)
        .map_err(|err| ASCOMError::new(ASCOMErrorCode::INVALID_VALUE, err.to_string()))?;
    Ok(OpaqueResponse(match json {
        serde_json::Value::Object(map) => map,
        serde_json::Value::Null => serde_json::Map::new(),
        value => {
            // Wrap into IntResponse / BoolResponse / ..., aka {"value": ...}
            std::iter::once(("Value".to_owned(), value)).collect()
        }
    }))
}

macro_rules! rpc {
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
                $(#[params($params_ty:ty)])?
                fn $method_name:ident(& $($mut_self:ident)* $(, $param:ident: $param_ty:ty)* $(,)?) $(-> $return_type:ty)?;
            )*
        }
    )*) => {
        $(
            #[allow(unused_variables)]
            $(#[doc = $doc])*
            pub trait $trait_name $(: $parent_trait_name)? {
                rpc!(@if_parent $($parent_trait_name)? {
                    const TYPE: &'static str = $path;
                } {
                    fn ty(&self) -> &'static str;

                    fn handle_action(&mut self, is_mut: bool, action: &str, params_str: &str) -> $crate::ASCOMResult<$crate::OpaqueResponse>;
                });

                $(
                    $(#[doc = $method_doc])*
                    fn $method_name(& $($mut_self)* $(, $param: $param_ty)*) -> $crate::ASCOMResult$(<$return_type>)? {
                        Err($crate::ASCOMError::ACTION_NOT_IMPLEMENTED)
                    }
                )*

                fn handle_action_impl(&mut self, is_mut: bool, action: &str, params_str: &str) -> $crate::ASCOMResult<$crate::OpaqueResponse> {
                    use $crate::rpc::to_response;

                    match (is_mut, action) {
                        $((rpc!(@is_mut $($mut_self)*), $method_path) => {
                            $(
                                let params: $params_ty =
                                    serde_urlencoded::from_str(params_str)
                                    .map_err(|err| $crate::ASCOMError::new($crate::ASCOMErrorCode::INVALID_VALUE, err.to_string()))?;
                            )?
                            let result = self.$method_name($(params.$param),*)?;
                            to_response(result)
                        })*
                        _ => {
                            rpc!(@if_parent $($parent_trait_name)? {
                                $($parent_trait_name)?::handle_action_impl(self, is_mut, action, params_str)
                            } {
                                Err($crate::ASCOMError::ACTION_NOT_IMPLEMENTED)
                            })
                        }
                    }
                }
            }
        )*
    };
}

pub(crate) use rpc;
