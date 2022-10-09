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

#![allow(unused_variables, rustdoc::broken_intra_doc_links)]

use serde::{Deserialize, Serialize};

mod common;
pub use common::*;

mod schemas {
    use super::*;

    #[derive(Serialize)]

    pub struct ImageArrayResponse {
        /// 0 = Unknown, 1 = Short(int16), 2 = Integer (int32), 3 = Double (Double precision real number).
        #[serde(rename = "Type")]
        pub r#type: Option<i32>,

        /// The array's rank, will be 2 (single plane image (monochrome)) or 3 (multi-plane image).
        #[serde(rename = "Rank")]
        pub rank: Option<i32>,

        /// Returned integer or double value
        #[serde(rename = "Value")]
        pub value: Option<Vec<Vec<f64>>>,
    }

    impl ToResponse for ImageArrayResponse {
        type Response = Self;

        fn to_response(self) -> Self::Response {
            self
        }
    }

    impl ToResponse for bool {
        type Response = ValueResponse<Self>;

        fn to_response(self) -> Self::Response {
            self.into()
        }
    }

    impl ToResponse for f64 {
        type Response = ValueResponse<Self>;

        fn to_response(self) -> Self::Response {
            self.into()
        }
    }

    impl ToResponse for i32 {
        type Response = ValueResponse<Self>;

        fn to_response(self) -> Self::Response {
            self.into()
        }
    }

    impl ToResponse for Vec<i32> {
        type Response = ValueResponse<Self>;

        fn to_response(self) -> Self::Response {
            self.into()
        }
    }

    impl ToResponse for String {
        type Response = ValueResponse<Self>;

        fn to_response(self) -> Self::Response {
            self.into()
        }
    }

    impl ToResponse for Vec<String> {
        type Response = ValueResponse<Self>;

        fn to_response(self) -> Self::Response {
            self.into()
        }
    }

    impl ToResponse for Vec<schemas::AxisRate> {
        type Response = ValueResponse<Self>;

        fn to_response(self) -> Self::Response {
            self.into()
        }
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

    impl ToResponse for Vec<schemas::DriveRate> {
        type Response = ValueResponse<Self>;

        fn to_response(self) -> Self::Response {
            self.into()
        }
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
}

rpc! {

    /// ASCOM Methods Common To All Devices
    #[http("{device_type}")]
    pub trait DeviceType {
        /**
        Actions and SupportedActions are a standardised means for drivers to extend functionality beyond the built-in capabilities of the ASCOM device interfaces.

        The key advantage of using Actions is that drivers can expose any device specific functionality required. The downside is that, in order to use these unique features, every application author would need to create bespoke code to present or exploit them.

        The Action parameter and return strings are deceptively simple, but can support transmission of arbitrarily complex data structures, for example through JSON encoding.

        This capability will be of primary value to
         * <span style="font-size:14px;">bespoke software and hardware configurations where a single entity controls both the consuming application software and the hardware / driver environment</span>
         * <span style="font-size:14px;">a group of application and device authors to quickly formulate and try out new interface capabilities without requiring an immediate change to the ASCOM device interface, which will take a lot longer than just agreeing a name, input parameters and a standard response for an Action command.</span>


        The list of Action commands supported by a driver can be discovered through the SupportedActions property.

        This method should return an error message and NotImplementedException error number (0x400) if the driver just implements the standard ASCOM device methods and has no bespoke, unique, functionality.
        */
        #[http("action")]
        fn set_action(&mut self, request: schemas::PutActionRequest) -> ASCOMResult<String>;

        /// Transmits an arbitrary string to the device and does not wait for a response. Optionally, protocol framing characters may be added to the string before transmission.
        #[http("commandblind")]
        fn set_commandblind(&mut self, request: schemas::PutCommandblindRequest) -> ASCOMResult<()>;

        /// Transmits an arbitrary string to the device and waits for a boolean response. Optionally, protocol framing characters may be added to the string before transmission.
        #[http("commandbool")]
        fn set_commandbool(&mut self, request: schemas::PutCommandblindRequest) -> ASCOMResult<bool>;

        /// Transmits an arbitrary string to the device and waits for a string response. Optionally, protocol framing characters may be added to the string before transmission.
        #[http("commandstring")]
        fn set_commandstring(&mut self, request: schemas::PutCommandblindRequest) -> ASCOMResult<String>;

        /// Retrieves the connected state of the device
        #[http("connected")]
        fn get_connected(&self) -> ASCOMResult<bool>;

        /// Sets the connected state of the device
        #[http("connected")]
        fn set_connected(&mut self, request: schemas::PutConnectedRequest) -> ASCOMResult<()>;

        /// The description of the device
        #[http("description")]
        fn get_description(&self) -> ASCOMResult<String>;

        /// The description of the driver
        #[http("driverinfo")]
        fn get_driverinfo(&self) -> ASCOMResult<String>;

        /// A string containing only the major and minor version of the driver.
        #[http("driverversion")]
        fn get_driverversion(&self) -> ASCOMResult<String>;

        /// This method returns the version of the ASCOM device interface contract to which this device complies. Only one interface version is current at a moment in time and all new devices should be built to the latest interface version. Applications can choose which device interface versions they support and it is in their interest to support  previous versions as well as the current version to ensure thay can use the largest number of devices.
        #[http("interfaceversion")]
        fn get_interfaceversion(&self) -> ASCOMResult<i32>;

        /// The name of the device
        #[http("name")]
        fn get_name(&self) -> ASCOMResult<String>;

        /// Returns the list of action names supported by this driver.
        #[http("supportedactions")]
        fn get_supportedactions(&self) -> ASCOMResult<Vec<String>>;
    }

    /// Camera Specific Methods
    #[http("camera")]
    pub trait Camera {
        /// Returns the X offset of the Bayer matrix, as defined in SensorType.
        #[http("bayeroffsetx")]
        fn get_bayeroffsetx(&self) -> ASCOMResult<i32>;

        /// Returns the Y offset of the Bayer matrix, as defined in SensorType.
        #[http("bayeroffsety")]
        fn get_bayeroffsety(&self) -> ASCOMResult<i32>;

        /// Returns the binning factor for the X axis.
        #[http("binx")]
        fn get_binx(&self) -> ASCOMResult<i32>;

        /// Sets the binning factor for the X axis.
        #[http("binx")]
        fn set_binx(&mut self, request: schemas::PutCameraBinxRequest) -> ASCOMResult<()>;

        /// Returns the binning factor for the Y axis.
        #[http("biny")]
        fn get_biny(&self) -> ASCOMResult<i32>;

        /// Sets the binning factor for the Y axis.
        #[http("biny")]
        fn set_biny(&mut self, request: schemas::PutCameraBinyRequest) -> ASCOMResult<()>;

        /// Returns the current camera operational state as an integer. 0 = CameraIdle , 1 = CameraWaiting , 2 = CameraExposing , 3 = CameraReading , 4 = CameraDownload , 5 = CameraError
        #[http("camerastate")]
        fn get_camerastate(&self) -> ASCOMResult<i32>;

        /// Returns the width of the CCD camera chip in unbinned pixels.
        #[http("cameraxsize")]
        fn get_cameraxsize(&self) -> ASCOMResult<i32>;

        /// Returns the height of the CCD camera chip in unbinned pixels.
        #[http("cameraysize")]
        fn get_cameraysize(&self) -> ASCOMResult<i32>;

        /// Returns true if the camera can abort exposures; false if not.
        #[http("canabortexposure")]
        fn get_canabortexposure(&self) -> ASCOMResult<bool>;

        /// Returns a flag showing whether this camera supports asymmetric binning
        #[http("canasymmetricbin")]
        fn get_canasymmetricbin(&self) -> ASCOMResult<bool>;

        /// Indicates whether the camera has a fast readout mode.
        #[http("canfastreadout")]
        fn get_canfastreadout(&self) -> ASCOMResult<bool>;

        /// If true, the camera's cooler power setting can be read.
        #[http("cangetcoolerpower")]
        fn get_cangetcoolerpower(&self) -> ASCOMResult<bool>;

        /// Returns a flag indicating whether this camera supports pulse guiding.
        #[http("canpulseguide")]
        fn get_canpulseguide(&self) -> ASCOMResult<bool>;

        /// Returns a flag indicatig whether this camera supports setting the CCD temperature
        #[http("cansetccdtemperature")]
        fn get_cansetccdtemperature(&self) -> ASCOMResult<bool>;

        /// Returns a flag indicating whether this camera can stop an exposure that is in progress
        #[http("canstopexposure")]
        fn get_canstopexposure(&self) -> ASCOMResult<bool>;

        /// Returns the current CCD temperature in degrees Celsius.
        #[http("ccdtemperature")]
        fn get_ccdtemperature(&self) -> ASCOMResult<f64>;

        /// Returns the current cooler on/off state.
        #[http("cooleron")]
        fn get_cooleron(&self) -> ASCOMResult<bool>;

        /// Turns on and off the camera cooler. True = cooler on, False = cooler off
        #[http("cooleron")]
        fn set_cooleron(&mut self, request: schemas::PutCameraCooleronRequest) -> ASCOMResult<()>;

        /// Returns the present cooler power level, in percent.
        #[http("coolerpower")]
        fn get_coolerpower(&self) -> ASCOMResult<f64>;

        /// Returns the gain of the camera in photoelectrons per A/D unit.
        #[http("electronsperadu")]
        fn get_electronsperadu(&self) -> ASCOMResult<f64>;

        /// Returns the maximum exposure time supported by StartExposure.
        #[http("exposuremax")]
        fn get_exposuremax(&self) -> ASCOMResult<f64>;

        /// Returns the Minimium exposure time in seconds that the camera supports through StartExposure.
        #[http("exposuremin")]
        fn get_exposuremin(&self) -> ASCOMResult<f64>;

        /// Returns the smallest increment in exposure time supported by StartExposure.
        #[http("exposureresolution")]
        fn get_exposureresolution(&self) -> ASCOMResult<f64>;

        /// Returns whenther Fast Readout Mode is enabled.
        #[http("fastreadout")]
        fn get_fastreadout(&self) -> ASCOMResult<bool>;

        /// Sets whether Fast Readout Mode is enabled.
        #[http("fastreadout")]
        fn set_fastreadout(&mut self, request: schemas::PutCameraFastreadoutRequest) -> ASCOMResult<()>;

        /// Reports the full well capacity of the camera in electrons, at the current camera settings (binning, SetupDialog settings, etc.).
        #[http("fullwellcapacity")]
        fn get_fullwellcapacity(&self) -> ASCOMResult<f64>;

        /// The camera's gain (GAIN VALUE MODE) OR the index of the selected camera gain description in the Gains array (GAINS INDEX MODE).
        #[http("gain")]
        fn get_gain(&self) -> ASCOMResult<i32>;

        /// The camera's gain (GAIN VALUE MODE) OR the index of the selected camera gain description in the Gains array (GAINS INDEX MODE).
        #[http("gain")]
        fn set_gain(&mut self, request: schemas::PutCameraGainRequest) -> ASCOMResult<()>;

        /// Returns the maximum value of Gain.
        #[http("gainmax")]
        fn get_gainmax(&self) -> ASCOMResult<i32>;

        /// Returns the Minimum value of Gain.
        #[http("gainmin")]
        fn get_gainmin(&self) -> ASCOMResult<i32>;

        /// Returns the Gains supported by the camera.
        #[http("gains")]
        fn get_gains(&self) -> ASCOMResult<Vec<String>>;

        /// Returns a flag indicating whether this camera has a mechanical shutter.
        #[http("hasshutter")]
        fn get_hasshutter(&self) -> ASCOMResult<bool>;

        /// Returns the current heat sink temperature (called "ambient temperature" by some manufacturers) in degrees Celsius.
        #[http("heatsinktemperature")]
        fn get_heatsinktemperature(&self) -> ASCOMResult<f64>;

        /**
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
        #[http("imagearray")]
        fn get_imagearray(&self) -> ASCOMResult<schemas::ImageArrayResponse>;

        /**
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
        #[http("imagearrayvariant")]
        fn get_imagearrayvariant(&self) -> ASCOMResult<schemas::ImageArrayResponse>;

        /// Returns a flag indicating whether the image is ready to be downloaded from the camera.
        #[http("imageready")]
        fn get_imageready(&self) -> ASCOMResult<bool>;

        /// Returns a flag indicating whether the camera is currrently in a PulseGuide operation.
        #[http("ispulseguiding")]
        fn get_ispulseguiding(&self) -> ASCOMResult<bool>;

        /// Reports the actual exposure duration in seconds (i.e. shutter open time).
        #[http("lastexposureduration")]
        fn get_lastexposureduration(&self) -> ASCOMResult<f64>;

        /// Reports the actual exposure start in the FITS-standard CCYY-MM-DDThh:mm:ss[.sss...] format.
        #[http("lastexposurestarttime")]
        fn get_lastexposurestarttime(&self) -> ASCOMResult<String>;

        /// Reports the maximum ADU value the camera can produce.
        #[http("maxadu")]
        fn get_maxadu(&self) -> ASCOMResult<i32>;

        /// Returns the maximum allowed binning for the X camera axis
        #[http("maxbinx")]
        fn get_maxbinx(&self) -> ASCOMResult<i32>;

        /// Returns the maximum allowed binning for the Y camera axis
        #[http("maxbiny")]
        fn get_maxbiny(&self) -> ASCOMResult<i32>;

        /// Returns the current subframe width, if binning is active, value is in binned pixels.
        #[http("numx")]
        fn get_numx(&self) -> ASCOMResult<i32>;

        /// Sets the current subframe width.
        #[http("numx")]
        fn set_numx(&mut self, request: schemas::PutCameraNumxRequest) -> ASCOMResult<()>;

        /// Returns the current subframe height, if binning is active, value is in binned pixels.
        #[http("numy")]
        fn get_numy(&self) -> ASCOMResult<i32>;

        /// Sets the current subframe height.
        #[http("numy")]
        fn set_numy(&mut self, request: schemas::PutCameraNumyRequest) -> ASCOMResult<()>;

        /// Returns the camera's offset (OFFSET VALUE MODE) OR the index of the selected camera offset description in the offsets array (OFFSETS INDEX MODE).
        #[http("offset")]
        fn get_offset(&self) -> ASCOMResult<i32>;

        /// Sets the camera's offset (OFFSET VALUE MODE) OR the index of the selected camera offset description in the offsets array (OFFSETS INDEX MODE).
        #[http("offset")]
        fn set_offset(&mut self, request: schemas::PutCameraOffsetRequest) -> ASCOMResult<()>;

        /// Returns the maximum value of offset.
        #[http("offsetmax")]
        fn get_offsetmax(&self) -> ASCOMResult<i32>;

        /// Returns the Minimum value of offset.
        #[http("offsetmin")]
        fn get_offsetmin(&self) -> ASCOMResult<i32>;

        /// Returns the offsets supported by the camera.
        #[http("offsets")]
        fn get_offsets(&self) -> ASCOMResult<Vec<String>>;

        /// Returns the percentage of the current operation that is complete. If valid, returns an integer between 0 and 100, where 0 indicates 0% progress (function just started) and 100 indicates 100% progress (i.e. completion).
        #[http("percentcompleted")]
        fn get_percentcompleted(&self) -> ASCOMResult<i32>;

        /// Returns the width of the CCD chip pixels in microns.
        #[http("pixelsizex")]
        fn get_pixelsizex(&self) -> ASCOMResult<f64>;

        /// Returns the Height of the CCD chip pixels in microns.
        #[http("pixelsizey")]
        fn get_pixelsizey(&self) -> ASCOMResult<f64>;

        /// ReadoutMode is an index into the array ReadoutModes and returns the desired readout mode for the camera. Defaults to 0 if not set.
        #[http("readoutmode")]
        fn get_readoutmode(&self) -> ASCOMResult<i32>;

        /// Sets the ReadoutMode as an index into the array ReadoutModes.
        #[http("readoutmode")]
        fn set_readoutmode(&mut self, request: schemas::PutCameraReadoutmodeRequest) -> ASCOMResult<()>;

        /// This property provides an array of strings, each of which describes an available readout mode of the camera. At least one string must be present in the list.
        #[http("readoutmodes")]
        fn get_readoutmodes(&self) -> ASCOMResult<Vec<String>>;

        /// The name of the sensor used within the camera.
        #[http("sensorname")]
        fn get_sensorname(&self) -> ASCOMResult<String>;

        /**
        Returns a value indicating whether the sensor is monochrome, or what Bayer matrix it encodes. Where:
        - 0 = Monochrome,
        - 1 = Colour not requiring Bayer decoding
        - 2 = RGGB Bayer encoding
        - 3 = CMYG Bayer encoding
        - 4 = CMYG2 Bayer encoding
        - 5 = LRGB TRUESENSE Bayer encoding.

        Please see the ASCOM Help fie for more informaiton on the SensorType.

        */
        #[http("sensortype")]
        fn get_sensortype(&self) -> ASCOMResult<i32>;

        /// Returns the current camera cooler setpoint in degrees Celsius.
        #[http("setccdtemperature")]
        fn get_setccdtemperature(&self) -> ASCOMResult<f64>;

        /// Set's the camera's cooler setpoint in degrees Celsius.
        #[http("setccdtemperature")]
        fn set_setccdtemperature(&mut self, request: schemas::PutCameraSetccdtemperatureRequest) -> ASCOMResult<()>;

        /// Sets the subframe start position for the X axis (0 based) and returns the current value. If binning is active, value is in binned pixels.
        #[http("startx")]
        fn get_startx(&self) -> ASCOMResult<i32>;

        /// Sets the current subframe X axis start position in binned pixels.
        #[http("startx")]
        fn set_startx(&mut self, request: schemas::PutCameraStartxRequest) -> ASCOMResult<()>;

        /// Sets the subframe start position for the Y axis (0 based) and returns the current value. If binning is active, value is in binned pixels.
        #[http("starty")]
        fn get_starty(&self) -> ASCOMResult<i32>;

        /// Sets the current subframe Y axis start position in binned pixels.
        #[http("starty")]
        fn set_starty(&mut self, request: schemas::PutCameraStartyRequest) -> ASCOMResult<()>;

        /// The Camera's sub exposure duration in seconds. Only available in Camera Interface Version 3 and later.
        #[http("subexposureduration")]
        fn get_subexposureduration(&self) -> ASCOMResult<f64>;

        /// Sets image sub exposure duration in seconds. Only available in Camera Interface Version 3 and later.
        #[http("subexposureduration")]
        fn set_subexposureduration(&mut self, request: schemas::PutCameraSubexposuredurationRequest) -> ASCOMResult<()>;

        /// Aborts the current exposure, if any, and returns the camera to Idle state.
        #[http("abortexposure")]
        fn set_abortexposure(&mut self) -> ASCOMResult<()>;

        /// Activates the Camera's mount control sytem to instruct the mount to move in a particular direction for a given period of time
        #[http("pulseguide")]
        fn set_pulseguide(&mut self, request: schemas::PutCameraPulseguideRequest) -> ASCOMResult<()>;

        /// Starts an exposure. Use ImageReady to check when the exposure is complete.
        #[http("startexposure")]
        fn set_startexposure(&mut self, request: schemas::PutCameraStartexposureRequest) -> ASCOMResult<()>;

        /// Stops the current exposure, if any. If an exposure is in progress, the readout process is initiated. Ignored if readout is already in process.
        #[http("stopexposure")]
        fn set_stopexposure(&mut self) -> ASCOMResult<()>;
    }

    /// CoverCalibrator Specific Methods
    #[http("covercalibrator")]
    pub trait Covercalibrator {
        /// Returns the current calibrator brightness in the range 0 (completely off) to MaxBrightness (fully on)
        #[http("brightness")]
        fn get_brightness(&self) -> ASCOMResult<i32>;

        /// Returns the state of the calibration device, if present, otherwise returns "NotPresent".  The calibrator state mode is specified as an integer value from the CalibratorStatus Enum.
        #[http("calibratorstate")]
        fn get_calibratorstate(&self) -> ASCOMResult<i32>;

        /// Returns the state of the device cover, if present, otherwise returns "NotPresent".  The cover state mode is specified as an integer value from the CoverStatus Enum.
        #[http("coverstate")]
        fn get_coverstate(&self) -> ASCOMResult<i32>;

        /// The Brightness value that makes the calibrator deliver its maximum illumination.
        #[http("maxbrightness")]
        fn get_maxbrightness(&self) -> ASCOMResult<i32>;

        /// Turns the calibrator off if the device has calibration capability.
        #[http("calibratoroff")]
        fn set_calibratoroff(&mut self) -> ASCOMResult<()>;

        /// Turns the calibrator on at the specified brightness if the device has calibration capability.
        #[http("calibratoron")]
        fn set_calibratoron(&mut self, request: schemas::PutCovercalibratorCalibratoronRequest) -> ASCOMResult<()>;

        /// Initiates cover closing if a cover is present.
        #[http("closecover")]
        fn set_closecover(&mut self) -> ASCOMResult<()>;

        /// Stops any cover movement that may be in progress if a cover is present and cover movement can be interrupted.
        #[http("haltcover")]
        fn set_haltcover(&mut self) -> ASCOMResult<()>;

        /// Initiates cover opening if a cover is present.
        #[http("opencover")]
        fn set_opencover(&mut self) -> ASCOMResult<()>;
    }

    /// Dome Specific Methods
    #[http("dome")]
    pub trait Dome {
        /// The dome altitude (degrees, horizon zero and increasing positive to 90 zenith).
        #[http("altitude")]
        fn get_altitude(&self) -> ASCOMResult<f64>;

        /// Indicates whether the dome is in the home position. This is normally used following a FindHome()  operation. The value is reset with any azimuth slew operation that moves the dome away from the home position. AtHome may also become true durng normal slew operations, if the dome passes through the home position and the dome controller hardware is capable of detecting that; or at the end of a slew operation if the dome comes to rest at the home position.
        #[http("athome")]
        fn get_athome(&self) -> ASCOMResult<bool>;

        /// True if the dome is in the programmed park position. Set only following a Park() operation and reset with any slew operation.
        #[http("atpark")]
        fn get_atpark(&self) -> ASCOMResult<bool>;

        /// Returns the dome azimuth (degrees, North zero and increasing clockwise, i.e., 90 East, 180 South, 270 West)
        #[http("azimuth")]
        fn get_azimuth(&self) -> ASCOMResult<f64>;

        /// True if the dome can move to the home position.
        #[http("canfindhome")]
        fn get_canfindhome(&self) -> ASCOMResult<bool>;

        /// True if the dome is capable of programmed parking (Park() method)
        #[http("canpark")]
        fn get_canpark(&self) -> ASCOMResult<bool>;

        /// True if driver is capable of setting the dome altitude.
        #[http("cansetaltitude")]
        fn get_cansetaltitude(&self) -> ASCOMResult<bool>;

        /// True if driver is capable of setting the dome azimuth.
        #[http("cansetazimuth")]
        fn get_cansetazimuth(&self) -> ASCOMResult<bool>;

        /// True if driver is capable of setting the dome park position.
        #[http("cansetpark")]
        fn get_cansetpark(&self) -> ASCOMResult<bool>;

        /// True if driver is capable of automatically operating shutter
        #[http("cansetshutter")]
        fn get_cansetshutter(&self) -> ASCOMResult<bool>;

        /// True if driver is capable of slaving to a telescope.
        #[http("canslave")]
        fn get_canslave(&self) -> ASCOMResult<bool>;

        /// True if driver is capable of synchronizing the dome azimuth position using the SyncToAzimuth(Double) method.
        #[http("cansyncazimuth")]
        fn get_cansyncazimuth(&self) -> ASCOMResult<bool>;

        /// Returns the status of the dome shutter or roll-off roof. 0 = Open, 1 = Closed, 2 = Opening, 3 = Closing, 4 = Shutter status error
        #[http("shutterstatus")]
        fn get_shutterstatus(&self) -> ASCOMResult<i32>;

        /// True if the dome is slaved to the telescope in its hardware, else False.
        #[http("slaved")]
        fn get_slaved(&self) -> ASCOMResult<bool>;

        /// Sets the current subframe height.
        #[http("slaved")]
        fn set_slaved(&mut self, request: schemas::PutDomeSlavedRequest) -> ASCOMResult<()>;

        /// True if any part of the dome is currently moving, False if all dome components are steady.
        #[http("slewing")]
        fn get_slewing(&self) -> ASCOMResult<bool>;

        /// Calling this method will immediately disable hardware slewing (Slaved will become False).
        #[http("abortslew")]
        fn set_abortslew(&mut self) -> ASCOMResult<()>;

        /// Close the shutter or otherwise shield telescope from the sky.
        #[http("closeshutter")]
        fn set_closeshutter(&mut self) -> ASCOMResult<()>;

        /// After Home position is established initializes Azimuth to the default value and sets the AtHome flag.
        #[http("findhome")]
        fn set_findhome(&mut self) -> ASCOMResult<()>;

        /// Open shutter or otherwise expose telescope to the sky.
        #[http("openshutter")]
        fn set_openshutter(&mut self) -> ASCOMResult<()>;

        /// After assuming programmed park position, sets AtPark flag.
        #[http("park")]
        fn set_park(&mut self) -> ASCOMResult<()>;

        /// Set the current azimuth, altitude position of dome to be the park position.
        #[http("setpark")]
        fn set_setpark(&mut self) -> ASCOMResult<()>;

        /// Slew the dome to the given altitude position.
        #[http("slewtoaltitude")]
        fn set_slewtoaltitude(&mut self, request: schemas::PutDomeSlewtoaltitudeRequest) -> ASCOMResult<()>;

        /// Slew the dome to the given azimuth position.
        #[http("slewtoazimuth")]
        fn set_slewtoazimuth(&mut self, request: schemas::PutDomeSlewtoazimuthRequest) -> ASCOMResult<()>;

        /// Synchronize the current position of the dome to the given azimuth.
        #[http("synctoazimuth")]
        fn set_synctoazimuth(&mut self, request: schemas::PutDomeSlewtoazimuthRequest) -> ASCOMResult<()>;
    }

    /// FilterWheel Specific Methods
    #[http("filterwheel")]
    pub trait Filterwheel {
        /// An integer array of filter focus offsets.
        #[http("focusoffsets")]
        fn get_focusoffsets(&self) -> ASCOMResult<Vec<i32>>;

        /// The names of the filters
        #[http("names")]
        fn get_names(&self) -> ASCOMResult<Vec<String>>;

        /// Returns the current filter wheel position
        #[http("position")]
        fn get_position(&self) -> ASCOMResult<i32>;

        /// Sets the filter wheel position
        #[http("position")]
        fn set_position(&mut self, request: schemas::PutFilterwheelPositionRequest) -> ASCOMResult<()>;
    }

    /// Focuser Specific Methods
    #[http("focuser")]
    pub trait Focuser {
        /// True if the focuser is capable of absolute position; that is, being commanded to a specific step location.
        #[http("absolute")]
        fn get_absolute(&self) -> ASCOMResult<bool>;

        /// True if the focuser is currently moving to a new position. False if the focuser is stationary.
        #[http("ismoving")]
        fn get_ismoving(&self) -> ASCOMResult<bool>;

        /// Maximum increment size allowed by the focuser; i.e. the maximum number of steps allowed in one move operation.
        #[http("maxincrement")]
        fn get_maxincrement(&self) -> ASCOMResult<i32>;

        /// Maximum step position permitted.
        #[http("maxstep")]
        fn get_maxstep(&self) -> ASCOMResult<i32>;

        /// Current focuser position, in steps.
        #[http("position")]
        fn get_position(&self) -> ASCOMResult<i32>;

        /// Step size (microns) for the focuser.
        #[http("stepsize")]
        fn get_stepsize(&self) -> ASCOMResult<f64>;

        /// Gets the state of temperature compensation mode (if available), else always False.
        #[http("tempcomp")]
        fn get_tempcomp(&self) -> ASCOMResult<bool>;

        /// Sets the state of temperature compensation mode.
        #[http("tempcomp")]
        fn set_tempcomp(&mut self, request: schemas::PutFocuserTempcompRequest) -> ASCOMResult<()>;

        /// True if focuser has temperature compensation available.
        #[http("tempcompavailable")]
        fn get_tempcompavailable(&self) -> ASCOMResult<bool>;

        /// Current ambient temperature as measured by the focuser.
        #[http("temperature")]
        fn get_temperature(&self) -> ASCOMResult<f64>;

        /// Immediately stop any focuser motion due to a previous Move(Int32) method call.
        #[http("halt")]
        fn set_halt(&mut self) -> ASCOMResult<()>;

        /// Moves the focuser by the specified amount or to the specified position depending on the value of the Absolute property.
        #[http("move")]
        fn set_move(&mut self, request: schemas::PutFocuserMoveRequest) -> ASCOMResult<()>;
    }

    /// ObservingConditions Specific Methods
    #[http("observingconditions")]
    pub trait Observingconditions {
        /// Gets the time period over which observations will be averaged
        #[http("averageperiod")]
        fn get_averageperiod(&self) -> ASCOMResult<f64>;

        /// Sets the time period over which observations will be averaged
        #[http("averageperiod")]
        fn set_averageperiod(&mut self, request: schemas::PutObservingconditionsAverageperiodRequest) -> ASCOMResult<()>;

        /// Gets the percentage of the sky obscured by cloud
        #[http("cloudcover")]
        fn get_cloudcover(&self) -> ASCOMResult<f64>;

        /// Gets the atmospheric dew point at the observatory reported in °C.
        #[http("dewpoint")]
        fn get_dewpoint(&self) -> ASCOMResult<f64>;

        /// Gets the atmospheric  humidity (%) at the observatory
        #[http("humidity")]
        fn get_humidity(&self) -> ASCOMResult<f64>;

        /// Gets the atmospheric pressure in hectoPascals at the observatory's altitude - NOT reduced to sea level.
        #[http("pressure")]
        fn get_pressure(&self) -> ASCOMResult<f64>;

        /// Gets the rain rate (mm/hour) at the observatory.
        #[http("rainrate")]
        fn get_rainrate(&self) -> ASCOMResult<f64>;

        /// Gets the sky brightness at the observatory (Lux)
        #[http("skybrightness")]
        fn get_skybrightness(&self) -> ASCOMResult<f64>;

        /// Gets the sky quality at the observatory (magnitudes per square arc second)
        #[http("skyquality")]
        fn get_skyquality(&self) -> ASCOMResult<f64>;

        /// Gets the sky temperature(°C) at the observatory.
        #[http("skytemperature")]
        fn get_skytemperature(&self) -> ASCOMResult<f64>;

        /// Gets the seeing at the observatory measured as star full width half maximum (FWHM) in arc secs.
        #[http("starfwhm")]
        fn get_starfwhm(&self) -> ASCOMResult<f64>;

        /// Gets the temperature(°C) at the observatory.
        #[http("temperature")]
        fn get_temperature(&self) -> ASCOMResult<f64>;

        /// Gets the wind direction. The returned value must be between 0.0 and 360.0, interpreted according to the metereological standard, where a special value of 0.0 is returned when the wind speed is 0.0. Wind direction is measured clockwise from north, through east, where East=90.0, South=180.0, West=270.0 and North=360.0.
        #[http("winddirection")]
        fn get_winddirection(&self) -> ASCOMResult<f64>;

        /// Gets the peak 3 second wind gust(m/s) at the observatory over the last 2 minutes.
        #[http("windgust")]
        fn get_windgust(&self) -> ASCOMResult<f64>;

        /// Gets the wind speed(m/s) at the observatory.
        #[http("windspeed")]
        fn get_windspeed(&self) -> ASCOMResult<f64>;

        /// Forces the driver to immediately query its attached hardware to refresh sensor values.
        #[http("refresh")]
        fn set_refresh(&mut self) -> ASCOMResult<()>;

        /// Gets a description of the sensor with the name specified in the SensorName parameter
        #[http("sensordescription")]
        fn get_sensordescription(&self, request: schemas::GetObservingconditionsSensordescriptionRequest) -> ASCOMResult<String>;

        /// Gets the time since the sensor specified in the SensorName parameter was last updated
        #[http("timesincelastupdate")]
        fn get_timesincelastupdate(&self, request: schemas::GetObservingconditionsTimesincelastupdateRequest) -> ASCOMResult<f64>;
    }

    /// Rotator Specific Methods
    #[http("rotator")]
    pub trait Rotator {
        /// True if the Rotator supports the Reverse method.
        #[http("canreverse")]
        fn get_canreverse(&self) -> ASCOMResult<bool>;

        /// True if the rotator is currently moving to a new position. False if the focuser is stationary.
        #[http("ismoving")]
        fn get_ismoving(&self) -> ASCOMResult<bool>;

        /// Returns the raw mechanical position of the rotator in degrees.
        #[http("mechanicalposition")]
        fn get_mechanicalposition(&self) -> ASCOMResult<f64>;

        /// Current instantaneous Rotator position, in degrees.
        #[http("position")]
        fn get_position(&self) -> ASCOMResult<f64>;

        /// Returns the rotator’s Reverse state.
        #[http("reverse")]
        fn get_reverse(&self) -> ASCOMResult<bool>;

        /// Sets the rotator’s Reverse state.
        #[http("reverse")]
        fn set_reverse(&mut self, request: schemas::PutRotatorReverseRequest) -> ASCOMResult<()>;

        /// The minimum StepSize, in degrees.
        #[http("stepsize")]
        fn get_stepsize(&self) -> ASCOMResult<f64>;

        /// The destination position angle for Move() and MoveAbsolute().
        #[http("targetposition")]
        fn get_targetposition(&self) -> ASCOMResult<f64>;

        /// Immediately stop any Rotator motion due to a previous Move or MoveAbsolute method call.
        #[http("halt")]
        fn set_halt(&mut self) -> ASCOMResult<()>;

        /// Causes the rotator to move Position degrees relative to the current Position value.
        #[http("move")]
        fn set_move(&mut self, request: schemas::PutRotatorMoveRequest) -> ASCOMResult<()>;

        /// Causes the rotator to move the absolute position of Position degrees.
        #[http("moveabsolute")]
        fn set_moveabsolute(&mut self, request: schemas::PutRotatorMoveabsoluteRequest) -> ASCOMResult<()>;

        /// Causes the rotator to move the mechanical position of Position degrees.
        #[http("movemechanical")]
        fn set_movemechanical(&mut self, request: schemas::PutRotatorMovemechanicalRequest) -> ASCOMResult<()>;

        /// Causes the rotator to sync to the position of Position degrees.
        #[http("sync")]
        fn set_sync(&mut self, request: schemas::PutRotatorSyncRequest) -> ASCOMResult<()>;
    }

    /// SafetyMonitor Specific Methods
    #[http("safetymonitor")]
    pub trait Safetymonitor {
        /// Indicates whether the monitored state is safe for use. True if the state is safe, False if it is unsafe.
        #[http("issafe")]
        fn get_issafe(&self) -> ASCOMResult<bool>;
    }

    /// Switch Specific Methods
    #[http("switch")]
    pub trait Switch {
        /// Returns the number of switch devices managed by this driver. Devices are numbered from 0 to MaxSwitch - 1
        #[http("maxswitch")]
        fn get_maxswitch(&self) -> ASCOMResult<i32>;

        /// Reports if the specified switch device can be written to, default true. This is false if the device cannot be written to, for example a limit switch or a sensor.  Devices are numbered from 0 to MaxSwitch - 1
        #[http("canwrite")]
        fn get_canwrite(&self, request: schemas::GetSwitchCanwriteRequest) -> ASCOMResult<bool>;

        /// Return the state of switch device id as a boolean.  Devices are numbered from 0 to MaxSwitch - 1
        #[http("getswitch")]
        fn get_getswitch(&self, request: schemas::GetSwitchGetswitchRequest) -> ASCOMResult<bool>;

        /// Gets the description of the specified switch device. This is to allow a fuller description of the device to be returned, for example for a tool tip. Devices are numbered from 0 to MaxSwitch - 1
        #[http("getswitchdescription")]
        fn get_getswitchdescription(&self, request: schemas::GetSwitchGetswitchdescriptionRequest) -> ASCOMResult<String>;

        /// Gets the name of the specified switch device. Devices are numbered from 0 to MaxSwitch - 1
        #[http("getswitchname")]
        fn get_getswitchname(&self, request: schemas::GetSwitchGetswitchnameRequest) -> ASCOMResult<String>;

        /// Gets the value of the specified switch device as a double. Devices are numbered from 0 to MaxSwitch - 1, The value of this switch is expected to be between MinSwitchValue and MaxSwitchValue.
        #[http("getswitchvalue")]
        fn get_getswitchvalue(&self, request: schemas::GetSwitchGetswitchvalueRequest) -> ASCOMResult<f64>;

        /// Gets the minimum value of the specified switch device as a double. Devices are numbered from 0 to MaxSwitch - 1.
        #[http("minswitchvalue")]
        fn get_minswitchvalue(&self, request: schemas::GetSwitchMinswitchvalueRequest) -> ASCOMResult<f64>;

        /// Gets the maximum value of the specified switch device as a double. Devices are numbered from 0 to MaxSwitch - 1.
        #[http("maxswitchvalue")]
        fn get_maxswitchvalue(&self, request: schemas::GetSwitchMaxswitchvalueRequest) -> ASCOMResult<f64>;

        /// Sets a switch controller device to the specified state, true or false.
        #[http("setswitch")]
        fn set_setswitch(&mut self, request: schemas::PutSwitchSetswitchRequest) -> ASCOMResult<()>;

        /// Sets a switch device name to the specified value.
        #[http("setswitchname")]
        fn set_setswitchname(&mut self, request: schemas::PutSwitchSetswitchnameRequest) -> ASCOMResult<()>;

        /// Sets a switch device value to the specified value.
        #[http("setswitchvalue")]
        fn set_setswitchvalue(&mut self, request: schemas::PutSwitchSetswitchvalueRequest) -> ASCOMResult<()>;

        /// Returns the step size that this device supports (the difference between successive values of the device). Devices are numbered from 0 to MaxSwitch - 1.
        #[http("switchstep")]
        fn get_switchstep(&self, request: schemas::GetSwitchSwitchstepRequest) -> ASCOMResult<f64>;
    }

    /// Telescope Specific Methods
    #[http("telescope")]
    pub trait Telescope {
        /// Returns the alignment mode of the mount (Alt/Az, Polar, German Polar).  The alignment mode is specified as an integer value from the AlignmentModes Enum.
        #[http("alignmentmode")]
        fn get_alignmentmode(&self) -> ASCOMResult<i32>;

        /// The altitude above the local horizon of the mount's current position (degrees, positive up)
        #[http("altitude")]
        fn get_altitude(&self) -> ASCOMResult<f64>;

        /// The area of the telescope's aperture, taking into account any obstructions (square meters)
        #[http("aperturearea")]
        fn get_aperturearea(&self) -> ASCOMResult<f64>;

        /// The telescope's effective aperture diameter (meters)
        #[http("aperturediameter")]
        fn get_aperturediameter(&self) -> ASCOMResult<f64>;

        /// True if the mount is stopped in the Home position. Set only following a FindHome()  operation, and reset with any slew operation. This property must be False if the telescope does not support homing.
        #[http("athome")]
        fn get_athome(&self) -> ASCOMResult<bool>;

        /// True if the telescope has been put into the parked state by the seee Park()  method. Set False by calling the Unpark() method.
        #[http("atpark")]
        fn get_atpark(&self) -> ASCOMResult<bool>;

        /// The azimuth at the local horizon of the mount's current position (degrees, North-referenced, positive East/clockwise).
        #[http("azimuth")]
        fn get_azimuth(&self) -> ASCOMResult<f64>;

        /// True if this telescope is capable of programmed finding its home position (FindHome()  method).
        #[http("canfindhome")]
        fn get_canfindhome(&self) -> ASCOMResult<bool>;

        /// True if this telescope is capable of programmed parking (Park() method)
        #[http("canpark")]
        fn get_canpark(&self) -> ASCOMResult<bool>;

        /// True if this telescope is capable of software-pulsed guiding (via the PulseGuide(GuideDirections, Int32) method)
        #[http("canpulseguide")]
        fn get_canpulseguide(&self) -> ASCOMResult<bool>;

        /// True if the DeclinationRate property can be changed to provide offset tracking in the declination axis.
        #[http("cansetdeclinationrate")]
        fn get_cansetdeclinationrate(&self) -> ASCOMResult<bool>;

        /// True if the guide rate properties used for PulseGuide(GuideDirections, Int32) can ba adjusted.
        #[http("cansetguiderates")]
        fn get_cansetguiderates(&self) -> ASCOMResult<bool>;

        /// True if this telescope is capable of programmed setting of its park position (SetPark() method)
        #[http("cansetpark")]
        fn get_cansetpark(&self) -> ASCOMResult<bool>;

        /// True if the SideOfPier property can be set, meaning that the mount can be forced to flip.
        #[http("cansetpierside")]
        fn get_cansetpierside(&self) -> ASCOMResult<bool>;

        /// True if the RightAscensionRate property can be changed to provide offset tracking in the right ascension axis. .
        #[http("cansetrightascensionrate")]
        fn get_cansetrightascensionrate(&self) -> ASCOMResult<bool>;

        /// True if the Tracking property can be changed, turning telescope sidereal tracking on and off.
        #[http("cansettracking")]
        fn get_cansettracking(&self) -> ASCOMResult<bool>;

        /// True if this telescope is capable of programmed slewing (synchronous or asynchronous) to equatorial coordinates
        #[http("canslew")]
        fn get_canslew(&self) -> ASCOMResult<bool>;

        /// True if this telescope is capable of programmed slewing (synchronous or asynchronous) to local horizontal coordinates
        #[http("canslewaltaz")]
        fn get_canslewaltaz(&self) -> ASCOMResult<bool>;

        /// True if this telescope is capable of programmed asynchronous slewing to local horizontal coordinates
        #[http("canslewaltazasync")]
        fn get_canslewaltazasync(&self) -> ASCOMResult<bool>;

        /// True if this telescope is capable of programmed asynchronous slewing to equatorial coordinates.
        #[http("canslewasync")]
        fn get_canslewasync(&self) -> ASCOMResult<bool>;

        /// True if this telescope is capable of programmed synching to equatorial coordinates.
        #[http("cansync")]
        fn get_cansync(&self) -> ASCOMResult<bool>;

        /// True if this telescope is capable of programmed synching to local horizontal coordinates
        #[http("cansyncaltaz")]
        fn get_cansyncaltaz(&self) -> ASCOMResult<bool>;

        /// True if this telescope is capable of programmed unparking (UnPark() method)
        #[http("canunpark")]
        fn get_canunpark(&self) -> ASCOMResult<bool>;

        /// The declination (degrees) of the mount's current equatorial coordinates, in the coordinate system given by the EquatorialSystem property. Reading the property will raise an error if the value is unavailable.
        #[http("declination")]
        fn get_declination(&self) -> ASCOMResult<f64>;

        /// The declination tracking rate (arcseconds per second, default = 0.0)
        #[http("declinationrate")]
        fn get_declinationrate(&self) -> ASCOMResult<f64>;

        /// Sets the declination tracking rate (arcseconds per second)
        #[http("declinationrate")]
        fn set_declinationrate(&mut self, request: schemas::PutTelescopeDeclinationrateRequest) -> ASCOMResult<()>;

        /// True if the telescope or driver applies atmospheric refraction to coordinates.
        #[http("doesrefraction")]
        fn get_doesrefraction(&self) -> ASCOMResult<bool>;

        /// Causes the rotator to move Position degrees relative to the current Position value.
        #[http("doesrefraction")]
        fn set_doesrefraction(&mut self, request: schemas::PutTelescopeDoesrefractionRequest) -> ASCOMResult<()>;

        /// Returns the current equatorial coordinate system used by this telescope (e.g. Topocentric or J2000).
        #[http("equatorialsystem")]
        fn get_equatorialsystem(&self) -> ASCOMResult<i32>;

        /// The telescope's focal length in meters
        #[http("focallength")]
        fn get_focallength(&self) -> ASCOMResult<f64>;

        /// The current Declination movement rate offset for telescope guiding (degrees/sec)
        #[http("guideratedeclination")]
        fn get_guideratedeclination(&self) -> ASCOMResult<f64>;

        /// Sets the current Declination movement rate offset for telescope guiding (degrees/sec).
        #[http("guideratedeclination")]
        fn set_guideratedeclination(&mut self, request: schemas::PutTelescopeGuideratedeclinationRequest) -> ASCOMResult<()>;

        /// The current RightAscension movement rate offset for telescope guiding (degrees/sec)
        #[http("guideraterightascension")]
        fn get_guideraterightascension(&self) -> ASCOMResult<f64>;

        /// Sets the current RightAscension movement rate offset for telescope guiding (degrees/sec).
        #[http("guideraterightascension")]
        fn set_guideraterightascension(&mut self, request: schemas::PutTelescopeGuideraterightascensionRequest) -> ASCOMResult<()>;

        /// True if a PulseGuide(GuideDirections, Int32) command is in progress, False otherwise
        #[http("ispulseguiding")]
        fn get_ispulseguiding(&self) -> ASCOMResult<bool>;

        /// The right ascension (hours) of the mount's current equatorial coordinates, in the coordinate system given by the EquatorialSystem property
        #[http("rightascension")]
        fn get_rightascension(&self) -> ASCOMResult<f64>;

        /// The right ascension tracking rate (arcseconds per second, default = 0.0)
        #[http("rightascensionrate")]
        fn get_rightascensionrate(&self) -> ASCOMResult<f64>;

        /// Sets the right ascension tracking rate (arcseconds per second)
        #[http("rightascensionrate")]
        fn set_rightascensionrate(&mut self, request: schemas::PutTelescopeRightascensionrateRequest) -> ASCOMResult<()>;

        /// Indicates the pointing state of the mount. 0 = pierEast, 1 = pierWest, -1= pierUnknown
        #[http("sideofpier")]
        fn get_sideofpier(&self) -> ASCOMResult<i32>;

        /// Sets the pointing state of the mount. 0 = pierEast, 1 = pierWest
        #[http("sideofpier")]
        fn set_sideofpier(&mut self, request: schemas::PutTelescopeSideofpierRequest) -> ASCOMResult<()>;

        /// The local apparent sidereal time from the telescope's internal clock (hours, sidereal).
        #[http("siderealtime")]
        fn get_siderealtime(&self) -> ASCOMResult<f64>;

        /// The elevation above mean sea level (meters) of the site at which the telescope is located.
        #[http("siteelevation")]
        fn get_siteelevation(&self) -> ASCOMResult<f64>;

        /// Sets the elevation above mean sea level (metres) of the site at which the telescope is located.
        #[http("siteelevation")]
        fn set_siteelevation(&mut self, request: schemas::PutTelescopeSiteelevationRequest) -> ASCOMResult<()>;

        /// The geodetic(map) latitude (degrees, positive North, WGS84) of the site at which the telescope is located.
        #[http("sitelatitude")]
        fn get_sitelatitude(&self) -> ASCOMResult<f64>;

        /// Sets the observing site's latitude (degrees).
        #[http("sitelatitude")]
        fn set_sitelatitude(&mut self, request: schemas::PutTelescopeSitelatitudeRequest) -> ASCOMResult<()>;

        /// The longitude (degrees, positive East, WGS84) of the site at which the telescope is located.
        #[http("sitelongitude")]
        fn get_sitelongitude(&self) -> ASCOMResult<f64>;

        /// Sets the observing site's longitude (degrees, positive East, WGS84).
        #[http("sitelongitude")]
        fn set_sitelongitude(&mut self, request: schemas::PutTelescopeSitelongitudeRequest) -> ASCOMResult<()>;

        /// True if telescope is currently moving in response to one of the Slew methods or the MoveAxis(TelescopeAxes, Double) method, False at all other times.
        #[http("slewing")]
        fn get_slewing(&self) -> ASCOMResult<bool>;

        /// Returns the post-slew settling time (sec.).
        #[http("slewsettletime")]
        fn get_slewsettletime(&self) -> ASCOMResult<i32>;

        /// Sets the  post-slew settling time (integer sec.).
        #[http("slewsettletime")]
        fn set_slewsettletime(&mut self, request: schemas::PutTelescopeSlewsettletimeRequest) -> ASCOMResult<()>;

        /// The declination (degrees, positive North) for the target of an equatorial slew or sync operation
        #[http("targetdeclination")]
        fn get_targetdeclination(&self) -> ASCOMResult<f64>;

        /// Sets the declination (degrees, positive North) for the target of an equatorial slew or sync operation
        #[http("targetdeclination")]
        fn set_targetdeclination(&mut self, request: schemas::PutTelescopeTargetdeclinationRequest) -> ASCOMResult<()>;

        /// The right ascension (hours) for the target of an equatorial slew or sync operation
        #[http("targetrightascension")]
        fn get_targetrightascension(&self) -> ASCOMResult<f64>;

        /// Sets the right ascension (hours) for the target of an equatorial slew or sync operation
        #[http("targetrightascension")]
        fn set_targetrightascension(&mut self, request: schemas::PutTelescopeTargetrightascensionRequest) -> ASCOMResult<()>;

        /// Returns the state of the telescope's sidereal tracking drive.
        #[http("tracking")]
        fn get_tracking(&self) -> ASCOMResult<bool>;

        /// Sets the state of the telescope's sidereal tracking drive.
        #[http("tracking")]
        fn set_tracking(&mut self, request: schemas::PutTelescopeTrackingRequest) -> ASCOMResult<()>;

        /// The current tracking rate of the telescope's sidereal drive.
        #[http("trackingrate")]
        fn get_trackingrate(&self) -> ASCOMResult<i32>;

        /// Sets the tracking rate of the telescope's sidereal drive. 0 = driveSidereal, 1 = driveLunar, 2 = driveSolar, 3 = driveKing
        #[http("trackingrate")]
        fn set_trackingrate(&mut self, request: schemas::PutTelescopeTrackingrateRequest) -> ASCOMResult<()>;

        /// Returns an array of supported DriveRates values that describe the permissible values of the TrackingRate property for this telescope type.
        #[http("trackingrates")]
        fn get_trackingrates(&self) -> ASCOMResult<Vec<schemas::DriveRate>>;

        /// The UTC date/time of the telescope's internal clock in ISO 8601 format including fractional seconds. The general format (in Microsoft custom date format style) is yyyy-MM-ddTHH:mm:ss.fffffffZ E.g. 2016-03-04T17:45:31.1234567Z or 2016-11-14T07:03:08.1234567Z Please note the compulsary trailing Z indicating the 'Zulu', UTC time zone.
        #[http("utcdate")]
        fn get_utcdate(&self) -> ASCOMResult<String>;

        /// The UTC date/time of the telescope's internal clock in ISO 8601 format including fractional seconds. The general format (in Microsoft custom date format style) is yyyy-MM-ddTHH:mm:ss.fffffffZ E.g. 2016-03-04T17:45:31.1234567Z or 2016-11-14T07:03:08.1234567Z Please note the compulsary trailing Z indicating the 'Zulu', UTC time zone.
        #[http("utcdate")]
        fn set_utcdate(&mut self, request: schemas::PutTelescopeUtcdateRequest) -> ASCOMResult<()>;

        /// Immediately Stops a slew in progress.
        #[http("abortslew")]
        fn set_abortslew(&mut self) -> ASCOMResult<()>;

        /// The rates at which the telescope may be moved about the specified axis by the MoveAxis(TelescopeAxes, Double) method.
        #[http("axisrates")]
        fn get_axisrates(&self, request: schemas::GetTelescopeAxisratesRequest) -> ASCOMResult<Vec<schemas::AxisRate>>;

        /// True if this telescope can move the requested axis.
        #[http("canmoveaxis")]
        fn get_canmoveaxis(&self, request: schemas::GetTelescopeCanmoveaxisRequest) -> ASCOMResult<bool>;

        /// Predicts the pointing state that a German equatorial mount will be in if it slews to the given coordinates. The  return value will be one of - 0 = pierEast, 1 = pierWest, -1 = pierUnknown
        #[http("destinationsideofpier")]
        fn get_destinationsideofpier(&self, request: schemas::GetTelescopeDestinationsideofpierRequest) -> ASCOMResult<i32>;

        /// Locates the telescope's "home" position (synchronous)
        #[http("findhome")]
        fn set_findhome(&mut self) -> ASCOMResult<()>;

        /// Move the telescope in one axis at the given rate.
        #[http("moveaxis")]
        fn set_moveaxis(&mut self, request: schemas::PutTelescopeMoveaxisRequest) -> ASCOMResult<()>;

        /// Move the telescope to its park position, stop all motion (or restrict to a small safe range), and set AtPark to True. )
        #[http("park")]
        fn set_park(&mut self) -> ASCOMResult<()>;

        /// Moves the scope in the given direction for the given interval or time at the rate given by the corresponding guide rate property
        #[http("pulseguide")]
        fn set_pulseguide(&mut self, request: schemas::PutTelescopePulseguideRequest) -> ASCOMResult<()>;

        /// Sets the telescope's park position to be its current position.
        #[http("setpark")]
        fn set_setpark(&mut self) -> ASCOMResult<()>;

        /// Move the telescope to the given local horizontal coordinates, return when slew is complete
        #[http("slewtoaltaz")]
        fn set_slewtoaltaz(&mut self, request: schemas::PutTelescopeSlewtoaltazRequest) -> ASCOMResult<()>;

        /// Move the telescope to the given local horizontal coordinates, return immediatley after the slew starts. The client can poll the Slewing method to determine when the mount reaches the intended coordinates.
        #[http("slewtoaltazasync")]
        fn set_slewtoaltazasync(&mut self, request: schemas::PutTelescopeSlewtoaltazRequest) -> ASCOMResult<()>;

        /// Move the telescope to the given equatorial coordinates, return when slew is complete
        #[http("slewtocoordinates")]
        fn set_slewtocoordinates(&mut self, request: schemas::PutTelescopeSlewtocoordinatesRequest) -> ASCOMResult<()>;

        /// Move the telescope to the given equatorial coordinates, return immediatley after the slew starts. The client can poll the Slewing method to determine when the mount reaches the intended coordinates.
        #[http("slewtocoordinatesasync")]
        fn set_slewtocoordinatesasync(&mut self, request: schemas::PutTelescopeSlewtocoordinatesRequest) -> ASCOMResult<()>;

        /// Move the telescope to the TargetRightAscension and TargetDeclination equatorial coordinates, return when slew is complete
        #[http("slewtotarget")]
        fn set_slewtotarget(&mut self) -> ASCOMResult<()>;

        /// Move the telescope to the TargetRightAscension and TargetDeclination equatorial coordinates, return immediatley after the slew starts. The client can poll the Slewing method to determine when the mount reaches the intended coordinates.
        #[http("slewtotargetasync")]
        fn set_slewtotargetasync(&mut self) -> ASCOMResult<()>;

        /// Matches the scope's local horizontal coordinates to the given local horizontal coordinates.
        #[http("synctoaltaz")]
        fn set_synctoaltaz(&mut self, request: schemas::PutTelescopeSlewtoaltazRequest) -> ASCOMResult<()>;

        /// Matches the scope's equatorial coordinates to the given equatorial coordinates.
        #[http("synctocoordinates")]
        fn set_synctocoordinates(&mut self, request: schemas::PutTelescopeSlewtocoordinatesRequest) -> ASCOMResult<()>;

        /// Matches the scope's equatorial coordinates to the TargetRightAscension and TargetDeclination equatorial coordinates.
        #[http("synctotarget")]
        fn set_synctotarget(&mut self) -> ASCOMResult<()>;

        /// Takes telescope out of the Parked state. )
        #[http("unpark")]
        fn set_unpark(&mut self) -> ASCOMResult<()>;
    }
}

pub fn service() -> actix_web::Scope {
    actix_web::web::scope("/api/v1")
        .service(RpcService::<dyn DeviceType>::default())
        .service(RpcService::<dyn Camera>::default())
        .service(RpcService::<dyn Covercalibrator>::default())
        .service(RpcService::<dyn Dome>::default())
        .service(RpcService::<dyn Filterwheel>::default())
        .service(RpcService::<dyn Focuser>::default())
        .service(RpcService::<dyn Observingconditions>::default())
        .service(RpcService::<dyn Rotator>::default())
        .service(RpcService::<dyn Safetymonitor>::default())
        .service(RpcService::<dyn Switch>::default())
        .service(RpcService::<dyn Telescope>::default())
}
