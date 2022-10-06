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
    extract::{Path, Query},
    response::{IntoResponse, Response},
    routing::{get, put},
    Form, Json,
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Deserialize)]
pub struct TransactionRequest {
    #[serde(rename = "ClientID")]
    pub client_id: Option<u32>,
    #[serde(rename = "ClientTransactionID")]
    pub client_transaction_id: Option<u32>,
}

#[derive(Serialize)]
pub struct TransactionResponse {
    #[serde(rename = "ClientTransactionID")]
    pub client_transaction_id: Option<u32>,
    #[serde(rename = "ServerTransactionID")]
    pub server_transaction_id: u32,
}

#[derive(Deserialize)]
pub struct ASCOMRequest<T> {
    #[serde(flatten)]
    pub transaction: TransactionRequest,
    #[serde(flatten)]
    pub request: T,
}

#[derive(Serialize)]
pub struct ASCOMErrorCode(u16);

impl ASCOMErrorCode {
    /// The starting value for driver-specific error numbers.
    const DriverBase: u16 = 0x500;
    /// The maximum value for driver-specific error numbers.
    const DriverMax: u16 = 0xFFF;

    /// Generate a driver-specific error code.
    pub const fn new_for_driver(driver_error: u16) -> Self {
        let code = Self::DriverBase + driver_error;
        assert!(code <= Self::DriverMax, "Driver error code is too large");
        Self(code)
    }
}

#[derive(Serialize)]
pub struct ASCOMError {
    pub code: ASCOMErrorCode,
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
    #[serde(flatten)]
    result: std::result::Result<T, ASCOMError>,
}

impl<T: Serialize> IntoResponse for ASCOMResponse<T> {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

mod schemas {
    use super::*;

    #[derive(Serialize)]

    pub struct ImageArrayResponse {
        /// 0 = Unknown, 1 = Short(int16), 2 = Integer (int32), 3 = Double (Double precision real number).
        #[serde(rename = "Type")]
        pub type_: Option<i32>,

        /// The array's rank, will be 2 (single plane image (monochrome)) or 3 (multi-plane image).
        #[serde(rename = "Rank")]
        pub rank: Option<i32>,

        /// Returned integer or double value
        #[serde(rename = "Value")]
        pub value: Option<Vec<Vec<f64>>>,
    }

    #[derive(Serialize)]

    pub struct BoolResponse {
        /// True or False value
        #[serde(rename = "Value")]
        pub value: Option<bool>,
    }

    #[derive(Serialize)]

    pub struct DoubleResponse {
        /// Returned double value
        #[serde(rename = "Value")]
        pub value: Option<f64>,
    }

    #[derive(Serialize)]

    pub struct IntResponse {
        /// Returned integer value
        #[serde(rename = "Value")]
        pub value: Option<i32>,
    }

    #[derive(Serialize)]

    pub struct IntArrayResponse {
        /// Array of integer values.
        #[serde(rename = "Value")]
        pub value: Option<Vec<i32>>,
    }

    #[derive(Serialize)]

    pub struct StringResponse {
        /// String response from the device.
        #[serde(rename = "Value")]
        pub value: Option<String>,
    }

    #[derive(Serialize)]

    pub struct StringArrayResponse {
        /// Array of string values.
        #[serde(rename = "Value")]
        pub value: Option<Vec<String>>,
    }

    #[derive(Serialize)]

    pub struct AxisRatesResponse {
        /// Array of AxisRate objects
        #[serde(rename = "Value")]
        pub value: Option<Vec<schemas::AxisRate>>,
    }

    #[derive(Serialize)]

    pub struct AxisRate {
        /// The maximum rate (degrees per second) This must always be a positive number. It indicates the maximum rate in either direction about the axis.
        #[serde(rename = "Maximum")]
        pub maximum: f64,

        /// The minimum rate (degrees per second) This must always be a positive number. It indicates the maximum rate in either direction about the axis.
        #[serde(rename = "Minimum")]
        pub minimum: f64,
    }

    #[derive(Serialize)]

    pub struct DriveRatesResponse {
        /// Array of DriveRate values
        #[serde(rename = "Value")]
        pub value: Option<Vec<schemas::DriveRate>>,
    }

    #[derive(Serialize)]
    #[repr(transparent)]
    pub struct DriveRate(f64);

    #[derive(Deserialize)]

    pub struct PutActionRequest {
        /// A well known name that represents the action to be carried out.
        #[serde(rename = "Action")]
        pub action: String,

        /// List of required parameters or an Empty String if none are required
        #[serde(rename = "Parameters")]
        pub parameters: String,
    }

    #[derive(Deserialize)]

    pub struct PutCommandblindRequest {
        /// The literal command string to be transmitted.
        #[serde(rename = "Command")]
        pub command: String,

        /// If set to true the string is transmitted 'as-is', if set to false then protocol framing characters may be added prior to transmission
        #[serde(rename = "Raw")]
        pub raw: String,
    }

    #[derive(Deserialize)]

    pub struct PutConnectedRequest {
        /// Set True to connect to the device hardware, set False to disconnect from the device hardware
        #[serde(rename = "Connected")]
        pub connected: bool,
    }

    #[derive(Deserialize)]

    pub struct PutCameraBinxRequest {
        /// The X binning value
        #[serde(rename = "BinX")]
        pub bin_x: i32,
    }

    #[derive(Deserialize)]

    pub struct PutCameraBinyRequest {
        /// The Y binning value
        #[serde(rename = "BinY")]
        pub bin_y: i32,
    }

    #[derive(Deserialize)]

    pub struct PutCameraCooleronRequest {
        /// Cooler state
        #[serde(rename = "CoolerOn")]
        pub cooler_on: bool,
    }

    #[derive(Deserialize)]

    pub struct PutCameraFastreadoutRequest {
        /// True to enable fast readout mode
        #[serde(rename = "FastReadout")]
        pub fast_readout: bool,
    }

    #[derive(Deserialize)]

    pub struct PutCameraGainRequest {
        /// Index of the current camera gain in the Gains string array.
        #[serde(rename = "Gain")]
        pub gain: i32,
    }

    #[derive(Deserialize)]

    pub struct PutCameraNumxRequest {
        /// Sets the subframe width, if binning is active, value is in binned pixels.
        #[serde(rename = "NumX")]
        pub num_x: i32,
    }

    #[derive(Deserialize)]

    pub struct PutCameraNumyRequest {
        /// Sets the subframe height, if binning is active, value is in binned pixels.
        #[serde(rename = "NumY")]
        pub num_y: i32,
    }

    #[derive(Deserialize)]

    pub struct PutCameraOffsetRequest {
        /// Index of the current camera offset in the offsets string array.
        #[serde(rename = "offset")]
        pub offset: i32,
    }

    #[derive(Deserialize)]

    pub struct PutCameraReadoutmodeRequest {
        /// Index into the ReadoutModes array of string readout mode names indicating the camera's current readout mode.
        #[serde(rename = "ReadoutMode")]
        pub readout_mode: i32,
    }

    #[derive(Deserialize)]

    pub struct PutCameraSetccdtemperatureRequest {
        /// Temperature set point (degrees Celsius).
        #[serde(rename = "SetCCDTemperature")]
        pub set_ccdtemperature: f64,
    }

    #[derive(Deserialize)]

    pub struct PutCameraStartxRequest {
        /// The subframe X axis start position in binned pixels.
        #[serde(rename = "StartX")]
        pub start_x: i32,
    }

    #[derive(Deserialize)]

    pub struct PutCameraStartyRequest {
        /// The subframe Y axis start position in binned pixels.
        #[serde(rename = "StartY")]
        pub start_y: i32,
    }

    #[derive(Deserialize)]

    pub struct PutCameraSubexposuredurationRequest {
        /// The request sub exposure duration in seconds
        #[serde(rename = "SubExposureDuration")]
        pub sub_exposure_duration: f64,
    }

    #[derive(Deserialize)]

    pub struct PutCameraPulseguideRequest {
        /// Direction of movement (0 = North, 1 = South, 2 = East, 3 = West)
        #[serde(rename = "Direction")]
        pub direction: i32,

        /// Duration of movement in milli-seconds
        #[serde(rename = "Duration")]
        pub duration: i32,
    }

    #[derive(Deserialize)]

    pub struct PutCameraStartexposureRequest {
        /// Duration of exposure in seconds
        #[serde(rename = "Duration")]
        pub duration: f64,

        /// True if light frame, false if dark frame.
        #[serde(rename = "Light")]
        pub light: bool,
    }

    #[derive(Deserialize)]

    pub struct PutCovercalibratorCalibratoronRequest {
        /// The required brightness in the range 0 to MaxBrightness
        #[serde(rename = "Brightness")]
        pub brightness: Option<i32>,
    }

    #[derive(Deserialize)]

    pub struct PutDomeSlavedRequest {
        /// True if telescope is slaved to dome, otherwise false
        #[serde(rename = "Slaved")]
        pub slaved: bool,
    }

    #[derive(Deserialize)]

    pub struct PutDomeSlewtoaltitudeRequest {
        /// Target dome altitude (degrees, horizon zero and increasing positive to 90 zenith)
        #[serde(rename = "Altitude")]
        pub altitude: f64,
    }

    #[derive(Deserialize)]

    pub struct PutDomeSlewtoazimuthRequest {
        /// Target dome azimuth (degrees, North zero and increasing clockwise. i.e., 90 East, 180 South, 270 West)
        #[serde(rename = "Azimuth")]
        pub azimuth: f64,
    }

    #[derive(Deserialize)]

    pub struct PutFilterwheelPositionRequest {
        /// The number of the filter wheel position to select
        #[serde(rename = "Position")]
        pub position: i32,
    }

    #[derive(Deserialize)]

    pub struct PutFocuserTempcompPath {
        /// Zero based device number as set on the server
        #[serde(rename = "device_number")]
        pub device_number: i32,
    }

    #[derive(Deserialize)]

    pub struct PutFocuserTempcompRequest {
        /// Set true to enable the focuser's temperature compensation mode, otherwise false for normal operation.
        #[serde(rename = "TempComp")]
        pub temp_comp: bool,
    }

    #[derive(Deserialize)]

    pub struct PutFocuserMoveRequest {
        /// Step distance or absolute position, depending on the value of the Absolute property
        #[serde(rename = "Position")]
        pub position: i32,
    }

    #[derive(Deserialize)]

    pub struct PutObservingconditionsAverageperiodRequest {
        /// Time period (hours) over which to average sensor readings
        #[serde(rename = "AveragePeriod")]
        pub average_period: f64,
    }

    #[derive(Deserialize)]

    pub struct GetObservingconditionsSensordescriptionRequest {
        /// Name of the sensor whose description is required
        #[serde(rename = "SensorName")]
        pub sensor_name: String,
    }

    #[derive(Deserialize)]

    pub struct GetObservingconditionsTimesincelastupdateRequest {
        /// Name of the sensor whose last update time is required
        #[serde(rename = "SensorName")]
        pub sensor_name: String,
    }

    #[derive(Deserialize)]

    pub struct PutRotatorReverseRequest {
        /// True if the rotation and angular direction must be reversed to match the optical characteristcs
        #[serde(rename = "Reverse")]
        pub reverse: bool,
    }

    #[derive(Deserialize)]

    pub struct PutRotatorMoveRequest {
        /// Relative position to move in degrees from current Position.
        #[serde(rename = "Position")]
        pub position: f64,
    }

    #[derive(Deserialize)]

    pub struct PutRotatorMoveabsoluteRequest {
        /// Absolute position in degrees.
        #[serde(rename = "Position")]
        pub position: f64,
    }

    #[derive(Deserialize)]

    pub struct PutRotatorMovemechanicalRequest {
        /// Absolute position in degrees.
        #[serde(rename = "Position")]
        pub position: f64,
    }

    #[derive(Deserialize)]

    pub struct PutRotatorSyncRequest {
        /// Absolute position in degrees.
        #[serde(rename = "Position")]
        pub position: f64,
    }

    #[derive(Deserialize)]

    pub struct GetSwitchCanwriteRequest {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        pub id: i32,
    }

    #[derive(Deserialize)]

    pub struct GetSwitchGetswitchRequest {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        pub id: i32,
    }

    #[derive(Deserialize)]

    pub struct GetSwitchGetswitchdescriptionRequest {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        pub id: i32,
    }

    #[derive(Deserialize)]

    pub struct GetSwitchGetswitchnameRequest {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        pub id: i32,
    }

    #[derive(Deserialize)]

    pub struct GetSwitchGetswitchvalueRequest {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        pub id: i32,
    }

    #[derive(Deserialize)]

    pub struct GetSwitchMinswitchvalueRequest {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        pub id: i32,
    }

    #[derive(Deserialize)]

    pub struct GetSwitchMaxswitchvalueRequest {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        pub id: i32,
    }

    #[derive(Deserialize)]

    pub struct PutSwitchSetswitchRequest {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        pub id: i32,

        /// The required control state (True or False)
        #[serde(rename = "State")]
        pub state: bool,
    }

    #[derive(Deserialize)]

    pub struct PutSwitchSetswitchnameRequest {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        pub id: i32,

        /// The name of the device
        #[serde(rename = "Name")]
        pub name: String,
    }

    #[derive(Deserialize)]

    pub struct PutSwitchSetswitchvalueRequest {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        pub id: i32,

        /// The value to be set, between MinSwitchValue and MaxSwitchValue
        #[serde(rename = "Value")]
        pub value: f64,
    }

    #[derive(Deserialize)]

    pub struct GetSwitchSwitchstepRequest {
        /// The device number (0 to MaxSwitch - 1)
        #[serde(rename = "Id")]
        pub id: i32,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopeDeclinationrateRequest {
        /// Declination tracking rate (arcseconds per second)
        #[serde(rename = "DeclinationRate")]
        pub declination_rate: f64,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopeDoesrefractionRequest {
        /// Set True to make the telescope or driver applie atmospheric refraction to coordinates.
        #[serde(rename = "DoesRefraction")]
        pub does_refraction: bool,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopeGuideratedeclinationRequest {
        /// Declination movement rate offset degrees/sec).
        #[serde(rename = "GuideRateDeclination")]
        pub guide_rate_declination: f64,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopeGuideraterightascensionRequest {
        /// RightAscension movement rate offset degrees/sec).
        #[serde(rename = "GuideRateRightAscension")]
        pub guide_rate_right_ascension: f64,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopeRightascensionrateRequest {
        /// Right ascension tracking rate (arcseconds per second)
        #[serde(rename = "RightAscensionRate")]
        pub right_ascension_rate: f64,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopeSideofpierRequest {
        /// New pointing state.
        #[serde(rename = "SideOfPier")]
        pub side_of_pier: i32,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopeSiteelevationRequest {
        /// Elevation above mean sea level (metres).
        #[serde(rename = "SiteElevation")]
        pub site_elevation: f64,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopeSitelatitudeRequest {
        /// Site latitude (degrees)
        #[serde(rename = "SiteLatitude")]
        pub site_latitude: f64,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopeSitelongitudeRequest {
        /// Site longitude (degrees, positive East, WGS84)
        #[serde(rename = "SiteLongitude")]
        pub site_longitude: f64,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopeSlewsettletimeRequest {
        /// Settling time (integer sec.).
        #[serde(rename = "SlewSettleTime")]
        pub slew_settle_time: i32,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopeTargetdeclinationRequest {
        /// Target declination(degrees)
        #[serde(rename = "TargetDeclination")]
        pub target_declination: f64,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopeTargetrightascensionRequest {
        /// Target right ascension(hours)
        #[serde(rename = "TargetRightAscension")]
        pub target_right_ascension: f64,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopeTrackingRequest {
        /// Tracking enabled / disabled
        #[serde(rename = "Tracking")]
        pub tracking: bool,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopeTrackingrateRequest {
        /// New tracking rate
        #[serde(rename = "TrackingRate")]
        pub tracking_rate: i32,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopeUtcdateRequest {
        /// UTC date/time in ISO 8601 format.
        #[serde(rename = "UTCDate")]
        pub utcdate: String,
    }

    #[derive(Deserialize)]

    pub struct GetTelescopeAxisratesRequest {
        /// The axis about which rate information is desired. 0 = axisPrimary, 1 = axisSecondary, 2 = axisTertiary.
        #[serde(rename = "Axis")]
        pub axis: i32,
    }

    #[derive(Deserialize)]

    pub struct GetTelescopeCanmoveaxisRequest {
        /// The axis about which rate information is desired. 0 = axisPrimary, 1 = axisSecondary, 2 = axisTertiary.
        #[serde(rename = "Axis")]
        pub axis: i32,
    }

    #[derive(Deserialize)]

    pub struct GetTelescopeDestinationsideofpierRequest {
        /// Right Ascension coordinate (0.0 to 23.99999999 hours)
        #[serde(rename = "RightAscension")]
        pub right_ascension: f64,

        /// Declination coordinate (-90.0 to +90.0 degrees)
        #[serde(rename = "Declination")]
        pub declination: f64,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopeMoveaxisRequest {
        /// The axis about which rate information is desired. 0 = axisPrimary, 1 = axisSecondary, 2 = axisTertiary.
        #[serde(rename = "Axis")]
        pub axis: i32,

        /// The rate of motion (deg/sec) about the specified axis
        #[serde(rename = "Rate")]
        pub rate: f64,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopePulseguideRequest {
        /// The direction in which the guide-rate motion is to be made. 0 = guideNorth, 1 = guideSouth, 2 = guideEast, 3 = guideWest
        #[serde(rename = "Direction")]
        pub direction: i32,

        /// The duration of the guide-rate motion (milliseconds)
        #[serde(rename = "Duration")]
        pub duration: i32,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopeSlewtoaltazRequest {
        /// Azimuth coordinate (degrees, North-referenced, positive East/clockwise)
        #[serde(rename = "Azimuth")]
        pub azimuth: f64,

        /// Altitude coordinate (degrees, positive up)
        #[serde(rename = "Altitude")]
        pub altitude: f64,
    }

    #[derive(Deserialize)]

    pub struct PutTelescopeSlewtocoordinatesRequest {
        /// Right Ascension coordinate (hours)
        #[serde(rename = "RightAscension")]
        pub right_ascension: f64,

        /// Declination coordinate (degrees)
        #[serde(rename = "Declination")]
        pub declination: f64,
    }

    #[derive(Deserialize)]

    pub struct DeviceNumberPath {
        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        pub device_number: u32,
    }

    #[derive(Deserialize)]

    pub struct DeviceTypeAndNumberPath {
        /// One of the recognised ASCOM device types e.g. telescope (must be lower case)
        #[serde(rename = "device_type")]
        pub device_type: String,

        /// Zero based device number as set on the server (0 to 4294967295)
        #[serde(rename = "device_number")]
        pub device_number: u32,
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
    Path(schemas::DeviceTypeAndNumberPath { device_type, device_number }): Path<schemas::DeviceTypeAndNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutActionRequest { action, parameters },
    }): Form<ASCOMRequest<schemas::PutActionRequest>>,
) -> ASCOMResponse<schemas::StringResponse> {
    unimplemented!()
}

/**
Transmits an arbitrary string to the device

Transmits an arbitrary string to the device and does not wait for a response. Optionally, protocol framing characters may be added to the string before transmission.
*/
#[put("/<device_type>/<device_number>/commandblind")]
fn put_commandblind(
    Path(schemas::DeviceTypeAndNumberPath { device_type, device_number }): Path<schemas::DeviceTypeAndNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCommandblindRequest { command, raw },
    }): Form<ASCOMRequest<schemas::PutCommandblindRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Transmits an arbitrary string to the device and returns a boolean value from the device.

Transmits an arbitrary string to the device and waits for a boolean response. Optionally, protocol framing characters may be added to the string before transmission.
*/
#[put("/<device_type>/<device_number>/commandbool")]
fn put_commandbool(
    Path(schemas::DeviceTypeAndNumberPath { device_type, device_number }): Path<schemas::DeviceTypeAndNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCommandblindRequest { command, raw },
    }): Form<ASCOMRequest<schemas::PutCommandblindRequest>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Transmits an arbitrary string to the device and returns a string value from the device.

Transmits an arbitrary string to the device and waits for a string response. Optionally, protocol framing characters may be added to the string before transmission.
*/
#[put("/<device_type>/<device_number>/commandstring")]
fn put_commandstring(
    Path(schemas::DeviceTypeAndNumberPath { device_type, device_number }): Path<schemas::DeviceTypeAndNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCommandblindRequest { command, raw },
    }): Form<ASCOMRequest<schemas::PutCommandblindRequest>>,
) -> ASCOMResponse<schemas::StringResponse> {
    unimplemented!()
}

/// Retrieves the connected state of the device
#[get("/<device_type>/<device_number>/connected")]
fn get_connected(
    Path(schemas::DeviceTypeAndNumberPath { device_type, device_number }): Path<schemas::DeviceTypeAndNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/// Sets the connected state of the device
#[put("/<device_type>/<device_number>/connected")]
fn put_connected(
    Path(schemas::DeviceTypeAndNumberPath { device_type, device_number }): Path<schemas::DeviceTypeAndNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutConnectedRequest { connected },
    }): Form<ASCOMRequest<schemas::PutConnectedRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Device description

The description of the device
*/
#[get("/<device_type>/<device_number>/description")]
fn get_description(
    Path(schemas::DeviceTypeAndNumberPath { device_type, device_number }): Path<schemas::DeviceTypeAndNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::StringResponse> {
    unimplemented!()
}

/**
Device driver description

The description of the driver
*/
#[get("/<device_type>/<device_number>/driverinfo")]
fn get_driverinfo(
    Path(schemas::DeviceTypeAndNumberPath { device_type, device_number }): Path<schemas::DeviceTypeAndNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::StringResponse> {
    unimplemented!()
}

/**
Driver Version

A string containing only the major and minor version of the driver.
*/
#[get("/<device_type>/<device_number>/driverversion")]
fn get_driverversion(
    Path(schemas::DeviceTypeAndNumberPath { device_type, device_number }): Path<schemas::DeviceTypeAndNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::StringResponse> {
    unimplemented!()
}

/**
The ASCOM Device interface version number that this device supports.

This method returns the version of the ASCOM device interface contract to which this device complies. Only one interface version is current at a moment in time and all new devices should be built to the latest interface version. Applications can choose which device interface versions they support and it is in their interest to support  previous versions as well as the current version to ensure thay can use the largest number of devices.
*/
#[get("/<device_type>/<device_number>/interfaceversion")]
fn get_interfaceversion(
    Path(schemas::DeviceTypeAndNumberPath { device_type, device_number }): Path<schemas::DeviceTypeAndNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Device name

The name of the device
*/
#[get("/<device_type>/<device_number>/name")]
fn get_name(
    Path(schemas::DeviceTypeAndNumberPath { device_type, device_number }): Path<schemas::DeviceTypeAndNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::StringResponse> {
    unimplemented!()
}

/// Returns the list of action names supported by this driver.
#[get("/<device_type>/<device_number>/supportedactions")]
fn get_supportedactions(
    Path(schemas::DeviceTypeAndNumberPath { device_type, device_number }): Path<schemas::DeviceTypeAndNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::StringArrayResponse> {
    unimplemented!()
}

/**
Returns the X offset of the Bayer matrix.

Returns the X offset of the Bayer matrix, as defined in SensorType.
*/
#[get("/camera/<device_number>/bayeroffsetx")]
fn get_camera_bayeroffsetx(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Returns the Y offset of the Bayer matrix.

Returns the Y offset of the Bayer matrix, as defined in SensorType.
*/
#[get("/camera/<device_number>/bayeroffsety")]
fn get_camera_bayeroffsety(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/// Returns the binning factor for the X axis.
#[get("/camera/<device_number>/binx")]
fn get_camera_binx(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/// Sets the binning factor for the X axis.
#[put("/camera/<device_number>/binx")]
fn put_camera_binx(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCameraBinxRequest { bin_x },
    }): Form<ASCOMRequest<schemas::PutCameraBinxRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/// Returns the binning factor for the Y axis.
#[get("/camera/<device_number>/biny")]
fn get_camera_biny(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/// Sets the binning factor for the Y axis.
#[put("/camera/<device_number>/biny")]
fn put_camera_biny(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCameraBinyRequest { bin_y },
    }): Form<ASCOMRequest<schemas::PutCameraBinyRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the camera operational state.

Returns the current camera operational state as an integer. 0 = CameraIdle , 1 = CameraWaiting , 2 = CameraExposing , 3 = CameraReading , 4 = CameraDownload , 5 = CameraError
*/
#[get("/camera/<device_number>/camerastate")]
fn get_camera_camerastate(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Returns the width of the CCD camera chip.

Returns the width of the CCD camera chip in unbinned pixels.
*/
#[get("/camera/<device_number>/cameraxsize")]
fn get_camera_cameraxsize(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Returns the height of the CCD camera chip.

Returns the height of the CCD camera chip in unbinned pixels.
*/
#[get("/camera/<device_number>/cameraysize")]
fn get_camera_cameraysize(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Indicates whether the camera can abort exposures.

Returns true if the camera can abort exposures; false if not.
*/
#[get("/camera/<device_number>/canabortexposure")]
fn get_camera_canabortexposure(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the camera supports asymmetric binning

Returns a flag showing whether this camera supports asymmetric binning
*/
#[get("/camera/<device_number>/canasymmetricbin")]
fn get_camera_canasymmetricbin(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/// Indicates whether the camera has a fast readout mode.
#[get("/camera/<device_number>/canfastreadout")]
fn get_camera_canfastreadout(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the camera's cooler power setting can be read.

If true, the camera's cooler power setting can be read.
*/
#[get("/camera/<device_number>/cangetcoolerpower")]
fn get_camera_cangetcoolerpower(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Returns a flag indicating whether this camera supports pulse guiding

Returns a flag indicating whether this camera supports pulse guiding.
*/
#[get("/camera/<device_number>/canpulseguide")]
fn get_camera_canpulseguide(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Returns a flag indicating whether this camera supports setting the CCD temperature

Returns a flag indicatig whether this camera supports setting the CCD temperature
*/
#[get("/camera/<device_number>/cansetccdtemperature")]
fn get_camera_cansetccdtemperature(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/// Returns a flag indicating whether this camera can stop an exposure that is in progress
#[get("/camera/<device_number>/canstopexposure")]
fn get_camera_canstopexposure(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Returns the current CCD temperature

Returns the current CCD temperature in degrees Celsius.
*/
#[get("/camera/<device_number>/ccdtemperature")]
fn get_camera_ccdtemperature(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/// Returns the current cooler on/off state.
#[get("/camera/<device_number>/cooleron")]
fn get_camera_cooleron(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Turns the camera cooler on and off

Turns on and off the camera cooler. True = cooler on, False = cooler off
*/
#[put("/camera/<device_number>/cooleron")]
fn put_camera_cooleron(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCameraCooleronRequest { cooler_on },
    }): Form<ASCOMRequest<schemas::PutCameraCooleronRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the present cooler power level

Returns the present cooler power level, in percent.
*/
#[get("/camera/<device_number>/coolerpower")]
fn get_camera_coolerpower(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the gain of the camera

Returns the gain of the camera in photoelectrons per A/D unit.
*/
#[get("/camera/<device_number>/electronsperadu")]
fn get_camera_electronsperadu(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/// Returns the maximum exposure time supported by StartExposure.
#[get("/camera/<device_number>/exposuremax")]
fn get_camera_exposuremax(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the Minimium exposure time

Returns the Minimium exposure time in seconds that the camera supports through StartExposure.
*/
#[get("/camera/<device_number>/exposuremin")]
fn get_camera_exposuremin(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/// Returns the smallest increment in exposure time supported by StartExposure.
#[get("/camera/<device_number>/exposureresolution")]
fn get_camera_exposureresolution(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/// Returns whenther Fast Readout Mode is enabled.
#[get("/camera/<device_number>/fastreadout")]
fn get_camera_fastreadout(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/// Sets whether Fast Readout Mode is enabled.
#[put("/camera/<device_number>/fastreadout")]
fn put_camera_fastreadout(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCameraFastreadoutRequest { fast_readout },
    }): Form<ASCOMRequest<schemas::PutCameraFastreadoutRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Reports the full well capacity of the camera

Reports the full well capacity of the camera in electrons, at the current camera settings (binning, SetupDialog settings, etc.).
*/
#[get("/camera/<device_number>/fullwellcapacity")]
fn get_camera_fullwellcapacity(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the camera's gain

The camera's gain (GAIN VALUE MODE) OR the index of the selected camera gain description in the Gains array (GAINS INDEX MODE).
*/
#[get("/camera/<device_number>/gain")]
fn get_camera_gain(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Sets the camera's gain.

The camera's gain (GAIN VALUE MODE) OR the index of the selected camera gain description in the Gains array (GAINS INDEX MODE).
*/
#[put("/camera/<device_number>/gain")]
fn put_camera_gain(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCameraGainRequest { gain },
    }): Form<ASCOMRequest<schemas::PutCameraGainRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Maximum Gain value of that this camera supports

Returns the maximum value of Gain.
*/
#[get("/camera/<device_number>/gainmax")]
fn get_camera_gainmax(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Minimum Gain value of that this camera supports

Returns the Minimum value of Gain.
*/
#[get("/camera/<device_number>/gainmin")]
fn get_camera_gainmin(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
List of Gain names supported by the camera

Returns the Gains supported by the camera.
*/
#[get("/camera/<device_number>/gains")]
fn get_camera_gains(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::StringArrayResponse> {
    unimplemented!()
}

/**
Indicates whether the camera has a mechanical shutter

Returns a flag indicating whether this camera has a mechanical shutter.
*/
#[get("/camera/<device_number>/hasshutter")]
fn get_camera_hasshutter(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Returns the current heat sink temperature.

Returns the current heat sink temperature (called "ambient temperature" by some manufacturers) in degrees Celsius.
*/
#[get("/camera/<device_number>/heatsinktemperature")]
fn get_camera_heatsinktemperature(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
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
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::ImageArrayResponse> {
    unimplemented!()
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
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::ImageArrayResponse> {
    unimplemented!()
}

/**
Indicates that an image is ready to be downloaded

Returns a flag indicating whether the image is ready to be downloaded from the camera.
*/
#[get("/camera/<device_number>/imageready")]
fn get_camera_imageready(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates that the camera is pulse guideing.

Returns a flag indicating whether the camera is currrently in a PulseGuide operation.
*/
#[get("/camera/<device_number>/ispulseguiding")]
fn get_camera_ispulseguiding(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Duration of the last exposure

Reports the actual exposure duration in seconds (i.e. shutter open time).
*/
#[get("/camera/<device_number>/lastexposureduration")]
fn get_camera_lastexposureduration(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Start time of the last exposure in FITS standard format.

Reports the actual exposure start in the FITS-standard CCYY-MM-DDThh:mm:ss[.sss...] format.
*/
#[get("/camera/<device_number>/lastexposurestarttime")]
fn get_camera_lastexposurestarttime(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::StringResponse> {
    unimplemented!()
}

/**
Camera's maximum ADU value

Reports the maximum ADU value the camera can produce.
*/
#[get("/camera/<device_number>/maxadu")]
fn get_camera_maxadu(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Maximum  binning for the camera X axis

Returns the maximum allowed binning for the X camera axis
*/
#[get("/camera/<device_number>/maxbinx")]
fn get_camera_maxbinx(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Maximum  binning for the camera Y axis

Returns the maximum allowed binning for the Y camera axis
*/
#[get("/camera/<device_number>/maxbiny")]
fn get_camera_maxbiny(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Returns the current subframe width

Returns the current subframe width, if binning is active, value is in binned pixels.
*/
#[get("/camera/<device_number>/numx")]
fn get_camera_numx(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Sets the current subframe width

Sets the current subframe width.
*/
#[put("/camera/<device_number>/numx")]
fn put_camera_numx(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCameraNumxRequest { num_x },
    }): Form<ASCOMRequest<schemas::PutCameraNumxRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the current subframe height

Returns the current subframe height, if binning is active, value is in binned pixels.
*/
#[get("/camera/<device_number>/numy")]
fn get_camera_numy(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Sets the current subframe height

Sets the current subframe height.
*/
#[put("/camera/<device_number>/numy")]
fn put_camera_numy(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCameraNumyRequest { num_y },
    }): Form<ASCOMRequest<schemas::PutCameraNumyRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the camera's offset

Returns the camera's offset (OFFSET VALUE MODE) OR the index of the selected camera offset description in the offsets array (OFFSETS INDEX MODE).
*/
#[get("/camera/<device_number>/offset")]
fn get_camera_offset(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Sets the camera's offset.

Sets the camera's offset (OFFSET VALUE MODE) OR the index of the selected camera offset description in the offsets array (OFFSETS INDEX MODE).
*/
#[put("/camera/<device_number>/offset")]
fn put_camera_offset(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCameraOffsetRequest { offset },
    }): Form<ASCOMRequest<schemas::PutCameraOffsetRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Maximum offset value of that this camera supports

Returns the maximum value of offset.
*/
#[get("/camera/<device_number>/offsetmax")]
fn get_camera_offsetmax(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Minimum offset value of that this camera supports

Returns the Minimum value of offset.
*/
#[get("/camera/<device_number>/offsetmin")]
fn get_camera_offsetmin(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
List of offset names supported by the camera

Returns the offsets supported by the camera.
*/
#[get("/camera/<device_number>/offsets")]
fn get_camera_offsets(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::StringArrayResponse> {
    unimplemented!()
}

/**
Indicates percentage completeness of the current operation

Returns the percentage of the current operation that is complete. If valid, returns an integer between 0 and 100, where 0 indicates 0% progress (function just started) and 100 indicates 100% progress (i.e. completion).
*/
#[get("/camera/<device_number>/percentcompleted")]
fn get_camera_percentcompleted(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Width of CCD chip pixels (microns)

Returns the width of the CCD chip pixels in microns.
*/
#[get("/camera/<device_number>/pixelsizex")]
fn get_camera_pixelsizex(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Height of CCD chip pixels (microns)

Returns the Height of the CCD chip pixels in microns.
*/
#[get("/camera/<device_number>/pixelsizey")]
fn get_camera_pixelsizey(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Indicates the canera's readout mode as an index into the array ReadoutModes

ReadoutMode is an index into the array ReadoutModes and returns the desired readout mode for the camera. Defaults to 0 if not set.
*/
#[get("/camera/<device_number>/readoutmode")]
fn get_camera_readoutmode(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Set the camera's readout mode

Sets the ReadoutMode as an index into the array ReadoutModes.
*/
#[put("/camera/<device_number>/readoutmode")]
fn put_camera_readoutmode(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCameraReadoutmodeRequest { readout_mode },
    }): Form<ASCOMRequest<schemas::PutCameraReadoutmodeRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
List of available readout modes

This property provides an array of strings, each of which describes an available readout mode of the camera. At least one string must be present in the list.
*/
#[get("/camera/<device_number>/readoutmodes")]
fn get_camera_readoutmodes(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::StringArrayResponse> {
    unimplemented!()
}

/**
Sensor name

The name of the sensor used within the camera.
*/
#[get("/camera/<device_number>/sensorname")]
fn get_camera_sensorname(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::StringResponse> {
    unimplemented!()
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
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/// Returns the current camera cooler setpoint in degrees Celsius.
#[get("/camera/<device_number>/setccdtemperature")]
fn get_camera_setccdtemperature(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Set the camera's cooler setpoint (degrees Celsius).

Set's the camera's cooler setpoint in degrees Celsius.
*/
#[put("/camera/<device_number>/setccdtemperature")]
fn put_camera_setccdtemperature(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCameraSetccdtemperatureRequest { set_ccdtemperature },
    }): Form<ASCOMRequest<schemas::PutCameraSetccdtemperatureRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Return the current subframe X axis start position

Sets the subframe start position for the X axis (0 based) and returns the current value. If binning is active, value is in binned pixels.
*/
#[get("/camera/<device_number>/startx")]
fn get_camera_startx(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Sets the current subframe X axis start position

Sets the current subframe X axis start position in binned pixels.
*/
#[put("/camera/<device_number>/startx")]
fn put_camera_startx(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCameraStartxRequest { start_x },
    }): Form<ASCOMRequest<schemas::PutCameraStartxRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Return the current subframe Y axis start position

Sets the subframe start position for the Y axis (0 based) and returns the current value. If binning is active, value is in binned pixels.
*/
#[get("/camera/<device_number>/starty")]
fn get_camera_starty(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Sets the current subframe Y axis start position

Sets the current subframe Y axis start position in binned pixels.
*/
#[put("/camera/<device_number>/starty")]
fn put_camera_starty(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCameraStartyRequest { start_y },
    }): Form<ASCOMRequest<schemas::PutCameraStartyRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Camera's sub-exposure interval

The Camera's sub exposure duration in seconds. Only available in Camera Interface Version 3 and later.
*/
#[get("/camera/<device_number>/subexposureduration")]
fn get_camera_subexposureduration(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Sets the current Sub Exposure Duration

Sets image sub exposure duration in seconds. Only available in Camera Interface Version 3 and later.
*/
#[put("/camera/<device_number>/subexposureduration")]
fn put_camera_subexposureduration(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCameraSubexposuredurationRequest { sub_exposure_duration },
    }): Form<ASCOMRequest<schemas::PutCameraSubexposuredurationRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Aborts the current exposure

Aborts the current exposure, if any, and returns the camera to Idle state.
*/
#[put("/camera/<device_number>/abortexposure")]
fn put_camera_abortexposure(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Pulse guide in the specified direction for the specified time.

Activates the Camera's mount control sytem to instruct the mount to move in a particular direction for a given period of time
*/
#[put("/camera/<device_number>/pulseguide")]
fn put_camera_pulseguide(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCameraPulseguideRequest { direction, duration },
    }): Form<ASCOMRequest<schemas::PutCameraPulseguideRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Starts an exposure

Starts an exposure. Use ImageReady to check when the exposure is complete.
*/
#[put("/camera/<device_number>/startexposure")]
fn put_camera_startexposure(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCameraStartexposureRequest { duration, light },
    }): Form<ASCOMRequest<schemas::PutCameraStartexposureRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Stops the current exposure

Stops the current exposure, if any. If an exposure is in progress, the readout process is initiated. Ignored if readout is already in process.
*/
#[put("/camera/<device_number>/stopexposure")]
fn put_camera_stopexposure(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the current calibrator brightness

Returns the current calibrator brightness in the range 0 (completely off) to MaxBrightness (fully on)
*/
#[get("/covercalibrator/<device_number>/brightness")]
fn get_covercalibrator_brightness(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Returns the state of the calibration device

Returns the state of the calibration device, if present, otherwise returns "NotPresent".  The calibrator state mode is specified as an integer value from the CalibratorStatus Enum.
*/
#[get("/covercalibrator/<device_number>/calibratorstate")]
fn get_covercalibrator_calibratorstate(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Returns the state of the device cover"

Returns the state of the device cover, if present, otherwise returns "NotPresent".  The cover state mode is specified as an integer value from the CoverStatus Enum.
*/
#[get("/covercalibrator/<device_number>/coverstate")]
fn get_covercalibrator_coverstate(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Returns the calibrator's maximum Brightness value.

The Brightness value that makes the calibrator deliver its maximum illumination.
*/
#[get("/covercalibrator/<device_number>/maxbrightness")]
fn get_covercalibrator_maxbrightness(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Turns the calibrator off

Turns the calibrator off if the device has calibration capability.
*/
#[put("/covercalibrator/<device_number>/calibratoroff")]
fn put_covercalibrator_calibratoroff(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Turns the calibrator on at the specified brightness

Turns the calibrator on at the specified brightness if the device has calibration capability.
*/
#[put("/covercalibrator/<device_number>/calibratoron")]
fn put_covercalibrator_calibratoron(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutCovercalibratorCalibratoronRequest { brightness },
    }): Form<ASCOMRequest<schemas::PutCovercalibratorCalibratoronRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Initiates cover closing

Initiates cover closing if a cover is present.
*/
#[put("/covercalibrator/<device_number>/closecover")]
fn put_covercalibrator_closecover(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Stops any cover movement that may be in progress

Stops any cover movement that may be in progress if a cover is present and cover movement can be interrupted.
*/
#[put("/covercalibrator/<device_number>/haltcover")]
fn put_covercalibrator_haltcover(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Initiates cover opening

Initiates cover opening if a cover is present.
*/
#[put("/covercalibrator/<device_number>/opencover")]
fn put_covercalibrator_opencover(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
The dome altitude

The dome altitude (degrees, horizon zero and increasing positive to 90 zenith).
*/
#[get("/dome/<device_number>/altitude")]
fn get_dome_altitude(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Indicates whether the dome is in the home position.

Indicates whether the dome is in the home position. This is normally used following a FindHome()  operation. The value is reset with any azimuth slew operation that moves the dome away from the home position. AtHome may also become true durng normal slew operations, if the dome passes through the home position and the dome controller hardware is capable of detecting that; or at the end of a slew operation if the dome comes to rest at the home position.
*/
#[get("/dome/<device_number>/athome")]
fn get_dome_athome(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the telescope is at the park position

True if the dome is in the programmed park position. Set only following a Park() operation and reset with any slew operation.
*/
#[get("/dome/<device_number>/atpark")]
fn get_dome_atpark(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
The dome azimuth

Returns the dome azimuth (degrees, North zero and increasing clockwise, i.e., 90 East, 180 South, 270 West)
*/
#[get("/dome/<device_number>/azimuth")]
fn get_dome_azimuth(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Indicates whether the dome can find the home position.

True if the dome can move to the home position.
*/
#[get("/dome/<device_number>/canfindhome")]
fn get_dome_canfindhome(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the dome can be parked.

True if the dome is capable of programmed parking (Park() method)
*/
#[get("/dome/<device_number>/canpark")]
fn get_dome_canpark(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the dome altitude can be set

True if driver is capable of setting the dome altitude.
*/
#[get("/dome/<device_number>/cansetaltitude")]
fn get_dome_cansetaltitude(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the dome azimuth can be set

True if driver is capable of setting the dome azimuth.
*/
#[get("/dome/<device_number>/cansetazimuth")]
fn get_dome_cansetazimuth(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the dome park position can be set

True if driver is capable of setting the dome park position.
*/
#[get("/dome/<device_number>/cansetpark")]
fn get_dome_cansetpark(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the dome shutter can be opened

True if driver is capable of automatically operating shutter
*/
#[get("/dome/<device_number>/cansetshutter")]
fn get_dome_cansetshutter(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the dome supports slaving to a telescope

True if driver is capable of slaving to a telescope.
*/
#[get("/dome/<device_number>/canslave")]
fn get_dome_canslave(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the dome azimuth position can be synched

True if driver is capable of synchronizing the dome azimuth position using the SyncToAzimuth(Double) method.
*/
#[get("/dome/<device_number>/cansyncazimuth")]
fn get_dome_cansyncazimuth(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Status of the dome shutter or roll-off roof

Returns the status of the dome shutter or roll-off roof. 0 = Open, 1 = Closed, 2 = Opening, 3 = Closing, 4 = Shutter status error
*/
#[get("/dome/<device_number>/shutterstatus")]
fn get_dome_shutterstatus(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Indicates whether the dome is slaved to the telescope

True if the dome is slaved to the telescope in its hardware, else False.
*/
#[get("/dome/<device_number>/slaved")]
fn get_dome_slaved(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Sets whether the dome is slaved to the telescope

Sets the current subframe height.
*/
#[put("/dome/<device_number>/slaved")]
fn put_dome_slaved(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutDomeSlavedRequest { slaved },
    }): Form<ASCOMRequest<schemas::PutDomeSlavedRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Indicates whether the any part of the dome is moving

True if any part of the dome is currently moving, False if all dome components are steady.
*/
#[get("/dome/<device_number>/slewing")]
fn get_dome_slewing(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Immediately cancel current dome operation.

Calling this method will immediately disable hardware slewing (Slaved will become False).
*/
#[put("/dome/<device_number>/abortslew")]
fn put_dome_abortslew(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/// Close the shutter or otherwise shield telescope from the sky.
#[put("/dome/<device_number>/closeshutter")]
fn put_dome_closeshutter(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Start operation to search for the dome home position.

After Home position is established initializes Azimuth to the default value and sets the AtHome flag.
*/
#[put("/dome/<device_number>/findhome")]
fn put_dome_findhome(Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>, Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>) -> ASCOMResponse<()> {
    unimplemented!()
}

/// Open shutter or otherwise expose telescope to the sky.
#[put("/dome/<device_number>/openshutter")]
fn put_dome_openshutter(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Rotate dome in azimuth to park position.

After assuming programmed park position, sets AtPark flag.
*/
#[put("/dome/<device_number>/park")]
fn put_dome_park(Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>, Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Set the current azimuth, altitude position of dome to be the park position

Set the current azimuth, altitude position of dome to be the park position.
*/
#[put("/dome/<device_number>/setpark")]
fn put_dome_setpark(Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>, Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>) -> ASCOMResponse<()> {
    unimplemented!()
}

/// Slew the dome to the given altitude position.
#[put("/dome/<device_number>/slewtoaltitude")]
fn put_dome_slewtoaltitude(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutDomeSlewtoaltitudeRequest { altitude },
    }): Form<ASCOMRequest<schemas::PutDomeSlewtoaltitudeRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/// Slew the dome to the given azimuth position.
#[put("/dome/<device_number>/slewtoazimuth")]
fn put_dome_slewtoazimuth(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutDomeSlewtoazimuthRequest { azimuth },
    }): Form<ASCOMRequest<schemas::PutDomeSlewtoazimuthRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/// Synchronize the current position of the dome to the given azimuth.
#[put("/dome/<device_number>/synctoazimuth")]
fn put_dome_synctoazimuth(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutDomeSlewtoazimuthRequest { azimuth },
    }): Form<ASCOMRequest<schemas::PutDomeSlewtoazimuthRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Filter focus offsets

An integer array of filter focus offsets.
*/
#[get("/filterwheel/<device_number>/focusoffsets")]
fn get_filterwheel_focusoffsets(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntArrayResponse> {
    unimplemented!()
}

/**
Filter wheel filter names

The names of the filters
*/
#[get("/filterwheel/<device_number>/names")]
fn get_filterwheel_names(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::StringArrayResponse> {
    unimplemented!()
}

/// Returns the current filter wheel position
#[get("/filterwheel/<device_number>/position")]
fn get_filterwheel_position(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/// Sets the filter wheel position
#[put("/filterwheel/<device_number>/position")]
fn put_filterwheel_position(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutFilterwheelPositionRequest { position },
    }): Form<ASCOMRequest<schemas::PutFilterwheelPositionRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Indicates whether the focuser is capable of absolute position.

True if the focuser is capable of absolute position; that is, being commanded to a specific step location.
*/
#[get("/focuser/<device_number>/absolute")]
fn get_focuser_absolute(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the focuser is currently moving.

True if the focuser is currently moving to a new position. False if the focuser is stationary.
*/
#[get("/focuser/<device_number>/ismoving")]
fn get_focuser_ismoving(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Returns the focuser's maximum increment size.

Maximum increment size allowed by the focuser; i.e. the maximum number of steps allowed in one move operation.
*/
#[get("/focuser/<device_number>/maxincrement")]
fn get_focuser_maxincrement(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Returns the focuser's maximum step size.

Maximum step position permitted.
*/
#[get("/focuser/<device_number>/maxstep")]
fn get_focuser_maxstep(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Returns the focuser's current position.

Current focuser position, in steps.
*/
#[get("/focuser/<device_number>/position")]
fn get_focuser_position(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Returns the focuser's step size.

Step size (microns) for the focuser.
*/
#[get("/focuser/<device_number>/stepsize")]
fn get_focuser_stepsize(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Retrieves the state of temperature compensation mode

Gets the state of temperature compensation mode (if available), else always False.
*/
#[get("/focuser/<device_number>/tempcomp")]
fn get_focuser_tempcomp(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Sets the device's temperature compensation mode.

Sets the state of temperature compensation mode.
*/
#[put("/focuser/<device_number>/tempcomp")]
fn put_focuser_tempcomp(
    Path(schemas::PutFocuserTempcompPath { device_number }): Path<schemas::PutFocuserTempcompPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutFocuserTempcompRequest { temp_comp },
    }): Form<ASCOMRequest<schemas::PutFocuserTempcompRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Indicates whether the focuser has temperature compensation.

True if focuser has temperature compensation available.
*/
#[get("/focuser/<device_number>/tempcompavailable")]
fn get_focuser_tempcompavailable(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Returns the focuser's current temperature.

Current ambient temperature as measured by the focuser.
*/
#[get("/focuser/<device_number>/temperature")]
fn get_focuser_temperature(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Immediatley stops focuser motion.

Immediately stop any focuser motion due to a previous Move(Int32) method call.
*/
#[put("/focuser/<device_number>/halt")]
fn put_focuser_halt(Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>, Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Moves the focuser to a new position.

Moves the focuser by the specified amount or to the specified position depending on the value of the Absolute property.
*/
#[put("/focuser/<device_number>/move")]
fn put_focuser_move(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutFocuserMoveRequest { position },
    }): Form<ASCOMRequest<schemas::PutFocuserMoveRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the time period over which observations will be averaged

Gets the time period over which observations will be averaged
*/
#[get("/observingconditions/<device_number>/averageperiod")]
fn get_observingconditions_averageperiod(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/// Sets the time period over which observations will be averaged
#[put("/observingconditions/<device_number>/averageperiod")]
fn put_observingconditions_averageperiod(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutObservingconditionsAverageperiodRequest { average_period },
    }): Form<ASCOMRequest<schemas::PutObservingconditionsAverageperiodRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the amount of sky obscured by cloud

Gets the percentage of the sky obscured by cloud
*/
#[get("/observingconditions/<device_number>/cloudcover")]
fn get_observingconditions_cloudcover(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the atmospheric dew point at the observatory

Gets the atmospheric dew point at the observatory reported in °C.
*/
#[get("/observingconditions/<device_number>/dewpoint")]
fn get_observingconditions_dewpoint(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the atmospheric humidity at the observatory

Gets the atmospheric  humidity (%) at the observatory
*/
#[get("/observingconditions/<device_number>/humidity")]
fn get_observingconditions_humidity(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the atmospheric pressure at the observatory.

Gets the atmospheric pressure in hectoPascals at the observatory's altitude - NOT reduced to sea level.
*/
#[get("/observingconditions/<device_number>/pressure")]
fn get_observingconditions_pressure(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the rain rate at the observatory.

Gets the rain rate (mm/hour) at the observatory.
*/
#[get("/observingconditions/<device_number>/rainrate")]
fn get_observingconditions_rainrate(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the sky brightness at the observatory

Gets the sky brightness at the observatory (Lux)
*/
#[get("/observingconditions/<device_number>/skybrightness")]
fn get_observingconditions_skybrightness(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the sky quality at the observatory

Gets the sky quality at the observatory (magnitudes per square arc second)
*/
#[get("/observingconditions/<device_number>/skyquality")]
fn get_observingconditions_skyquality(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the sky temperature at the observatory

Gets the sky temperature(°C) at the observatory.
*/
#[get("/observingconditions/<device_number>/skytemperature")]
fn get_observingconditions_skytemperature(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the seeing at the observatory

Gets the seeing at the observatory measured as star full width half maximum (FWHM) in arc secs.
*/
#[get("/observingconditions/<device_number>/starfwhm")]
fn get_observingconditions_starfwhm(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the temperature at the observatory

Gets the temperature(°C) at the observatory.
*/
#[get("/observingconditions/<device_number>/temperature")]
fn get_observingconditions_temperature(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the wind direction at the observatory

Gets the wind direction. The returned value must be between 0.0 and 360.0, interpreted according to the metereological standard, where a special value of 0.0 is returned when the wind speed is 0.0. Wind direction is measured clockwise from north, through east, where East=90.0, South=180.0, West=270.0 and North=360.0.
*/
#[get("/observingconditions/<device_number>/winddirection")]
fn get_observingconditions_winddirection(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the peak 3 second wind gust at the observatory over the last 2 minutes

Gets the peak 3 second wind gust(m/s) at the observatory over the last 2 minutes.
*/
#[get("/observingconditions/<device_number>/windgust")]
fn get_observingconditions_windgust(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the wind speed at the observatory.

Gets the wind speed(m/s) at the observatory.
*/
#[get("/observingconditions/<device_number>/windspeed")]
fn get_observingconditions_windspeed(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Refreshes sensor values from hardware.

Forces the driver to immediately query its attached hardware to refresh sensor values.
*/
#[put("/observingconditions/<device_number>/refresh")]
fn put_observingconditions_refresh(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Return a sensor description

Gets a description of the sensor with the name specified in the SensorName parameter
*/
#[get("/observingconditions/<device_number>/sensordescription")]
fn get_observingconditions_sensordescription(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest {
        transaction,
        request: schemas::GetObservingconditionsSensordescriptionRequest { sensor_name },
    }): Query<ASCOMRequest<schemas::GetObservingconditionsSensordescriptionRequest>>,
) -> ASCOMResponse<schemas::StringResponse> {
    unimplemented!()
}

/**
Return the time since the sensor value was last updated

Gets the time since the sensor specified in the SensorName parameter was last updated
*/
#[get("/observingconditions/<device_number>/timesincelastupdate")]
fn get_observingconditions_timesincelastupdate(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest {
        transaction,
        request: schemas::GetObservingconditionsTimesincelastupdateRequest { sensor_name },
    }): Query<ASCOMRequest<schemas::GetObservingconditionsTimesincelastupdateRequest>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
IIndicates whether the Rotator supports the Reverse method.

True if the Rotator supports the Reverse method.
*/
#[get("/rotator/<device_number>/canreverse")]
fn get_rotator_canreverse(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the rotator is currently moving.

True if the rotator is currently moving to a new position. False if the focuser is stationary.
*/
#[get("/rotator/<device_number>/ismoving")]
fn get_rotator_ismoving(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Returns the rotator's mechanical current position.

Returns the raw mechanical position of the rotator in degrees.
*/
#[get("/rotator/<device_number>/mechanicalposition")]
fn get_rotator_mechanicalposition(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the rotator's current position.

Current instantaneous Rotator position, in degrees.
*/
#[get("/rotator/<device_number>/position")]
fn get_rotator_position(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/// Returns the rotator’s Reverse state.
#[get("/rotator/<device_number>/reverse")]
fn get_rotator_reverse(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/// Sets the rotator’s Reverse state.
#[put("/rotator/<device_number>/reverse")]
fn put_rotator_reverse(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutRotatorReverseRequest { reverse },
    }): Form<ASCOMRequest<schemas::PutRotatorReverseRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the minimum StepSize

The minimum StepSize, in degrees.
*/
#[get("/rotator/<device_number>/stepsize")]
fn get_rotator_stepsize(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the destination position angle.

The destination position angle for Move() and MoveAbsolute().
*/
#[get("/rotator/<device_number>/targetposition")]
fn get_rotator_targetposition(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Immediatley stops rotator motion.

Immediately stop any Rotator motion due to a previous Move or MoveAbsolute method call.
*/
#[put("/rotator/<device_number>/halt")]
fn put_rotator_halt(Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>, Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Moves the rotator to a new relative position.

Causes the rotator to move Position degrees relative to the current Position value.
*/
#[put("/rotator/<device_number>/move")]
fn put_rotator_move(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutRotatorMoveRequest { position },
    }): Form<ASCOMRequest<schemas::PutRotatorMoveRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Moves the rotator to a new absolute position.

Causes the rotator to move the absolute position of Position degrees.
*/
#[put("/rotator/<device_number>/moveabsolute")]
fn put_rotator_moveabsolute(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutRotatorMoveabsoluteRequest { position },
    }): Form<ASCOMRequest<schemas::PutRotatorMoveabsoluteRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Moves the rotator to a new raw mechanical position.

Causes the rotator to move the mechanical position of Position degrees.
*/
#[put("/rotator/<device_number>/movemechanical")]
fn put_rotator_movemechanical(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutRotatorMovemechanicalRequest { position },
    }): Form<ASCOMRequest<schemas::PutRotatorMovemechanicalRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Syncs the rotator to the specified position angle without moving it.

Causes the rotator to sync to the position of Position degrees.
*/
#[put("/rotator/<device_number>/sync")]
fn put_rotator_sync(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutRotatorSyncRequest { position },
    }): Form<ASCOMRequest<schemas::PutRotatorSyncRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Indicates whether the monitored state is safe for use.

Indicates whether the monitored state is safe for use. True if the state is safe, False if it is unsafe.
*/
#[get("/safetymonitor/<device_number>/issafe")]
fn get_safetymonitor_issafe(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
The number of switch devices managed by this driver

Returns the number of switch devices managed by this driver. Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/maxswitch")]
fn get_switch_maxswitch(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Indicates whether the specified switch device can be written to

Reports if the specified switch device can be written to, default true. This is false if the device cannot be written to, for example a limit switch or a sensor.  Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/canwrite")]
fn get_switch_canwrite(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest {
        transaction,
        request: schemas::GetSwitchCanwriteRequest { id },
    }): Query<ASCOMRequest<schemas::GetSwitchCanwriteRequest>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Return the state of switch device id as a boolean

Return the state of switch device id as a boolean.  Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/getswitch")]
fn get_switch_getswitch(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest {
        transaction,
        request: schemas::GetSwitchGetswitchRequest { id },
    }): Query<ASCOMRequest<schemas::GetSwitchGetswitchRequest>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Gets the description of the specified switch device

Gets the description of the specified switch device. This is to allow a fuller description of the device to be returned, for example for a tool tip. Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/getswitchdescription")]
fn get_switch_getswitchdescription(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest {
        transaction,
        request: schemas::GetSwitchGetswitchdescriptionRequest { id },
    }): Query<ASCOMRequest<schemas::GetSwitchGetswitchdescriptionRequest>>,
) -> ASCOMResponse<schemas::StringResponse> {
    unimplemented!()
}

/**
Gets the name of the specified switch device

Gets the name of the specified switch device. Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/getswitchname")]
fn get_switch_getswitchname(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest {
        transaction,
        request: schemas::GetSwitchGetswitchnameRequest { id },
    }): Query<ASCOMRequest<schemas::GetSwitchGetswitchnameRequest>>,
) -> ASCOMResponse<schemas::StringResponse> {
    unimplemented!()
}

/**
Gets the value of the specified switch device as a double

Gets the value of the specified switch device as a double. Devices are numbered from 0 to MaxSwitch - 1, The value of this switch is expected to be between MinSwitchValue and MaxSwitchValue.
*/
#[get("/switch/<device_number>/getswitchvalue")]
fn get_switch_getswitchvalue(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest {
        transaction,
        request: schemas::GetSwitchGetswitchvalueRequest { id },
    }): Query<ASCOMRequest<schemas::GetSwitchGetswitchvalueRequest>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Gets the minimum value of the specified switch device as a double

Gets the minimum value of the specified switch device as a double. Devices are numbered from 0 to MaxSwitch - 1.
*/
#[get("/switch/<device_number>/minswitchvalue")]
fn get_switch_minswitchvalue(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest {
        transaction,
        request: schemas::GetSwitchMinswitchvalueRequest { id },
    }): Query<ASCOMRequest<schemas::GetSwitchMinswitchvalueRequest>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Gets the maximum value of the specified switch device as a double

Gets the maximum value of the specified switch device as a double. Devices are numbered from 0 to MaxSwitch - 1.
*/
#[get("/switch/<device_number>/maxswitchvalue")]
fn get_switch_maxswitchvalue(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest {
        transaction,
        request: schemas::GetSwitchMaxswitchvalueRequest { id },
    }): Query<ASCOMRequest<schemas::GetSwitchMaxswitchvalueRequest>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Sets a switch controller device to the specified state, true or false

Sets a switch controller device to the specified state, true or false.
*/
#[put("/switch/<device_number>/setswitch")]
fn put_switch_setswitch(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutSwitchSetswitchRequest { id, state },
    }): Form<ASCOMRequest<schemas::PutSwitchSetswitchRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Sets a switch device name to the specified value

Sets a switch device name to the specified value.
*/
#[put("/switch/<device_number>/setswitchname")]
fn put_switch_setswitchname(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutSwitchSetswitchnameRequest { id, name },
    }): Form<ASCOMRequest<schemas::PutSwitchSetswitchnameRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Sets a switch device value to the specified value

Sets a switch device value to the specified value.
*/
#[put("/switch/<device_number>/setswitchvalue")]
fn put_switch_setswitchvalue(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutSwitchSetswitchvalueRequest { id, value },
    }): Form<ASCOMRequest<schemas::PutSwitchSetswitchvalueRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the step size that this device supports (the difference between successive values of the device).

Returns the step size that this device supports (the difference between successive values of the device). Devices are numbered from 0 to MaxSwitch - 1.
*/
#[get("/switch/<device_number>/switchstep")]
fn get_switch_switchstep(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest {
        transaction,
        request: schemas::GetSwitchSwitchstepRequest { id },
    }): Query<ASCOMRequest<schemas::GetSwitchSwitchstepRequest>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the current mount alignment mode

Returns the alignment mode of the mount (Alt/Az, Polar, German Polar).  The alignment mode is specified as an integer value from the AlignmentModes Enum.
*/
#[get("/telescope/<device_number>/alignmentmode")]
fn get_telescope_alignmentmode(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Returns the mount's altitude above the horizon.

The altitude above the local horizon of the mount's current position (degrees, positive up)
*/
#[get("/telescope/<device_number>/altitude")]
fn get_telescope_altitude(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the telescope's aperture.

The area of the telescope's aperture, taking into account any obstructions (square meters)
*/
#[get("/telescope/<device_number>/aperturearea")]
fn get_telescope_aperturearea(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the telescope's effective aperture.

The telescope's effective aperture diameter (meters)
*/
#[get("/telescope/<device_number>/aperturediameter")]
fn get_telescope_aperturediameter(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Indicates whether the mount is at the home position.

True if the mount is stopped in the Home position. Set only following a FindHome()  operation, and reset with any slew operation. This property must be False if the telescope does not support homing.
*/
#[get("/telescope/<device_number>/athome")]
fn get_telescope_athome(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the telescope is at the park position.

True if the telescope has been put into the parked state by the seee Park()  method. Set False by calling the Unpark() method.
*/
#[get("/telescope/<device_number>/atpark")]
fn get_telescope_atpark(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Returns the mount's azimuth.

The azimuth at the local horizon of the mount's current position (degrees, North-referenced, positive East/clockwise).
*/
#[get("/telescope/<device_number>/azimuth")]
fn get_telescope_azimuth(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Indicates whether the mount can find the home position.

True if this telescope is capable of programmed finding its home position (FindHome()  method).
*/
#[get("/telescope/<device_number>/canfindhome")]
fn get_telescope_canfindhome(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the telescope can be parked.

True if this telescope is capable of programmed parking (Park() method)
*/
#[get("/telescope/<device_number>/canpark")]
fn get_telescope_canpark(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the telescope can be pulse guided.

True if this telescope is capable of software-pulsed guiding (via the PulseGuide(GuideDirections, Int32) method)
*/
#[get("/telescope/<device_number>/canpulseguide")]
fn get_telescope_canpulseguide(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the DeclinationRate property can be changed.

True if the DeclinationRate property can be changed to provide offset tracking in the declination axis.
*/
#[get("/telescope/<device_number>/cansetdeclinationrate")]
fn get_telescope_cansetdeclinationrate(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the DeclinationRate property can be changed.

True if the guide rate properties used for PulseGuide(GuideDirections, Int32) can ba adjusted.
*/
#[get("/telescope/<device_number>/cansetguiderates")]
fn get_telescope_cansetguiderates(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the telescope park position can be set.

True if this telescope is capable of programmed setting of its park position (SetPark() method)
*/
#[get("/telescope/<device_number>/cansetpark")]
fn get_telescope_cansetpark(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the telescope SideOfPier can be set.

True if the SideOfPier property can be set, meaning that the mount can be forced to flip.
*/
#[get("/telescope/<device_number>/cansetpierside")]
fn get_telescope_cansetpierside(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the RightAscensionRate property can be changed.

True if the RightAscensionRate property can be changed to provide offset tracking in the right ascension axis. .
*/
#[get("/telescope/<device_number>/cansetrightascensionrate")]
fn get_telescope_cansetrightascensionrate(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the Tracking property can be changed.

True if the Tracking property can be changed, turning telescope sidereal tracking on and off.
*/
#[get("/telescope/<device_number>/cansettracking")]
fn get_telescope_cansettracking(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the telescope can slew synchronously.

True if this telescope is capable of programmed slewing (synchronous or asynchronous) to equatorial coordinates
*/
#[get("/telescope/<device_number>/canslew")]
fn get_telescope_canslew(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the telescope can slew synchronously to AltAz coordinates.

True if this telescope is capable of programmed slewing (synchronous or asynchronous) to local horizontal coordinates
*/
#[get("/telescope/<device_number>/canslewaltaz")]
fn get_telescope_canslewaltaz(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the telescope can slew asynchronously to AltAz coordinates.

True if this telescope is capable of programmed asynchronous slewing to local horizontal coordinates
*/
#[get("/telescope/<device_number>/canslewaltazasync")]
fn get_telescope_canslewaltazasync(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the telescope can slew asynchronously.

True if this telescope is capable of programmed asynchronous slewing to equatorial coordinates.
*/
#[get("/telescope/<device_number>/canslewasync")]
fn get_telescope_canslewasync(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the telescope can sync to equatorial coordinates.

True if this telescope is capable of programmed synching to equatorial coordinates.
*/
#[get("/telescope/<device_number>/cansync")]
fn get_telescope_cansync(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the telescope can sync to local horizontal coordinates.

True if this telescope is capable of programmed synching to local horizontal coordinates
*/
#[get("/telescope/<device_number>/cansyncaltaz")]
fn get_telescope_cansyncaltaz(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Indicates whether the telescope can be unparked.

True if this telescope is capable of programmed unparking (UnPark() method)
*/
#[get("/telescope/<device_number>/canunpark")]
fn get_telescope_canunpark(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Returns the mount's declination.

The declination (degrees) of the mount's current equatorial coordinates, in the coordinate system given by the EquatorialSystem property. Reading the property will raise an error if the value is unavailable.
*/
#[get("/telescope/<device_number>/declination")]
fn get_telescope_declination(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the telescope's declination tracking rate.

The declination tracking rate (arcseconds per second, default = 0.0)
*/
#[get("/telescope/<device_number>/declinationrate")]
fn get_telescope_declinationrate(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Sets the telescope's declination tracking rate.

Sets the declination tracking rate (arcseconds per second)
*/
#[put("/telescope/<device_number>/declinationrate")]
fn put_telescope_declinationrate(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeDeclinationrateRequest { declination_rate },
    }): Form<ASCOMRequest<schemas::PutTelescopeDeclinationrateRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Indicates whether atmospheric refraction is applied to coordinates.

True if the telescope or driver applies atmospheric refraction to coordinates.
*/
#[get("/telescope/<device_number>/doesrefraction")]
fn get_telescope_doesrefraction(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Determines whether atmospheric refraction is applied to coordinates.

Causes the rotator to move Position degrees relative to the current Position value.
*/
#[put("/telescope/<device_number>/doesrefraction")]
fn put_telescope_doesrefraction(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeDoesrefractionRequest { does_refraction },
    }): Form<ASCOMRequest<schemas::PutTelescopeDoesrefractionRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the current equatorial coordinate system used by this telescope.

Returns the current equatorial coordinate system used by this telescope (e.g. Topocentric or J2000).
*/
#[get("/telescope/<device_number>/equatorialsystem")]
fn get_telescope_equatorialsystem(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Returns the telescope's focal length in meters.

The telescope's focal length in meters
*/
#[get("/telescope/<device_number>/focallength")]
fn get_telescope_focallength(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the  current Declination rate offset for telescope guiding

The current Declination movement rate offset for telescope guiding (degrees/sec)
*/
#[get("/telescope/<device_number>/guideratedeclination")]
fn get_telescope_guideratedeclination(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Sets the  current Declination rate offset for telescope guiding.

Sets the current Declination movement rate offset for telescope guiding (degrees/sec).
*/
#[put("/telescope/<device_number>/guideratedeclination")]
fn put_telescope_guideratedeclination(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeGuideratedeclinationRequest { guide_rate_declination },
    }): Form<ASCOMRequest<schemas::PutTelescopeGuideratedeclinationRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the  current RightAscension rate offset for telescope guiding

The current RightAscension movement rate offset for telescope guiding (degrees/sec)
*/
#[get("/telescope/<device_number>/guideraterightascension")]
fn get_telescope_guideraterightascension(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Sets the  current RightAscension rate offset for telescope guiding.

Sets the current RightAscension movement rate offset for telescope guiding (degrees/sec).
*/
#[put("/telescope/<device_number>/guideraterightascension")]
fn put_telescope_guideraterightascension(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeGuideraterightascensionRequest { guide_rate_right_ascension },
    }): Form<ASCOMRequest<schemas::PutTelescopeGuideraterightascensionRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Indicates whether the telescope is currently executing a PulseGuide command

True if a PulseGuide(GuideDirections, Int32) command is in progress, False otherwise
*/
#[get("/telescope/<device_number>/ispulseguiding")]
fn get_telescope_ispulseguiding(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Returns the mount's right ascension coordinate.

The right ascension (hours) of the mount's current equatorial coordinates, in the coordinate system given by the EquatorialSystem property
*/
#[get("/telescope/<device_number>/rightascension")]
fn get_telescope_rightascension(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the telescope's right ascension tracking rate.

The right ascension tracking rate (arcseconds per second, default = 0.0)
*/
#[get("/telescope/<device_number>/rightascensionrate")]
fn get_telescope_rightascensionrate(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Sets the telescope's right ascension tracking rate.

Sets the right ascension tracking rate (arcseconds per second)
*/
#[put("/telescope/<device_number>/rightascensionrate")]
fn put_telescope_rightascensionrate(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeRightascensionrateRequest { right_ascension_rate },
    }): Form<ASCOMRequest<schemas::PutTelescopeRightascensionrateRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the mount's pointing state.

Indicates the pointing state of the mount. 0 = pierEast, 1 = pierWest, -1= pierUnknown
*/
#[get("/telescope/<device_number>/sideofpier")]
fn get_telescope_sideofpier(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Sets the mount's pointing state.

Sets the pointing state of the mount. 0 = pierEast, 1 = pierWest
*/
#[put("/telescope/<device_number>/sideofpier")]
fn put_telescope_sideofpier(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSideofpierRequest { side_of_pier },
    }): Form<ASCOMRequest<schemas::PutTelescopeSideofpierRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the local apparent sidereal time.

The local apparent sidereal time from the telescope's internal clock (hours, sidereal).
*/
#[get("/telescope/<device_number>/siderealtime")]
fn get_telescope_siderealtime(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Returns the observing site's elevation above mean sea level.

The elevation above mean sea level (meters) of the site at which the telescope is located.
*/
#[get("/telescope/<device_number>/siteelevation")]
fn get_telescope_siteelevation(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Sets the observing site's elevation above mean sea level.

Sets the elevation above mean sea level (metres) of the site at which the telescope is located.
*/
#[put("/telescope/<device_number>/siteelevation")]
fn put_telescope_siteelevation(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSiteelevationRequest { site_elevation },
    }): Form<ASCOMRequest<schemas::PutTelescopeSiteelevationRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the observing site's latitude.

The geodetic(map) latitude (degrees, positive North, WGS84) of the site at which the telescope is located.
*/
#[get("/telescope/<device_number>/sitelatitude")]
fn get_telescope_sitelatitude(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Sets the observing site's latitude.

Sets the observing site's latitude (degrees).
*/
#[put("/telescope/<device_number>/sitelatitude")]
fn put_telescope_sitelatitude(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSitelatitudeRequest { site_latitude },
    }): Form<ASCOMRequest<schemas::PutTelescopeSitelatitudeRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the observing site's longitude.

The longitude (degrees, positive East, WGS84) of the site at which the telescope is located.
*/
#[get("/telescope/<device_number>/sitelongitude")]
fn get_telescope_sitelongitude(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Sets the observing site's longitude.

Sets the observing site's longitude (degrees, positive East, WGS84).
*/
#[put("/telescope/<device_number>/sitelongitude")]
fn put_telescope_sitelongitude(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSitelongitudeRequest { site_longitude },
    }): Form<ASCOMRequest<schemas::PutTelescopeSitelongitudeRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Indicates whether the telescope is currently slewing.

True if telescope is currently moving in response to one of the Slew methods or the MoveAxis(TelescopeAxes, Double) method, False at all other times.
*/
#[get("/telescope/<device_number>/slewing")]
fn get_telescope_slewing(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Returns the post-slew settling time.

Returns the post-slew settling time (sec.).
*/
#[get("/telescope/<device_number>/slewsettletime")]
fn get_telescope_slewsettletime(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Sets the post-slew settling time.

Sets the  post-slew settling time (integer sec.).
*/
#[put("/telescope/<device_number>/slewsettletime")]
fn put_telescope_slewsettletime(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSlewsettletimeRequest { slew_settle_time },
    }): Form<ASCOMRequest<schemas::PutTelescopeSlewsettletimeRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the current target declination.

The declination (degrees, positive North) for the target of an equatorial slew or sync operation
*/
#[get("/telescope/<device_number>/targetdeclination")]
fn get_telescope_targetdeclination(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Sets the target declination of a slew or sync.

Sets the declination (degrees, positive North) for the target of an equatorial slew or sync operation
*/
#[put("/telescope/<device_number>/targetdeclination")]
fn put_telescope_targetdeclination(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeTargetdeclinationRequest { target_declination },
    }): Form<ASCOMRequest<schemas::PutTelescopeTargetdeclinationRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the current target right ascension.

The right ascension (hours) for the target of an equatorial slew or sync operation
*/
#[get("/telescope/<device_number>/targetrightascension")]
fn get_telescope_targetrightascension(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DoubleResponse> {
    unimplemented!()
}

/**
Sets the target right ascension of a slew or sync.

Sets the right ascension (hours) for the target of an equatorial slew or sync operation
*/
#[put("/telescope/<device_number>/targetrightascension")]
fn put_telescope_targetrightascension(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeTargetrightascensionRequest { target_right_ascension },
    }): Form<ASCOMRequest<schemas::PutTelescopeTargetrightascensionRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Indicates whether the telescope is tracking.

Returns the state of the telescope's sidereal tracking drive.
*/
#[get("/telescope/<device_number>/tracking")]
fn get_telescope_tracking(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Enables or disables telescope tracking.

Sets the state of the telescope's sidereal tracking drive.
*/
#[put("/telescope/<device_number>/tracking")]
fn put_telescope_tracking(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeTrackingRequest { tracking },
    }): Form<ASCOMRequest<schemas::PutTelescopeTrackingRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the current tracking rate.

The current tracking rate of the telescope's sidereal drive.
*/
#[get("/telescope/<device_number>/trackingrate")]
fn get_telescope_trackingrate(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Sets the mount's tracking rate.

Sets the tracking rate of the telescope's sidereal drive. 0 = driveSidereal, 1 = driveLunar, 2 = driveSolar, 3 = driveKing
*/
#[put("/telescope/<device_number>/trackingrate")]
fn put_telescope_trackingrate(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeTrackingrateRequest { tracking_rate },
    }): Form<ASCOMRequest<schemas::PutTelescopeTrackingrateRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns a collection of supported DriveRates values.

Returns an array of supported DriveRates values that describe the permissible values of the TrackingRate property for this telescope type.
*/
#[get("/telescope/<device_number>/trackingrates")]
fn get_telescope_trackingrates(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::DriveRatesResponse> {
    unimplemented!()
}

/**
Returns the UTC date/time of the telescope's internal clock.

The UTC date/time of the telescope's internal clock in ISO 8601 format including fractional seconds. The general format (in Microsoft custom date format style) is yyyy-MM-ddTHH:mm:ss.fffffffZ E.g. 2016-03-04T17:45:31.1234567Z or 2016-11-14T07:03:08.1234567Z Please note the compulsary trailing Z indicating the 'Zulu', UTC time zone.
*/
#[get("/telescope/<device_number>/utcdate")]
fn get_telescope_utcdate(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest { transaction, request: () }): Query<ASCOMRequest<()>>,
) -> ASCOMResponse<schemas::StringResponse> {
    unimplemented!()
}

/**
Sets the UTC date/time of the telescope's internal clock.

The UTC date/time of the telescope's internal clock in ISO 8601 format including fractional seconds. The general format (in Microsoft custom date format style) is yyyy-MM-ddTHH:mm:ss.fffffffZ E.g. 2016-03-04T17:45:31.1234567Z or 2016-11-14T07:03:08.1234567Z Please note the compulsary trailing Z indicating the 'Zulu', UTC time zone.
*/
#[put("/telescope/<device_number>/utcdate")]
fn put_telescope_utcdate(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeUtcdateRequest { utcdate },
    }): Form<ASCOMRequest<schemas::PutTelescopeUtcdateRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Immediatley stops a slew in progress.

Immediately Stops a slew in progress.
*/
#[put("/telescope/<device_number>/abortslew")]
fn put_telescope_abortslew(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Returns the rates at which the telescope may be moved about the specified axis.

The rates at which the telescope may be moved about the specified axis by the MoveAxis(TelescopeAxes, Double) method.
*/
#[get("/telescope/<device_number>/axisrates")]
fn get_telescope_axisrates(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeAxisratesRequest { axis },
    }): Query<ASCOMRequest<schemas::GetTelescopeAxisratesRequest>>,
) -> ASCOMResponse<schemas::AxisRatesResponse> {
    unimplemented!()
}

/**
Indicates whether the telescope can move the requested axis.

True if this telescope can move the requested axis.
*/
#[get("/telescope/<device_number>/canmoveaxis")]
fn get_telescope_canmoveaxis(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeCanmoveaxisRequest { axis },
    }): Query<ASCOMRequest<schemas::GetTelescopeCanmoveaxisRequest>>,
) -> ASCOMResponse<schemas::BoolResponse> {
    unimplemented!()
}

/**
Predicts the pointing state after a German equatorial mount slews to given coordinates.

Predicts the pointing state that a German equatorial mount will be in if it slews to the given coordinates. The  return value will be one of - 0 = pierEast, 1 = pierWest, -1 = pierUnknown
*/
#[get("/telescope/<device_number>/destinationsideofpier")]
fn get_telescope_destinationsideofpier(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Query(ASCOMRequest {
        transaction,
        request: schemas::GetTelescopeDestinationsideofpierRequest { right_ascension, declination },
    }): Query<ASCOMRequest<schemas::GetTelescopeDestinationsideofpierRequest>>,
) -> ASCOMResponse<schemas::IntResponse> {
    unimplemented!()
}

/**
Moves the mount to the "home" position.

Locates the telescope's "home" position (synchronous)
*/
#[put("/telescope/<device_number>/findhome")]
fn put_telescope_findhome(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Moves a telescope axis at the given rate.

Move the telescope in one axis at the given rate.
*/
#[put("/telescope/<device_number>/moveaxis")]
fn put_telescope_moveaxis(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeMoveaxisRequest { axis, rate },
    }): Form<ASCOMRequest<schemas::PutTelescopeMoveaxisRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Park the mount

Move the telescope to its park position, stop all motion (or restrict to a small safe range), and set AtPark to True. )
*/
#[put("/telescope/<device_number>/park")]
fn put_telescope_park(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Moves the scope in the given direction for the given time.

Moves the scope in the given direction for the given interval or time at the rate given by the corresponding guide rate property
*/
#[put("/telescope/<device_number>/pulseguide")]
fn put_telescope_pulseguide(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopePulseguideRequest { direction, duration },
    }): Form<ASCOMRequest<schemas::PutTelescopePulseguideRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Sets the telescope's park position

Sets the telescope's park position to be its current position.
*/
#[put("/telescope/<device_number>/setpark")]
fn put_telescope_setpark(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Synchronously slew to the given local horizontal coordinates.

Move the telescope to the given local horizontal coordinates, return when slew is complete
*/
#[put("/telescope/<device_number>/slewtoaltaz")]
fn put_telescope_slewtoaltaz(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSlewtoaltazRequest { azimuth, altitude },
    }): Form<ASCOMRequest<schemas::PutTelescopeSlewtoaltazRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Asynchronously slew to the given local horizontal coordinates.

Move the telescope to the given local horizontal coordinates, return immediatley after the slew starts. The client can poll the Slewing method to determine when the mount reaches the intended coordinates.
*/
#[put("/telescope/<device_number>/slewtoaltazasync")]
fn put_telescope_slewtoaltazasync(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSlewtoaltazRequest { azimuth, altitude },
    }): Form<ASCOMRequest<schemas::PutTelescopeSlewtoaltazRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Synchronously slew to the given equatorial coordinates.

Move the telescope to the given equatorial coordinates, return when slew is complete
*/
#[put("/telescope/<device_number>/slewtocoordinates")]
fn put_telescope_slewtocoordinates(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSlewtocoordinatesRequest { right_ascension, declination },
    }): Form<ASCOMRequest<schemas::PutTelescopeSlewtocoordinatesRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Asynchronously slew to the given equatorial coordinates.

Move the telescope to the given equatorial coordinates, return immediatley after the slew starts. The client can poll the Slewing method to determine when the mount reaches the intended coordinates.
*/
#[put("/telescope/<device_number>/slewtocoordinatesasync")]
fn put_telescope_slewtocoordinatesasync(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSlewtocoordinatesRequest { right_ascension, declination },
    }): Form<ASCOMRequest<schemas::PutTelescopeSlewtocoordinatesRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Synchronously slew to the TargetRightAscension and TargetDeclination coordinates.

Move the telescope to the TargetRightAscension and TargetDeclination equatorial coordinates, return when slew is complete
*/
#[put("/telescope/<device_number>/slewtotarget")]
fn put_telescope_slewtotarget(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Asynchronously slew to the TargetRightAscension and TargetDeclination coordinates.

Move the telescope to the TargetRightAscension and TargetDeclination equatorial coordinates, return immediatley after the slew starts. The client can poll the Slewing method to determine when the mount reaches the intended coordinates.
*/
#[put("/telescope/<device_number>/slewtotargetasync")]
fn put_telescope_slewtotargetasync(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Syncs to the given local horizontal coordinates.

Matches the scope's local horizontal coordinates to the given local horizontal coordinates.
*/
#[put("/telescope/<device_number>/synctoaltaz")]
fn put_telescope_synctoaltaz(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSlewtoaltazRequest { azimuth, altitude },
    }): Form<ASCOMRequest<schemas::PutTelescopeSlewtoaltazRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Syncs to the given equatorial coordinates.

Matches the scope's equatorial coordinates to the given equatorial coordinates.
*/
#[put("/telescope/<device_number>/synctocoordinates")]
fn put_telescope_synctocoordinates(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest {
        transaction,
        request: schemas::PutTelescopeSlewtocoordinatesRequest { right_ascension, declination },
    }): Form<ASCOMRequest<schemas::PutTelescopeSlewtocoordinatesRequest>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Syncs to the TargetRightAscension and TargetDeclination coordinates.

Matches the scope's equatorial coordinates to the TargetRightAscension and TargetDeclination equatorial coordinates.
*/
#[put("/telescope/<device_number>/synctotarget")]
fn put_telescope_synctotarget(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>,
) -> ASCOMResponse<()> {
    unimplemented!()
}

/**
Unparks the mount.

Takes telescope out of the Parked state. )
*/
#[put("/telescope/<device_number>/unpark")]
fn put_telescope_unpark(
    Path(schemas::DeviceNumberPath { device_number }): Path<schemas::DeviceNumberPath>,

    Form(ASCOMRequest { transaction, request: () }): Form<ASCOMRequest<()>>,
) -> ASCOMResponse<()> {
    unimplemented!()
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
