/*!
# ASCOM Alpaca Device API v1

The Alpaca API uses RESTful techniques and TCP/IP to enable ASCOM applications and devices to communicate across modern network environments.

## Interface Behaviour
The ASCOM Interface behavioural requirements for Alpaca drivers are the same as for COM based drivers and are documented in the <a href="https://ascom-standards.org/Help/Developer/html/N_ASCOM_DeviceInterface.htm">API Interface Definitions</a> e.g. the <a href="https://ascom-standards.org/Help/Developer/html/M_ASCOM_DeviceInterface_ITelescopeV3_SlewToCoordinates.htm">Telescope.SlewToCoordinates</a> method.       This document focuses on how to use the ASCOM Interface standards in their RESTful Alpaca form.
## Alpaca URLs, Case Sensitivity, Parameters and Returned values
**Alpaca Device API URLs** are of the form **http(s)://host:port/path** where path comprises **"/api/v1/"** followed by one of the method names below. e.g. for an Alpaca interface running on port 7843 of a device with IP address 192.168.1.89:
* A telescope "Interface Version" method URL would be **http://192.168.1.89:7843/api/v1/telescope/0/interfaceversion**

* A first focuser "Position" method URL would be  **http://192.168.1.89:7843/api/v1/focuser/0/position**

* A second focuser "StepSize" method URL would be  **http://192.168.1.89:7843/api/v1/focuser/1/stepsize**
* A rotator "Halt" method URL would be  **http://192.168.1.89:7843/api/v1/rotator/0/halt**


URLs are case sensitive and all elements must be in lower case. This means that both the device type and command name must always be in lower case. Parameter names are not case sensitive, so clients and drivers should be prepared for parameter names to be supplied and returned with any casing. Parameter values can be in mixed case as required.

For GET operations, parameters should be placed in the URL query string and for PUT operations they should be placed in the body of the message.

Responses, as described below, are returned in JSON format and always include a common set of values including the client's transaction number,  the server's transaction number together with any error message and error number.
If the transaction completes successfully, the ErrorMessage field will be an empty string and the ErrorNumber field will be zero.

## HTTP Status Codes and ASCOM Error codes
The returned HTTP status code gives a high level view of whether the device understood the request and whether it attempted to process it.

Under most circumstances the returned status will be `200`, indicating that the request was correctly formatted and that it was passed to the device's handler to execute. A `200` status does not necessarily mean that the operation completed as expected, without error, and you must always check the ErrorMessage and ErrorNumber fields to confirm whether the returned result is valid. The `200` status simply means that the transaction was successfully managed by the device's transaction management layer.

An HTTP status code of `400` indicates that the device could not interpret the request e.g. an invalid device number or misspelt device type was supplied. Check the body of the response for a text error message.

An HTTP status code of `500` indicates an unexpected error within the device from which it could not recover. Check the body of the response for a text error message.
## SetupDialog and Alpaca Device Configuration
The SetupDialog method has been omitted from the Alpaca Device API because it presents a user interface rather than returning data. Alpaca device configuration is covered in the "ASCOM Alpaca Management API" specification, which can be selected through the drop-down box at the head of this page.

*/

use axum::{
    routing::{get, post},
    Router,
};

#[derive(Deserialize)]
pub struct TransactionRequest {
    #[serde(rename = "ClientID")]
    client_id: u32,
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

#[derive(Serialize)]
pub struct TransactionResponse {
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
    #[serde(rename = "ServerTransactionID")]
    server_transaction_id: u32,
}

#[derive(Deserialize)]
pub struct ASCOMRequest<T> {
    #[serde(flatten)]
    transaction: TransactionRequest,
    #[serde(flatten)]
    request: T,
}

#[derive(Serialize)]
pub struct ASCOMError {
    code: i32,
    message: String,
}

#[derive(Serialize)]
pub struct ASCOMResponse<T> {
    #[serde(flatten)]
    transaction: TransactionResponse,
    #[serde(flatten)]
    result: std::result::Result<T, ASCOMError>,
}

impl<T: Serialize> IntoResponse for ASCOMResponse<T> {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

#[derive(Debug)]
pub enum AlpacaError {
    /// Method or parameter value error, check error message
    ValueError(String),

    // Server internal error, check error message
    InternalError(String),
}

impl IntoResponse for AlpacaError {
    fn into_response(self) -> Response {
        match self {
            Self::ValueError(message) => (StatusCode::BAD_REQUEST, message),
            Self::InternalError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        }
        .into_response()
    }
}

pub type Result<T> = std::result::Result<ASCOMResponse<T>, AlpacaError>;

mod parameters {

    /// One of the recognised ASCOM device types e.g. telescope (must be lower case)
    #[derive(Deserialize)]
    #[repr(transparent)]
    struct DeviceType(String);

    /// Zero based device number as set on the server (0 to 4294967295)
    #[derive(Deserialize)]
    #[repr(transparent)]
    struct DeviceNumber(u32);

    /// Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    #[derive(Deserialize)]
    #[repr(transparent)]
    struct ClientIdquery(u32);

    /// Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    #[derive(Deserialize)]
    #[repr(transparent)]
    struct ClientTransactionIdquery(u32);

    /// Right Ascension coordinate (0.0 to 23.99999999 hours)
    #[derive(Deserialize)]
    #[repr(transparent)]
    struct RightAscensionQuery(f64);

    /// Declination coordinate (-90.0 to +90.0 degrees)
    #[derive(Deserialize)]
    #[repr(transparent)]
    struct DeclinationQuery(f64);

    /// The axis about which rate information is desired. 0 = axisPrimary, 1 = axisSecondary, 2 = axisTertiary.
    #[derive(Deserialize)]
    #[repr(transparent)]
    struct AxisQuery(i32);

    /// The device number (0 to MaxSwitch - 1)
    #[derive(Deserialize)]
    #[repr(transparent)]
    struct SwitchNumberQuery(i32);
}

mod schemas {

    #[derive(Serialize)]

    struct ImageArrayResponse {
        /// 0 = Unknown, 1 = Short(int16), 2 = Integer (int32), 3 = Double (Double precision real number).
        #[serde(rename = "Type")]
        type_: Option<i32>,

        /// The array's rank, will be 2 (single plane image (monochrome)) or 3 (multi-plane image).
        #[serde(rename = "Rank")]
        rank: Option<i32>,

        /// Returned integer or double value
        #[serde(rename = "Value")]
        value: Option<Vec<Vec<f64>>>,
    }

    #[derive(Serialize)]

    struct BoolResponse {
        /// True or False value
        #[serde(rename = "Value")]
        value: Option<bool>,
    }

    #[derive(Serialize)]

    struct DoubleResponse {
        /// Returned double value
        #[serde(rename = "Value")]
        value: Option<f64>,
    }

    #[derive(Serialize)]

    struct IntResponse {
        /// Returned integer value
        #[serde(rename = "Value")]
        value: Option<i32>,
    }

    #[derive(Serialize)]

    struct IntArrayResponse {
        /// Array of integer values.
        #[serde(rename = "Value")]
        value: Option<Vec<i32>>,
    }

    #[derive(Serialize)]

    struct MethodResponse {}

    #[derive(Serialize)]

    struct StringResponse {
        /// String response from the device.
        #[serde(rename = "Value")]
        value: Option<String>,
    }

    #[derive(Serialize)]

    struct StringArrayResponse {
        /// Array of string values.
        #[serde(rename = "Value")]
        value: Option<Vec<String>>,
    }

    #[derive(Serialize)]

    struct AxisRatesResponse {
        /// Array of AxisRate objects
        #[serde(rename = "Value")]
        value: Option<Vec<schemas::AxisRate>>,
    }

    #[derive(Deserialize)]

    struct AxisRate {
        /// The maximum rate (degrees per second) This must always be a positive number. It indicates the maximum rate in either direction about the axis.
        #[serde(rename = "Maximum")]
        maximum: f64,

        /// The minimum rate (degrees per second) This must always be a positive number. It indicates the maximum rate in either direction about the axis.
        #[serde(rename = "Minimum")]
        minimum: f64,
    }

    #[derive(Serialize)]

    struct DriveRatesResponse {
        /// Array of DriveRate values
        #[serde(rename = "Value")]
        value: Option<Vec<schemas::DriveRate>>,
    }

    #[derive(Deserialize)]
    #[repr(transparent)]
    struct DriveRate(f64);

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutActionPath {
        /// One of the recognised ASCOM device types e.g. telescope (must be lower case)
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutActionRequest {
        /// A well known name that represents the action to be carried out.
        #[serde(rename = "Action")]
        action: String,

        /// List of required parameters or an Empty String if none are required
        #[serde(rename = "Parameters")]
        parameters: String,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCommandblindPath {
        /// One of the recognised ASCOM device types e.g. telescope (must be lower case)
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCommandblindRequest {
        /// The literal command string to be transmitted.
        #[serde(rename = "Command")]
        command: String,

        /// If set to true the string is transmitted 'as-is', if set to false then protocol framing characters may be added prior to transmission
        #[serde(rename = "Raw")]
        raw: String,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCommandboolPath {
        /// One of the recognised ASCOM device types e.g. telescope (must be lower case)
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCommandstringPath {
        /// One of the recognised ASCOM device types e.g. telescope (must be lower case)
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetConnectedPath {
        /// One of the recognised ASCOM device types e.g. telescope (must be lower case)
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetConnectedQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutConnectedPath {
        /// One of the recognised ASCOM device types e.g. telescope (must be lower case)
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutConnectedRequest {
        /// Set True to connect to the device hardware, set False to disconnect from the device hardware
        #[serde(rename = "Connected")]
        connected: bool,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDescriptionPath {
        /// One of the recognised ASCOM device types e.g. telescope (must be lower case)
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDescriptionQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDriverinfoPath {
        /// One of the recognised ASCOM device types e.g. telescope (must be lower case)
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDriverinfoQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDriverversionPath {
        /// One of the recognised ASCOM device types e.g. telescope (must be lower case)
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDriverversionQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetInterfaceversionPath {
        /// One of the recognised ASCOM device types e.g. telescope (must be lower case)
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetInterfaceversionQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetNamePath {
        /// One of the recognised ASCOM device types e.g. telescope (must be lower case)
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetNameQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSupportedactionsPath {
        /// One of the recognised ASCOM device types e.g. telescope (must be lower case)
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSupportedactionsQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraBayeroffsetxPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraBayeroffsetxQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraBayeroffsetyPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraBayeroffsetyQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraBinxPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraBinxQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraBinxPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraBinxRequest {
        /// The X binning value
        #[serde(rename = "BinX")]
        bin_x: i32,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraBinyPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraBinyQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraBinyPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraBinyRequest {
        /// The Y binning value
        #[serde(rename = "BinY")]
        bin_y: i32,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCamerastatePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCamerastateQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCameraxsizePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCameraxsizeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCameraysizePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCameraysizeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCanabortexposurePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCanabortexposureQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCanasymmetricbinPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCanasymmetricbinQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCanfastreadoutPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCanfastreadoutQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCangetcoolerpowerPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCangetcoolerpowerQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCanpulseguidePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCanpulseguideQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCansetccdtemperaturePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCansetccdtemperatureQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCanstopexposurePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCanstopexposureQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCcdtemperaturePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCcdtemperatureQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCooleronPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCooleronQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraCooleronPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraCooleronRequest {
        /// Cooler state
        #[serde(rename = "CoolerOn")]
        cooler_on: bool,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCoolerpowerPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCoolerpowerQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraElectronsperaduPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraElectronsperaduQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraExposuremaxPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraExposuremaxQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraExposureminPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraExposureminQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraExposureresolutionPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraExposureresolutionQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraFastreadoutPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraFastreadoutQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraFastreadoutPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraFastreadoutRequest {
        /// True to enable fast readout mode
        #[serde(rename = "FastReadout")]
        fast_readout: bool,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraFullwellcapacityPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraFullwellcapacityQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraGainPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraGainQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraGainPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraGainRequest {
        /// Index of the current camera gain in the Gains string array.
        #[serde(rename = "Gain")]
        gain: i32,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraGainmaxPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraGainmaxQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraGainminPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraGainminQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraGainsPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraGainsQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraHasshutterPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraHasshutterQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraHeatsinktemperaturePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraHeatsinktemperatureQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraImagearrayPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraImagearrayQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraImagearrayvariantPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraImagearrayvariantQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraImagereadyPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraImagereadyQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraIspulseguidingPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraIspulseguidingQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraLastexposuredurationPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraLastexposuredurationQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraLastexposurestarttimePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraLastexposurestarttimeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraMaxaduPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraMaxaduQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraMaxbinxPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraMaxbinxQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraMaxbinyPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraMaxbinyQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraNumxPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraNumxQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraNumxPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraNumxRequest {
        /// Sets the subframe width, if binning is active, value is in binned pixels.
        #[serde(rename = "NumX")]
        num_x: i32,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraNumyPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraNumyQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraNumyPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraNumyRequest {
        /// Sets the subframe height, if binning is active, value is in binned pixels.
        #[serde(rename = "NumY")]
        num_y: i32,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraOffsetPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraOffsetQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraOffsetPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraOffsetRequest {
        /// Index of the current camera offset in the offsets string array.
        #[serde(rename = "offset")]
        offset: i32,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraOffsetmaxPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraOffsetmaxQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraOffsetminPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraOffsetminQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraOffsetsPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraOffsetsQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraPercentcompletedPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraPercentcompletedQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraPixelsizexPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraPixelsizexQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraPixelsizeyPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraPixelsizeyQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraReadoutmodePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraReadoutmodeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraReadoutmodePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraReadoutmodeRequest {
        /// Index into the ReadoutModes array of string readout mode names indicating the camera's current readout mode.
        #[serde(rename = "ReadoutMode")]
        readout_mode: i32,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraReadoutmodesPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraReadoutmodesQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraSensornamePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraSensornameQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraSensortypePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraSensortypeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraSetccdtemperaturePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraSetccdtemperatureQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraSetccdtemperaturePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraSetccdtemperatureRequest {
        /// Temperature set point (degrees Celsius).
        #[serde(rename = "SetCCDTemperature")]
        set_ccdtemperature: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraStartxPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraStartxQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraStartxPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraStartxRequest {
        /// The subframe X axis start position in binned pixels.
        #[serde(rename = "StartX")]
        start_x: i32,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraStartyPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraStartyQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraStartyPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraStartyRequest {
        /// The subframe Y axis start position in binned pixels.
        #[serde(rename = "StartY")]
        start_y: i32,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraSubexposuredurationPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraSubexposuredurationQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraSubexposuredurationPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraSubexposuredurationRequest {
        /// The request sub exposure duration in seconds
        #[serde(rename = "SubExposureDuration")]
        sub_exposure_duration: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraAbortexposurePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraAbortexposureRequest {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraPulseguidePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraPulseguideRequest {
        /// Direction of movement (0 = North, 1 = South, 2 = East, 3 = West)
        #[serde(rename = "Direction")]
        direction: i32,

        /// Duration of movement in milli-seconds
        #[serde(rename = "Duration")]
        duration: i32,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraStartexposurePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraStartexposureRequest {
        /// Duration of exposure in seconds
        #[serde(rename = "Duration")]
        duration: f64,

        /// True if light frame, false if dark frame.
        #[serde(rename = "Light")]
        light: bool,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraStopexposurePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCovercalibratorBrightnessPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCovercalibratorBrightnessQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCovercalibratorCalibratorstatePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCovercalibratorCalibratorstateQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCovercalibratorCoverstatePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCovercalibratorCoverstateQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCovercalibratorMaxbrightnessPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCovercalibratorMaxbrightnessQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCovercalibratorCalibratoroffPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCovercalibratorCalibratoronPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCovercalibratorCalibratoronRequest {
        /// The required brightness in the range 0 to MaxBrightness
        #[serde(rename = "Brightness")]
        brightness: Option<i32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCovercalibratorClosecoverPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCovercalibratorHaltcoverPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCovercalibratorOpencoverPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeAltitudePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeAltitudeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeAthomePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeAthomeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeAtparkPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeAtparkQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeAzimuthPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeAzimuthQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeCanfindhomePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeCanfindhomeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeCanparkPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeCanparkQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeCansetaltitudePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeCansetaltitudeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeCansetazimuthPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeCansetazimuthQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeCansetparkPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeCansetparkQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeCansetshutterPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeCansetshutterQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeCanslavePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeCanslaveQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeCansyncazimuthPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeCansyncazimuthQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeShutterstatusPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeShutterstatusQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeSlavedPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeSlavedQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeSlavedPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutDomeSlavedRequest {
        /// True if telescope is slaved to dome, otherwise false
        #[serde(rename = "Slaved")]
        slaved: bool,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeSlewingPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeSlewingQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeAbortslewPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeCloseshutterPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeFindhomePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeOpenshutterPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeParkPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeSetparkPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeSlewtoaltitudePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutDomeSlewtoaltitudeRequest {
        /// Target dome altitude (degrees, horizon zero and increasing positive to 90 zenith)
        #[serde(rename = "Altitude")]
        altitude: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeSlewtoazimuthPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutDomeSlewtoazimuthRequest {
        /// Target dome azimuth (degrees, North zero and increasing clockwise. i.e., 90 East, 180 South, 270 West)
        #[serde(rename = "Azimuth")]
        azimuth: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeSynctoazimuthPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFilterwheelFocusoffsetsPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFilterwheelFocusoffsetsQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFilterwheelNamesPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFilterwheelNamesQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFilterwheelPositionPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFilterwheelPositionQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutFilterwheelPositionPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutFilterwheelPositionRequest {
        /// The number of the filter wheel position to select
        #[serde(rename = "Position")]
        position: i32,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFocuserAbsolutePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFocuserAbsoluteQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFocuserIsmovingPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFocuserIsmovingQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFocuserMaxincrementPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFocuserMaxincrementQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFocuserMaxstepPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFocuserMaxstepQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFocuserPositionPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFocuserPositionQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFocuserStepsizePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFocuserStepsizeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFocuserTempcompPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFocuserTempcompQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutFocuserTempcompPath {
        /// Zero based device number as set on the server
        #[serde(rename = "device_number")]
        device_number: Option<i32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutFocuserTempcompRequest {
        /// Set true to enable the focuser's temperature compensation mode, otherwise false for normal operation.
        #[serde(rename = "TempComp")]
        temp_comp: bool,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFocuserTempcompavailablePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFocuserTempcompavailableQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFocuserTemperaturePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFocuserTemperatureQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutFocuserHaltPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutFocuserMovePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutFocuserMoveRequest {
        /// Step distance or absolute position, depending on the value of the Absolute property
        #[serde(rename = "Position")]
        position: i32,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsAverageperiodPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsAverageperiodQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutObservingconditionsAverageperiodPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutObservingconditionsAverageperiodRequest {
        /// Time period (hours) over which to average sensor readings
        #[serde(rename = "AveragePeriod")]
        average_period: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsCloudcoverPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsCloudcoverQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsDewpointPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsDewpointQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsHumidityPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsHumidityQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsPressurePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsPressureQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsRainratePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsRainrateQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsSkybrightnessPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsSkybrightnessQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsSkyqualityPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsSkyqualityQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsSkytemperaturePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsSkytemperatureQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsStarfwhmPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsStarfwhmQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsTemperaturePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsTemperatureQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsWinddirectionPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsWinddirectionQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsWindgustPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsWindgustQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsWindspeedPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsWindspeedQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutObservingconditionsRefreshPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsSensordescriptionPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsSensordescriptionQuery {
        /// Name of the sensor whose description is required
        #[serde(rename = "SensorName")]
        sensor_name: Option<String>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsTimesincelastupdatePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsTimesincelastupdateQuery {
        /// Name of the sensor whose last update time is required
        #[serde(rename = "SensorName")]
        sensor_name: Option<String>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetRotatorCanreversePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetRotatorCanreverseQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetRotatorIsmovingPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetRotatorIsmovingQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetRotatorMechanicalpositionPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetRotatorMechanicalpositionQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetRotatorPositionPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetRotatorPositionQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetRotatorReversePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetRotatorReverseQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutRotatorReversePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutRotatorReverseRequest {
        /// True if the rotation and angular direction must be reversed to match the optical characteristcs
        #[serde(rename = "Reverse")]
        reverse: bool,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetRotatorStepsizePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetRotatorStepsizeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetRotatorTargetpositionPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetRotatorTargetpositionQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutRotatorHaltPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutRotatorMovePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutRotatorMoveRequest {
        /// Relative position to move in degrees from current Position.
        #[serde(rename = "Position")]
        position: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutRotatorMoveabsolutePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutRotatorMoveabsoluteRequest {
        /// Absolute position in degrees.
        #[serde(rename = "Position")]
        position: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutRotatorMovemechanicalPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutRotatorMovemechanicalRequest {
        /// Absolute position in degrees.
        #[serde(rename = "Position")]
        position: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutRotatorSyncPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutRotatorSyncRequest {
        /// Absolute position in degrees.
        #[serde(rename = "Position")]
        position: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSafetymonitorIssafePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSafetymonitorIssafeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSwitchMaxswitchPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSwitchMaxswitchQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSwitchCanwritePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSwitchCanwriteQuery {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        id: Option<i32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSwitchGetswitchPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSwitchGetswitchQuery {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        id: Option<i32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSwitchGetswitchdescriptionPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSwitchGetswitchdescriptionQuery {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        id: Option<i32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSwitchGetswitchnamePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSwitchGetswitchnameQuery {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        id: Option<i32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSwitchGetswitchvaluePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSwitchGetswitchvalueQuery {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        id: Option<i32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSwitchMinswitchvaluePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSwitchMinswitchvalueQuery {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        id: Option<i32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSwitchMaxswitchvaluePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSwitchMaxswitchvalueQuery {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        id: Option<i32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutSwitchSetswitchPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutSwitchSetswitchRequest {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        id: i32,

        /// The required control state (True or False)
        #[serde(rename = "State")]
        state: bool,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutSwitchSetswitchnamePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutSwitchSetswitchnameRequest {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        id: i32,

        /// The name of the device
        #[serde(rename = "Name")]
        name: String,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutSwitchSetswitchvaluePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutSwitchSetswitchvalueRequest {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        id: i32,

        /// The value to be set, between MinSwitchValue and MaxSwitchValue
        #[serde(rename = "Value")]
        value: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSwitchSwitchstepPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSwitchSwitchstepQuery {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        id: Option<i32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeAlignmentmodePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeAlignmentmodeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeAltitudePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeAltitudeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeApertureareaPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeApertureareaQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeAperturediameterPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeAperturediameterQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeAthomePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeAthomeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeAtparkPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeAtparkQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeAzimuthPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeAzimuthQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCanfindhomePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCanfindhomeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCanparkPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCanparkQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCanpulseguidePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCanpulseguideQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCansetdeclinationratePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCansetdeclinationrateQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCansetguideratesPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCansetguideratesQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCansetparkPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCansetparkQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCansetpiersidePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCansetpiersideQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCansetrightascensionratePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCansetrightascensionrateQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCansettrackingPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCansettrackingQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCanslewPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCanslewQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCanslewaltazPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCanslewaltazQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCanslewaltazasyncPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCanslewaltazasyncQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCanslewasyncPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCanslewasyncQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCansyncPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCansyncQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCansyncaltazPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCansyncaltazQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCanunparkPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCanunparkQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeDeclinationPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeDeclinationQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeDeclinationratePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeDeclinationrateQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeDeclinationratePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeDeclinationrateRequest {
        /// Declination tracking rate (arcseconds per second)
        #[serde(rename = "DeclinationRate")]
        declination_rate: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeDoesrefractionPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeDoesrefractionQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeDoesrefractionPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeDoesrefractionRequest {
        /// Set True to make the telescope or driver applie atmospheric refraction to coordinates.
        #[serde(rename = "DoesRefraction")]
        does_refraction: bool,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeEquatorialsystemPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeEquatorialsystemQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeFocallengthPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeFocallengthQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeGuideratedeclinationPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeGuideratedeclinationQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeGuideratedeclinationPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeGuideratedeclinationRequest {
        /// Declination movement rate offset degrees/sec).
        #[serde(rename = "GuideRateDeclination")]
        guide_rate_declination: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeGuideraterightascensionPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeGuideraterightascensionQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeGuideraterightascensionPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeGuideraterightascensionRequest {
        /// RightAscension movement rate offset degrees/sec).
        #[serde(rename = "GuideRateRightAscension")]
        guide_rate_right_ascension: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeIspulseguidingPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeIspulseguidingQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeRightascensionPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeRightascensionQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeRightascensionratePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeRightascensionrateQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeRightascensionratePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeRightascensionrateRequest {
        /// Right ascension tracking rate (arcseconds per second)
        #[serde(rename = "RightAscensionRate")]
        right_ascension_rate: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeSideofpierPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeSideofpierQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSideofpierPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeSideofpierRequest {
        /// New pointing state.
        #[serde(rename = "SideOfPier")]
        side_of_pier: i32,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeSiderealtimePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeSiderealtimeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeSiteelevationPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeSiteelevationQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSiteelevationPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeSiteelevationRequest {
        /// Elevation above mean sea level (metres).
        #[serde(rename = "SiteElevation")]
        site_elevation: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeSitelatitudePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeSitelatitudeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSitelatitudePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeSitelatitudeRequest {
        /// Site latitude (degrees)
        #[serde(rename = "SiteLatitude")]
        site_latitude: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeSitelongitudePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeSitelongitudeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSitelongitudePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeSitelongitudeRequest {
        /// Site longitude (degrees, positive East, WGS84)
        #[serde(rename = "SiteLongitude")]
        site_longitude: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeSlewingPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeSlewingQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeSlewsettletimePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeSlewsettletimeQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSlewsettletimePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeSlewsettletimeRequest {
        /// Settling time (integer sec.).
        #[serde(rename = "SlewSettleTime")]
        slew_settle_time: i32,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeTargetdeclinationPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeTargetdeclinationQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeTargetdeclinationPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeTargetdeclinationRequest {
        /// Target declination(degrees)
        #[serde(rename = "TargetDeclination")]
        target_declination: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeTargetrightascensionPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeTargetrightascensionQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeTargetrightascensionPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeTargetrightascensionRequest {
        /// Target right ascension(hours)
        #[serde(rename = "TargetRightAscension")]
        target_right_ascension: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeTrackingPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeTrackingQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeTrackingPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeTrackingRequest {
        /// Tracking enabled / disabled
        #[serde(rename = "Tracking")]
        tracking: bool,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeTrackingratePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeTrackingrateQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeTrackingratePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeTrackingrateRequest {
        /// New tracking rate
        #[serde(rename = "TrackingRate")]
        tracking_rate: i32,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeTrackingratesPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeTrackingratesQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeUtcdatePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeUtcdateQuery {}

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeUtcdatePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeUtcdateRequest {
        /// UTC date/time in ISO 8601 format.
        #[serde(rename = "UTCDate")]
        utcdate: String,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeAbortslewPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeAxisratesPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeAxisratesQuery {
        /// The axis about which rate information is desired. 0 = axisPrimary, 1 = axisSecondary, 2 = axisTertiary.
        #[serde(rename = "Axis")]
        axis: Option<i32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCanmoveaxisPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCanmoveaxisQuery {
        /// The axis about which rate information is desired. 0 = axisPrimary, 1 = axisSecondary, 2 = axisTertiary.
        #[serde(rename = "Axis")]
        axis: Option<i32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeDestinationsideofpierPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeDestinationsideofpierQuery {
        /// Right Ascension coordinate (0.0 to 23.99999999 hours)
        #[serde(rename = "RightAscension")]
        right_ascension: Option<f64>,

        /// Declination coordinate (-90.0 to +90.0 degrees)
        #[serde(rename = "Declination")]
        declination: Option<f64>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeFindhomePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeMoveaxisPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeMoveaxisRequest {
        /// The axis about which rate information is desired. 0 = axisPrimary, 1 = axisSecondary, 2 = axisTertiary.
        #[serde(rename = "Axis")]
        axis: i32,

        /// The rate of motion (deg/sec) about the specified axis
        #[serde(rename = "Rate")]
        rate: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeParkPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopePulseguidePath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopePulseguideRequest {
        /// The direction in which the guide-rate motion is to be made. 0 = guideNorth, 1 = guideSouth, 2 = guideEast, 3 = guideWest
        #[serde(rename = "Direction")]
        direction: i32,

        /// The duration of the guide-rate motion (milliseconds)
        #[serde(rename = "Duration")]
        duration: i32,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSetparkPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSlewtoaltazPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeSlewtoaltazRequest {
        /// Azimuth coordinate (degrees, North-referenced, positive East/clockwise)
        #[serde(rename = "Azimuth")]
        azimuth: f64,

        /// Altitude coordinate (degrees, positive up)
        #[serde(rename = "Altitude")]
        altitude: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSlewtoaltazasyncPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSlewtocoordinatesPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeSlewtocoordinatesRequest {
        /// Right Ascension coordinate (hours)
        #[serde(rename = "RightAscension")]
        right_ascension: f64,

        /// Declination coordinate (degrees)
        #[serde(rename = "Declination")]
        declination: f64,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSlewtocoordinatesasyncPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSlewtotargetPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSlewtotargetasyncPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSynctoaltazPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSynctocoordinatesPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSynctotargetPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeUnparkPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }
}

/**
Invokes the named device-specific action.

Actions and SupportedActions are a standardised means for drivers to extend functionality beyond the built-in capabilities of the ASCOM device interfaces.

The key advantage of using Actions is that drivers can expose any device specific functionality required. The downside is that, in order to use these unique features, every application author would need to create bespoke code to present or exploit them.

The Action parameter and return strings are deceptively simple, but can support transmission of arbitrarily complex data structures, for example through JSON encoding.

This capability will be of primary value to
 * <span style="font-size:14px;">bespoke software and hardware configurations where a single entity controls both the consuming application software and the hardware / driver environment</span>
 * <span style="font-size:14px;">a group of application and device authors to quickly formulate and try out new interface capabilities without requiring an immediate change to the ASCOM device interface, which will take a lot longer than just agreeing a name, input parameters and a standard response for an Action command.</span>


The list of Action commands supported by a driver can be discovered through the SupportedActions property.

This method should return an error message and NotImplementedException error number (0x400) if the driver just implements the standard ASCOM device methods and has no bespoke, unique, functionality.
*/
#[put("/<device_type>/<device_number>/action")]
fn put_action(
    schemas::PutActionPath { device_type, device_number }: schemas::PutActionPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutActionRequest { action, parameters },
    }: ASCOMRequest<schemas::PutActionRequest>,
) -> Result<schemas::StringResponse> {
}

/**
Transmits an arbitrary string to the device

Transmits an arbitrary string to the device and does not wait for a response. Optionally, protocol framing characters may be added to the string before transmission.
*/
#[put("/<device_type>/<device_number>/commandblind")]
fn put_commandblind(
    schemas::PutCommandblindPath { device_type, device_number }: schemas::PutCommandblindPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCommandblindRequest { command, raw },
    }: ASCOMRequest<schemas::PutCommandblindRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Transmits an arbitrary string to the device and returns a boolean value from the device.

Transmits an arbitrary string to the device and waits for a boolean response. Optionally, protocol framing characters may be added to the string before transmission.
*/
#[put("/<device_type>/<device_number>/commandbool")]
fn put_commandbool(
    schemas::PutCommandboolPath { device_type, device_number }: schemas::PutCommandboolPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCommandblindRequest { command, raw },
    }: ASCOMRequest<schemas::PutCommandblindRequest>,
) -> Result<schemas::BoolResponse> {
}

/**
Transmits an arbitrary string to the device and returns a string value from the device.

Transmits an arbitrary string to the device and waits for a string response. Optionally, protocol framing characters may be added to the string before transmission.
*/
#[put("/<device_type>/<device_number>/commandstring")]
fn put_commandstring(
    schemas::PutCommandstringPath { device_type, device_number }: schemas::PutCommandstringPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCommandblindRequest { command, raw },
    }: ASCOMRequest<schemas::PutCommandblindRequest>,
) -> Result<schemas::StringResponse> {
}

/// Retrieves the connected state of the device
#[get("/<device_type>/<device_number>/connected")]
fn get_connected(
    schemas::GetConnectedPath { device_type, device_number }: schemas::GetConnectedPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetConnectedQuery {},
    }: ASCOMRequest<schemas::GetConnectedQuery>,
) -> Result<schemas::BoolResponse> {
}

/// Sets the connected state of the device
#[put("/<device_type>/<device_number>/connected")]
fn put_connected(
    schemas::PutConnectedPath { device_type, device_number }: schemas::PutConnectedPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutConnectedRequest { connected },
    }: ASCOMRequest<schemas::PutConnectedRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Device description

The description of the device
*/
#[get("/<device_type>/<device_number>/description")]
fn get_description(
    schemas::GetDescriptionPath { device_type, device_number }: schemas::GetDescriptionPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetDescriptionQuery {},
    }: ASCOMRequest<schemas::GetDescriptionQuery>,
) -> Result<schemas::StringResponse> {
}

/**
Device driver description

The description of the driver
*/
#[get("/<device_type>/<device_number>/driverinfo")]
fn get_driverinfo(
    schemas::GetDriverinfoPath { device_type, device_number }: schemas::GetDriverinfoPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetDriverinfoQuery {},
    }: ASCOMRequest<schemas::GetDriverinfoQuery>,
) -> Result<schemas::StringResponse> {
}

/**
Driver Version

A string containing only the major and minor version of the driver.
*/
#[get("/<device_type>/<device_number>/driverversion")]
fn get_driverversion(
    schemas::GetDriverversionPath { device_type, device_number }: schemas::GetDriverversionPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetDriverversionQuery {},
    }: ASCOMRequest<schemas::GetDriverversionQuery>,
) -> Result<schemas::StringResponse> {
}

/**
The ASCOM Device interface version number that this device supports.

This method returns the version of the ASCOM device interface contract to which this device complies. Only one interface version is current at a moment in time and all new devices should be built to the latest interface version. Applications can choose which device interface versions they support and it is in their interest to support  previous versions as well as the current version to ensure thay can use the largest number of devices.
*/
#[get("/<device_type>/<device_number>/interfaceversion")]
fn get_interfaceversion(
    schemas::GetInterfaceversionPath { device_type, device_number }: schemas::GetInterfaceversionPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetInterfaceversionQuery {},
    }: ASCOMRequest<schemas::GetInterfaceversionQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Device name

The name of the device
*/
#[get("/<device_type>/<device_number>/name")]
fn get_name(
    schemas::GetNamePath { device_type, device_number }: schemas::GetNamePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetNameQuery {},
    }: ASCOMRequest<schemas::GetNameQuery>,
) -> Result<schemas::StringResponse> {
}

/// Returns the list of action names supported by this driver.
#[get("/<device_type>/<device_number>/supportedactions")]
fn get_supportedactions(
    schemas::GetSupportedactionsPath { device_type, device_number }: schemas::GetSupportedactionsPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetSupportedactionsQuery {},
    }: ASCOMRequest<schemas::GetSupportedactionsQuery>,
) -> Result<schemas::StringArrayResponse> {
}

/**
Returns the X offset of the Bayer matrix.

Returns the X offset of the Bayer matrix, as defined in SensorType.
*/
#[get("/camera/<device_number>/bayeroffsetx")]
fn get_camera_bayeroffsetx(
    schemas::GetCameraBayeroffsetxPath { device_number }: schemas::GetCameraBayeroffsetxPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraBayeroffsetxQuery {},
    }: ASCOMRequest<schemas::GetCameraBayeroffsetxQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Returns the Y offset of the Bayer matrix.

Returns the Y offset of the Bayer matrix, as defined in SensorType.
*/
#[get("/camera/<device_number>/bayeroffsety")]
fn get_camera_bayeroffsety(
    schemas::GetCameraBayeroffsetyPath { device_number }: schemas::GetCameraBayeroffsetyPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraBayeroffsetyQuery {},
    }: ASCOMRequest<schemas::GetCameraBayeroffsetyQuery>,
) -> Result<schemas::IntResponse> {
}

/// Returns the binning factor for the X axis.
#[get("/camera/<device_number>/binx")]
fn get_camera_binx(
    schemas::GetCameraBinxPath { device_number }: schemas::GetCameraBinxPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraBinxQuery {},
    }: ASCOMRequest<schemas::GetCameraBinxQuery>,
) -> Result<schemas::IntResponse> {
}

/// Sets the binning factor for the X axis.
#[put("/camera/<device_number>/binx")]
fn put_camera_binx(
    schemas::PutCameraBinxPath { device_number }: schemas::PutCameraBinxPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraBinxRequest { bin_x },
    }: ASCOMRequest<schemas::PutCameraBinxRequest>,
) -> Result<schemas::MethodResponse> {
}

/// Returns the binning factor for the Y axis.
#[get("/camera/<device_number>/biny")]
fn get_camera_biny(
    schemas::GetCameraBinyPath { device_number }: schemas::GetCameraBinyPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraBinyQuery {},
    }: ASCOMRequest<schemas::GetCameraBinyQuery>,
) -> Result<schemas::IntResponse> {
}

/// Sets the binning factor for the Y axis.
#[put("/camera/<device_number>/biny")]
fn put_camera_biny(
    schemas::PutCameraBinyPath { device_number }: schemas::PutCameraBinyPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraBinyRequest { bin_y },
    }: ASCOMRequest<schemas::PutCameraBinyRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the camera operational state.

Returns the current camera operational state as an integer. 0 = CameraIdle , 1 = CameraWaiting , 2 = CameraExposing , 3 = CameraReading , 4 = CameraDownload , 5 = CameraError
*/
#[get("/camera/<device_number>/camerastate")]
fn get_camera_camerastate(
    schemas::GetCameraCamerastatePath { device_number }: schemas::GetCameraCamerastatePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraCamerastateQuery {},
    }: ASCOMRequest<schemas::GetCameraCamerastateQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Returns the width of the CCD camera chip.

Returns the width of the CCD camera chip in unbinned pixels.
*/
#[get("/camera/<device_number>/cameraxsize")]
fn get_camera_cameraxsize(
    schemas::GetCameraCameraxsizePath { device_number }: schemas::GetCameraCameraxsizePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraCameraxsizeQuery {},
    }: ASCOMRequest<schemas::GetCameraCameraxsizeQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Returns the height of the CCD camera chip.

Returns the height of the CCD camera chip in unbinned pixels.
*/
#[get("/camera/<device_number>/cameraysize")]
fn get_camera_cameraysize(
    schemas::GetCameraCameraysizePath { device_number }: schemas::GetCameraCameraysizePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraCameraysizeQuery {},
    }: ASCOMRequest<schemas::GetCameraCameraysizeQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Indicates whether the camera can abort exposures.

Returns true if the camera can abort exposures; false if not.
*/
#[get("/camera/<device_number>/canabortexposure")]
fn get_camera_canabortexposure(
    schemas::GetCameraCanabortexposurePath { device_number }: schemas::GetCameraCanabortexposurePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraCanabortexposureQuery {},
    }: ASCOMRequest<schemas::GetCameraCanabortexposureQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the camera supports asymmetric binning

Returns a flag showing whether this camera supports asymmetric binning
*/
#[get("/camera/<device_number>/canasymmetricbin")]
fn get_camera_canasymmetricbin(
    schemas::GetCameraCanasymmetricbinPath { device_number }: schemas::GetCameraCanasymmetricbinPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraCanasymmetricbinQuery {},
    }: ASCOMRequest<schemas::GetCameraCanasymmetricbinQuery>,
) -> Result<schemas::BoolResponse> {
}

/// Indicates whether the camera has a fast readout mode.
#[get("/camera/<device_number>/canfastreadout")]
fn get_camera_canfastreadout(
    schemas::GetCameraCanfastreadoutPath { device_number }: schemas::GetCameraCanfastreadoutPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraCanfastreadoutQuery {},
    }: ASCOMRequest<schemas::GetCameraCanfastreadoutQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the camera's cooler power setting can be read.

If true, the camera's cooler power setting can be read.
*/
#[get("/camera/<device_number>/cangetcoolerpower")]
fn get_camera_cangetcoolerpower(
    schemas::GetCameraCangetcoolerpowerPath { device_number }: schemas::GetCameraCangetcoolerpowerPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraCangetcoolerpowerQuery {},
    }: ASCOMRequest<schemas::GetCameraCangetcoolerpowerQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Returns a flag indicating whether this camera supports pulse guiding

Returns a flag indicating whether this camera supports pulse guiding.
*/
#[get("/camera/<device_number>/canpulseguide")]
fn get_camera_canpulseguide(
    schemas::GetCameraCanpulseguidePath { device_number }: schemas::GetCameraCanpulseguidePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraCanpulseguideQuery {},
    }: ASCOMRequest<schemas::GetCameraCanpulseguideQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Returns a flag indicating whether this camera supports setting the CCD temperature

Returns a flag indicatig whether this camera supports setting the CCD temperature
*/
#[get("/camera/<device_number>/cansetccdtemperature")]
fn get_camera_cansetccdtemperature(
    schemas::GetCameraCansetccdtemperaturePath { device_number }: schemas::GetCameraCansetccdtemperaturePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraCansetccdtemperatureQuery {},
    }: ASCOMRequest<schemas::GetCameraCansetccdtemperatureQuery>,
) -> Result<schemas::BoolResponse> {
}

/// Returns a flag indicating whether this camera can stop an exposure that is in progress
#[get("/camera/<device_number>/canstopexposure")]
fn get_camera_canstopexposure(
    schemas::GetCameraCanstopexposurePath { device_number }: schemas::GetCameraCanstopexposurePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraCanstopexposureQuery {},
    }: ASCOMRequest<schemas::GetCameraCanstopexposureQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Returns the current CCD temperature

Returns the current CCD temperature in degrees Celsius.
*/
#[get("/camera/<device_number>/ccdtemperature")]
fn get_camera_ccdtemperature(
    schemas::GetCameraCcdtemperaturePath { device_number }: schemas::GetCameraCcdtemperaturePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraCcdtemperatureQuery {},
    }: ASCOMRequest<schemas::GetCameraCcdtemperatureQuery>,
) -> Result<schemas::DoubleResponse> {
}

/// Returns the current cooler on/off state.
#[get("/camera/<device_number>/cooleron")]
fn get_camera_cooleron(
    schemas::GetCameraCooleronPath { device_number }: schemas::GetCameraCooleronPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraCooleronQuery {},
    }: ASCOMRequest<schemas::GetCameraCooleronQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Turns the camera cooler on and off

Turns on and off the camera cooler. True = cooler on, False = cooler off
*/
#[put("/camera/<device_number>/cooleron")]
fn put_camera_cooleron(
    schemas::PutCameraCooleronPath { device_number }: schemas::PutCameraCooleronPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraCooleronRequest { cooler_on },
    }: ASCOMRequest<schemas::PutCameraCooleronRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the present cooler power level

Returns the present cooler power level, in percent.
*/
#[get("/camera/<device_number>/coolerpower")]
fn get_camera_coolerpower(
    schemas::GetCameraCoolerpowerPath { device_number }: schemas::GetCameraCoolerpowerPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraCoolerpowerQuery {},
    }: ASCOMRequest<schemas::GetCameraCoolerpowerQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the gain of the camera

Returns the gain of the camera in photoelectrons per A/D unit.
*/
#[get("/camera/<device_number>/electronsperadu")]
fn get_camera_electronsperadu(
    schemas::GetCameraElectronsperaduPath { device_number }: schemas::GetCameraElectronsperaduPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraElectronsperaduQuery {},
    }: ASCOMRequest<schemas::GetCameraElectronsperaduQuery>,
) -> Result<schemas::DoubleResponse> {
}

/// Returns the maximum exposure time supported by StartExposure.
#[get("/camera/<device_number>/exposuremax")]
fn get_camera_exposuremax(
    schemas::GetCameraExposuremaxPath { device_number }: schemas::GetCameraExposuremaxPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraExposuremaxQuery {},
    }: ASCOMRequest<schemas::GetCameraExposuremaxQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the Minimium exposure time

Returns the Minimium exposure time in seconds that the camera supports through StartExposure.
*/
#[get("/camera/<device_number>/exposuremin")]
fn get_camera_exposuremin(
    schemas::GetCameraExposureminPath { device_number }: schemas::GetCameraExposureminPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraExposureminQuery {},
    }: ASCOMRequest<schemas::GetCameraExposureminQuery>,
) -> Result<schemas::DoubleResponse> {
}

/// Returns the smallest increment in exposure time supported by StartExposure.
#[get("/camera/<device_number>/exposureresolution")]
fn get_camera_exposureresolution(
    schemas::GetCameraExposureresolutionPath { device_number }: schemas::GetCameraExposureresolutionPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraExposureresolutionQuery {},
    }: ASCOMRequest<schemas::GetCameraExposureresolutionQuery>,
) -> Result<schemas::DoubleResponse> {
}

/// Returns whenther Fast Readout Mode is enabled.
#[get("/camera/<device_number>/fastreadout")]
fn get_camera_fastreadout(
    schemas::GetCameraFastreadoutPath { device_number }: schemas::GetCameraFastreadoutPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraFastreadoutQuery {},
    }: ASCOMRequest<schemas::GetCameraFastreadoutQuery>,
) -> Result<schemas::BoolResponse> {
}

/// Sets whether Fast Readout Mode is enabled.
#[put("/camera/<device_number>/fastreadout")]
fn put_camera_fastreadout(
    schemas::PutCameraFastreadoutPath { device_number }: schemas::PutCameraFastreadoutPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraFastreadoutRequest { fast_readout },
    }: ASCOMRequest<schemas::PutCameraFastreadoutRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Reports the full well capacity of the camera

Reports the full well capacity of the camera in electrons, at the current camera settings (binning, SetupDialog settings, etc.).
*/
#[get("/camera/<device_number>/fullwellcapacity")]
fn get_camera_fullwellcapacity(
    schemas::GetCameraFullwellcapacityPath { device_number }: schemas::GetCameraFullwellcapacityPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraFullwellcapacityQuery {},
    }: ASCOMRequest<schemas::GetCameraFullwellcapacityQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the camera's gain

The camera's gain (GAIN VALUE MODE) OR the index of the selected camera gain description in the Gains array (GAINS INDEX MODE).
*/
#[get("/camera/<device_number>/gain")]
fn get_camera_gain(
    schemas::GetCameraGainPath { device_number }: schemas::GetCameraGainPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraGainQuery {},
    }: ASCOMRequest<schemas::GetCameraGainQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Sets the camera's gain.

The camera's gain (GAIN VALUE MODE) OR the index of the selected camera gain description in the Gains array (GAINS INDEX MODE).
*/
#[put("/camera/<device_number>/gain")]
fn put_camera_gain(
    schemas::PutCameraGainPath { device_number }: schemas::PutCameraGainPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraGainRequest { gain },
    }: ASCOMRequest<schemas::PutCameraGainRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Maximum Gain value of that this camera supports

Returns the maximum value of Gain.
*/
#[get("/camera/<device_number>/gainmax")]
fn get_camera_gainmax(
    schemas::GetCameraGainmaxPath { device_number }: schemas::GetCameraGainmaxPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraGainmaxQuery {},
    }: ASCOMRequest<schemas::GetCameraGainmaxQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Minimum Gain value of that this camera supports

Returns the Minimum value of Gain.
*/
#[get("/camera/<device_number>/gainmin")]
fn get_camera_gainmin(
    schemas::GetCameraGainminPath { device_number }: schemas::GetCameraGainminPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraGainminQuery {},
    }: ASCOMRequest<schemas::GetCameraGainminQuery>,
) -> Result<schemas::IntResponse> {
}

/**
List of Gain names supported by the camera

Returns the Gains supported by the camera.
*/
#[get("/camera/<device_number>/gains")]
fn get_camera_gains(
    schemas::GetCameraGainsPath { device_number }: schemas::GetCameraGainsPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraGainsQuery {},
    }: ASCOMRequest<schemas::GetCameraGainsQuery>,
) -> Result<schemas::StringArrayResponse> {
}

/**
Indicates whether the camera has a mechanical shutter

Returns a flag indicating whether this camera has a mechanical shutter.
*/
#[get("/camera/<device_number>/hasshutter")]
fn get_camera_hasshutter(
    schemas::GetCameraHasshutterPath { device_number }: schemas::GetCameraHasshutterPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraHasshutterQuery {},
    }: ASCOMRequest<schemas::GetCameraHasshutterQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Returns the current heat sink temperature.

Returns the current heat sink temperature (called "ambient temperature" by some manufacturers) in degrees Celsius.
*/
#[get("/camera/<device_number>/heatsinktemperature")]
fn get_camera_heatsinktemperature(
    schemas::GetCameraHeatsinktemperaturePath { device_number }: schemas::GetCameraHeatsinktemperaturePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraHeatsinktemperatureQuery {},
    }: ASCOMRequest<schemas::GetCameraHeatsinktemperatureQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns an array of integers containing the exposure pixel values

Returns an array of 32bit integers containing the pixel values from the last exposure. This call can return either a 2 dimension (monochrome images) or 3 dimension (colour or multi-plane images) array of size NumX \* NumY or NumX \* NumY \* NumPlanes. Where applicable, the size of NumPlanes has to be determined by inspection of the returned Array.

Since 32bit integers are always returned by this call, the returned JSON Type value (0 = Unknown, 1 = short(16bit), 2 = int(32bit), 3 = Double) is always 2. The number of planes is given in the returned Rank value.

When de-serialising to an object it is essential to know the array Rank beforehand so that the correct data class can be used. This can be achieved through a regular expression or by direct parsing of the returned JSON string to extract the Type and Rank values before de-serialising.

This regular expression accomplishes the extraction into two named groups Type and Rank, which can then be used to select the correct de-serialisation data class:

__`^*"Type":(?<Type>\d*),"Rank":(?<Rank>\d*)`__

When the SensorType is Monochrome, RGGB, CMYG, CMYG2 or LRGB, the serialised JSON array should have 2 dimensions. For example, the returned array should appear as below if NumX = 7, NumY = 5  and Pxy represents the pixel value at the zero based position x across and y down the image with the origin in the top left corner of the image.

Please note that this is "column-major" order (column changes most rapidly) from the image's row and column perspective, while, from the array's perspective, serialisation is actually effected in "row-major" order (rightmost index changes most rapidly).  This unintuitive outcome arises because the ASCOM Camera Interface specification defines the image column dimension as the rightmost array dimension.

[

[P00,P01,P02,P03,P04],

[P10,P11,P12,P13,P14],

[P20,P21,P22,P23,P24],

[P30,P31,P32,P33,P34],

[P40,P41,P42,P43,P44],

[P50,P51,P52,P53,P54],

[P60,P61,P62,P63,P64]

]

When the SensorType is Color, the serialised JSON array will have 3 dimensions. For example, the returned array should appear as below if NumX = 7, NumY = 5  and Rxy, Gxy and Bxy represent the red, green and blue pixel values at the zero based position x across and y down the image with the origin in the top left corner of the image.  Please see note above regarding element ordering.

[

[[R00,G00,B00],[R01,G01,B01],[R02,G02,B02],[R03,G03,B03],[R04,G04,B04]],

[[R10,G10,B10],[R11,G11,B11],[R12,G12,B12],[R13,G13,B13],[R14,G14,B14]],

[[R20,G20,B20],[R21,G21,B21],[R22,G22,B22],[R23,G23,B23],[R24,G24,B24]],

[[R30,G30,B30],[R31,G31,B31],[R32,G32,B32],[R33,G33,B33],[R34,G34,B34]],

[[R40,G40,B40],[R41,G41,B41],[R42,G42,B42],[R43,G43,B43],[R44,G44,B44]],

[[R50,G50,B50],[R51,G51,B51],[R52,G52,B52],[R53,G53,B53],[R54,G54,B54]],

[[R60,G60,B60],[R61,G61,B61],[R62,G62,B62],[R63,G63,B63],[R64,G64,B64]],

]

__`Performance`__

Returning an image from an Alpaca device as a JSON array is very inefficient and can result in delays of 30 or more seconds while client and device process and send the huge JSON string over the network.  A new, much faster mechanic called ImageBytes - [Alpaca ImageBytes Concepts and Implementation](https://www.ascom-standards.org/Developer/AlpacaImageBytes.pdf) has been developed that sends data as a binary byte stream and can offer a 10 to 20 fold reduction in transfer time.  It is strongly recommended that Alpaca Cameras implement the ImageBytes mechanic as well as the JSON mechanic.

*/
#[get("/camera/<device_number>/imagearray")]
fn get_camera_imagearray(
    schemas::GetCameraImagearrayPath { device_number }: schemas::GetCameraImagearrayPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraImagearrayQuery {},
    }: ASCOMRequest<schemas::GetCameraImagearrayQuery>,
) -> Result<schemas::ImageArrayResponse> {
}

/**
Returns an array of int containing the exposure pixel values

Returns an array containing the pixel values from the last exposure. This call can return either a 2 dimension (monochrome images) or 3 dimension (colour or multi-plane images) array of size NumX \* NumY  or NumX \* NumY \* NumPlanes. Where applicable, the size of NumPlanes has to be determined by inspection of the returned Array.

This call can return values as short(16bit) integers, int(32bit) integers or double floating point values. The nature of the returned values is given in the Type parameter: 0 = Unknown, 1 = short(16bit), 2 = int(32bit), 3 = Double. The number of planes is given in the returned Rank value.

When deserialising to an object it helps enormously to know the Type and Rank beforehand so that the correct data class can be used. This can be achieved through a regular expression or by direct parsing of the returned JSON string to extract the Type and Rank values before deserialising.

This regular expression accomplishes the extraction into two named groups Type and Rank, which can then be used to select the correct de-serialisation data class:

__`^*"Type":(?<Type>\d*),"Rank":(?<Rank>\d*)`__

When the SensorType is Monochrome, RGGB, CMYG, CMYG2 or LRGB, the serialised JSON array should have 2 dimensions. For example, the returned array should appear as below if NumX = 7, NumY = 5  and Pxy represents the pixel value at the zero based position x across and y down the image with the origin in the top left corner of the image.

Please note that this is "column-major" order (column changes most rapidly) from the image's row and column perspective, while, from the array's perspective, serialisation is actually effected in "row-major" order (rightmost index changes most rapidly).  This unintuitive outcome arises because the ASCOM Camera Interface specification defines the image column dimension as the rightmost array dimension.

[

[P00,P01,P02,P03,P04],

[P10,P11,P12,P13,P14],

[P20,P21,P22,P23,P24],

[P30,P31,P32,P33,P34],

[P40,P41,P42,P43,P44],

[P50,P51,P52,P53,P54],

[P60,P61,P62,P63,P64]

]

When the SensorType is Color, the serialised JSON array should have 3 dimensions. For example, the returned array should appear as below if NumX = 7, NumY = 5  and Rxy, Gxy and Bxy represent the red, green and blue pixel values at the zero based position x across and y down the image with the origin in the top left corner of the image.  Please see note above regarding element ordering.

[

[[R00,G00,B00],[R01,G01,B01],[R02,G02,B02],[R03,G03,B03],[R04,G04,B04]],

[[R10,G10,B10],[R11,G11,B11],[R12,G12,B12],[R13,G13,B13],[R14,G14,B14]],

[[R20,G20,B20],[R21,G21,B21],[R22,G22,B22],[R23,G23,B23],[R24,G24,B24]],

[[R30,G30,B30],[R31,G31,B31],[R32,G32,B32],[R33,G33,B33],[R34,G34,B34]],

[[R40,G40,B40],[R41,G41,B41],[R42,G42,B42],[R43,G43,B43],[R44,G44,B44]],

[[R50,G50,B50],[R51,G51,B51],[R52,G52,B52],[R53,G53,B53],[R54,G54,B54]],

[[R60,G60,B60],[R61,G61,B61],[R62,G62,B62],[R63,G63,B63],[R64,G64,B64]],

]

__`Performance`__

Returning an image from an Alpaca device as a JSON array is very inefficient and can result in delays of 30 or more seconds while client and device process and send the huge JSON string over the network.  A new, much faster mechanic called ImageBytes - [Alpaca ImageBytes Concepts and Implementation](https://www.ascom-standards.org/Developer/AlpacaImageBytes.pdf) has been developed that sends data as a binary byte stream and can offer a 10 to 20 fold reduction in transfer time.  It is strongly recommended that Alpaca Cameras implement the ImageBytes mechanic as well as the JSON mechanic.

*/
#[get("/camera/<device_number>/imagearrayvariant")]
fn get_camera_imagearrayvariant(
    schemas::GetCameraImagearrayvariantPath { device_number }: schemas::GetCameraImagearrayvariantPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraImagearrayvariantQuery {},
    }: ASCOMRequest<schemas::GetCameraImagearrayvariantQuery>,
) -> Result<schemas::ImageArrayResponse> {
}

/**
Indicates that an image is ready to be downloaded

Returns a flag indicating whether the image is ready to be downloaded from the camera.
*/
#[get("/camera/<device_number>/imageready")]
fn get_camera_imageready(
    schemas::GetCameraImagereadyPath { device_number }: schemas::GetCameraImagereadyPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraImagereadyQuery {},
    }: ASCOMRequest<schemas::GetCameraImagereadyQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates that the camera is pulse guideing.

Returns a flag indicating whether the camera is currrently in a PulseGuide operation.
*/
#[get("/camera/<device_number>/ispulseguiding")]
fn get_camera_ispulseguiding(
    schemas::GetCameraIspulseguidingPath { device_number }: schemas::GetCameraIspulseguidingPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraIspulseguidingQuery {},
    }: ASCOMRequest<schemas::GetCameraIspulseguidingQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Duration of the last exposure

Reports the actual exposure duration in seconds (i.e. shutter open time).
*/
#[get("/camera/<device_number>/lastexposureduration")]
fn get_camera_lastexposureduration(
    schemas::GetCameraLastexposuredurationPath { device_number }: schemas::GetCameraLastexposuredurationPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraLastexposuredurationQuery {},
    }: ASCOMRequest<schemas::GetCameraLastexposuredurationQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Start time of the last exposure in FITS standard format.

Reports the actual exposure start in the FITS-standard CCYY-MM-DDThh:mm:ss[.sss...] format.
*/
#[get("/camera/<device_number>/lastexposurestarttime")]
fn get_camera_lastexposurestarttime(
    schemas::GetCameraLastexposurestarttimePath { device_number }: schemas::GetCameraLastexposurestarttimePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraLastexposurestarttimeQuery {},
    }: ASCOMRequest<schemas::GetCameraLastexposurestarttimeQuery>,
) -> Result<schemas::StringResponse> {
}

/**
Camera's maximum ADU value

Reports the maximum ADU value the camera can produce.
*/
#[get("/camera/<device_number>/maxadu")]
fn get_camera_maxadu(
    schemas::GetCameraMaxaduPath { device_number }: schemas::GetCameraMaxaduPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraMaxaduQuery {},
    }: ASCOMRequest<schemas::GetCameraMaxaduQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Maximum  binning for the camera X axis

Returns the maximum allowed binning for the X camera axis
*/
#[get("/camera/<device_number>/maxbinx")]
fn get_camera_maxbinx(
    schemas::GetCameraMaxbinxPath { device_number }: schemas::GetCameraMaxbinxPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraMaxbinxQuery {},
    }: ASCOMRequest<schemas::GetCameraMaxbinxQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Maximum  binning for the camera Y axis

Returns the maximum allowed binning for the Y camera axis
*/
#[get("/camera/<device_number>/maxbiny")]
fn get_camera_maxbiny(
    schemas::GetCameraMaxbinyPath { device_number }: schemas::GetCameraMaxbinyPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraMaxbinyQuery {},
    }: ASCOMRequest<schemas::GetCameraMaxbinyQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Returns the current subframe width

Returns the current subframe width, if binning is active, value is in binned pixels.
*/
#[get("/camera/<device_number>/numx")]
fn get_camera_numx(
    schemas::GetCameraNumxPath { device_number }: schemas::GetCameraNumxPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraNumxQuery {},
    }: ASCOMRequest<schemas::GetCameraNumxQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Sets the current subframe width

Sets the current subframe width.
*/
#[put("/camera/<device_number>/numx")]
fn put_camera_numx(
    schemas::PutCameraNumxPath { device_number }: schemas::PutCameraNumxPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraNumxRequest { num_x },
    }: ASCOMRequest<schemas::PutCameraNumxRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the current subframe height

Returns the current subframe height, if binning is active, value is in binned pixels.
*/
#[get("/camera/<device_number>/numy")]
fn get_camera_numy(
    schemas::GetCameraNumyPath { device_number }: schemas::GetCameraNumyPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraNumyQuery {},
    }: ASCOMRequest<schemas::GetCameraNumyQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Sets the current subframe height

Sets the current subframe height.
*/
#[put("/camera/<device_number>/numy")]
fn put_camera_numy(
    schemas::PutCameraNumyPath { device_number }: schemas::PutCameraNumyPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraNumyRequest { num_y },
    }: ASCOMRequest<schemas::PutCameraNumyRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the camera's offset

Returns the camera's offset (OFFSET VALUE MODE) OR the index of the selected camera offset description in the offsets array (OFFSETS INDEX MODE).
*/
#[get("/camera/<device_number>/offset")]
fn get_camera_offset(
    schemas::GetCameraOffsetPath { device_number }: schemas::GetCameraOffsetPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraOffsetQuery {},
    }: ASCOMRequest<schemas::GetCameraOffsetQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Sets the camera's offset.

Sets the camera's offset (OFFSET VALUE MODE) OR the index of the selected camera offset description in the offsets array (OFFSETS INDEX MODE).
*/
#[put("/camera/<device_number>/offset")]
fn put_camera_offset(
    schemas::PutCameraOffsetPath { device_number }: schemas::PutCameraOffsetPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraOffsetRequest { offset },
    }: ASCOMRequest<schemas::PutCameraOffsetRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Maximum offset value of that this camera supports

Returns the maximum value of offset.
*/
#[get("/camera/<device_number>/offsetmax")]
fn get_camera_offsetmax(
    schemas::GetCameraOffsetmaxPath { device_number }: schemas::GetCameraOffsetmaxPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraOffsetmaxQuery {},
    }: ASCOMRequest<schemas::GetCameraOffsetmaxQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Minimum offset value of that this camera supports

Returns the Minimum value of offset.
*/
#[get("/camera/<device_number>/offsetmin")]
fn get_camera_offsetmin(
    schemas::GetCameraOffsetminPath { device_number }: schemas::GetCameraOffsetminPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraOffsetminQuery {},
    }: ASCOMRequest<schemas::GetCameraOffsetminQuery>,
) -> Result<schemas::IntResponse> {
}

/**
List of offset names supported by the camera

Returns the offsets supported by the camera.
*/
#[get("/camera/<device_number>/offsets")]
fn get_camera_offsets(
    schemas::GetCameraOffsetsPath { device_number }: schemas::GetCameraOffsetsPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraOffsetsQuery {},
    }: ASCOMRequest<schemas::GetCameraOffsetsQuery>,
) -> Result<schemas::StringArrayResponse> {
}

/**
Indicates percentage completeness of the current operation

Returns the percentage of the current operation that is complete. If valid, returns an integer between 0 and 100, where 0 indicates 0% progress (function just started) and 100 indicates 100% progress (i.e. completion).
*/
#[get("/camera/<device_number>/percentcompleted")]
fn get_camera_percentcompleted(
    schemas::GetCameraPercentcompletedPath { device_number }: schemas::GetCameraPercentcompletedPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraPercentcompletedQuery {},
    }: ASCOMRequest<schemas::GetCameraPercentcompletedQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Width of CCD chip pixels (microns)

Returns the width of the CCD chip pixels in microns.
*/
#[get("/camera/<device_number>/pixelsizex")]
fn get_camera_pixelsizex(
    schemas::GetCameraPixelsizexPath { device_number }: schemas::GetCameraPixelsizexPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraPixelsizexQuery {},
    }: ASCOMRequest<schemas::GetCameraPixelsizexQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Height of CCD chip pixels (microns)

Returns the Height of the CCD chip pixels in microns.
*/
#[get("/camera/<device_number>/pixelsizey")]
fn get_camera_pixelsizey(
    schemas::GetCameraPixelsizeyPath { device_number }: schemas::GetCameraPixelsizeyPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraPixelsizeyQuery {},
    }: ASCOMRequest<schemas::GetCameraPixelsizeyQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Indicates the canera's readout mode as an index into the array ReadoutModes

ReadoutMode is an index into the array ReadoutModes and returns the desired readout mode for the camera. Defaults to 0 if not set.
*/
#[get("/camera/<device_number>/readoutmode")]
fn get_camera_readoutmode(
    schemas::GetCameraReadoutmodePath { device_number }: schemas::GetCameraReadoutmodePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraReadoutmodeQuery {},
    }: ASCOMRequest<schemas::GetCameraReadoutmodeQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Set the camera's readout mode

Sets the ReadoutMode as an index into the array ReadoutModes.
*/
#[put("/camera/<device_number>/readoutmode")]
fn put_camera_readoutmode(
    schemas::PutCameraReadoutmodePath { device_number }: schemas::PutCameraReadoutmodePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraReadoutmodeRequest { readout_mode },
    }: ASCOMRequest<schemas::PutCameraReadoutmodeRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
List of available readout modes

This property provides an array of strings, each of which describes an available readout mode of the camera. At least one string must be present in the list.
*/
#[get("/camera/<device_number>/readoutmodes")]
fn get_camera_readoutmodes(
    schemas::GetCameraReadoutmodesPath { device_number }: schemas::GetCameraReadoutmodesPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraReadoutmodesQuery {},
    }: ASCOMRequest<schemas::GetCameraReadoutmodesQuery>,
) -> Result<schemas::StringArrayResponse> {
}

/**
Sensor name

The name of the sensor used within the camera.
*/
#[get("/camera/<device_number>/sensorname")]
fn get_camera_sensorname(
    schemas::GetCameraSensornamePath { device_number }: schemas::GetCameraSensornamePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraSensornameQuery {},
    }: ASCOMRequest<schemas::GetCameraSensornameQuery>,
) -> Result<schemas::StringResponse> {
}

/**
Type of information returned by the the camera sensor (monochrome or colour)

Returns a value indicating whether the sensor is monochrome, or what Bayer matrix it encodes. Where:
- 0 = Monochrome,
- 1 = Colour not requiring Bayer decoding
- 2 = RGGB Bayer encoding
- 3 = CMYG Bayer encoding
- 4 = CMYG2 Bayer encoding
- 5 = LRGB TRUESENSE Bayer encoding.

Please see the ASCOM Help fie for more informaiton on the SensorType.

*/
#[get("/camera/<device_number>/sensortype")]
fn get_camera_sensortype(
    schemas::GetCameraSensortypePath { device_number }: schemas::GetCameraSensortypePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraSensortypeQuery {},
    }: ASCOMRequest<schemas::GetCameraSensortypeQuery>,
) -> Result<schemas::IntResponse> {
}

/// Returns the current camera cooler setpoint in degrees Celsius.
#[get("/camera/<device_number>/setccdtemperature")]
fn get_camera_setccdtemperature(
    schemas::GetCameraSetccdtemperaturePath { device_number }: schemas::GetCameraSetccdtemperaturePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraSetccdtemperatureQuery {},
    }: ASCOMRequest<schemas::GetCameraSetccdtemperatureQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Set the camera's cooler setpoint (degrees Celsius).

Set's the camera's cooler setpoint in degrees Celsius.
*/
#[put("/camera/<device_number>/setccdtemperature")]
fn put_camera_setccdtemperature(
    schemas::PutCameraSetccdtemperaturePath { device_number }: schemas::PutCameraSetccdtemperaturePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraSetccdtemperatureRequest { set_ccdtemperature },
    }: ASCOMRequest<schemas::PutCameraSetccdtemperatureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Return the current subframe X axis start position

Sets the subframe start position for the X axis (0 based) and returns the current value. If binning is active, value is in binned pixels.
*/
#[get("/camera/<device_number>/startx")]
fn get_camera_startx(
    schemas::GetCameraStartxPath { device_number }: schemas::GetCameraStartxPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraStartxQuery {},
    }: ASCOMRequest<schemas::GetCameraStartxQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Sets the current subframe X axis start position

Sets the current subframe X axis start position in binned pixels.
*/
#[put("/camera/<device_number>/startx")]
fn put_camera_startx(
    schemas::PutCameraStartxPath { device_number }: schemas::PutCameraStartxPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraStartxRequest { start_x },
    }: ASCOMRequest<schemas::PutCameraStartxRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Return the current subframe Y axis start position

Sets the subframe start position for the Y axis (0 based) and returns the current value. If binning is active, value is in binned pixels.
*/
#[get("/camera/<device_number>/starty")]
fn get_camera_starty(
    schemas::GetCameraStartyPath { device_number }: schemas::GetCameraStartyPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraStartyQuery {},
    }: ASCOMRequest<schemas::GetCameraStartyQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Sets the current subframe Y axis start position

Sets the current subframe Y axis start position in binned pixels.
*/
#[put("/camera/<device_number>/starty")]
fn put_camera_starty(
    schemas::PutCameraStartyPath { device_number }: schemas::PutCameraStartyPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraStartyRequest { start_y },
    }: ASCOMRequest<schemas::PutCameraStartyRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Camera's sub-exposure interval

The Camera's sub exposure duration in seconds. Only available in Camera Interface Version 3 and later.
*/
#[get("/camera/<device_number>/subexposureduration")]
fn get_camera_subexposureduration(
    schemas::GetCameraSubexposuredurationPath { device_number }: schemas::GetCameraSubexposuredurationPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCameraSubexposuredurationQuery {},
    }: ASCOMRequest<schemas::GetCameraSubexposuredurationQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Sets the current Sub Exposure Duration

Sets image sub exposure duration in seconds. Only available in Camera Interface Version 3 and later.
*/
#[put("/camera/<device_number>/subexposureduration")]
fn put_camera_subexposureduration(
    schemas::PutCameraSubexposuredurationPath { device_number }: schemas::PutCameraSubexposuredurationPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraSubexposuredurationRequest { sub_exposure_duration },
    }: ASCOMRequest<schemas::PutCameraSubexposuredurationRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Aborts the current exposure

Aborts the current exposure, if any, and returns the camera to Idle state.
*/
#[put("/camera/<device_number>/abortexposure")]
fn put_camera_abortexposure(
    schemas::PutCameraAbortexposurePath { device_number }: schemas::PutCameraAbortexposurePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Pulse guide in the specified direction for the specified time.

Activates the Camera's mount control sytem to instruct the mount to move in a particular direction for a given period of time
*/
#[put("/camera/<device_number>/pulseguide")]
fn put_camera_pulseguide(
    schemas::PutCameraPulseguidePath { device_number }: schemas::PutCameraPulseguidePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraPulseguideRequest { direction, duration },
    }: ASCOMRequest<schemas::PutCameraPulseguideRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Starts an exposure

Starts an exposure. Use ImageReady to check when the exposure is complete.
*/
#[put("/camera/<device_number>/startexposure")]
fn put_camera_startexposure(
    schemas::PutCameraStartexposurePath { device_number }: schemas::PutCameraStartexposurePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraStartexposureRequest { duration, light },
    }: ASCOMRequest<schemas::PutCameraStartexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Stops the current exposure

Stops the current exposure, if any. If an exposure is in progress, the readout process is initiated. Ignored if readout is already in process.
*/
#[put("/camera/<device_number>/stopexposure")]
fn put_camera_stopexposure(
    schemas::PutCameraStopexposurePath { device_number }: schemas::PutCameraStopexposurePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the current calibrator brightness

Returns the current calibrator brightness in the range 0 (completely off) to MaxBrightness (fully on)
*/
#[get("/covercalibrator/<device_number>/brightness")]
fn get_covercalibrator_brightness(
    schemas::GetCovercalibratorBrightnessPath { device_number }: schemas::GetCovercalibratorBrightnessPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCovercalibratorBrightnessQuery {},
    }: ASCOMRequest<schemas::GetCovercalibratorBrightnessQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Returns the state of the calibration device

Returns the state of the calibration device, if present, otherwise returns "NotPresent".  The calibrator state mode is specified as an integer value from the CalibratorStatus Enum.
*/
#[get("/covercalibrator/<device_number>/calibratorstate")]
fn get_covercalibrator_calibratorstate(
    schemas::GetCovercalibratorCalibratorstatePath { device_number }: schemas::GetCovercalibratorCalibratorstatePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCovercalibratorCalibratorstateQuery {},
    }: ASCOMRequest<schemas::GetCovercalibratorCalibratorstateQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Returns the state of the device cover"

Returns the state of the device cover, if present, otherwise returns "NotPresent".  The cover state mode is specified as an integer value from the CoverStatus Enum.
*/
#[get("/covercalibrator/<device_number>/coverstate")]
fn get_covercalibrator_coverstate(
    schemas::GetCovercalibratorCoverstatePath { device_number }: schemas::GetCovercalibratorCoverstatePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCovercalibratorCoverstateQuery {},
    }: ASCOMRequest<schemas::GetCovercalibratorCoverstateQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Returns the calibrator's maximum Brightness value.

The Brightness value that makes the calibrator deliver its maximum illumination.
*/
#[get("/covercalibrator/<device_number>/maxbrightness")]
fn get_covercalibrator_maxbrightness(
    schemas::GetCovercalibratorMaxbrightnessPath { device_number }: schemas::GetCovercalibratorMaxbrightnessPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetCovercalibratorMaxbrightnessQuery {},
    }: ASCOMRequest<schemas::GetCovercalibratorMaxbrightnessQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Turns the calibrator off

Turns the calibrator off if the device has calibration capability.
*/
#[put("/covercalibrator/<device_number>/calibratoroff")]
fn put_covercalibrator_calibratoroff(
    schemas::PutCovercalibratorCalibratoroffPath { device_number }: schemas::PutCovercalibratorCalibratoroffPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Turns the calibrator on at the specified brightness

Turns the calibrator on at the specified brightness if the device has calibration capability.
*/
#[put("/covercalibrator/<device_number>/calibratoron")]
fn put_covercalibrator_calibratoron(
    schemas::PutCovercalibratorCalibratoronPath { device_number }: schemas::PutCovercalibratorCalibratoronPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCovercalibratorCalibratoronRequest { brightness },
    }: ASCOMRequest<schemas::PutCovercalibratorCalibratoronRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Initiates cover closing

Initiates cover closing if a cover is present.
*/
#[put("/covercalibrator/<device_number>/closecover")]
fn put_covercalibrator_closecover(
    schemas::PutCovercalibratorClosecoverPath { device_number }: schemas::PutCovercalibratorClosecoverPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Stops any cover movement that may be in progress

Stops any cover movement that may be in progress if a cover is present and cover movement can be interrupted.
*/
#[put("/covercalibrator/<device_number>/haltcover")]
fn put_covercalibrator_haltcover(
    schemas::PutCovercalibratorHaltcoverPath { device_number }: schemas::PutCovercalibratorHaltcoverPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Initiates cover opening

Initiates cover opening if a cover is present.
*/
#[put("/covercalibrator/<device_number>/opencover")]
fn put_covercalibrator_opencover(
    schemas::PutCovercalibratorOpencoverPath { device_number }: schemas::PutCovercalibratorOpencoverPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
The dome altitude

The dome altitude (degrees, horizon zero and increasing positive to 90 zenith).
*/
#[get("/dome/<device_number>/altitude")]
fn get_dome_altitude(
    schemas::GetDomeAltitudePath { device_number }: schemas::GetDomeAltitudePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetDomeAltitudeQuery {},
    }: ASCOMRequest<schemas::GetDomeAltitudeQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Indicates whether the dome is in the home position.

Indicates whether the dome is in the home position. This is normally used following a FindHome()  operation. The value is reset with any azimuth slew operation that moves the dome away from the home position. AtHome may also become true durng normal slew operations, if the dome passes through the home position and the dome controller hardware is capable of detecting that; or at the end of a slew operation if the dome comes to rest at the home position.
*/
#[get("/dome/<device_number>/athome")]
fn get_dome_athome(
    schemas::GetDomeAthomePath { device_number }: schemas::GetDomeAthomePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetDomeAthomeQuery {},
    }: ASCOMRequest<schemas::GetDomeAthomeQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the telescope is at the park position

True if the dome is in the programmed park position. Set only following a Park() operation and reset with any slew operation.
*/
#[get("/dome/<device_number>/atpark")]
fn get_dome_atpark(
    schemas::GetDomeAtparkPath { device_number }: schemas::GetDomeAtparkPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetDomeAtparkQuery {},
    }: ASCOMRequest<schemas::GetDomeAtparkQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
The dome azimuth

Returns the dome azimuth (degrees, North zero and increasing clockwise, i.e., 90 East, 180 South, 270 West)
*/
#[get("/dome/<device_number>/azimuth")]
fn get_dome_azimuth(
    schemas::GetDomeAzimuthPath { device_number }: schemas::GetDomeAzimuthPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetDomeAzimuthQuery {},
    }: ASCOMRequest<schemas::GetDomeAzimuthQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Indicates whether the dome can find the home position.

True if the dome can move to the home position.
*/
#[get("/dome/<device_number>/canfindhome")]
fn get_dome_canfindhome(
    schemas::GetDomeCanfindhomePath { device_number }: schemas::GetDomeCanfindhomePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetDomeCanfindhomeQuery {},
    }: ASCOMRequest<schemas::GetDomeCanfindhomeQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the dome can be parked.

True if the dome is capable of programmed parking (Park() method)
*/
#[get("/dome/<device_number>/canpark")]
fn get_dome_canpark(
    schemas::GetDomeCanparkPath { device_number }: schemas::GetDomeCanparkPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetDomeCanparkQuery {},
    }: ASCOMRequest<schemas::GetDomeCanparkQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the dome altitude can be set

True if driver is capable of setting the dome altitude.
*/
#[get("/dome/<device_number>/cansetaltitude")]
fn get_dome_cansetaltitude(
    schemas::GetDomeCansetaltitudePath { device_number }: schemas::GetDomeCansetaltitudePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetDomeCansetaltitudeQuery {},
    }: ASCOMRequest<schemas::GetDomeCansetaltitudeQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the dome azimuth can be set

True if driver is capable of setting the dome azimuth.
*/
#[get("/dome/<device_number>/cansetazimuth")]
fn get_dome_cansetazimuth(
    schemas::GetDomeCansetazimuthPath { device_number }: schemas::GetDomeCansetazimuthPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetDomeCansetazimuthQuery {},
    }: ASCOMRequest<schemas::GetDomeCansetazimuthQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the dome park position can be set

True if driver is capable of setting the dome park position.
*/
#[get("/dome/<device_number>/cansetpark")]
fn get_dome_cansetpark(
    schemas::GetDomeCansetparkPath { device_number }: schemas::GetDomeCansetparkPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetDomeCansetparkQuery {},
    }: ASCOMRequest<schemas::GetDomeCansetparkQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the dome shutter can be opened

True if driver is capable of automatically operating shutter
*/
#[get("/dome/<device_number>/cansetshutter")]
fn get_dome_cansetshutter(
    schemas::GetDomeCansetshutterPath { device_number }: schemas::GetDomeCansetshutterPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetDomeCansetshutterQuery {},
    }: ASCOMRequest<schemas::GetDomeCansetshutterQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the dome supports slaving to a telescope

True if driver is capable of slaving to a telescope.
*/
#[get("/dome/<device_number>/canslave")]
fn get_dome_canslave(
    schemas::GetDomeCanslavePath { device_number }: schemas::GetDomeCanslavePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetDomeCanslaveQuery {},
    }: ASCOMRequest<schemas::GetDomeCanslaveQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the dome azimuth position can be synched

True if driver is capable of synchronizing the dome azimuth position using the SyncToAzimuth(Double) method.
*/
#[get("/dome/<device_number>/cansyncazimuth")]
fn get_dome_cansyncazimuth(
    schemas::GetDomeCansyncazimuthPath { device_number }: schemas::GetDomeCansyncazimuthPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetDomeCansyncazimuthQuery {},
    }: ASCOMRequest<schemas::GetDomeCansyncazimuthQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Status of the dome shutter or roll-off roof

Returns the status of the dome shutter or roll-off roof. 0 = Open, 1 = Closed, 2 = Opening, 3 = Closing, 4 = Shutter status error
*/
#[get("/dome/<device_number>/shutterstatus")]
fn get_dome_shutterstatus(
    schemas::GetDomeShutterstatusPath { device_number }: schemas::GetDomeShutterstatusPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetDomeShutterstatusQuery {},
    }: ASCOMRequest<schemas::GetDomeShutterstatusQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Indicates whether the dome is slaved to the telescope

True if the dome is slaved to the telescope in its hardware, else False.
*/
#[get("/dome/<device_number>/slaved")]
fn get_dome_slaved(
    schemas::GetDomeSlavedPath { device_number }: schemas::GetDomeSlavedPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetDomeSlavedQuery {},
    }: ASCOMRequest<schemas::GetDomeSlavedQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Sets whether the dome is slaved to the telescope

Sets the current subframe height.
*/
#[put("/dome/<device_number>/slaved")]
fn put_dome_slaved(
    schemas::PutDomeSlavedPath { device_number }: schemas::PutDomeSlavedPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutDomeSlavedRequest { slaved },
    }: ASCOMRequest<schemas::PutDomeSlavedRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Indicates whether the any part of the dome is moving

True if any part of the dome is currently moving, False if all dome components are steady.
*/
#[get("/dome/<device_number>/slewing")]
fn get_dome_slewing(
    schemas::GetDomeSlewingPath { device_number }: schemas::GetDomeSlewingPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetDomeSlewingQuery {},
    }: ASCOMRequest<schemas::GetDomeSlewingQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Immediately cancel current dome operation.

Calling this method will immediately disable hardware slewing (Slaved will become False).
*/
#[put("/dome/<device_number>/abortslew")]
fn put_dome_abortslew(
    schemas::PutDomeAbortslewPath { device_number }: schemas::PutDomeAbortslewPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/// Close the shutter or otherwise shield telescope from the sky.
#[put("/dome/<device_number>/closeshutter")]
fn put_dome_closeshutter(
    schemas::PutDomeCloseshutterPath { device_number }: schemas::PutDomeCloseshutterPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Start operation to search for the dome home position.

After Home position is established initializes Azimuth to the default value and sets the AtHome flag.
*/
#[put("/dome/<device_number>/findhome")]
fn put_dome_findhome(
    schemas::PutDomeFindhomePath { device_number }: schemas::PutDomeFindhomePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/// Open shutter or otherwise expose telescope to the sky.
#[put("/dome/<device_number>/openshutter")]
fn put_dome_openshutter(
    schemas::PutDomeOpenshutterPath { device_number }: schemas::PutDomeOpenshutterPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Rotate dome in azimuth to park position.

After assuming programmed park position, sets AtPark flag.
*/
#[put("/dome/<device_number>/park")]
fn put_dome_park(
    schemas::PutDomeParkPath { device_number }: schemas::PutDomeParkPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Set the current azimuth, altitude position of dome to be the park position

Set the current azimuth, altitude position of dome to be the park position.
*/
#[put("/dome/<device_number>/setpark")]
fn put_dome_setpark(
    schemas::PutDomeSetparkPath { device_number }: schemas::PutDomeSetparkPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/// Slew the dome to the given altitude position.
#[put("/dome/<device_number>/slewtoaltitude")]
fn put_dome_slewtoaltitude(
    schemas::PutDomeSlewtoaltitudePath { device_number }: schemas::PutDomeSlewtoaltitudePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutDomeSlewtoaltitudeRequest { altitude },
    }: ASCOMRequest<schemas::PutDomeSlewtoaltitudeRequest>,
) -> Result<schemas::MethodResponse> {
}

/// Slew the dome to the given azimuth position.
#[put("/dome/<device_number>/slewtoazimuth")]
fn put_dome_slewtoazimuth(
    schemas::PutDomeSlewtoazimuthPath { device_number }: schemas::PutDomeSlewtoazimuthPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutDomeSlewtoazimuthRequest { azimuth },
    }: ASCOMRequest<schemas::PutDomeSlewtoazimuthRequest>,
) -> Result<schemas::MethodResponse> {
}

/// Synchronize the current position of the dome to the given azimuth.
#[put("/dome/<device_number>/synctoazimuth")]
fn put_dome_synctoazimuth(
    schemas::PutDomeSynctoazimuthPath { device_number }: schemas::PutDomeSynctoazimuthPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutDomeSlewtoazimuthRequest { azimuth },
    }: ASCOMRequest<schemas::PutDomeSlewtoazimuthRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Filter focus offsets

An integer array of filter focus offsets.
*/
#[get("/filterwheel/<device_number>/focusoffsets")]
fn get_filterwheel_focusoffsets(
    schemas::GetFilterwheelFocusoffsetsPath { device_number }: schemas::GetFilterwheelFocusoffsetsPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetFilterwheelFocusoffsetsQuery {},
    }: ASCOMRequest<schemas::GetFilterwheelFocusoffsetsQuery>,
) -> Result<schemas::IntArrayResponse> {
}

/**
Filter wheel filter names

The names of the filters
*/
#[get("/filterwheel/<device_number>/names")]
fn get_filterwheel_names(
    schemas::GetFilterwheelNamesPath { device_number }: schemas::GetFilterwheelNamesPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetFilterwheelNamesQuery {},
    }: ASCOMRequest<schemas::GetFilterwheelNamesQuery>,
) -> Result<schemas::StringArrayResponse> {
}

/// Returns the current filter wheel position
#[get("/filterwheel/<device_number>/position")]
fn get_filterwheel_position(
    schemas::GetFilterwheelPositionPath { device_number }: schemas::GetFilterwheelPositionPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetFilterwheelPositionQuery {},
    }: ASCOMRequest<schemas::GetFilterwheelPositionQuery>,
) -> Result<schemas::IntResponse> {
}

/// Sets the filter wheel position
#[put("/filterwheel/<device_number>/position")]
fn put_filterwheel_position(
    schemas::PutFilterwheelPositionPath { device_number }: schemas::PutFilterwheelPositionPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutFilterwheelPositionRequest { position },
    }: ASCOMRequest<schemas::PutFilterwheelPositionRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Indicates whether the focuser is capable of absolute position.

True if the focuser is capable of absolute position; that is, being commanded to a specific step location.
*/
#[get("/focuser/<device_number>/absolute")]
fn get_focuser_absolute(
    schemas::GetFocuserAbsolutePath { device_number }: schemas::GetFocuserAbsolutePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetFocuserAbsoluteQuery {},
    }: ASCOMRequest<schemas::GetFocuserAbsoluteQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the focuser is currently moving.

True if the focuser is currently moving to a new position. False if the focuser is stationary.
*/
#[get("/focuser/<device_number>/ismoving")]
fn get_focuser_ismoving(
    schemas::GetFocuserIsmovingPath { device_number }: schemas::GetFocuserIsmovingPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetFocuserIsmovingQuery {},
    }: ASCOMRequest<schemas::GetFocuserIsmovingQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Returns the focuser's maximum increment size.

Maximum increment size allowed by the focuser; i.e. the maximum number of steps allowed in one move operation.
*/
#[get("/focuser/<device_number>/maxincrement")]
fn get_focuser_maxincrement(
    schemas::GetFocuserMaxincrementPath { device_number }: schemas::GetFocuserMaxincrementPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetFocuserMaxincrementQuery {},
    }: ASCOMRequest<schemas::GetFocuserMaxincrementQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Returns the focuser's maximum step size.

Maximum step position permitted.
*/
#[get("/focuser/<device_number>/maxstep")]
fn get_focuser_maxstep(
    schemas::GetFocuserMaxstepPath { device_number }: schemas::GetFocuserMaxstepPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetFocuserMaxstepQuery {},
    }: ASCOMRequest<schemas::GetFocuserMaxstepQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Returns the focuser's current position.

Current focuser position, in steps.
*/
#[get("/focuser/<device_number>/position")]
fn get_focuser_position(
    schemas::GetFocuserPositionPath { device_number }: schemas::GetFocuserPositionPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetFocuserPositionQuery {},
    }: ASCOMRequest<schemas::GetFocuserPositionQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Returns the focuser's step size.

Step size (microns) for the focuser.
*/
#[get("/focuser/<device_number>/stepsize")]
fn get_focuser_stepsize(
    schemas::GetFocuserStepsizePath { device_number }: schemas::GetFocuserStepsizePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetFocuserStepsizeQuery {},
    }: ASCOMRequest<schemas::GetFocuserStepsizeQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Retrieves the state of temperature compensation mode

Gets the state of temperature compensation mode (if available), else always False.
*/
#[get("/focuser/<device_number>/tempcomp")]
fn get_focuser_tempcomp(
    schemas::GetFocuserTempcompPath { device_number }: schemas::GetFocuserTempcompPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetFocuserTempcompQuery {},
    }: ASCOMRequest<schemas::GetFocuserTempcompQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Sets the device's temperature compensation mode.

Sets the state of temperature compensation mode.
*/
#[put("/focuser/<device_number>/tempcomp")]
fn put_focuser_tempcomp(
    schemas::PutFocuserTempcompPath { device_number }: schemas::PutFocuserTempcompPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutFocuserTempcompRequest { temp_comp },
    }: ASCOMRequest<schemas::PutFocuserTempcompRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Indicates whether the focuser has temperature compensation.

True if focuser has temperature compensation available.
*/
#[get("/focuser/<device_number>/tempcompavailable")]
fn get_focuser_tempcompavailable(
    schemas::GetFocuserTempcompavailablePath { device_number }: schemas::GetFocuserTempcompavailablePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetFocuserTempcompavailableQuery {},
    }: ASCOMRequest<schemas::GetFocuserTempcompavailableQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Returns the focuser's current temperature.

Current ambient temperature as measured by the focuser.
*/
#[get("/focuser/<device_number>/temperature")]
fn get_focuser_temperature(
    schemas::GetFocuserTemperaturePath { device_number }: schemas::GetFocuserTemperaturePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetFocuserTemperatureQuery {},
    }: ASCOMRequest<schemas::GetFocuserTemperatureQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Immediatley stops focuser motion.

Immediately stop any focuser motion due to a previous Move(Int32) method call.
*/
#[put("/focuser/<device_number>/halt")]
fn put_focuser_halt(
    schemas::PutFocuserHaltPath { device_number }: schemas::PutFocuserHaltPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Moves the focuser to a new position.

Moves the focuser by the specified amount or to the specified position depending on the value of the Absolute property.
*/
#[put("/focuser/<device_number>/move")]
fn put_focuser_move(
    schemas::PutFocuserMovePath { device_number }: schemas::PutFocuserMovePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutFocuserMoveRequest { position },
    }: ASCOMRequest<schemas::PutFocuserMoveRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the time period over which observations will be averaged

Gets the time period over which observations will be averaged
*/
#[get("/observingconditions/<device_number>/averageperiod")]
fn get_observingconditions_averageperiod(
    schemas::GetObservingconditionsAverageperiodPath { device_number }: schemas::GetObservingconditionsAverageperiodPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetObservingconditionsAverageperiodQuery {},
    }: ASCOMRequest<schemas::GetObservingconditionsAverageperiodQuery>,
) -> Result<schemas::DoubleResponse> {
}

/// Sets the time period over which observations will be averaged
#[put("/observingconditions/<device_number>/averageperiod")]
fn put_observingconditions_averageperiod(
    schemas::PutObservingconditionsAverageperiodPath { device_number }: schemas::PutObservingconditionsAverageperiodPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutObservingconditionsAverageperiodRequest { average_period },
    }: ASCOMRequest<schemas::PutObservingconditionsAverageperiodRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the amount of sky obscured by cloud

Gets the percentage of the sky obscured by cloud
*/
#[get("/observingconditions/<device_number>/cloudcover")]
fn get_observingconditions_cloudcover(
    schemas::GetObservingconditionsCloudcoverPath { device_number }: schemas::GetObservingconditionsCloudcoverPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetObservingconditionsCloudcoverQuery {},
    }: ASCOMRequest<schemas::GetObservingconditionsCloudcoverQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the atmospheric dew point at the observatory

Gets the atmospheric dew point at the observatory reported in °C.
*/
#[get("/observingconditions/<device_number>/dewpoint")]
fn get_observingconditions_dewpoint(
    schemas::GetObservingconditionsDewpointPath { device_number }: schemas::GetObservingconditionsDewpointPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetObservingconditionsDewpointQuery {},
    }: ASCOMRequest<schemas::GetObservingconditionsDewpointQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the atmospheric humidity at the observatory

Gets the atmospheric  humidity (%) at the observatory
*/
#[get("/observingconditions/<device_number>/humidity")]
fn get_observingconditions_humidity(
    schemas::GetObservingconditionsHumidityPath { device_number }: schemas::GetObservingconditionsHumidityPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetObservingconditionsHumidityQuery {},
    }: ASCOMRequest<schemas::GetObservingconditionsHumidityQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the atmospheric pressure at the observatory.

Gets the atmospheric pressure in hectoPascals at the observatory's altitude - NOT reduced to sea level.
*/
#[get("/observingconditions/<device_number>/pressure")]
fn get_observingconditions_pressure(
    schemas::GetObservingconditionsPressurePath { device_number }: schemas::GetObservingconditionsPressurePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetObservingconditionsPressureQuery {},
    }: ASCOMRequest<schemas::GetObservingconditionsPressureQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the rain rate at the observatory.

Gets the rain rate (mm/hour) at the observatory.
*/
#[get("/observingconditions/<device_number>/rainrate")]
fn get_observingconditions_rainrate(
    schemas::GetObservingconditionsRainratePath { device_number }: schemas::GetObservingconditionsRainratePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetObservingconditionsRainrateQuery {},
    }: ASCOMRequest<schemas::GetObservingconditionsRainrateQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the sky brightness at the observatory

Gets the sky brightness at the observatory (Lux)
*/
#[get("/observingconditions/<device_number>/skybrightness")]
fn get_observingconditions_skybrightness(
    schemas::GetObservingconditionsSkybrightnessPath { device_number }: schemas::GetObservingconditionsSkybrightnessPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetObservingconditionsSkybrightnessQuery {},
    }: ASCOMRequest<schemas::GetObservingconditionsSkybrightnessQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the sky quality at the observatory

Gets the sky quality at the observatory (magnitudes per square arc second)
*/
#[get("/observingconditions/<device_number>/skyquality")]
fn get_observingconditions_skyquality(
    schemas::GetObservingconditionsSkyqualityPath { device_number }: schemas::GetObservingconditionsSkyqualityPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetObservingconditionsSkyqualityQuery {},
    }: ASCOMRequest<schemas::GetObservingconditionsSkyqualityQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the sky temperature at the observatory

Gets the sky temperature(°C) at the observatory.
*/
#[get("/observingconditions/<device_number>/skytemperature")]
fn get_observingconditions_skytemperature(
    schemas::GetObservingconditionsSkytemperaturePath { device_number }: schemas::GetObservingconditionsSkytemperaturePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetObservingconditionsSkytemperatureQuery {},
    }: ASCOMRequest<schemas::GetObservingconditionsSkytemperatureQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the seeing at the observatory

Gets the seeing at the observatory measured as star full width half maximum (FWHM) in arc secs.
*/
#[get("/observingconditions/<device_number>/starfwhm")]
fn get_observingconditions_starfwhm(
    schemas::GetObservingconditionsStarfwhmPath { device_number }: schemas::GetObservingconditionsStarfwhmPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetObservingconditionsStarfwhmQuery {},
    }: ASCOMRequest<schemas::GetObservingconditionsStarfwhmQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the temperature at the observatory

Gets the temperature(°C) at the observatory.
*/
#[get("/observingconditions/<device_number>/temperature")]
fn get_observingconditions_temperature(
    schemas::GetObservingconditionsTemperaturePath { device_number }: schemas::GetObservingconditionsTemperaturePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetObservingconditionsTemperatureQuery {},
    }: ASCOMRequest<schemas::GetObservingconditionsTemperatureQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the wind direction at the observatory

Gets the wind direction. The returned value must be between 0.0 and 360.0, interpreted according to the metereological standard, where a special value of 0.0 is returned when the wind speed is 0.0. Wind direction is measured clockwise from north, through east, where East=90.0, South=180.0, West=270.0 and North=360.0.
*/
#[get("/observingconditions/<device_number>/winddirection")]
fn get_observingconditions_winddirection(
    schemas::GetObservingconditionsWinddirectionPath { device_number }: schemas::GetObservingconditionsWinddirectionPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetObservingconditionsWinddirectionQuery {},
    }: ASCOMRequest<schemas::GetObservingconditionsWinddirectionQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the peak 3 second wind gust at the observatory over the last 2 minutes

Gets the peak 3 second wind gust(m/s) at the observatory over the last 2 minutes.
*/
#[get("/observingconditions/<device_number>/windgust")]
fn get_observingconditions_windgust(
    schemas::GetObservingconditionsWindgustPath { device_number }: schemas::GetObservingconditionsWindgustPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetObservingconditionsWindgustQuery {},
    }: ASCOMRequest<schemas::GetObservingconditionsWindgustQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the wind speed at the observatory.

Gets the wind speed(m/s) at the observatory.
*/
#[get("/observingconditions/<device_number>/windspeed")]
fn get_observingconditions_windspeed(
    schemas::GetObservingconditionsWindspeedPath { device_number }: schemas::GetObservingconditionsWindspeedPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetObservingconditionsWindspeedQuery {},
    }: ASCOMRequest<schemas::GetObservingconditionsWindspeedQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Refreshes sensor values from hardware.

Forces the driver to immediately query its attached hardware to refresh sensor values.
*/
#[put("/observingconditions/<device_number>/refresh")]
fn put_observingconditions_refresh(
    schemas::PutObservingconditionsRefreshPath { device_number }: schemas::PutObservingconditionsRefreshPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Return a sensor description

Gets a description of the sensor with the name specified in the SensorName parameter
*/
#[get("/observingconditions/<device_number>/sensordescription")]
fn get_observingconditions_sensordescription(
    schemas::GetObservingconditionsSensordescriptionPath { device_number }: schemas::GetObservingconditionsSensordescriptionPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetObservingconditionsSensordescriptionQuery { sensor_name },
    }: ASCOMRequest<schemas::GetObservingconditionsSensordescriptionQuery>,
) -> Result<schemas::StringResponse> {
}

/**
Return the time since the sensor value was last updated

Gets the time since the sensor specified in the SensorName parameter was last updated
*/
#[get("/observingconditions/<device_number>/timesincelastupdate")]
fn get_observingconditions_timesincelastupdate(
    schemas::GetObservingconditionsTimesincelastupdatePath { device_number }: schemas::GetObservingconditionsTimesincelastupdatePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetObservingconditionsTimesincelastupdateQuery { sensor_name },
    }: ASCOMRequest<schemas::GetObservingconditionsTimesincelastupdateQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
IIndicates whether the Rotator supports the Reverse method.

True if the Rotator supports the Reverse method.
*/
#[get("/rotator/<device_number>/canreverse")]
fn get_rotator_canreverse(
    schemas::GetRotatorCanreversePath { device_number }: schemas::GetRotatorCanreversePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetRotatorCanreverseQuery {},
    }: ASCOMRequest<schemas::GetRotatorCanreverseQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the rotator is currently moving.

True if the rotator is currently moving to a new position. False if the focuser is stationary.
*/
#[get("/rotator/<device_number>/ismoving")]
fn get_rotator_ismoving(
    schemas::GetRotatorIsmovingPath { device_number }: schemas::GetRotatorIsmovingPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetRotatorIsmovingQuery {},
    }: ASCOMRequest<schemas::GetRotatorIsmovingQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Returns the rotator's mechanical current position.

Returns the raw mechanical position of the rotator in degrees.
*/
#[get("/rotator/<device_number>/mechanicalposition")]
fn get_rotator_mechanicalposition(
    schemas::GetRotatorMechanicalpositionPath { device_number }: schemas::GetRotatorMechanicalpositionPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetRotatorMechanicalpositionQuery {},
    }: ASCOMRequest<schemas::GetRotatorMechanicalpositionQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the rotator's current position.

Current instantaneous Rotator position, in degrees.
*/
#[get("/rotator/<device_number>/position")]
fn get_rotator_position(
    schemas::GetRotatorPositionPath { device_number }: schemas::GetRotatorPositionPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetRotatorPositionQuery {},
    }: ASCOMRequest<schemas::GetRotatorPositionQuery>,
) -> Result<schemas::DoubleResponse> {
}

/// Returns the rotator’s Reverse state.
#[get("/rotator/<device_number>/reverse")]
fn get_rotator_reverse(
    schemas::GetRotatorReversePath { device_number }: schemas::GetRotatorReversePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetRotatorReverseQuery {},
    }: ASCOMRequest<schemas::GetRotatorReverseQuery>,
) -> Result<schemas::BoolResponse> {
}

/// Sets the rotator’s Reverse state.
#[put("/rotator/<device_number>/reverse")]
fn put_rotator_reverse(
    schemas::PutRotatorReversePath { device_number }: schemas::PutRotatorReversePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutRotatorReverseRequest { reverse },
    }: ASCOMRequest<schemas::PutRotatorReverseRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the minimum StepSize

The minimum StepSize, in degrees.
*/
#[get("/rotator/<device_number>/stepsize")]
fn get_rotator_stepsize(
    schemas::GetRotatorStepsizePath { device_number }: schemas::GetRotatorStepsizePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetRotatorStepsizeQuery {},
    }: ASCOMRequest<schemas::GetRotatorStepsizeQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the destination position angle.

The destination position angle for Move() and MoveAbsolute().
*/
#[get("/rotator/<device_number>/targetposition")]
fn get_rotator_targetposition(
    schemas::GetRotatorTargetpositionPath { device_number }: schemas::GetRotatorTargetpositionPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetRotatorTargetpositionQuery {},
    }: ASCOMRequest<schemas::GetRotatorTargetpositionQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Immediatley stops rotator motion.

Immediately stop any Rotator motion due to a previous Move or MoveAbsolute method call.
*/
#[put("/rotator/<device_number>/halt")]
fn put_rotator_halt(
    schemas::PutRotatorHaltPath { device_number }: schemas::PutRotatorHaltPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Moves the rotator to a new relative position.

Causes the rotator to move Position degrees relative to the current Position value.
*/
#[put("/rotator/<device_number>/move")]
fn put_rotator_move(
    schemas::PutRotatorMovePath { device_number }: schemas::PutRotatorMovePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutRotatorMoveRequest { position },
    }: ASCOMRequest<schemas::PutRotatorMoveRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Moves the rotator to a new absolute position.

Causes the rotator to move the absolute position of Position degrees.
*/
#[put("/rotator/<device_number>/moveabsolute")]
fn put_rotator_moveabsolute(
    schemas::PutRotatorMoveabsolutePath { device_number }: schemas::PutRotatorMoveabsolutePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutRotatorMoveabsoluteRequest { position },
    }: ASCOMRequest<schemas::PutRotatorMoveabsoluteRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Moves the rotator to a new raw mechanical position.

Causes the rotator to move the mechanical position of Position degrees.
*/
#[put("/rotator/<device_number>/movemechanical")]
fn put_rotator_movemechanical(
    schemas::PutRotatorMovemechanicalPath { device_number }: schemas::PutRotatorMovemechanicalPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutRotatorMovemechanicalRequest { position },
    }: ASCOMRequest<schemas::PutRotatorMovemechanicalRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Syncs the rotator to the specified position angle without moving it.

Causes the rotator to sync to the position of Position degrees.
*/
#[put("/rotator/<device_number>/sync")]
fn put_rotator_sync(
    schemas::PutRotatorSyncPath { device_number }: schemas::PutRotatorSyncPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutRotatorSyncRequest { position },
    }: ASCOMRequest<schemas::PutRotatorSyncRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Indicates whether the monitored state is safe for use.

Indicates whether the monitored state is safe for use. True if the state is safe, False if it is unsafe.
*/
#[get("/safetymonitor/<device_number>/issafe")]
fn get_safetymonitor_issafe(
    schemas::GetSafetymonitorIssafePath { device_number }: schemas::GetSafetymonitorIssafePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetSafetymonitorIssafeQuery {},
    }: ASCOMRequest<schemas::GetSafetymonitorIssafeQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
The number of switch devices managed by this driver

Returns the number of switch devices managed by this driver. Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/maxswitch")]
fn get_switch_maxswitch(
    schemas::GetSwitchMaxswitchPath { device_number }: schemas::GetSwitchMaxswitchPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetSwitchMaxswitchQuery {},
    }: ASCOMRequest<schemas::GetSwitchMaxswitchQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Indicates whether the specified switch device can be written to

Reports if the specified switch device can be written to, default true. This is false if the device cannot be written to, for example a limit switch or a sensor.  Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/canwrite")]
fn get_switch_canwrite(
    schemas::GetSwitchCanwritePath { device_number }: schemas::GetSwitchCanwritePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetSwitchCanwriteQuery { id },
    }: ASCOMRequest<schemas::GetSwitchCanwriteQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Return the state of switch device id as a boolean

Return the state of switch device id as a boolean.  Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/getswitch")]
fn get_switch_getswitch(
    schemas::GetSwitchGetswitchPath { device_number }: schemas::GetSwitchGetswitchPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetSwitchGetswitchQuery { id },
    }: ASCOMRequest<schemas::GetSwitchGetswitchQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Gets the description of the specified switch device

Gets the description of the specified switch device. This is to allow a fuller description of the device to be returned, for example for a tool tip. Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/getswitchdescription")]
fn get_switch_getswitchdescription(
    schemas::GetSwitchGetswitchdescriptionPath { device_number }: schemas::GetSwitchGetswitchdescriptionPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetSwitchGetswitchdescriptionQuery { id },
    }: ASCOMRequest<schemas::GetSwitchGetswitchdescriptionQuery>,
) -> Result<schemas::StringResponse> {
}

/**
Gets the name of the specified switch device

Gets the name of the specified switch device. Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/getswitchname")]
fn get_switch_getswitchname(
    schemas::GetSwitchGetswitchnamePath { device_number }: schemas::GetSwitchGetswitchnamePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetSwitchGetswitchnameQuery { id },
    }: ASCOMRequest<schemas::GetSwitchGetswitchnameQuery>,
) -> Result<schemas::StringResponse> {
}

/**
Gets the value of the specified switch device as a double

Gets the value of the specified switch device as a double. Devices are numbered from 0 to MaxSwitch - 1, The value of this switch is expected to be between MinSwitchValue and MaxSwitchValue.
*/
#[get("/switch/<device_number>/getswitchvalue")]
fn get_switch_getswitchvalue(
    schemas::GetSwitchGetswitchvaluePath { device_number }: schemas::GetSwitchGetswitchvaluePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetSwitchGetswitchvalueQuery { id },
    }: ASCOMRequest<schemas::GetSwitchGetswitchvalueQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Gets the minimum value of the specified switch device as a double

Gets the minimum value of the specified switch device as a double. Devices are numbered from 0 to MaxSwitch - 1.
*/
#[get("/switch/<device_number>/minswitchvalue")]
fn get_switch_minswitchvalue(
    schemas::GetSwitchMinswitchvaluePath { device_number }: schemas::GetSwitchMinswitchvaluePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetSwitchMinswitchvalueQuery { id },
    }: ASCOMRequest<schemas::GetSwitchMinswitchvalueQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Gets the maximum value of the specified switch device as a double

Gets the maximum value of the specified switch device as a double. Devices are numbered from 0 to MaxSwitch - 1.
*/
#[get("/switch/<device_number>/maxswitchvalue")]
fn get_switch_maxswitchvalue(
    schemas::GetSwitchMaxswitchvaluePath { device_number }: schemas::GetSwitchMaxswitchvaluePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetSwitchMaxswitchvalueQuery { id },
    }: ASCOMRequest<schemas::GetSwitchMaxswitchvalueQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Sets a switch controller device to the specified state, true or false

Sets a switch controller device to the specified state, true or false.
*/
#[put("/switch/<device_number>/setswitch")]
fn put_switch_setswitch(
    schemas::PutSwitchSetswitchPath { device_number }: schemas::PutSwitchSetswitchPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutSwitchSetswitchRequest { id, state },
    }: ASCOMRequest<schemas::PutSwitchSetswitchRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Sets a switch device name to the specified value

Sets a switch device name to the specified value.
*/
#[put("/switch/<device_number>/setswitchname")]
fn put_switch_setswitchname(
    schemas::PutSwitchSetswitchnamePath { device_number }: schemas::PutSwitchSetswitchnamePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutSwitchSetswitchnameRequest { id, name },
    }: ASCOMRequest<schemas::PutSwitchSetswitchnameRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Sets a switch device value to the specified value

Sets a switch device value to the specified value.
*/
#[put("/switch/<device_number>/setswitchvalue")]
fn put_switch_setswitchvalue(
    schemas::PutSwitchSetswitchvaluePath { device_number }: schemas::PutSwitchSetswitchvaluePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutSwitchSetswitchvalueRequest { id, value },
    }: ASCOMRequest<schemas::PutSwitchSetswitchvalueRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the step size that this device supports (the difference between successive values of the device).

Returns the step size that this device supports (the difference between successive values of the device). Devices are numbered from 0 to MaxSwitch - 1.
*/
#[get("/switch/<device_number>/switchstep")]
fn get_switch_switchstep(
    schemas::GetSwitchSwitchstepPath { device_number }: schemas::GetSwitchSwitchstepPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetSwitchSwitchstepQuery { id },
    }: ASCOMRequest<schemas::GetSwitchSwitchstepQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the current mount alignment mode

Returns the alignment mode of the mount (Alt/Az, Polar, German Polar).  The alignment mode is specified as an integer value from the AlignmentModes Enum.
*/
#[get("/telescope/<device_number>/alignmentmode")]
fn get_telescope_alignmentmode(
    schemas::GetTelescopeAlignmentmodePath { device_number }: schemas::GetTelescopeAlignmentmodePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeAlignmentmodeQuery {},
    }: ASCOMRequest<schemas::GetTelescopeAlignmentmodeQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Returns the mount's altitude above the horizon.

The altitude above the local horizon of the mount's current position (degrees, positive up)
*/
#[get("/telescope/<device_number>/altitude")]
fn get_telescope_altitude(
    schemas::GetTelescopeAltitudePath { device_number }: schemas::GetTelescopeAltitudePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeAltitudeQuery {},
    }: ASCOMRequest<schemas::GetTelescopeAltitudeQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the telescope's aperture.

The area of the telescope's aperture, taking into account any obstructions (square meters)
*/
#[get("/telescope/<device_number>/aperturearea")]
fn get_telescope_aperturearea(
    schemas::GetTelescopeApertureareaPath { device_number }: schemas::GetTelescopeApertureareaPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeApertureareaQuery {},
    }: ASCOMRequest<schemas::GetTelescopeApertureareaQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the telescope's effective aperture.

The telescope's effective aperture diameter (meters)
*/
#[get("/telescope/<device_number>/aperturediameter")]
fn get_telescope_aperturediameter(
    schemas::GetTelescopeAperturediameterPath { device_number }: schemas::GetTelescopeAperturediameterPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeAperturediameterQuery {},
    }: ASCOMRequest<schemas::GetTelescopeAperturediameterQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Indicates whether the mount is at the home position.

True if the mount is stopped in the Home position. Set only following a FindHome()  operation, and reset with any slew operation. This property must be False if the telescope does not support homing.
*/
#[get("/telescope/<device_number>/athome")]
fn get_telescope_athome(
    schemas::GetTelescopeAthomePath { device_number }: schemas::GetTelescopeAthomePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeAthomeQuery {},
    }: ASCOMRequest<schemas::GetTelescopeAthomeQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the telescope is at the park position.

True if the telescope has been put into the parked state by the seee Park()  method. Set False by calling the Unpark() method.
*/
#[get("/telescope/<device_number>/atpark")]
fn get_telescope_atpark(
    schemas::GetTelescopeAtparkPath { device_number }: schemas::GetTelescopeAtparkPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeAtparkQuery {},
    }: ASCOMRequest<schemas::GetTelescopeAtparkQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Returns the mount's azimuth.

The azimuth at the local horizon of the mount's current position (degrees, North-referenced, positive East/clockwise).
*/
#[get("/telescope/<device_number>/azimuth")]
fn get_telescope_azimuth(
    schemas::GetTelescopeAzimuthPath { device_number }: schemas::GetTelescopeAzimuthPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeAzimuthQuery {},
    }: ASCOMRequest<schemas::GetTelescopeAzimuthQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Indicates whether the mount can find the home position.

True if this telescope is capable of programmed finding its home position (FindHome()  method).
*/
#[get("/telescope/<device_number>/canfindhome")]
fn get_telescope_canfindhome(
    schemas::GetTelescopeCanfindhomePath { device_number }: schemas::GetTelescopeCanfindhomePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeCanfindhomeQuery {},
    }: ASCOMRequest<schemas::GetTelescopeCanfindhomeQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the telescope can be parked.

True if this telescope is capable of programmed parking (Park() method)
*/
#[get("/telescope/<device_number>/canpark")]
fn get_telescope_canpark(
    schemas::GetTelescopeCanparkPath { device_number }: schemas::GetTelescopeCanparkPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeCanparkQuery {},
    }: ASCOMRequest<schemas::GetTelescopeCanparkQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the telescope can be pulse guided.

True if this telescope is capable of software-pulsed guiding (via the PulseGuide(GuideDirections, Int32) method)
*/
#[get("/telescope/<device_number>/canpulseguide")]
fn get_telescope_canpulseguide(
    schemas::GetTelescopeCanpulseguidePath { device_number }: schemas::GetTelescopeCanpulseguidePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeCanpulseguideQuery {},
    }: ASCOMRequest<schemas::GetTelescopeCanpulseguideQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the DeclinationRate property can be changed.

True if the DeclinationRate property can be changed to provide offset tracking in the declination axis.
*/
#[get("/telescope/<device_number>/cansetdeclinationrate")]
fn get_telescope_cansetdeclinationrate(
    schemas::GetTelescopeCansetdeclinationratePath { device_number }: schemas::GetTelescopeCansetdeclinationratePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeCansetdeclinationrateQuery {},
    }: ASCOMRequest<schemas::GetTelescopeCansetdeclinationrateQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the DeclinationRate property can be changed.

True if the guide rate properties used for PulseGuide(GuideDirections, Int32) can ba adjusted.
*/
#[get("/telescope/<device_number>/cansetguiderates")]
fn get_telescope_cansetguiderates(
    schemas::GetTelescopeCansetguideratesPath { device_number }: schemas::GetTelescopeCansetguideratesPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeCansetguideratesQuery {},
    }: ASCOMRequest<schemas::GetTelescopeCansetguideratesQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the telescope park position can be set.

True if this telescope is capable of programmed setting of its park position (SetPark() method)
*/
#[get("/telescope/<device_number>/cansetpark")]
fn get_telescope_cansetpark(
    schemas::GetTelescopeCansetparkPath { device_number }: schemas::GetTelescopeCansetparkPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeCansetparkQuery {},
    }: ASCOMRequest<schemas::GetTelescopeCansetparkQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the telescope SideOfPier can be set.

True if the SideOfPier property can be set, meaning that the mount can be forced to flip.
*/
#[get("/telescope/<device_number>/cansetpierside")]
fn get_telescope_cansetpierside(
    schemas::GetTelescopeCansetpiersidePath { device_number }: schemas::GetTelescopeCansetpiersidePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeCansetpiersideQuery {},
    }: ASCOMRequest<schemas::GetTelescopeCansetpiersideQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the RightAscensionRate property can be changed.

True if the RightAscensionRate property can be changed to provide offset tracking in the right ascension axis. .
*/
#[get("/telescope/<device_number>/cansetrightascensionrate")]
fn get_telescope_cansetrightascensionrate(
    schemas::GetTelescopeCansetrightascensionratePath { device_number }: schemas::GetTelescopeCansetrightascensionratePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeCansetrightascensionrateQuery {},
    }: ASCOMRequest<schemas::GetTelescopeCansetrightascensionrateQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the Tracking property can be changed.

True if the Tracking property can be changed, turning telescope sidereal tracking on and off.
*/
#[get("/telescope/<device_number>/cansettracking")]
fn get_telescope_cansettracking(
    schemas::GetTelescopeCansettrackingPath { device_number }: schemas::GetTelescopeCansettrackingPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeCansettrackingQuery {},
    }: ASCOMRequest<schemas::GetTelescopeCansettrackingQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the telescope can slew synchronously.

True if this telescope is capable of programmed slewing (synchronous or asynchronous) to equatorial coordinates
*/
#[get("/telescope/<device_number>/canslew")]
fn get_telescope_canslew(
    schemas::GetTelescopeCanslewPath { device_number }: schemas::GetTelescopeCanslewPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeCanslewQuery {},
    }: ASCOMRequest<schemas::GetTelescopeCanslewQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the telescope can slew synchronously to AltAz coordinates.

True if this telescope is capable of programmed slewing (synchronous or asynchronous) to local horizontal coordinates
*/
#[get("/telescope/<device_number>/canslewaltaz")]
fn get_telescope_canslewaltaz(
    schemas::GetTelescopeCanslewaltazPath { device_number }: schemas::GetTelescopeCanslewaltazPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeCanslewaltazQuery {},
    }: ASCOMRequest<schemas::GetTelescopeCanslewaltazQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the telescope can slew asynchronously to AltAz coordinates.

True if this telescope is capable of programmed asynchronous slewing to local horizontal coordinates
*/
#[get("/telescope/<device_number>/canslewaltazasync")]
fn get_telescope_canslewaltazasync(
    schemas::GetTelescopeCanslewaltazasyncPath { device_number }: schemas::GetTelescopeCanslewaltazasyncPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeCanslewaltazasyncQuery {},
    }: ASCOMRequest<schemas::GetTelescopeCanslewaltazasyncQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the telescope can slew asynchronously.

True if this telescope is capable of programmed asynchronous slewing to equatorial coordinates.
*/
#[get("/telescope/<device_number>/canslewasync")]
fn get_telescope_canslewasync(
    schemas::GetTelescopeCanslewasyncPath { device_number }: schemas::GetTelescopeCanslewasyncPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeCanslewasyncQuery {},
    }: ASCOMRequest<schemas::GetTelescopeCanslewasyncQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the telescope can sync to equatorial coordinates.

True if this telescope is capable of programmed synching to equatorial coordinates.
*/
#[get("/telescope/<device_number>/cansync")]
fn get_telescope_cansync(
    schemas::GetTelescopeCansyncPath { device_number }: schemas::GetTelescopeCansyncPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeCansyncQuery {},
    }: ASCOMRequest<schemas::GetTelescopeCansyncQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the telescope can sync to local horizontal coordinates.

True if this telescope is capable of programmed synching to local horizontal coordinates
*/
#[get("/telescope/<device_number>/cansyncaltaz")]
fn get_telescope_cansyncaltaz(
    schemas::GetTelescopeCansyncaltazPath { device_number }: schemas::GetTelescopeCansyncaltazPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeCansyncaltazQuery {},
    }: ASCOMRequest<schemas::GetTelescopeCansyncaltazQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Indicates whether the telescope can be unparked.

True if this telescope is capable of programmed unparking (UnPark() method)
*/
#[get("/telescope/<device_number>/canunpark")]
fn get_telescope_canunpark(
    schemas::GetTelescopeCanunparkPath { device_number }: schemas::GetTelescopeCanunparkPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeCanunparkQuery {},
    }: ASCOMRequest<schemas::GetTelescopeCanunparkQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Returns the mount's declination.

The declination (degrees) of the mount's current equatorial coordinates, in the coordinate system given by the EquatorialSystem property. Reading the property will raise an error if the value is unavailable.
*/
#[get("/telescope/<device_number>/declination")]
fn get_telescope_declination(
    schemas::GetTelescopeDeclinationPath { device_number }: schemas::GetTelescopeDeclinationPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeDeclinationQuery {},
    }: ASCOMRequest<schemas::GetTelescopeDeclinationQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the telescope's declination tracking rate.

The declination tracking rate (arcseconds per second, default = 0.0)
*/
#[get("/telescope/<device_number>/declinationrate")]
fn get_telescope_declinationrate(
    schemas::GetTelescopeDeclinationratePath { device_number }: schemas::GetTelescopeDeclinationratePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeDeclinationrateQuery {},
    }: ASCOMRequest<schemas::GetTelescopeDeclinationrateQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Sets the telescope's declination tracking rate.

Sets the declination tracking rate (arcseconds per second)
*/
#[put("/telescope/<device_number>/declinationrate")]
fn put_telescope_declinationrate(
    schemas::PutTelescopeDeclinationratePath { device_number }: schemas::PutTelescopeDeclinationratePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeDeclinationrateRequest { declination_rate },
    }: ASCOMRequest<schemas::PutTelescopeDeclinationrateRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Indicates whether atmospheric refraction is applied to coordinates.

True if the telescope or driver applies atmospheric refraction to coordinates.
*/
#[get("/telescope/<device_number>/doesrefraction")]
fn get_telescope_doesrefraction(
    schemas::GetTelescopeDoesrefractionPath { device_number }: schemas::GetTelescopeDoesrefractionPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeDoesrefractionQuery {},
    }: ASCOMRequest<schemas::GetTelescopeDoesrefractionQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Determines whether atmospheric refraction is applied to coordinates.

Causes the rotator to move Position degrees relative to the current Position value.
*/
#[put("/telescope/<device_number>/doesrefraction")]
fn put_telescope_doesrefraction(
    schemas::PutTelescopeDoesrefractionPath { device_number }: schemas::PutTelescopeDoesrefractionPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeDoesrefractionRequest { does_refraction },
    }: ASCOMRequest<schemas::PutTelescopeDoesrefractionRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the current equatorial coordinate system used by this telescope.

Returns the current equatorial coordinate system used by this telescope (e.g. Topocentric or J2000).
*/
#[get("/telescope/<device_number>/equatorialsystem")]
fn get_telescope_equatorialsystem(
    schemas::GetTelescopeEquatorialsystemPath { device_number }: schemas::GetTelescopeEquatorialsystemPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeEquatorialsystemQuery {},
    }: ASCOMRequest<schemas::GetTelescopeEquatorialsystemQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Returns the telescope's focal length in meters.

The telescope's focal length in meters
*/
#[get("/telescope/<device_number>/focallength")]
fn get_telescope_focallength(
    schemas::GetTelescopeFocallengthPath { device_number }: schemas::GetTelescopeFocallengthPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeFocallengthQuery {},
    }: ASCOMRequest<schemas::GetTelescopeFocallengthQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the  current Declination rate offset for telescope guiding

The current Declination movement rate offset for telescope guiding (degrees/sec)
*/
#[get("/telescope/<device_number>/guideratedeclination")]
fn get_telescope_guideratedeclination(
    schemas::GetTelescopeGuideratedeclinationPath { device_number }: schemas::GetTelescopeGuideratedeclinationPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeGuideratedeclinationQuery {},
    }: ASCOMRequest<schemas::GetTelescopeGuideratedeclinationQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Sets the  current Declination rate offset for telescope guiding.

Sets the current Declination movement rate offset for telescope guiding (degrees/sec).
*/
#[put("/telescope/<device_number>/guideratedeclination")]
fn put_telescope_guideratedeclination(
    schemas::PutTelescopeGuideratedeclinationPath { device_number }: schemas::PutTelescopeGuideratedeclinationPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeGuideratedeclinationRequest { guide_rate_declination },
    }: ASCOMRequest<schemas::PutTelescopeGuideratedeclinationRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the  current RightAscension rate offset for telescope guiding

The current RightAscension movement rate offset for telescope guiding (degrees/sec)
*/
#[get("/telescope/<device_number>/guideraterightascension")]
fn get_telescope_guideraterightascension(
    schemas::GetTelescopeGuideraterightascensionPath { device_number }: schemas::GetTelescopeGuideraterightascensionPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeGuideraterightascensionQuery {},
    }: ASCOMRequest<schemas::GetTelescopeGuideraterightascensionQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Sets the  current RightAscension rate offset for telescope guiding.

Sets the current RightAscension movement rate offset for telescope guiding (degrees/sec).
*/
#[put("/telescope/<device_number>/guideraterightascension")]
fn put_telescope_guideraterightascension(
    schemas::PutTelescopeGuideraterightascensionPath { device_number }: schemas::PutTelescopeGuideraterightascensionPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeGuideraterightascensionRequest { guide_rate_right_ascension },
    }: ASCOMRequest<schemas::PutTelescopeGuideraterightascensionRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Indicates whether the telescope is currently executing a PulseGuide command

True if a PulseGuide(GuideDirections, Int32) command is in progress, False otherwise
*/
#[get("/telescope/<device_number>/ispulseguiding")]
fn get_telescope_ispulseguiding(
    schemas::GetTelescopeIspulseguidingPath { device_number }: schemas::GetTelescopeIspulseguidingPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeIspulseguidingQuery {},
    }: ASCOMRequest<schemas::GetTelescopeIspulseguidingQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Returns the mount's right ascension coordinate.

The right ascension (hours) of the mount's current equatorial coordinates, in the coordinate system given by the EquatorialSystem property
*/
#[get("/telescope/<device_number>/rightascension")]
fn get_telescope_rightascension(
    schemas::GetTelescopeRightascensionPath { device_number }: schemas::GetTelescopeRightascensionPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeRightascensionQuery {},
    }: ASCOMRequest<schemas::GetTelescopeRightascensionQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the telescope's right ascension tracking rate.

The right ascension tracking rate (arcseconds per second, default = 0.0)
*/
#[get("/telescope/<device_number>/rightascensionrate")]
fn get_telescope_rightascensionrate(
    schemas::GetTelescopeRightascensionratePath { device_number }: schemas::GetTelescopeRightascensionratePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeRightascensionrateQuery {},
    }: ASCOMRequest<schemas::GetTelescopeRightascensionrateQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Sets the telescope's right ascension tracking rate.

Sets the right ascension tracking rate (arcseconds per second)
*/
#[put("/telescope/<device_number>/rightascensionrate")]
fn put_telescope_rightascensionrate(
    schemas::PutTelescopeRightascensionratePath { device_number }: schemas::PutTelescopeRightascensionratePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeRightascensionrateRequest { right_ascension_rate },
    }: ASCOMRequest<schemas::PutTelescopeRightascensionrateRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the mount's pointing state.

Indicates the pointing state of the mount. 0 = pierEast, 1 = pierWest, -1= pierUnknown
*/
#[get("/telescope/<device_number>/sideofpier")]
fn get_telescope_sideofpier(
    schemas::GetTelescopeSideofpierPath { device_number }: schemas::GetTelescopeSideofpierPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeSideofpierQuery {},
    }: ASCOMRequest<schemas::GetTelescopeSideofpierQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Sets the mount's pointing state.

Sets the pointing state of the mount. 0 = pierEast, 1 = pierWest
*/
#[put("/telescope/<device_number>/sideofpier")]
fn put_telescope_sideofpier(
    schemas::PutTelescopeSideofpierPath { device_number }: schemas::PutTelescopeSideofpierPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSideofpierRequest { side_of_pier },
    }: ASCOMRequest<schemas::PutTelescopeSideofpierRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the local apparent sidereal time.

The local apparent sidereal time from the telescope's internal clock (hours, sidereal).
*/
#[get("/telescope/<device_number>/siderealtime")]
fn get_telescope_siderealtime(
    schemas::GetTelescopeSiderealtimePath { device_number }: schemas::GetTelescopeSiderealtimePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeSiderealtimeQuery {},
    }: ASCOMRequest<schemas::GetTelescopeSiderealtimeQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Returns the observing site's elevation above mean sea level.

The elevation above mean sea level (meters) of the site at which the telescope is located.
*/
#[get("/telescope/<device_number>/siteelevation")]
fn get_telescope_siteelevation(
    schemas::GetTelescopeSiteelevationPath { device_number }: schemas::GetTelescopeSiteelevationPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeSiteelevationQuery {},
    }: ASCOMRequest<schemas::GetTelescopeSiteelevationQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Sets the observing site's elevation above mean sea level.

Sets the elevation above mean sea level (metres) of the site at which the telescope is located.
*/
#[put("/telescope/<device_number>/siteelevation")]
fn put_telescope_siteelevation(
    schemas::PutTelescopeSiteelevationPath { device_number }: schemas::PutTelescopeSiteelevationPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSiteelevationRequest { site_elevation },
    }: ASCOMRequest<schemas::PutTelescopeSiteelevationRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the observing site's latitude.

The geodetic(map) latitude (degrees, positive North, WGS84) of the site at which the telescope is located.
*/
#[get("/telescope/<device_number>/sitelatitude")]
fn get_telescope_sitelatitude(
    schemas::GetTelescopeSitelatitudePath { device_number }: schemas::GetTelescopeSitelatitudePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeSitelatitudeQuery {},
    }: ASCOMRequest<schemas::GetTelescopeSitelatitudeQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Sets the observing site's latitude.

Sets the observing site's latitude (degrees).
*/
#[put("/telescope/<device_number>/sitelatitude")]
fn put_telescope_sitelatitude(
    schemas::PutTelescopeSitelatitudePath { device_number }: schemas::PutTelescopeSitelatitudePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSitelatitudeRequest { site_latitude },
    }: ASCOMRequest<schemas::PutTelescopeSitelatitudeRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the observing site's longitude.

The longitude (degrees, positive East, WGS84) of the site at which the telescope is located.
*/
#[get("/telescope/<device_number>/sitelongitude")]
fn get_telescope_sitelongitude(
    schemas::GetTelescopeSitelongitudePath { device_number }: schemas::GetTelescopeSitelongitudePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeSitelongitudeQuery {},
    }: ASCOMRequest<schemas::GetTelescopeSitelongitudeQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Sets the observing site's longitude.

Sets the observing site's longitude (degrees, positive East, WGS84).
*/
#[put("/telescope/<device_number>/sitelongitude")]
fn put_telescope_sitelongitude(
    schemas::PutTelescopeSitelongitudePath { device_number }: schemas::PutTelescopeSitelongitudePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSitelongitudeRequest { site_longitude },
    }: ASCOMRequest<schemas::PutTelescopeSitelongitudeRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Indicates whether the telescope is currently slewing.

True if telescope is currently moving in response to one of the Slew methods or the MoveAxis(TelescopeAxes, Double) method, False at all other times.
*/
#[get("/telescope/<device_number>/slewing")]
fn get_telescope_slewing(
    schemas::GetTelescopeSlewingPath { device_number }: schemas::GetTelescopeSlewingPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeSlewingQuery {},
    }: ASCOMRequest<schemas::GetTelescopeSlewingQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Returns the post-slew settling time.

Returns the post-slew settling time (sec.).
*/
#[get("/telescope/<device_number>/slewsettletime")]
fn get_telescope_slewsettletime(
    schemas::GetTelescopeSlewsettletimePath { device_number }: schemas::GetTelescopeSlewsettletimePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeSlewsettletimeQuery {},
    }: ASCOMRequest<schemas::GetTelescopeSlewsettletimeQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Sets the post-slew settling time.

Sets the  post-slew settling time (integer sec.).
*/
#[put("/telescope/<device_number>/slewsettletime")]
fn put_telescope_slewsettletime(
    schemas::PutTelescopeSlewsettletimePath { device_number }: schemas::PutTelescopeSlewsettletimePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSlewsettletimeRequest { slew_settle_time },
    }: ASCOMRequest<schemas::PutTelescopeSlewsettletimeRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the current target declination.

The declination (degrees, positive North) for the target of an equatorial slew or sync operation
*/
#[get("/telescope/<device_number>/targetdeclination")]
fn get_telescope_targetdeclination(
    schemas::GetTelescopeTargetdeclinationPath { device_number }: schemas::GetTelescopeTargetdeclinationPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeTargetdeclinationQuery {},
    }: ASCOMRequest<schemas::GetTelescopeTargetdeclinationQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Sets the target declination of a slew or sync.

Sets the declination (degrees, positive North) for the target of an equatorial slew or sync operation
*/
#[put("/telescope/<device_number>/targetdeclination")]
fn put_telescope_targetdeclination(
    schemas::PutTelescopeTargetdeclinationPath { device_number }: schemas::PutTelescopeTargetdeclinationPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeTargetdeclinationRequest { target_declination },
    }: ASCOMRequest<schemas::PutTelescopeTargetdeclinationRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the current target right ascension.

The right ascension (hours) for the target of an equatorial slew or sync operation
*/
#[get("/telescope/<device_number>/targetrightascension")]
fn get_telescope_targetrightascension(
    schemas::GetTelescopeTargetrightascensionPath { device_number }: schemas::GetTelescopeTargetrightascensionPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeTargetrightascensionQuery {},
    }: ASCOMRequest<schemas::GetTelescopeTargetrightascensionQuery>,
) -> Result<schemas::DoubleResponse> {
}

/**
Sets the target right ascension of a slew or sync.

Sets the right ascension (hours) for the target of an equatorial slew or sync operation
*/
#[put("/telescope/<device_number>/targetrightascension")]
fn put_telescope_targetrightascension(
    schemas::PutTelescopeTargetrightascensionPath { device_number }: schemas::PutTelescopeTargetrightascensionPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeTargetrightascensionRequest { target_right_ascension },
    }: ASCOMRequest<schemas::PutTelescopeTargetrightascensionRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Indicates whether the telescope is tracking.

Returns the state of the telescope's sidereal tracking drive.
*/
#[get("/telescope/<device_number>/tracking")]
fn get_telescope_tracking(
    schemas::GetTelescopeTrackingPath { device_number }: schemas::GetTelescopeTrackingPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeTrackingQuery {},
    }: ASCOMRequest<schemas::GetTelescopeTrackingQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Enables or disables telescope tracking.

Sets the state of the telescope's sidereal tracking drive.
*/
#[put("/telescope/<device_number>/tracking")]
fn put_telescope_tracking(
    schemas::PutTelescopeTrackingPath { device_number }: schemas::PutTelescopeTrackingPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeTrackingRequest { tracking },
    }: ASCOMRequest<schemas::PutTelescopeTrackingRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the current tracking rate.

The current tracking rate of the telescope's sidereal drive.
*/
#[get("/telescope/<device_number>/trackingrate")]
fn get_telescope_trackingrate(
    schemas::GetTelescopeTrackingratePath { device_number }: schemas::GetTelescopeTrackingratePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeTrackingrateQuery {},
    }: ASCOMRequest<schemas::GetTelescopeTrackingrateQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Sets the mount's tracking rate.

Sets the tracking rate of the telescope's sidereal drive. 0 = driveSidereal, 1 = driveLunar, 2 = driveSolar, 3 = driveKing
*/
#[put("/telescope/<device_number>/trackingrate")]
fn put_telescope_trackingrate(
    schemas::PutTelescopeTrackingratePath { device_number }: schemas::PutTelescopeTrackingratePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeTrackingrateRequest { tracking_rate },
    }: ASCOMRequest<schemas::PutTelescopeTrackingrateRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns a collection of supported DriveRates values.

Returns an array of supported DriveRates values that describe the permissible values of the TrackingRate property for this telescope type.
*/
#[get("/telescope/<device_number>/trackingrates")]
fn get_telescope_trackingrates(
    schemas::GetTelescopeTrackingratesPath { device_number }: schemas::GetTelescopeTrackingratesPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeTrackingratesQuery {},
    }: ASCOMRequest<schemas::GetTelescopeTrackingratesQuery>,
) -> Result<schemas::DriveRatesResponse> {
}

/**
Returns the UTC date/time of the telescope's internal clock.

The UTC date/time of the telescope's internal clock in ISO 8601 format including fractional seconds. The general format (in Microsoft custom date format style) is yyyy-MM-ddTHH:mm:ss.fffffffZ E.g. 2016-03-04T17:45:31.1234567Z or 2016-11-14T07:03:08.1234567Z Please note the compulsary trailing Z indicating the 'Zulu', UTC time zone.
*/
#[get("/telescope/<device_number>/utcdate")]
fn get_telescope_utcdate(
    schemas::GetTelescopeUtcdatePath { device_number }: schemas::GetTelescopeUtcdatePath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeUtcdateQuery {},
    }: ASCOMRequest<schemas::GetTelescopeUtcdateQuery>,
) -> Result<schemas::StringResponse> {
}

/**
Sets the UTC date/time of the telescope's internal clock.

The UTC date/time of the telescope's internal clock in ISO 8601 format including fractional seconds. The general format (in Microsoft custom date format style) is yyyy-MM-ddTHH:mm:ss.fffffffZ E.g. 2016-03-04T17:45:31.1234567Z or 2016-11-14T07:03:08.1234567Z Please note the compulsary trailing Z indicating the 'Zulu', UTC time zone.
*/
#[put("/telescope/<device_number>/utcdate")]
fn put_telescope_utcdate(
    schemas::PutTelescopeUtcdatePath { device_number }: schemas::PutTelescopeUtcdatePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeUtcdateRequest { utcdate },
    }: ASCOMRequest<schemas::PutTelescopeUtcdateRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Immediatley stops a slew in progress.

Immediately Stops a slew in progress.
*/
#[put("/telescope/<device_number>/abortslew")]
fn put_telescope_abortslew(
    schemas::PutTelescopeAbortslewPath { device_number }: schemas::PutTelescopeAbortslewPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Returns the rates at which the telescope may be moved about the specified axis.

The rates at which the telescope may be moved about the specified axis by the MoveAxis(TelescopeAxes, Double) method.
*/
#[get("/telescope/<device_number>/axisrates")]
fn get_telescope_axisrates(
    schemas::GetTelescopeAxisratesPath { device_number }: schemas::GetTelescopeAxisratesPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeAxisratesQuery { axis },
    }: ASCOMRequest<schemas::GetTelescopeAxisratesQuery>,
) -> Result<schemas::AxisRatesResponse> {
}

/**
Indicates whether the telescope can move the requested axis.

True if this telescope can move the requested axis.
*/
#[get("/telescope/<device_number>/canmoveaxis")]
fn get_telescope_canmoveaxis(
    schemas::GetTelescopeCanmoveaxisPath { device_number }: schemas::GetTelescopeCanmoveaxisPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeCanmoveaxisQuery { axis },
    }: ASCOMRequest<schemas::GetTelescopeCanmoveaxisQuery>,
) -> Result<schemas::BoolResponse> {
}

/**
Predicts the pointing state after a German equatorial mount slews to given coordinates.

Predicts the pointing state that a German equatorial mount will be in if it slews to the given coordinates. The  return value will be one of - 0 = pierEast, 1 = pierWest, -1 = pierUnknown
*/
#[get("/telescope/<device_number>/destinationsideofpier")]
fn get_telescope_destinationsideofpier(
    schemas::GetTelescopeDestinationsideofpierPath { device_number }: schemas::GetTelescopeDestinationsideofpierPath,
    ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeDestinationsideofpierQuery { right_ascension, declination },
    }: ASCOMRequest<schemas::GetTelescopeDestinationsideofpierQuery>,
) -> Result<schemas::IntResponse> {
}

/**
Moves the mount to the "home" position.

Locates the telescope's "home" position (synchronous)
*/
#[put("/telescope/<device_number>/findhome")]
fn put_telescope_findhome(
    schemas::PutTelescopeFindhomePath { device_number }: schemas::PutTelescopeFindhomePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Moves a telescope axis at the given rate.

Move the telescope in one axis at the given rate.
*/
#[put("/telescope/<device_number>/moveaxis")]
fn put_telescope_moveaxis(
    schemas::PutTelescopeMoveaxisPath { device_number }: schemas::PutTelescopeMoveaxisPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeMoveaxisRequest { axis, rate },
    }: ASCOMRequest<schemas::PutTelescopeMoveaxisRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Park the mount

Move the telescope to its park position, stop all motion (or restrict to a small safe range), and set AtPark to True. )
*/
#[put("/telescope/<device_number>/park")]
fn put_telescope_park(
    schemas::PutTelescopeParkPath { device_number }: schemas::PutTelescopeParkPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Moves the scope in the given direction for the given time.

Moves the scope in the given direction for the given interval or time at the rate given by the corresponding guide rate property
*/
#[put("/telescope/<device_number>/pulseguide")]
fn put_telescope_pulseguide(
    schemas::PutTelescopePulseguidePath { device_number }: schemas::PutTelescopePulseguidePath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopePulseguideRequest { direction, duration },
    }: ASCOMRequest<schemas::PutTelescopePulseguideRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Sets the telescope's park position

Sets the telescope's park position to be its current position.
*/
#[put("/telescope/<device_number>/setpark")]
fn put_telescope_setpark(
    schemas::PutTelescopeSetparkPath { device_number }: schemas::PutTelescopeSetparkPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Synchronously slew to the given local horizontal coordinates.

Move the telescope to the given local horizontal coordinates, return when slew is complete
*/
#[put("/telescope/<device_number>/slewtoaltaz")]
fn put_telescope_slewtoaltaz(
    schemas::PutTelescopeSlewtoaltazPath { device_number }: schemas::PutTelescopeSlewtoaltazPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSlewtoaltazRequest { azimuth, altitude },
    }: ASCOMRequest<schemas::PutTelescopeSlewtoaltazRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Asynchronously slew to the given local horizontal coordinates.

Move the telescope to the given local horizontal coordinates, return immediatley after the slew starts. The client can poll the Slewing method to determine when the mount reaches the intended coordinates.
*/
#[put("/telescope/<device_number>/slewtoaltazasync")]
fn put_telescope_slewtoaltazasync(
    schemas::PutTelescopeSlewtoaltazasyncPath { device_number }: schemas::PutTelescopeSlewtoaltazasyncPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSlewtoaltazRequest { azimuth, altitude },
    }: ASCOMRequest<schemas::PutTelescopeSlewtoaltazRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Synchronously slew to the given equatorial coordinates.

Move the telescope to the given equatorial coordinates, return when slew is complete
*/
#[put("/telescope/<device_number>/slewtocoordinates")]
fn put_telescope_slewtocoordinates(
    schemas::PutTelescopeSlewtocoordinatesPath { device_number }: schemas::PutTelescopeSlewtocoordinatesPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSlewtocoordinatesRequest { right_ascension, declination },
    }: ASCOMRequest<schemas::PutTelescopeSlewtocoordinatesRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Asynchronously slew to the given equatorial coordinates.

Move the telescope to the given equatorial coordinates, return immediatley after the slew starts. The client can poll the Slewing method to determine when the mount reaches the intended coordinates.
*/
#[put("/telescope/<device_number>/slewtocoordinatesasync")]
fn put_telescope_slewtocoordinatesasync(
    schemas::PutTelescopeSlewtocoordinatesasyncPath { device_number }: schemas::PutTelescopeSlewtocoordinatesasyncPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSlewtocoordinatesRequest { right_ascension, declination },
    }: ASCOMRequest<schemas::PutTelescopeSlewtocoordinatesRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Synchronously slew to the TargetRightAscension and TargetDeclination coordinates.

Move the telescope to the TargetRightAscension and TargetDeclination equatorial coordinates, return when slew is complete
*/
#[put("/telescope/<device_number>/slewtotarget")]
fn put_telescope_slewtotarget(
    schemas::PutTelescopeSlewtotargetPath { device_number }: schemas::PutTelescopeSlewtotargetPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Asynchronously slew to the TargetRightAscension and TargetDeclination coordinates.

Move the telescope to the TargetRightAscension and TargetDeclination equatorial coordinates, return immediatley after the slew starts. The client can poll the Slewing method to determine when the mount reaches the intended coordinates.
*/
#[put("/telescope/<device_number>/slewtotargetasync")]
fn put_telescope_slewtotargetasync(
    schemas::PutTelescopeSlewtotargetasyncPath { device_number }: schemas::PutTelescopeSlewtotargetasyncPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Syncs to the given local horizontal coordinates.

Matches the scope's local horizontal coordinates to the given local horizontal coordinates.
*/
#[put("/telescope/<device_number>/synctoaltaz")]
fn put_telescope_synctoaltaz(
    schemas::PutTelescopeSynctoaltazPath { device_number }: schemas::PutTelescopeSynctoaltazPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSlewtoaltazRequest { azimuth, altitude },
    }: ASCOMRequest<schemas::PutTelescopeSlewtoaltazRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Syncs to the given equatorial coordinates.

Matches the scope's equatorial coordinates to the given equatorial coordinates.
*/
#[put("/telescope/<device_number>/synctocoordinates")]
fn put_telescope_synctocoordinates(
    schemas::PutTelescopeSynctocoordinatesPath { device_number }: schemas::PutTelescopeSynctocoordinatesPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSlewtocoordinatesRequest { right_ascension, declination },
    }: ASCOMRequest<schemas::PutTelescopeSlewtocoordinatesRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Syncs to the TargetRightAscension and TargetDeclination coordinates.

Matches the scope's equatorial coordinates to the TargetRightAscension and TargetDeclination equatorial coordinates.
*/
#[put("/telescope/<device_number>/synctotarget")]
fn put_telescope_synctotarget(
    schemas::PutTelescopeSynctotargetPath { device_number }: schemas::PutTelescopeSynctotargetPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

/**
Unparks the mount.

Takes telescope out of the Parked state. )
*/
#[put("/telescope/<device_number>/unpark")]
fn put_telescope_unpark(
    schemas::PutTelescopeUnparkPath { device_number }: schemas::PutTelescopeUnparkPath,
    ASCOMRequest {
        transaction,
        request: schemas::PutCameraAbortexposureRequest {},
    }: ASCOMRequest<schemas::PutCameraAbortexposureRequest>,
) -> Result<schemas::MethodResponse> {
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(put_action)
            .service(put_commandblind)
            .service(put_commandbool)
            .service(put_commandstring)
            .service(get_connected)
            .service(put_connected)
            .service(get_description)
            .service(get_driverinfo)
            .service(get_driverversion)
            .service(get_interfaceversion)
            .service(get_name)
            .service(get_supportedactions)
            .service(get_camera_bayeroffsetx)
            .service(get_camera_bayeroffsety)
            .service(get_camera_binx)
            .service(put_camera_binx)
            .service(get_camera_biny)
            .service(put_camera_biny)
            .service(get_camera_camerastate)
            .service(get_camera_cameraxsize)
            .service(get_camera_cameraysize)
            .service(get_camera_canabortexposure)
            .service(get_camera_canasymmetricbin)
            .service(get_camera_canfastreadout)
            .service(get_camera_cangetcoolerpower)
            .service(get_camera_canpulseguide)
            .service(get_camera_cansetccdtemperature)
            .service(get_camera_canstopexposure)
            .service(get_camera_ccdtemperature)
            .service(get_camera_cooleron)
            .service(put_camera_cooleron)
            .service(get_camera_coolerpower)
            .service(get_camera_electronsperadu)
            .service(get_camera_exposuremax)
            .service(get_camera_exposuremin)
            .service(get_camera_exposureresolution)
            .service(get_camera_fastreadout)
            .service(put_camera_fastreadout)
            .service(get_camera_fullwellcapacity)
            .service(get_camera_gain)
            .service(put_camera_gain)
            .service(get_camera_gainmax)
            .service(get_camera_gainmin)
            .service(get_camera_gains)
            .service(get_camera_hasshutter)
            .service(get_camera_heatsinktemperature)
            .service(get_camera_imagearray)
            .service(get_camera_imagearrayvariant)
            .service(get_camera_imageready)
            .service(get_camera_ispulseguiding)
            .service(get_camera_lastexposureduration)
            .service(get_camera_lastexposurestarttime)
            .service(get_camera_maxadu)
            .service(get_camera_maxbinx)
            .service(get_camera_maxbiny)
            .service(get_camera_numx)
            .service(put_camera_numx)
            .service(get_camera_numy)
            .service(put_camera_numy)
            .service(get_camera_offset)
            .service(put_camera_offset)
            .service(get_camera_offsetmax)
            .service(get_camera_offsetmin)
            .service(get_camera_offsets)
            .service(get_camera_percentcompleted)
            .service(get_camera_pixelsizex)
            .service(get_camera_pixelsizey)
            .service(get_camera_readoutmode)
            .service(put_camera_readoutmode)
            .service(get_camera_readoutmodes)
            .service(get_camera_sensorname)
            .service(get_camera_sensortype)
            .service(get_camera_setccdtemperature)
            .service(put_camera_setccdtemperature)
            .service(get_camera_startx)
            .service(put_camera_startx)
            .service(get_camera_starty)
            .service(put_camera_starty)
            .service(get_camera_subexposureduration)
            .service(put_camera_subexposureduration)
            .service(put_camera_abortexposure)
            .service(put_camera_pulseguide)
            .service(put_camera_startexposure)
            .service(put_camera_stopexposure)
            .service(get_covercalibrator_brightness)
            .service(get_covercalibrator_calibratorstate)
            .service(get_covercalibrator_coverstate)
            .service(get_covercalibrator_maxbrightness)
            .service(put_covercalibrator_calibratoroff)
            .service(put_covercalibrator_calibratoron)
            .service(put_covercalibrator_closecover)
            .service(put_covercalibrator_haltcover)
            .service(put_covercalibrator_opencover)
            .service(get_dome_altitude)
            .service(get_dome_athome)
            .service(get_dome_atpark)
            .service(get_dome_azimuth)
            .service(get_dome_canfindhome)
            .service(get_dome_canpark)
            .service(get_dome_cansetaltitude)
            .service(get_dome_cansetazimuth)
            .service(get_dome_cansetpark)
            .service(get_dome_cansetshutter)
            .service(get_dome_canslave)
            .service(get_dome_cansyncazimuth)
            .service(get_dome_shutterstatus)
            .service(get_dome_slaved)
            .service(put_dome_slaved)
            .service(get_dome_slewing)
            .service(put_dome_abortslew)
            .service(put_dome_closeshutter)
            .service(put_dome_findhome)
            .service(put_dome_openshutter)
            .service(put_dome_park)
            .service(put_dome_setpark)
            .service(put_dome_slewtoaltitude)
            .service(put_dome_slewtoazimuth)
            .service(put_dome_synctoazimuth)
            .service(get_filterwheel_focusoffsets)
            .service(get_filterwheel_names)
            .service(get_filterwheel_position)
            .service(put_filterwheel_position)
            .service(get_focuser_absolute)
            .service(get_focuser_ismoving)
            .service(get_focuser_maxincrement)
            .service(get_focuser_maxstep)
            .service(get_focuser_position)
            .service(get_focuser_stepsize)
            .service(get_focuser_tempcomp)
            .service(put_focuser_tempcomp)
            .service(get_focuser_tempcompavailable)
            .service(get_focuser_temperature)
            .service(put_focuser_halt)
            .service(put_focuser_move)
            .service(get_observingconditions_averageperiod)
            .service(put_observingconditions_averageperiod)
            .service(get_observingconditions_cloudcover)
            .service(get_observingconditions_dewpoint)
            .service(get_observingconditions_humidity)
            .service(get_observingconditions_pressure)
            .service(get_observingconditions_rainrate)
            .service(get_observingconditions_skybrightness)
            .service(get_observingconditions_skyquality)
            .service(get_observingconditions_skytemperature)
            .service(get_observingconditions_starfwhm)
            .service(get_observingconditions_temperature)
            .service(get_observingconditions_winddirection)
            .service(get_observingconditions_windgust)
            .service(get_observingconditions_windspeed)
            .service(put_observingconditions_refresh)
            .service(get_observingconditions_sensordescription)
            .service(get_observingconditions_timesincelastupdate)
            .service(get_rotator_canreverse)
            .service(get_rotator_ismoving)
            .service(get_rotator_mechanicalposition)
            .service(get_rotator_position)
            .service(get_rotator_reverse)
            .service(put_rotator_reverse)
            .service(get_rotator_stepsize)
            .service(get_rotator_targetposition)
            .service(put_rotator_halt)
            .service(put_rotator_move)
            .service(put_rotator_moveabsolute)
            .service(put_rotator_movemechanical)
            .service(put_rotator_sync)
            .service(get_safetymonitor_issafe)
            .service(get_switch_maxswitch)
            .service(get_switch_canwrite)
            .service(get_switch_getswitch)
            .service(get_switch_getswitchdescription)
            .service(get_switch_getswitchname)
            .service(get_switch_getswitchvalue)
            .service(get_switch_minswitchvalue)
            .service(get_switch_maxswitchvalue)
            .service(put_switch_setswitch)
            .service(put_switch_setswitchname)
            .service(put_switch_setswitchvalue)
            .service(get_switch_switchstep)
            .service(get_telescope_alignmentmode)
            .service(get_telescope_altitude)
            .service(get_telescope_aperturearea)
            .service(get_telescope_aperturediameter)
            .service(get_telescope_athome)
            .service(get_telescope_atpark)
            .service(get_telescope_azimuth)
            .service(get_telescope_canfindhome)
            .service(get_telescope_canpark)
            .service(get_telescope_canpulseguide)
            .service(get_telescope_cansetdeclinationrate)
            .service(get_telescope_cansetguiderates)
            .service(get_telescope_cansetpark)
            .service(get_telescope_cansetpierside)
            .service(get_telescope_cansetrightascensionrate)
            .service(get_telescope_cansettracking)
            .service(get_telescope_canslew)
            .service(get_telescope_canslewaltaz)
            .service(get_telescope_canslewaltazasync)
            .service(get_telescope_canslewasync)
            .service(get_telescope_cansync)
            .service(get_telescope_cansyncaltaz)
            .service(get_telescope_canunpark)
            .service(get_telescope_declination)
            .service(get_telescope_declinationrate)
            .service(put_telescope_declinationrate)
            .service(get_telescope_doesrefraction)
            .service(put_telescope_doesrefraction)
            .service(get_telescope_equatorialsystem)
            .service(get_telescope_focallength)
            .service(get_telescope_guideratedeclination)
            .service(put_telescope_guideratedeclination)
            .service(get_telescope_guideraterightascension)
            .service(put_telescope_guideraterightascension)
            .service(get_telescope_ispulseguiding)
            .service(get_telescope_rightascension)
            .service(get_telescope_rightascensionrate)
            .service(put_telescope_rightascensionrate)
            .service(get_telescope_sideofpier)
            .service(put_telescope_sideofpier)
            .service(get_telescope_siderealtime)
            .service(get_telescope_siteelevation)
            .service(put_telescope_siteelevation)
            .service(get_telescope_sitelatitude)
            .service(put_telescope_sitelatitude)
            .service(get_telescope_sitelongitude)
            .service(put_telescope_sitelongitude)
            .service(get_telescope_slewing)
            .service(get_telescope_slewsettletime)
            .service(put_telescope_slewsettletime)
            .service(get_telescope_targetdeclination)
            .service(put_telescope_targetdeclination)
            .service(get_telescope_targetrightascension)
            .service(put_telescope_targetrightascension)
            .service(get_telescope_tracking)
            .service(put_telescope_tracking)
            .service(get_telescope_trackingrate)
            .service(put_telescope_trackingrate)
            .service(get_telescope_trackingrates)
            .service(get_telescope_utcdate)
            .service(put_telescope_utcdate)
            .service(put_telescope_abortslew)
            .service(get_telescope_axisrates)
            .service(get_telescope_canmoveaxis)
            .service(get_telescope_destinationsideofpier)
            .service(put_telescope_findhome)
            .service(put_telescope_moveaxis)
            .service(put_telescope_park)
            .service(put_telescope_pulseguide)
            .service(put_telescope_setpark)
            .service(put_telescope_slewtoaltaz)
            .service(put_telescope_slewtoaltazasync)
            .service(put_telescope_slewtocoordinates)
            .service(put_telescope_slewtocoordinatesasync)
            .service(put_telescope_slewtotarget)
            .service(put_telescope_slewtotargetasync)
            .service(put_telescope_synctoaltaz)
            .service(put_telescope_synctocoordinates)
            .service(put_telescope_synctotarget)
            .service(put_telescope_unpark)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
