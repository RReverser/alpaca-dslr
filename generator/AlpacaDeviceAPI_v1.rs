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

#![allow(unused_variables)]

use actix_web::{error::BlockingError, get, put, web::Path};
use serde::{Deserialize, Serialize};
use std::future::Future;
use tracing_actix_web::RootSpan;

mod common;
pub use common::*;

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
#[put("/{device_type}/{device_number}/action")]
fn put_action(
    root_span: RootSpan,

    path: Path<schemas::DeviceTypeAndNumberPath>,

    request: ASCOMRequest<schemas::PutActionRequest>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::StringResponse>, BlockingError>> {
    let schemas::DeviceTypeAndNumberPath { device_type, device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutActionRequest { action, parameters }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Transmits an arbitrary string to the device

Transmits an arbitrary string to the device and does not wait for a response. Optionally, protocol framing characters may be added to the string before transmission.
*/
#[put("/{device_type}/{device_number}/commandblind")]
fn put_commandblind(
    root_span: RootSpan,

    path: Path<schemas::DeviceTypeAndNumberPath>,

    request: ASCOMRequest<schemas::PutCommandblindRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceTypeAndNumberPath { device_type, device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCommandblindRequest { command, raw }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Transmits an arbitrary string to the device and returns a boolean value from the device.

Transmits an arbitrary string to the device and waits for a boolean response. Optionally, protocol framing characters may be added to the string before transmission.
*/
#[put("/{device_type}/{device_number}/commandbool")]
fn put_commandbool(
    root_span: RootSpan,

    path: Path<schemas::DeviceTypeAndNumberPath>,

    request: ASCOMRequest<schemas::PutCommandblindRequest>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceTypeAndNumberPath { device_type, device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCommandblindRequest { command, raw }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Transmits an arbitrary string to the device and returns a string value from the device.

Transmits an arbitrary string to the device and waits for a string response. Optionally, protocol framing characters may be added to the string before transmission.
*/
#[put("/{device_type}/{device_number}/commandstring")]
fn put_commandstring(
    root_span: RootSpan,

    path: Path<schemas::DeviceTypeAndNumberPath>,

    request: ASCOMRequest<schemas::PutCommandblindRequest>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::StringResponse>, BlockingError>> {
    let schemas::DeviceTypeAndNumberPath { device_type, device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCommandblindRequest { command, raw }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Retrieves the connected state of the device
#[get("/{device_type}/{device_number}/connected")]
fn get_connected(root_span: RootSpan, path: Path<schemas::DeviceTypeAndNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceTypeAndNumberPath { device_type, device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Sets the connected state of the device
#[put("/{device_type}/{device_number}/connected")]
fn put_connected(
    root_span: RootSpan,

    path: Path<schemas::DeviceTypeAndNumberPath>,

    request: ASCOMRequest<schemas::PutConnectedRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceTypeAndNumberPath { device_type, device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutConnectedRequest { connected }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Device description

The description of the device
*/
#[get("/{device_type}/{device_number}/description")]
fn get_description(
    root_span: RootSpan,

    path: Path<schemas::DeviceTypeAndNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::StringResponse>, BlockingError>> {
    let schemas::DeviceTypeAndNumberPath { device_type, device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Device driver description

The description of the driver
*/
#[get("/{device_type}/{device_number}/driverinfo")]
fn get_driverinfo(root_span: RootSpan, path: Path<schemas::DeviceTypeAndNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::StringResponse>, BlockingError>> {
    let schemas::DeviceTypeAndNumberPath { device_type, device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Driver Version

A string containing only the major and minor version of the driver.
*/
#[get("/{device_type}/{device_number}/driverversion")]
fn get_driverversion(
    root_span: RootSpan,

    path: Path<schemas::DeviceTypeAndNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::StringResponse>, BlockingError>> {
    let schemas::DeviceTypeAndNumberPath { device_type, device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
The ASCOM Device interface version number that this device supports.

This method returns the version of the ASCOM device interface contract to which this device complies. Only one interface version is current at a moment in time and all new devices should be built to the latest interface version. Applications can choose which device interface versions they support and it is in their interest to support  previous versions as well as the current version to ensure thay can use the largest number of devices.
*/
#[get("/{device_type}/{device_number}/interfaceversion")]
fn get_interfaceversion(
    root_span: RootSpan,

    path: Path<schemas::DeviceTypeAndNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceTypeAndNumberPath { device_type, device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Device name

The name of the device
*/
#[get("/{device_type}/{device_number}/name")]
fn get_name(root_span: RootSpan, path: Path<schemas::DeviceTypeAndNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::StringResponse>, BlockingError>> {
    let schemas::DeviceTypeAndNumberPath { device_type, device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Returns the list of action names supported by this driver.
#[get("/{device_type}/{device_number}/supportedactions")]
fn get_supportedactions(
    root_span: RootSpan,

    path: Path<schemas::DeviceTypeAndNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::StringArrayResponse>, BlockingError>> {
    let schemas::DeviceTypeAndNumberPath { device_type, device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the X offset of the Bayer matrix.

Returns the X offset of the Bayer matrix, as defined in SensorType.
*/
#[get("/camera/{device_number}/bayeroffsetx")]
fn get_camera_bayeroffsetx(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the Y offset of the Bayer matrix.

Returns the Y offset of the Bayer matrix, as defined in SensorType.
*/
#[get("/camera/{device_number}/bayeroffsety")]
fn get_camera_bayeroffsety(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Returns the binning factor for the X axis.
#[get("/camera/{device_number}/binx")]
fn get_camera_binx(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Sets the binning factor for the X axis.
#[put("/camera/{device_number}/binx")]
fn put_camera_binx(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<schemas::PutCameraBinxRequest>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCameraBinxRequest { bin_x }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Returns the binning factor for the Y axis.
#[get("/camera/{device_number}/biny")]
fn get_camera_biny(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Sets the binning factor for the Y axis.
#[put("/camera/{device_number}/biny")]
fn put_camera_biny(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<schemas::PutCameraBinyRequest>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCameraBinyRequest { bin_y }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the camera operational state.

Returns the current camera operational state as an integer. 0 = CameraIdle , 1 = CameraWaiting , 2 = CameraExposing , 3 = CameraReading , 4 = CameraDownload , 5 = CameraError
*/
#[get("/camera/{device_number}/camerastate")]
fn get_camera_camerastate(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the width of the CCD camera chip.

Returns the width of the CCD camera chip in unbinned pixels.
*/
#[get("/camera/{device_number}/cameraxsize")]
fn get_camera_cameraxsize(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the height of the CCD camera chip.

Returns the height of the CCD camera chip in unbinned pixels.
*/
#[get("/camera/{device_number}/cameraysize")]
fn get_camera_cameraysize(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the camera can abort exposures.

Returns true if the camera can abort exposures; false if not.
*/
#[get("/camera/{device_number}/canabortexposure")]
fn get_camera_canabortexposure(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the camera supports asymmetric binning

Returns a flag showing whether this camera supports asymmetric binning
*/
#[get("/camera/{device_number}/canasymmetricbin")]
fn get_camera_canasymmetricbin(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Indicates whether the camera has a fast readout mode.
#[get("/camera/{device_number}/canfastreadout")]
fn get_camera_canfastreadout(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the camera's cooler power setting can be read.

If true, the camera's cooler power setting can be read.
*/
#[get("/camera/{device_number}/cangetcoolerpower")]
fn get_camera_cangetcoolerpower(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns a flag indicating whether this camera supports pulse guiding

Returns a flag indicating whether this camera supports pulse guiding.
*/
#[get("/camera/{device_number}/canpulseguide")]
fn get_camera_canpulseguide(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns a flag indicating whether this camera supports setting the CCD temperature

Returns a flag indicatig whether this camera supports setting the CCD temperature
*/
#[get("/camera/{device_number}/cansetccdtemperature")]
fn get_camera_cansetccdtemperature(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Returns a flag indicating whether this camera can stop an exposure that is in progress
#[get("/camera/{device_number}/canstopexposure")]
fn get_camera_canstopexposure(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the current CCD temperature

Returns the current CCD temperature in degrees Celsius.
*/
#[get("/camera/{device_number}/ccdtemperature")]
fn get_camera_ccdtemperature(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Returns the current cooler on/off state.
#[get("/camera/{device_number}/cooleron")]
fn get_camera_cooleron(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Turns the camera cooler on and off

Turns on and off the camera cooler. True = cooler on, False = cooler off
*/
#[put("/camera/{device_number}/cooleron")]
fn put_camera_cooleron(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutCameraCooleronRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCameraCooleronRequest { cooler_on }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the present cooler power level

Returns the present cooler power level, in percent.
*/
#[get("/camera/{device_number}/coolerpower")]
fn get_camera_coolerpower(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the gain of the camera

Returns the gain of the camera in photoelectrons per A/D unit.
*/
#[get("/camera/{device_number}/electronsperadu")]
fn get_camera_electronsperadu(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Returns the maximum exposure time supported by StartExposure.
#[get("/camera/{device_number}/exposuremax")]
fn get_camera_exposuremax(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the Minimium exposure time

Returns the Minimium exposure time in seconds that the camera supports through StartExposure.
*/
#[get("/camera/{device_number}/exposuremin")]
fn get_camera_exposuremin(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Returns the smallest increment in exposure time supported by StartExposure.
#[get("/camera/{device_number}/exposureresolution")]
fn get_camera_exposureresolution(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Returns whenther Fast Readout Mode is enabled.
#[get("/camera/{device_number}/fastreadout")]
fn get_camera_fastreadout(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Sets whether Fast Readout Mode is enabled.
#[put("/camera/{device_number}/fastreadout")]
fn put_camera_fastreadout(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutCameraFastreadoutRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCameraFastreadoutRequest { fast_readout }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Reports the full well capacity of the camera

Reports the full well capacity of the camera in electrons, at the current camera settings (binning, SetupDialog settings, etc.).
*/
#[get("/camera/{device_number}/fullwellcapacity")]
fn get_camera_fullwellcapacity(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the camera's gain

The camera's gain (GAIN VALUE MODE) OR the index of the selected camera gain description in the Gains array (GAINS INDEX MODE).
*/
#[get("/camera/{device_number}/gain")]
fn get_camera_gain(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the camera's gain.

The camera's gain (GAIN VALUE MODE) OR the index of the selected camera gain description in the Gains array (GAINS INDEX MODE).
*/
#[put("/camera/{device_number}/gain")]
fn put_camera_gain(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<schemas::PutCameraGainRequest>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCameraGainRequest { gain }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Maximum Gain value of that this camera supports

Returns the maximum value of Gain.
*/
#[get("/camera/{device_number}/gainmax")]
fn get_camera_gainmax(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Minimum Gain value of that this camera supports

Returns the Minimum value of Gain.
*/
#[get("/camera/{device_number}/gainmin")]
fn get_camera_gainmin(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
List of Gain names supported by the camera

Returns the Gains supported by the camera.
*/
#[get("/camera/{device_number}/gains")]
fn get_camera_gains(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::StringArrayResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the camera has a mechanical shutter

Returns a flag indicating whether this camera has a mechanical shutter.
*/
#[get("/camera/{device_number}/hasshutter")]
fn get_camera_hasshutter(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the current heat sink temperature.

Returns the current heat sink temperature (called "ambient temperature" by some manufacturers) in degrees Celsius.
*/
#[get("/camera/{device_number}/heatsinktemperature")]
fn get_camera_heatsinktemperature(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
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
#[get("/camera/{device_number}/imagearray")]
fn get_camera_imagearray(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::ImageArrayResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
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
#[get("/camera/{device_number}/imagearrayvariant")]
fn get_camera_imagearrayvariant(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::ImageArrayResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates that an image is ready to be downloaded

Returns a flag indicating whether the image is ready to be downloaded from the camera.
*/
#[get("/camera/{device_number}/imageready")]
fn get_camera_imageready(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates that the camera is pulse guideing.

Returns a flag indicating whether the camera is currrently in a PulseGuide operation.
*/
#[get("/camera/{device_number}/ispulseguiding")]
fn get_camera_ispulseguiding(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Duration of the last exposure

Reports the actual exposure duration in seconds (i.e. shutter open time).
*/
#[get("/camera/{device_number}/lastexposureduration")]
fn get_camera_lastexposureduration(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Start time of the last exposure in FITS standard format.

Reports the actual exposure start in the FITS-standard CCYY-MM-DDThh:mm:ss[.sss...] format.
*/
#[get("/camera/{device_number}/lastexposurestarttime")]
fn get_camera_lastexposurestarttime(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::StringResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Camera's maximum ADU value

Reports the maximum ADU value the camera can produce.
*/
#[get("/camera/{device_number}/maxadu")]
fn get_camera_maxadu(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Maximum  binning for the camera X axis

Returns the maximum allowed binning for the X camera axis
*/
#[get("/camera/{device_number}/maxbinx")]
fn get_camera_maxbinx(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Maximum  binning for the camera Y axis

Returns the maximum allowed binning for the Y camera axis
*/
#[get("/camera/{device_number}/maxbiny")]
fn get_camera_maxbiny(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the current subframe width

Returns the current subframe width, if binning is active, value is in binned pixels.
*/
#[get("/camera/{device_number}/numx")]
fn get_camera_numx(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the current subframe width

Sets the current subframe width.
*/
#[put("/camera/{device_number}/numx")]
fn put_camera_numx(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<schemas::PutCameraNumxRequest>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCameraNumxRequest { num_x }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the current subframe height

Returns the current subframe height, if binning is active, value is in binned pixels.
*/
#[get("/camera/{device_number}/numy")]
fn get_camera_numy(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the current subframe height

Sets the current subframe height.
*/
#[put("/camera/{device_number}/numy")]
fn put_camera_numy(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<schemas::PutCameraNumyRequest>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCameraNumyRequest { num_y }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the camera's offset

Returns the camera's offset (OFFSET VALUE MODE) OR the index of the selected camera offset description in the offsets array (OFFSETS INDEX MODE).
*/
#[get("/camera/{device_number}/offset")]
fn get_camera_offset(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the camera's offset.

Sets the camera's offset (OFFSET VALUE MODE) OR the index of the selected camera offset description in the offsets array (OFFSETS INDEX MODE).
*/
#[put("/camera/{device_number}/offset")]
fn put_camera_offset(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutCameraOffsetRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCameraOffsetRequest { offset }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Maximum offset value of that this camera supports

Returns the maximum value of offset.
*/
#[get("/camera/{device_number}/offsetmax")]
fn get_camera_offsetmax(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Minimum offset value of that this camera supports

Returns the Minimum value of offset.
*/
#[get("/camera/{device_number}/offsetmin")]
fn get_camera_offsetmin(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
List of offset names supported by the camera

Returns the offsets supported by the camera.
*/
#[get("/camera/{device_number}/offsets")]
fn get_camera_offsets(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::StringArrayResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates percentage completeness of the current operation

Returns the percentage of the current operation that is complete. If valid, returns an integer between 0 and 100, where 0 indicates 0% progress (function just started) and 100 indicates 100% progress (i.e. completion).
*/
#[get("/camera/{device_number}/percentcompleted")]
fn get_camera_percentcompleted(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Width of CCD chip pixels (microns)

Returns the width of the CCD chip pixels in microns.
*/
#[get("/camera/{device_number}/pixelsizex")]
fn get_camera_pixelsizex(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Height of CCD chip pixels (microns)

Returns the Height of the CCD chip pixels in microns.
*/
#[get("/camera/{device_number}/pixelsizey")]
fn get_camera_pixelsizey(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates the canera's readout mode as an index into the array ReadoutModes

ReadoutMode is an index into the array ReadoutModes and returns the desired readout mode for the camera. Defaults to 0 if not set.
*/
#[get("/camera/{device_number}/readoutmode")]
fn get_camera_readoutmode(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Set the camera's readout mode

Sets the ReadoutMode as an index into the array ReadoutModes.
*/
#[put("/camera/{device_number}/readoutmode")]
fn put_camera_readoutmode(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutCameraReadoutmodeRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCameraReadoutmodeRequest { readout_mode }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
List of available readout modes

This property provides an array of strings, each of which describes an available readout mode of the camera. At least one string must be present in the list.
*/
#[get("/camera/{device_number}/readoutmodes")]
fn get_camera_readoutmodes(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::StringArrayResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sensor name

The name of the sensor used within the camera.
*/
#[get("/camera/{device_number}/sensorname")]
fn get_camera_sensorname(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::StringResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
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
#[get("/camera/{device_number}/sensortype")]
fn get_camera_sensortype(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Returns the current camera cooler setpoint in degrees Celsius.
#[get("/camera/{device_number}/setccdtemperature")]
fn get_camera_setccdtemperature(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Set the camera's cooler setpoint (degrees Celsius).

Set's the camera's cooler setpoint in degrees Celsius.
*/
#[put("/camera/{device_number}/setccdtemperature")]
fn put_camera_setccdtemperature(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutCameraSetccdtemperatureRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCameraSetccdtemperatureRequest { set_ccdtemperature }| {
        Err(ASCOMError::ACTION_NOT_IMPLEMENTED)
    })
}

/**
Return the current subframe X axis start position

Sets the subframe start position for the X axis (0 based) and returns the current value. If binning is active, value is in binned pixels.
*/
#[get("/camera/{device_number}/startx")]
fn get_camera_startx(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the current subframe X axis start position

Sets the current subframe X axis start position in binned pixels.
*/
#[put("/camera/{device_number}/startx")]
fn put_camera_startx(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutCameraStartxRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCameraStartxRequest { start_x }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Return the current subframe Y axis start position

Sets the subframe start position for the Y axis (0 based) and returns the current value. If binning is active, value is in binned pixels.
*/
#[get("/camera/{device_number}/starty")]
fn get_camera_starty(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the current subframe Y axis start position

Sets the current subframe Y axis start position in binned pixels.
*/
#[put("/camera/{device_number}/starty")]
fn put_camera_starty(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutCameraStartyRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCameraStartyRequest { start_y }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Camera's sub-exposure interval

The Camera's sub exposure duration in seconds. Only available in Camera Interface Version 3 and later.
*/
#[get("/camera/{device_number}/subexposureduration")]
fn get_camera_subexposureduration(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the current Sub Exposure Duration

Sets image sub exposure duration in seconds. Only available in Camera Interface Version 3 and later.
*/
#[put("/camera/{device_number}/subexposureduration")]
fn put_camera_subexposureduration(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutCameraSubexposuredurationRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCameraSubexposuredurationRequest { sub_exposure_duration }| {
        Err(ASCOMError::ACTION_NOT_IMPLEMENTED)
    })
}

/**
Aborts the current exposure

Aborts the current exposure, if any, and returns the camera to Idle state.
*/
#[put("/camera/{device_number}/abortexposure")]
fn put_camera_abortexposure(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Pulse guide in the specified direction for the specified time.

Activates the Camera's mount control sytem to instruct the mount to move in a particular direction for a given period of time
*/
#[put("/camera/{device_number}/pulseguide")]
fn put_camera_pulseguide(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutCameraPulseguideRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCameraPulseguideRequest { direction, duration }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Starts an exposure

Starts an exposure. Use ImageReady to check when the exposure is complete.
*/
#[put("/camera/{device_number}/startexposure")]
fn put_camera_startexposure(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutCameraStartexposureRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCameraStartexposureRequest { duration, light }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Stops the current exposure

Stops the current exposure, if any. If an exposure is in progress, the readout process is initiated. Ignored if readout is already in process.
*/
#[put("/camera/{device_number}/stopexposure")]
fn put_camera_stopexposure(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the current calibrator brightness

Returns the current calibrator brightness in the range 0 (completely off) to MaxBrightness (fully on)
*/
#[get("/covercalibrator/{device_number}/brightness")]
fn get_covercalibrator_brightness(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the state of the calibration device

Returns the state of the calibration device, if present, otherwise returns "NotPresent".  The calibrator state mode is specified as an integer value from the CalibratorStatus Enum.
*/
#[get("/covercalibrator/{device_number}/calibratorstate")]
fn get_covercalibrator_calibratorstate(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the state of the device cover"

Returns the state of the device cover, if present, otherwise returns "NotPresent".  The cover state mode is specified as an integer value from the CoverStatus Enum.
*/
#[get("/covercalibrator/{device_number}/coverstate")]
fn get_covercalibrator_coverstate(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the calibrator's maximum Brightness value.

The Brightness value that makes the calibrator deliver its maximum illumination.
*/
#[get("/covercalibrator/{device_number}/maxbrightness")]
fn get_covercalibrator_maxbrightness(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Turns the calibrator off

Turns the calibrator off if the device has calibration capability.
*/
#[put("/covercalibrator/{device_number}/calibratoroff")]
fn put_covercalibrator_calibratoroff(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Turns the calibrator on at the specified brightness

Turns the calibrator on at the specified brightness if the device has calibration capability.
*/
#[put("/covercalibrator/{device_number}/calibratoron")]
fn put_covercalibrator_calibratoron(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutCovercalibratorCalibratoronRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutCovercalibratorCalibratoronRequest { brightness }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Initiates cover closing

Initiates cover closing if a cover is present.
*/
#[put("/covercalibrator/{device_number}/closecover")]
fn put_covercalibrator_closecover(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Stops any cover movement that may be in progress

Stops any cover movement that may be in progress if a cover is present and cover movement can be interrupted.
*/
#[put("/covercalibrator/{device_number}/haltcover")]
fn put_covercalibrator_haltcover(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Initiates cover opening

Initiates cover opening if a cover is present.
*/
#[put("/covercalibrator/{device_number}/opencover")]
fn put_covercalibrator_opencover(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
The dome altitude

The dome altitude (degrees, horizon zero and increasing positive to 90 zenith).
*/
#[get("/dome/{device_number}/altitude")]
fn get_dome_altitude(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the dome is in the home position.

Indicates whether the dome is in the home position. This is normally used following a FindHome()  operation. The value is reset with any azimuth slew operation that moves the dome away from the home position. AtHome may also become true durng normal slew operations, if the dome passes through the home position and the dome controller hardware is capable of detecting that; or at the end of a slew operation if the dome comes to rest at the home position.
*/
#[get("/dome/{device_number}/athome")]
fn get_dome_athome(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the telescope is at the park position

True if the dome is in the programmed park position. Set only following a Park() operation and reset with any slew operation.
*/
#[get("/dome/{device_number}/atpark")]
fn get_dome_atpark(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
The dome azimuth

Returns the dome azimuth (degrees, North zero and increasing clockwise, i.e., 90 East, 180 South, 270 West)
*/
#[get("/dome/{device_number}/azimuth")]
fn get_dome_azimuth(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the dome can find the home position.

True if the dome can move to the home position.
*/
#[get("/dome/{device_number}/canfindhome")]
fn get_dome_canfindhome(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the dome can be parked.

True if the dome is capable of programmed parking (Park() method)
*/
#[get("/dome/{device_number}/canpark")]
fn get_dome_canpark(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the dome altitude can be set

True if driver is capable of setting the dome altitude.
*/
#[get("/dome/{device_number}/cansetaltitude")]
fn get_dome_cansetaltitude(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the dome azimuth can be set

True if driver is capable of setting the dome azimuth.
*/
#[get("/dome/{device_number}/cansetazimuth")]
fn get_dome_cansetazimuth(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the dome park position can be set

True if driver is capable of setting the dome park position.
*/
#[get("/dome/{device_number}/cansetpark")]
fn get_dome_cansetpark(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the dome shutter can be opened

True if driver is capable of automatically operating shutter
*/
#[get("/dome/{device_number}/cansetshutter")]
fn get_dome_cansetshutter(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the dome supports slaving to a telescope

True if driver is capable of slaving to a telescope.
*/
#[get("/dome/{device_number}/canslave")]
fn get_dome_canslave(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the dome azimuth position can be synched

True if driver is capable of synchronizing the dome azimuth position using the SyncToAzimuth(Double) method.
*/
#[get("/dome/{device_number}/cansyncazimuth")]
fn get_dome_cansyncazimuth(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Status of the dome shutter or roll-off roof

Returns the status of the dome shutter or roll-off roof. 0 = Open, 1 = Closed, 2 = Opening, 3 = Closing, 4 = Shutter status error
*/
#[get("/dome/{device_number}/shutterstatus")]
fn get_dome_shutterstatus(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the dome is slaved to the telescope

True if the dome is slaved to the telescope in its hardware, else False.
*/
#[get("/dome/{device_number}/slaved")]
fn get_dome_slaved(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets whether the dome is slaved to the telescope

Sets the current subframe height.
*/
#[put("/dome/{device_number}/slaved")]
fn put_dome_slaved(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<schemas::PutDomeSlavedRequest>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutDomeSlavedRequest { slaved }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the any part of the dome is moving

True if any part of the dome is currently moving, False if all dome components are steady.
*/
#[get("/dome/{device_number}/slewing")]
fn get_dome_slewing(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Immediately cancel current dome operation.

Calling this method will immediately disable hardware slewing (Slaved will become False).
*/
#[put("/dome/{device_number}/abortslew")]
fn put_dome_abortslew(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Close the shutter or otherwise shield telescope from the sky.
#[put("/dome/{device_number}/closeshutter")]
fn put_dome_closeshutter(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Start operation to search for the dome home position.

After Home position is established initializes Azimuth to the default value and sets the AtHome flag.
*/
#[put("/dome/{device_number}/findhome")]
fn put_dome_findhome(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Open shutter or otherwise expose telescope to the sky.
#[put("/dome/{device_number}/openshutter")]
fn put_dome_openshutter(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Rotate dome in azimuth to park position.

After assuming programmed park position, sets AtPark flag.
*/
#[put("/dome/{device_number}/park")]
fn put_dome_park(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Set the current azimuth, altitude position of dome to be the park position

Set the current azimuth, altitude position of dome to be the park position.
*/
#[put("/dome/{device_number}/setpark")]
fn put_dome_setpark(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Slew the dome to the given altitude position.
#[put("/dome/{device_number}/slewtoaltitude")]
fn put_dome_slewtoaltitude(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutDomeSlewtoaltitudeRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutDomeSlewtoaltitudeRequest { altitude }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Slew the dome to the given azimuth position.
#[put("/dome/{device_number}/slewtoazimuth")]
fn put_dome_slewtoazimuth(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutDomeSlewtoazimuthRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutDomeSlewtoazimuthRequest { azimuth }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Synchronize the current position of the dome to the given azimuth.
#[put("/dome/{device_number}/synctoazimuth")]
fn put_dome_synctoazimuth(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutDomeSlewtoazimuthRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutDomeSlewtoazimuthRequest { azimuth }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Filter focus offsets

An integer array of filter focus offsets.
*/
#[get("/filterwheel/{device_number}/focusoffsets")]
fn get_filterwheel_focusoffsets(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::IntArrayResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Filter wheel filter names

The names of the filters
*/
#[get("/filterwheel/{device_number}/names")]
fn get_filterwheel_names(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::StringArrayResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Returns the current filter wheel position
#[get("/filterwheel/{device_number}/position")]
fn get_filterwheel_position(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Sets the filter wheel position
#[put("/filterwheel/{device_number}/position")]
fn put_filterwheel_position(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutFilterwheelPositionRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutFilterwheelPositionRequest { position }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the focuser is capable of absolute position.

True if the focuser is capable of absolute position; that is, being commanded to a specific step location.
*/
#[get("/focuser/{device_number}/absolute")]
fn get_focuser_absolute(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the focuser is currently moving.

True if the focuser is currently moving to a new position. False if the focuser is stationary.
*/
#[get("/focuser/{device_number}/ismoving")]
fn get_focuser_ismoving(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the focuser's maximum increment size.

Maximum increment size allowed by the focuser; i.e. the maximum number of steps allowed in one move operation.
*/
#[get("/focuser/{device_number}/maxincrement")]
fn get_focuser_maxincrement(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the focuser's maximum step size.

Maximum step position permitted.
*/
#[get("/focuser/{device_number}/maxstep")]
fn get_focuser_maxstep(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the focuser's current position.

Current focuser position, in steps.
*/
#[get("/focuser/{device_number}/position")]
fn get_focuser_position(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the focuser's step size.

Step size (microns) for the focuser.
*/
#[get("/focuser/{device_number}/stepsize")]
fn get_focuser_stepsize(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Retrieves the state of temperature compensation mode

Gets the state of temperature compensation mode (if available), else always False.
*/
#[get("/focuser/{device_number}/tempcomp")]
fn get_focuser_tempcomp(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the device's temperature compensation mode.

Sets the state of temperature compensation mode.
*/
#[put("/focuser/{device_number}/tempcomp")]
fn put_focuser_tempcomp(
    root_span: RootSpan,

    path: Path<schemas::PutFocuserTempcompPath>,

    request: ASCOMRequest<schemas::PutFocuserTempcompRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::PutFocuserTempcompPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutFocuserTempcompRequest { temp_comp }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the focuser has temperature compensation.

True if focuser has temperature compensation available.
*/
#[get("/focuser/{device_number}/tempcompavailable")]
fn get_focuser_tempcompavailable(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the focuser's current temperature.

Current ambient temperature as measured by the focuser.
*/
#[get("/focuser/{device_number}/temperature")]
fn get_focuser_temperature(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Immediatley stops focuser motion.

Immediately stop any focuser motion due to a previous Move(Int32) method call.
*/
#[put("/focuser/{device_number}/halt")]
fn put_focuser_halt(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Moves the focuser to a new position.

Moves the focuser by the specified amount or to the specified position depending on the value of the Absolute property.
*/
#[put("/focuser/{device_number}/move")]
fn put_focuser_move(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutFocuserMoveRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutFocuserMoveRequest { position }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the time period over which observations will be averaged

Gets the time period over which observations will be averaged
*/
#[get("/observingconditions/{device_number}/averageperiod")]
fn get_observingconditions_averageperiod(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Sets the time period over which observations will be averaged
#[put("/observingconditions/{device_number}/averageperiod")]
fn put_observingconditions_averageperiod(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutObservingconditionsAverageperiodRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutObservingconditionsAverageperiodRequest { average_period }| {
        Err(ASCOMError::ACTION_NOT_IMPLEMENTED)
    })
}

/**
Returns the amount of sky obscured by cloud

Gets the percentage of the sky obscured by cloud
*/
#[get("/observingconditions/{device_number}/cloudcover")]
fn get_observingconditions_cloudcover(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the atmospheric dew point at the observatory

Gets the atmospheric dew point at the observatory reported in °C.
*/
#[get("/observingconditions/{device_number}/dewpoint")]
fn get_observingconditions_dewpoint(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the atmospheric humidity at the observatory

Gets the atmospheric  humidity (%) at the observatory
*/
#[get("/observingconditions/{device_number}/humidity")]
fn get_observingconditions_humidity(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the atmospheric pressure at the observatory.

Gets the atmospheric pressure in hectoPascals at the observatory's altitude - NOT reduced to sea level.
*/
#[get("/observingconditions/{device_number}/pressure")]
fn get_observingconditions_pressure(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the rain rate at the observatory.

Gets the rain rate (mm/hour) at the observatory.
*/
#[get("/observingconditions/{device_number}/rainrate")]
fn get_observingconditions_rainrate(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the sky brightness at the observatory

Gets the sky brightness at the observatory (Lux)
*/
#[get("/observingconditions/{device_number}/skybrightness")]
fn get_observingconditions_skybrightness(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the sky quality at the observatory

Gets the sky quality at the observatory (magnitudes per square arc second)
*/
#[get("/observingconditions/{device_number}/skyquality")]
fn get_observingconditions_skyquality(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the sky temperature at the observatory

Gets the sky temperature(°C) at the observatory.
*/
#[get("/observingconditions/{device_number}/skytemperature")]
fn get_observingconditions_skytemperature(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the seeing at the observatory

Gets the seeing at the observatory measured as star full width half maximum (FWHM) in arc secs.
*/
#[get("/observingconditions/{device_number}/starfwhm")]
fn get_observingconditions_starfwhm(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the temperature at the observatory

Gets the temperature(°C) at the observatory.
*/
#[get("/observingconditions/{device_number}/temperature")]
fn get_observingconditions_temperature(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the wind direction at the observatory

Gets the wind direction. The returned value must be between 0.0 and 360.0, interpreted according to the metereological standard, where a special value of 0.0 is returned when the wind speed is 0.0. Wind direction is measured clockwise from north, through east, where East=90.0, South=180.0, West=270.0 and North=360.0.
*/
#[get("/observingconditions/{device_number}/winddirection")]
fn get_observingconditions_winddirection(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the peak 3 second wind gust at the observatory over the last 2 minutes

Gets the peak 3 second wind gust(m/s) at the observatory over the last 2 minutes.
*/
#[get("/observingconditions/{device_number}/windgust")]
fn get_observingconditions_windgust(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the wind speed at the observatory.

Gets the wind speed(m/s) at the observatory.
*/
#[get("/observingconditions/{device_number}/windspeed")]
fn get_observingconditions_windspeed(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Refreshes sensor values from hardware.

Forces the driver to immediately query its attached hardware to refresh sensor values.
*/
#[put("/observingconditions/{device_number}/refresh")]
fn put_observingconditions_refresh(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Return a sensor description

Gets a description of the sensor with the name specified in the SensorName parameter
*/
#[get("/observingconditions/{device_number}/sensordescription")]
fn get_observingconditions_sensordescription(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::GetObservingconditionsSensordescriptionRequest>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::StringResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::GetObservingconditionsSensordescriptionRequest { sensor_name }| {
        Err(ASCOMError::ACTION_NOT_IMPLEMENTED)
    })
}

/**
Return the time since the sensor value was last updated

Gets the time since the sensor specified in the SensorName parameter was last updated
*/
#[get("/observingconditions/{device_number}/timesincelastupdate")]
fn get_observingconditions_timesincelastupdate(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::GetObservingconditionsTimesincelastupdateRequest>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::GetObservingconditionsTimesincelastupdateRequest { sensor_name }| {
        Err(ASCOMError::ACTION_NOT_IMPLEMENTED)
    })
}

/**
IIndicates whether the Rotator supports the Reverse method.

True if the Rotator supports the Reverse method.
*/
#[get("/rotator/{device_number}/canreverse")]
fn get_rotator_canreverse(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the rotator is currently moving.

True if the rotator is currently moving to a new position. False if the focuser is stationary.
*/
#[get("/rotator/{device_number}/ismoving")]
fn get_rotator_ismoving(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the rotator's mechanical current position.

Returns the raw mechanical position of the rotator in degrees.
*/
#[get("/rotator/{device_number}/mechanicalposition")]
fn get_rotator_mechanicalposition(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the rotator's current position.

Current instantaneous Rotator position, in degrees.
*/
#[get("/rotator/{device_number}/position")]
fn get_rotator_position(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Returns the rotator’s Reverse state.
#[get("/rotator/{device_number}/reverse")]
fn get_rotator_reverse(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/// Sets the rotator’s Reverse state.
#[put("/rotator/{device_number}/reverse")]
fn put_rotator_reverse(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutRotatorReverseRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutRotatorReverseRequest { reverse }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the minimum StepSize

The minimum StepSize, in degrees.
*/
#[get("/rotator/{device_number}/stepsize")]
fn get_rotator_stepsize(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the destination position angle.

The destination position angle for Move() and MoveAbsolute().
*/
#[get("/rotator/{device_number}/targetposition")]
fn get_rotator_targetposition(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Immediatley stops rotator motion.

Immediately stop any Rotator motion due to a previous Move or MoveAbsolute method call.
*/
#[put("/rotator/{device_number}/halt")]
fn put_rotator_halt(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Moves the rotator to a new relative position.

Causes the rotator to move Position degrees relative to the current Position value.
*/
#[put("/rotator/{device_number}/move")]
fn put_rotator_move(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutRotatorMoveRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutRotatorMoveRequest { position }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Moves the rotator to a new absolute position.

Causes the rotator to move the absolute position of Position degrees.
*/
#[put("/rotator/{device_number}/moveabsolute")]
fn put_rotator_moveabsolute(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutRotatorMoveabsoluteRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutRotatorMoveabsoluteRequest { position }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Moves the rotator to a new raw mechanical position.

Causes the rotator to move the mechanical position of Position degrees.
*/
#[put("/rotator/{device_number}/movemechanical")]
fn put_rotator_movemechanical(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutRotatorMovemechanicalRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutRotatorMovemechanicalRequest { position }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Syncs the rotator to the specified position angle without moving it.

Causes the rotator to sync to the position of Position degrees.
*/
#[put("/rotator/{device_number}/sync")]
fn put_rotator_sync(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutRotatorSyncRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutRotatorSyncRequest { position }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the monitored state is safe for use.

Indicates whether the monitored state is safe for use. True if the state is safe, False if it is unsafe.
*/
#[get("/safetymonitor/{device_number}/issafe")]
fn get_safetymonitor_issafe(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
The number of switch devices managed by this driver

Returns the number of switch devices managed by this driver. Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/{device_number}/maxswitch")]
fn get_switch_maxswitch(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the specified switch device can be written to

Reports if the specified switch device can be written to, default true. This is false if the device cannot be written to, for example a limit switch or a sensor.  Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/{device_number}/canwrite")]
fn get_switch_canwrite(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::GetSwitchCanwriteRequest>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::GetSwitchCanwriteRequest { id }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Return the state of switch device id as a boolean

Return the state of switch device id as a boolean.  Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/{device_number}/getswitch")]
fn get_switch_getswitch(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::GetSwitchGetswitchRequest>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::GetSwitchGetswitchRequest { id }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Gets the description of the specified switch device

Gets the description of the specified switch device. This is to allow a fuller description of the device to be returned, for example for a tool tip. Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/{device_number}/getswitchdescription")]
fn get_switch_getswitchdescription(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::GetSwitchGetswitchdescriptionRequest>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::StringResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::GetSwitchGetswitchdescriptionRequest { id }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Gets the name of the specified switch device

Gets the name of the specified switch device. Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/{device_number}/getswitchname")]
fn get_switch_getswitchname(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::GetSwitchGetswitchnameRequest>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::StringResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::GetSwitchGetswitchnameRequest { id }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Gets the value of the specified switch device as a double

Gets the value of the specified switch device as a double. Devices are numbered from 0 to MaxSwitch - 1, The value of this switch is expected to be between MinSwitchValue and MaxSwitchValue.
*/
#[get("/switch/{device_number}/getswitchvalue")]
fn get_switch_getswitchvalue(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::GetSwitchGetswitchvalueRequest>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::GetSwitchGetswitchvalueRequest { id }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Gets the minimum value of the specified switch device as a double

Gets the minimum value of the specified switch device as a double. Devices are numbered from 0 to MaxSwitch - 1.
*/
#[get("/switch/{device_number}/minswitchvalue")]
fn get_switch_minswitchvalue(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::GetSwitchMinswitchvalueRequest>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::GetSwitchMinswitchvalueRequest { id }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Gets the maximum value of the specified switch device as a double

Gets the maximum value of the specified switch device as a double. Devices are numbered from 0 to MaxSwitch - 1.
*/
#[get("/switch/{device_number}/maxswitchvalue")]
fn get_switch_maxswitchvalue(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::GetSwitchMaxswitchvalueRequest>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::GetSwitchMaxswitchvalueRequest { id }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets a switch controller device to the specified state, true or false

Sets a switch controller device to the specified state, true or false.
*/
#[put("/switch/{device_number}/setswitch")]
fn put_switch_setswitch(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutSwitchSetswitchRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutSwitchSetswitchRequest { id, state }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets a switch device name to the specified value

Sets a switch device name to the specified value.
*/
#[put("/switch/{device_number}/setswitchname")]
fn put_switch_setswitchname(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutSwitchSetswitchnameRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutSwitchSetswitchnameRequest { id, name }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets a switch device value to the specified value

Sets a switch device value to the specified value.
*/
#[put("/switch/{device_number}/setswitchvalue")]
fn put_switch_setswitchvalue(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutSwitchSetswitchvalueRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutSwitchSetswitchvalueRequest { id, value }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the step size that this device supports (the difference between successive values of the device).

Returns the step size that this device supports (the difference between successive values of the device). Devices are numbered from 0 to MaxSwitch - 1.
*/
#[get("/switch/{device_number}/switchstep")]
fn get_switch_switchstep(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::GetSwitchSwitchstepRequest>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::GetSwitchSwitchstepRequest { id }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the current mount alignment mode

Returns the alignment mode of the mount (Alt/Az, Polar, German Polar).  The alignment mode is specified as an integer value from the AlignmentModes Enum.
*/
#[get("/telescope/{device_number}/alignmentmode")]
fn get_telescope_alignmentmode(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the mount's altitude above the horizon.

The altitude above the local horizon of the mount's current position (degrees, positive up)
*/
#[get("/telescope/{device_number}/altitude")]
fn get_telescope_altitude(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the telescope's aperture.

The area of the telescope's aperture, taking into account any obstructions (square meters)
*/
#[get("/telescope/{device_number}/aperturearea")]
fn get_telescope_aperturearea(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the telescope's effective aperture.

The telescope's effective aperture diameter (meters)
*/
#[get("/telescope/{device_number}/aperturediameter")]
fn get_telescope_aperturediameter(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the mount is at the home position.

True if the mount is stopped in the Home position. Set only following a FindHome()  operation, and reset with any slew operation. This property must be False if the telescope does not support homing.
*/
#[get("/telescope/{device_number}/athome")]
fn get_telescope_athome(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the telescope is at the park position.

True if the telescope has been put into the parked state by the seee Park()  method. Set False by calling the Unpark() method.
*/
#[get("/telescope/{device_number}/atpark")]
fn get_telescope_atpark(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the mount's azimuth.

The azimuth at the local horizon of the mount's current position (degrees, North-referenced, positive East/clockwise).
*/
#[get("/telescope/{device_number}/azimuth")]
fn get_telescope_azimuth(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the mount can find the home position.

True if this telescope is capable of programmed finding its home position (FindHome()  method).
*/
#[get("/telescope/{device_number}/canfindhome")]
fn get_telescope_canfindhome(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the telescope can be parked.

True if this telescope is capable of programmed parking (Park() method)
*/
#[get("/telescope/{device_number}/canpark")]
fn get_telescope_canpark(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the telescope can be pulse guided.

True if this telescope is capable of software-pulsed guiding (via the PulseGuide(GuideDirections, Int32) method)
*/
#[get("/telescope/{device_number}/canpulseguide")]
fn get_telescope_canpulseguide(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the DeclinationRate property can be changed.

True if the DeclinationRate property can be changed to provide offset tracking in the declination axis.
*/
#[get("/telescope/{device_number}/cansetdeclinationrate")]
fn get_telescope_cansetdeclinationrate(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the DeclinationRate property can be changed.

True if the guide rate properties used for PulseGuide(GuideDirections, Int32) can ba adjusted.
*/
#[get("/telescope/{device_number}/cansetguiderates")]
fn get_telescope_cansetguiderates(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the telescope park position can be set.

True if this telescope is capable of programmed setting of its park position (SetPark() method)
*/
#[get("/telescope/{device_number}/cansetpark")]
fn get_telescope_cansetpark(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the telescope SideOfPier can be set.

True if the SideOfPier property can be set, meaning that the mount can be forced to flip.
*/
#[get("/telescope/{device_number}/cansetpierside")]
fn get_telescope_cansetpierside(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the RightAscensionRate property can be changed.

True if the RightAscensionRate property can be changed to provide offset tracking in the right ascension axis. .
*/
#[get("/telescope/{device_number}/cansetrightascensionrate")]
fn get_telescope_cansetrightascensionrate(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the Tracking property can be changed.

True if the Tracking property can be changed, turning telescope sidereal tracking on and off.
*/
#[get("/telescope/{device_number}/cansettracking")]
fn get_telescope_cansettracking(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the telescope can slew synchronously.

True if this telescope is capable of programmed slewing (synchronous or asynchronous) to equatorial coordinates
*/
#[get("/telescope/{device_number}/canslew")]
fn get_telescope_canslew(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the telescope can slew synchronously to AltAz coordinates.

True if this telescope is capable of programmed slewing (synchronous or asynchronous) to local horizontal coordinates
*/
#[get("/telescope/{device_number}/canslewaltaz")]
fn get_telescope_canslewaltaz(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the telescope can slew asynchronously to AltAz coordinates.

True if this telescope is capable of programmed asynchronous slewing to local horizontal coordinates
*/
#[get("/telescope/{device_number}/canslewaltazasync")]
fn get_telescope_canslewaltazasync(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the telescope can slew asynchronously.

True if this telescope is capable of programmed asynchronous slewing to equatorial coordinates.
*/
#[get("/telescope/{device_number}/canslewasync")]
fn get_telescope_canslewasync(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the telescope can sync to equatorial coordinates.

True if this telescope is capable of programmed synching to equatorial coordinates.
*/
#[get("/telescope/{device_number}/cansync")]
fn get_telescope_cansync(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the telescope can sync to local horizontal coordinates.

True if this telescope is capable of programmed synching to local horizontal coordinates
*/
#[get("/telescope/{device_number}/cansyncaltaz")]
fn get_telescope_cansyncaltaz(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the telescope can be unparked.

True if this telescope is capable of programmed unparking (UnPark() method)
*/
#[get("/telescope/{device_number}/canunpark")]
fn get_telescope_canunpark(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the mount's declination.

The declination (degrees) of the mount's current equatorial coordinates, in the coordinate system given by the EquatorialSystem property. Reading the property will raise an error if the value is unavailable.
*/
#[get("/telescope/{device_number}/declination")]
fn get_telescope_declination(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the telescope's declination tracking rate.

The declination tracking rate (arcseconds per second, default = 0.0)
*/
#[get("/telescope/{device_number}/declinationrate")]
fn get_telescope_declinationrate(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the telescope's declination tracking rate.

Sets the declination tracking rate (arcseconds per second)
*/
#[put("/telescope/{device_number}/declinationrate")]
fn put_telescope_declinationrate(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeDeclinationrateRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeDeclinationrateRequest { declination_rate }| {
        Err(ASCOMError::ACTION_NOT_IMPLEMENTED)
    })
}

/**
Indicates whether atmospheric refraction is applied to coordinates.

True if the telescope or driver applies atmospheric refraction to coordinates.
*/
#[get("/telescope/{device_number}/doesrefraction")]
fn get_telescope_doesrefraction(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Determines whether atmospheric refraction is applied to coordinates.

Causes the rotator to move Position degrees relative to the current Position value.
*/
#[put("/telescope/{device_number}/doesrefraction")]
fn put_telescope_doesrefraction(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeDoesrefractionRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeDoesrefractionRequest { does_refraction }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the current equatorial coordinate system used by this telescope.

Returns the current equatorial coordinate system used by this telescope (e.g. Topocentric or J2000).
*/
#[get("/telescope/{device_number}/equatorialsystem")]
fn get_telescope_equatorialsystem(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the telescope's focal length in meters.

The telescope's focal length in meters
*/
#[get("/telescope/{device_number}/focallength")]
fn get_telescope_focallength(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the  current Declination rate offset for telescope guiding

The current Declination movement rate offset for telescope guiding (degrees/sec)
*/
#[get("/telescope/{device_number}/guideratedeclination")]
fn get_telescope_guideratedeclination(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the  current Declination rate offset for telescope guiding.

Sets the current Declination movement rate offset for telescope guiding (degrees/sec).
*/
#[put("/telescope/{device_number}/guideratedeclination")]
fn put_telescope_guideratedeclination(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeGuideratedeclinationRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeGuideratedeclinationRequest { guide_rate_declination }| {
        Err(ASCOMError::ACTION_NOT_IMPLEMENTED)
    })
}

/**
Returns the  current RightAscension rate offset for telescope guiding

The current RightAscension movement rate offset for telescope guiding (degrees/sec)
*/
#[get("/telescope/{device_number}/guideraterightascension")]
fn get_telescope_guideraterightascension(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the  current RightAscension rate offset for telescope guiding.

Sets the current RightAscension movement rate offset for telescope guiding (degrees/sec).
*/
#[put("/telescope/{device_number}/guideraterightascension")]
fn put_telescope_guideraterightascension(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeGuideraterightascensionRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeGuideraterightascensionRequest { guide_rate_right_ascension }| {
        Err(ASCOMError::ACTION_NOT_IMPLEMENTED)
    })
}

/**
Indicates whether the telescope is currently executing a PulseGuide command

True if a PulseGuide(GuideDirections, Int32) command is in progress, False otherwise
*/
#[get("/telescope/{device_number}/ispulseguiding")]
fn get_telescope_ispulseguiding(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the mount's right ascension coordinate.

The right ascension (hours) of the mount's current equatorial coordinates, in the coordinate system given by the EquatorialSystem property
*/
#[get("/telescope/{device_number}/rightascension")]
fn get_telescope_rightascension(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the telescope's right ascension tracking rate.

The right ascension tracking rate (arcseconds per second, default = 0.0)
*/
#[get("/telescope/{device_number}/rightascensionrate")]
fn get_telescope_rightascensionrate(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the telescope's right ascension tracking rate.

Sets the right ascension tracking rate (arcseconds per second)
*/
#[put("/telescope/{device_number}/rightascensionrate")]
fn put_telescope_rightascensionrate(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeRightascensionrateRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeRightascensionrateRequest { right_ascension_rate }| {
        Err(ASCOMError::ACTION_NOT_IMPLEMENTED)
    })
}

/**
Returns the mount's pointing state.

Indicates the pointing state of the mount. 0 = pierEast, 1 = pierWest, -1= pierUnknown
*/
#[get("/telescope/{device_number}/sideofpier")]
fn get_telescope_sideofpier(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the mount's pointing state.

Sets the pointing state of the mount. 0 = pierEast, 1 = pierWest
*/
#[put("/telescope/{device_number}/sideofpier")]
fn put_telescope_sideofpier(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeSideofpierRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeSideofpierRequest { side_of_pier }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the local apparent sidereal time.

The local apparent sidereal time from the telescope's internal clock (hours, sidereal).
*/
#[get("/telescope/{device_number}/siderealtime")]
fn get_telescope_siderealtime(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the observing site's elevation above mean sea level.

The elevation above mean sea level (meters) of the site at which the telescope is located.
*/
#[get("/telescope/{device_number}/siteelevation")]
fn get_telescope_siteelevation(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the observing site's elevation above mean sea level.

Sets the elevation above mean sea level (metres) of the site at which the telescope is located.
*/
#[put("/telescope/{device_number}/siteelevation")]
fn put_telescope_siteelevation(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeSiteelevationRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeSiteelevationRequest { site_elevation }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the observing site's latitude.

The geodetic(map) latitude (degrees, positive North, WGS84) of the site at which the telescope is located.
*/
#[get("/telescope/{device_number}/sitelatitude")]
fn get_telescope_sitelatitude(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the observing site's latitude.

Sets the observing site's latitude (degrees).
*/
#[put("/telescope/{device_number}/sitelatitude")]
fn put_telescope_sitelatitude(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeSitelatitudeRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeSitelatitudeRequest { site_latitude }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the observing site's longitude.

The longitude (degrees, positive East, WGS84) of the site at which the telescope is located.
*/
#[get("/telescope/{device_number}/sitelongitude")]
fn get_telescope_sitelongitude(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the observing site's longitude.

Sets the observing site's longitude (degrees, positive East, WGS84).
*/
#[put("/telescope/{device_number}/sitelongitude")]
fn put_telescope_sitelongitude(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeSitelongitudeRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeSitelongitudeRequest { site_longitude }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the telescope is currently slewing.

True if telescope is currently moving in response to one of the Slew methods or the MoveAxis(TelescopeAxes, Double) method, False at all other times.
*/
#[get("/telescope/{device_number}/slewing")]
fn get_telescope_slewing(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the post-slew settling time.

Returns the post-slew settling time (sec.).
*/
#[get("/telescope/{device_number}/slewsettletime")]
fn get_telescope_slewsettletime(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the post-slew settling time.

Sets the  post-slew settling time (integer sec.).
*/
#[put("/telescope/{device_number}/slewsettletime")]
fn put_telescope_slewsettletime(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeSlewsettletimeRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeSlewsettletimeRequest { slew_settle_time }| {
        Err(ASCOMError::ACTION_NOT_IMPLEMENTED)
    })
}

/**
Returns the current target declination.

The declination (degrees, positive North) for the target of an equatorial slew or sync operation
*/
#[get("/telescope/{device_number}/targetdeclination")]
fn get_telescope_targetdeclination(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the target declination of a slew or sync.

Sets the declination (degrees, positive North) for the target of an equatorial slew or sync operation
*/
#[put("/telescope/{device_number}/targetdeclination")]
fn put_telescope_targetdeclination(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeTargetdeclinationRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeTargetdeclinationRequest { target_declination }| {
        Err(ASCOMError::ACTION_NOT_IMPLEMENTED)
    })
}

/**
Returns the current target right ascension.

The right ascension (hours) for the target of an equatorial slew or sync operation
*/
#[get("/telescope/{device_number}/targetrightascension")]
fn get_telescope_targetrightascension(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DoubleResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the target right ascension of a slew or sync.

Sets the right ascension (hours) for the target of an equatorial slew or sync operation
*/
#[put("/telescope/{device_number}/targetrightascension")]
fn put_telescope_targetrightascension(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeTargetrightascensionRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeTargetrightascensionRequest { target_right_ascension }| {
        Err(ASCOMError::ACTION_NOT_IMPLEMENTED)
    })
}

/**
Indicates whether the telescope is tracking.

Returns the state of the telescope's sidereal tracking drive.
*/
#[get("/telescope/{device_number}/tracking")]
fn get_telescope_tracking(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Enables or disables telescope tracking.

Sets the state of the telescope's sidereal tracking drive.
*/
#[put("/telescope/{device_number}/tracking")]
fn put_telescope_tracking(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeTrackingRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeTrackingRequest { tracking }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the current tracking rate.

The current tracking rate of the telescope's sidereal drive.
*/
#[get("/telescope/{device_number}/trackingrate")]
fn get_telescope_trackingrate(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the mount's tracking rate.

Sets the tracking rate of the telescope's sidereal drive. 0 = driveSidereal, 1 = driveLunar, 2 = driveSolar, 3 = driveKing
*/
#[put("/telescope/{device_number}/trackingrate")]
fn put_telescope_trackingrate(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeTrackingrateRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeTrackingrateRequest { tracking_rate }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns a collection of supported DriveRates values.

Returns an array of supported DriveRates values that describe the permissible values of the TrackingRate property for this telescope type.
*/
#[get("/telescope/{device_number}/trackingrates")]
fn get_telescope_trackingrates(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<()>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::DriveRatesResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the UTC date/time of the telescope's internal clock.

The UTC date/time of the telescope's internal clock in ISO 8601 format including fractional seconds. The general format (in Microsoft custom date format style) is yyyy-MM-ddTHH:mm:ss.fffffffZ E.g. 2016-03-04T17:45:31.1234567Z or 2016-11-14T07:03:08.1234567Z Please note the compulsary trailing Z indicating the 'Zulu', UTC time zone.
*/
#[get("/telescope/{device_number}/utcdate")]
fn get_telescope_utcdate(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<schemas::StringResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the UTC date/time of the telescope's internal clock.

The UTC date/time of the telescope's internal clock in ISO 8601 format including fractional seconds. The general format (in Microsoft custom date format style) is yyyy-MM-ddTHH:mm:ss.fffffffZ E.g. 2016-03-04T17:45:31.1234567Z or 2016-11-14T07:03:08.1234567Z Please note the compulsary trailing Z indicating the 'Zulu', UTC time zone.
*/
#[put("/telescope/{device_number}/utcdate")]
fn put_telescope_utcdate(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeUtcdateRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeUtcdateRequest { utcdate }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Immediatley stops a slew in progress.

Immediately Stops a slew in progress.
*/
#[put("/telescope/{device_number}/abortslew")]
fn put_telescope_abortslew(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Returns the rates at which the telescope may be moved about the specified axis.

The rates at which the telescope may be moved about the specified axis by the MoveAxis(TelescopeAxes, Double) method.
*/
#[get("/telescope/{device_number}/axisrates")]
fn get_telescope_axisrates(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::GetTelescopeAxisratesRequest>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::AxisRatesResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::GetTelescopeAxisratesRequest { axis }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Indicates whether the telescope can move the requested axis.

True if this telescope can move the requested axis.
*/
#[get("/telescope/{device_number}/canmoveaxis")]
fn get_telescope_canmoveaxis(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::GetTelescopeCanmoveaxisRequest>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::BoolResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::GetTelescopeCanmoveaxisRequest { axis }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Predicts the pointing state after a German equatorial mount slews to given coordinates.

Predicts the pointing state that a German equatorial mount will be in if it slews to the given coordinates. The  return value will be one of - 0 = pierEast, 1 = pierWest, -1 = pierUnknown
*/
#[get("/telescope/{device_number}/destinationsideofpier")]
fn get_telescope_destinationsideofpier(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::GetTelescopeDestinationsideofpierRequest>,
) -> impl Future<Output = Result<ASCOMResponse<schemas::IntResponse>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::GetTelescopeDestinationsideofpierRequest { right_ascension, declination }| {
        Err(ASCOMError::ACTION_NOT_IMPLEMENTED)
    })
}

/**
Moves the mount to the "home" position.

Locates the telescope's "home" position (synchronous)
*/
#[put("/telescope/{device_number}/findhome")]
fn put_telescope_findhome(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Moves a telescope axis at the given rate.

Move the telescope in one axis at the given rate.
*/
#[put("/telescope/{device_number}/moveaxis")]
fn put_telescope_moveaxis(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeMoveaxisRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeMoveaxisRequest { axis, rate }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Park the mount

Move the telescope to its park position, stop all motion (or restrict to a small safe range), and set AtPark to True. )
*/
#[put("/telescope/{device_number}/park")]
fn put_telescope_park(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Moves the scope in the given direction for the given time.

Moves the scope in the given direction for the given interval or time at the rate given by the corresponding guide rate property
*/
#[put("/telescope/{device_number}/pulseguide")]
fn put_telescope_pulseguide(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopePulseguideRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopePulseguideRequest { direction, duration }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Sets the telescope's park position

Sets the telescope's park position to be its current position.
*/
#[put("/telescope/{device_number}/setpark")]
fn put_telescope_setpark(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Synchronously slew to the given local horizontal coordinates.

Move the telescope to the given local horizontal coordinates, return when slew is complete
*/
#[put("/telescope/{device_number}/slewtoaltaz")]
fn put_telescope_slewtoaltaz(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeSlewtoaltazRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeSlewtoaltazRequest { azimuth, altitude }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Asynchronously slew to the given local horizontal coordinates.

Move the telescope to the given local horizontal coordinates, return immediatley after the slew starts. The client can poll the Slewing method to determine when the mount reaches the intended coordinates.
*/
#[put("/telescope/{device_number}/slewtoaltazasync")]
fn put_telescope_slewtoaltazasync(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeSlewtoaltazRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeSlewtoaltazRequest { azimuth, altitude }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Synchronously slew to the given equatorial coordinates.

Move the telescope to the given equatorial coordinates, return when slew is complete
*/
#[put("/telescope/{device_number}/slewtocoordinates")]
fn put_telescope_slewtocoordinates(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeSlewtocoordinatesRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeSlewtocoordinatesRequest { right_ascension, declination }| {
        Err(ASCOMError::ACTION_NOT_IMPLEMENTED)
    })
}

/**
Asynchronously slew to the given equatorial coordinates.

Move the telescope to the given equatorial coordinates, return immediatley after the slew starts. The client can poll the Slewing method to determine when the mount reaches the intended coordinates.
*/
#[put("/telescope/{device_number}/slewtocoordinatesasync")]
fn put_telescope_slewtocoordinatesasync(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeSlewtocoordinatesRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeSlewtocoordinatesRequest { right_ascension, declination }| {
        Err(ASCOMError::ACTION_NOT_IMPLEMENTED)
    })
}

/**
Synchronously slew to the TargetRightAscension and TargetDeclination coordinates.

Move the telescope to the TargetRightAscension and TargetDeclination equatorial coordinates, return when slew is complete
*/
#[put("/telescope/{device_number}/slewtotarget")]
fn put_telescope_slewtotarget(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Asynchronously slew to the TargetRightAscension and TargetDeclination coordinates.

Move the telescope to the TargetRightAscension and TargetDeclination equatorial coordinates, return immediatley after the slew starts. The client can poll the Slewing method to determine when the mount reaches the intended coordinates.
*/
#[put("/telescope/{device_number}/slewtotargetasync")]
fn put_telescope_slewtotargetasync(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Syncs to the given local horizontal coordinates.

Matches the scope's local horizontal coordinates to the given local horizontal coordinates.
*/
#[put("/telescope/{device_number}/synctoaltaz")]
fn put_telescope_synctoaltaz(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeSlewtoaltazRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeSlewtoaltazRequest { azimuth, altitude }| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Syncs to the given equatorial coordinates.

Matches the scope's equatorial coordinates to the given equatorial coordinates.
*/
#[put("/telescope/{device_number}/synctocoordinates")]
fn put_telescope_synctocoordinates(
    root_span: RootSpan,

    path: Path<schemas::DeviceNumberPath>,

    request: ASCOMRequest<schemas::PutTelescopeSlewtocoordinatesRequest>,
) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |schemas::PutTelescopeSlewtocoordinatesRequest { right_ascension, declination }| {
        Err(ASCOMError::ACTION_NOT_IMPLEMENTED)
    })
}

/**
Syncs to the TargetRightAscension and TargetDeclination coordinates.

Matches the scope's equatorial coordinates to the TargetRightAscension and TargetDeclination equatorial coordinates.
*/
#[put("/telescope/{device_number}/synctotarget")]
fn put_telescope_synctotarget(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

/**
Unparks the mount.

Takes telescope out of the Parked state. )
*/
#[put("/telescope/{device_number}/unpark")]
fn put_telescope_unpark(root_span: RootSpan, path: Path<schemas::DeviceNumberPath>, request: ASCOMRequest<()>) -> impl Future<Output = Result<ASCOMResponse<()>, BlockingError>> {
    let schemas::DeviceNumberPath { device_number } = path.into_inner();

    request.respond_with(root_span, move |()| Err(ASCOMError::ACTION_NOT_IMPLEMENTED))
}

pub fn service() -> actix_web::Scope {
    actix_web::web::scope("/api/v1")
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
}
