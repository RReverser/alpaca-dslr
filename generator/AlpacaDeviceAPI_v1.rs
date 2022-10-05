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

mod parameters {

    /**
    One of the recognised ASCOM device types e.g. telescope (must be lower case)
    */
    #[derive(Deserialize)]
    #[repr(transparent)]
    struct DeviceType(String);

    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[derive(Deserialize)]
    #[repr(transparent)]
    struct DeviceNumber(u32);

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[derive(Deserialize)]
    #[repr(transparent)]
    struct ClientIdquery(u32);

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[derive(Deserialize)]
    #[repr(transparent)]
    struct ClientTransactionIdquery(u32);

    /**
    Right Ascension coordinate (0.0 to 23.99999999 hours)
    */
    #[derive(Deserialize)]
    #[repr(transparent)]
    struct RightAscensionQuery(f64);

    /**
    Declination coordinate (-90.0 to +90.0 degrees)
    */
    #[derive(Deserialize)]
    #[repr(transparent)]
    struct DeclinationQuery(f64);

    /**
    The axis about which rate information is desired. 0 = axisPrimary, 1 = axisSecondary, 2 = axisTertiary.
    */
    #[derive(Deserialize)]
    #[repr(transparent)]
    struct AxisQuery(i32);

    /**
    The device number (0 to MaxSwitch - 1)
    */
    #[derive(Deserialize)]
    #[repr(transparent)]
    struct SwitchNumberQuery(i32);
}

mod schemas {

    #[derive(Serialize)]

    struct ImageArrayResponse {
        /**
        0 = Unknown, 1 = Short(int16), 2 = Integer (int32), 3 = Double (Double precision real number).
        */
        #[serde(rename = "Type")]
        type_: Option<i32>,

        /**
        The array's rank, will be 2 (single plane image (monochrome)) or 3 (multi-plane image).
        */
        #[serde(rename = "Rank")]
        rank: Option<i32>,

        /**
        Returned integer or double value
        */
        #[serde(rename = "Value")]
        value: Option<Vec<Vec<f64>>>,

        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: Option<u32>,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: Option<i32>,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: Option<String>,
    }

    impl IntoResponse for ImageArrayResponse {
        fn into_response(self) -> Response<UnsyncBoxBody<Bytes, Error>> {
            Json(self).into_response()
        }
    }

    #[derive(Serialize)]

    struct BoolResponse {
        /**
        True or False value
        */
        #[serde(rename = "Value")]
        value: Option<bool>,

        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: Option<u32>,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: Option<i32>,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered. If an error message is returned, a non zero error number must also be returned.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: Option<String>,
    }

    impl IntoResponse for BoolResponse {
        fn into_response(self) -> Response<UnsyncBoxBody<Bytes, Error>> {
            Json(self).into_response()
        }
    }

    #[derive(Serialize)]

    struct DoubleResponse {
        /**
        Returned double value
        */
        #[serde(rename = "Value")]
        value: Option<f64>,

        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: Option<u32>,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: Option<i32>,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered. If an error message is returned, a non zero error number must also be returned.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: Option<String>,
    }

    impl IntoResponse for DoubleResponse {
        fn into_response(self) -> Response<UnsyncBoxBody<Bytes, Error>> {
            Json(self).into_response()
        }
    }

    #[derive(Serialize)]

    struct IntResponse {
        /**
        Returned integer value
        */
        #[serde(rename = "Value")]
        value: Option<i32>,

        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: Option<u32>,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: Option<i32>,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered. If an error message is returned, a non zero error number must also be returned.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: Option<String>,
    }

    impl IntoResponse for IntResponse {
        fn into_response(self) -> Response<UnsyncBoxBody<Bytes, Error>> {
            Json(self).into_response()
        }
    }

    #[derive(Serialize)]

    struct IntArrayResponse {
        /**
        Array of integer values.
        */
        #[serde(rename = "Value")]
        value: Option<Vec<i32>>,

        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: Option<u32>,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: Option<i32>,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered. If an error message is returned, a non zero error number must also be returned.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: Option<String>,
    }

    impl IntoResponse for IntArrayResponse {
        fn into_response(self) -> Response<UnsyncBoxBody<Bytes, Error>> {
            Json(self).into_response()
        }
    }

    #[derive(Serialize)]

    struct MethodResponse {
        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: Option<u32>,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: Option<i32>,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered. If an error message is returned, a non zero error number must also be returned.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: Option<String>,
    }

    impl IntoResponse for MethodResponse {
        fn into_response(self) -> Response<UnsyncBoxBody<Bytes, Error>> {
            Json(self).into_response()
        }
    }

    #[derive(Serialize)]

    struct StringResponse {
        /**
        String response from the device.
        */
        #[serde(rename = "Value")]
        value: Option<String>,

        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: Option<u32>,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: Option<i32>,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered. If an error message is returned, a non zero error number must also be returned.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: Option<String>,
    }

    impl IntoResponse for StringResponse {
        fn into_response(self) -> Response<UnsyncBoxBody<Bytes, Error>> {
            Json(self).into_response()
        }
    }

    #[derive(Serialize)]

    struct StringArrayResponse {
        /**
        Array of string values.
        */
        #[serde(rename = "Value")]
        value: Option<Vec<String>>,

        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: Option<u32>,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: Option<i32>,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered. If an error message is returned, a non zero error number must also be returned.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: Option<String>,
    }

    impl IntoResponse for StringArrayResponse {
        fn into_response(self) -> Response<UnsyncBoxBody<Bytes, Error>> {
            Json(self).into_response()
        }
    }

    #[derive(Serialize)]

    struct AxisRatesResponse {
        /**
        Array of AxisRate objects
        */
        #[serde(rename = "Value")]
        value: Option<Vec<schemas::AxisRate>>,

        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: Option<u32>,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: Option<i32>,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered. If an error message is returned, a non zero error number must also be returned.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: Option<String>,
    }

    impl IntoResponse for AxisRatesResponse {
        fn into_response(self) -> Response<UnsyncBoxBody<Bytes, Error>> {
            Json(self).into_response()
        }
    }

    struct AxisRate {
        /**
        The maximum rate (degrees per second) This must always be a positive number. It indicates the maximum rate in either direction about the axis.
        */
        #[serde(rename = "Maximum")]
        maximum: f64,

        /**
        The minimum rate (degrees per second) This must always be a positive number. It indicates the maximum rate in either direction about the axis.
        */
        #[serde(rename = "Minimum")]
        minimum: f64,
    }

    #[derive(Serialize)]

    struct DriveRatesResponse {
        /**
        Array of DriveRate values
        */
        #[serde(rename = "Value")]
        value: Option<Vec<schemas::DriveRate>>,

        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: Option<u32>,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: Option<i32>,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered. If an error message is returned, a non zero error number must also be returned.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: Option<String>,
    }

    impl IntoResponse for DriveRatesResponse {
        fn into_response(self) -> Response<UnsyncBoxBody<Bytes, Error>> {
            Json(self).into_response()
        }
    }

    #[repr(transparent)]
    struct DriveRate(f64);

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutActionPath {
        /**
        One of the recognised ASCOM device types e.g. telescope (must be lower case)
        */
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutActionRequest {
        /**
        A well known name that represents the action to be carried out.
        */
        #[serde(rename = "Action")]
        action: String,

        /**
        List of required parameters or an Empty String if none are required
        */
        #[serde(rename = "Parameters")]
        parameters: String,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCommandblindPath {
        /**
        One of the recognised ASCOM device types e.g. telescope (must be lower case)
        */
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCommandblindRequest {
        /**
        The literal command string to be transmitted.
        */
        #[serde(rename = "Command")]
        command: String,

        /**
        If set to true the string is transmitted 'as-is', if set to false then protocol framing characters may be added prior to transmission
        */
        #[serde(rename = "Raw")]
        raw: String,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCommandboolPath {
        /**
        One of the recognised ASCOM device types e.g. telescope (must be lower case)
        */
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCommandstringPath {
        /**
        One of the recognised ASCOM device types e.g. telescope (must be lower case)
        */
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetConnectedPath {
        /**
        One of the recognised ASCOM device types e.g. telescope (must be lower case)
        */
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetConnectedQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutConnectedPath {
        /**
        One of the recognised ASCOM device types e.g. telescope (must be lower case)
        */
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutConnectedRequest {
        /**
        Set True to connect to the device hardware, set False to disconnect from the device hardware
        */
        #[serde(rename = "Connected")]
        connected: bool,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDescriptionPath {
        /**
        One of the recognised ASCOM device types e.g. telescope (must be lower case)
        */
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDescriptionQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDriverinfoPath {
        /**
        One of the recognised ASCOM device types e.g. telescope (must be lower case)
        */
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDriverinfoQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDriverversionPath {
        /**
        One of the recognised ASCOM device types e.g. telescope (must be lower case)
        */
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDriverversionQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetInterfaceversionPath {
        /**
        One of the recognised ASCOM device types e.g. telescope (must be lower case)
        */
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetInterfaceversionQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetNamePath {
        /**
        One of the recognised ASCOM device types e.g. telescope (must be lower case)
        */
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetNameQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSupportedactionsPath {
        /**
        One of the recognised ASCOM device types e.g. telescope (must be lower case)
        */
        #[serde(rename = "device_type")]
        device_type: Option<String>,

        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSupportedactionsQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraBayeroffsetxPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraBayeroffsetxQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraBayeroffsetyPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraBayeroffsetyQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraBinxPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraBinxQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraBinxPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraBinxRequest {
        /**
        The X binning value
        */
        #[serde(rename = "BinX")]
        bin_x: i32,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraBinyPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraBinyQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraBinyPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraBinyRequest {
        /**
        The Y binning value
        */
        #[serde(rename = "BinY")]
        bin_y: i32,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCamerastatePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCamerastateQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCameraxsizePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCameraxsizeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCameraysizePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCameraysizeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCanabortexposurePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCanabortexposureQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCanasymmetricbinPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCanasymmetricbinQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCanfastreadoutPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCanfastreadoutQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCangetcoolerpowerPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCangetcoolerpowerQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCanpulseguidePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCanpulseguideQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCansetccdtemperaturePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCansetccdtemperatureQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCanstopexposurePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCanstopexposureQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCcdtemperaturePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCcdtemperatureQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCooleronPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCooleronQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraCooleronPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraCooleronRequest {
        /**
        Cooler state
        */
        #[serde(rename = "CoolerOn")]
        cooler_on: bool,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraCoolerpowerPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraCoolerpowerQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraElectronsperaduPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraElectronsperaduQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraExposuremaxPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraExposuremaxQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraExposureminPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraExposureminQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraExposureresolutionPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraExposureresolutionQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraFastreadoutPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraFastreadoutQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraFastreadoutPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraFastreadoutRequest {
        /**
        True to enable fast readout mode
        */
        #[serde(rename = "FastReadout")]
        fast_readout: bool,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraFullwellcapacityPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraFullwellcapacityQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraGainPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraGainQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraGainPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraGainRequest {
        /**
        Index of the current camera gain in the Gains string array.
        */
        #[serde(rename = "Gain")]
        gain: i32,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraGainmaxPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraGainmaxQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraGainminPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraGainminQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraGainsPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraGainsQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraHasshutterPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraHasshutterQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraHeatsinktemperaturePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraHeatsinktemperatureQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraImagearrayPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraImagearrayQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraImagearrayvariantPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraImagearrayvariantQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraImagereadyPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraImagereadyQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraIspulseguidingPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraIspulseguidingQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraLastexposuredurationPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraLastexposuredurationQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraLastexposurestarttimePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraLastexposurestarttimeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraMaxaduPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraMaxaduQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraMaxbinxPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraMaxbinxQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraMaxbinyPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraMaxbinyQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraNumxPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraNumxQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraNumxPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraNumxRequest {
        /**
        Sets the subframe width, if binning is active, value is in binned pixels.
        */
        #[serde(rename = "NumX")]
        num_x: i32,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraNumyPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraNumyQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraNumyPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraNumyRequest {
        /**
        Sets the subframe height, if binning is active, value is in binned pixels.
        */
        #[serde(rename = "NumY")]
        num_y: i32,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraOffsetPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraOffsetQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraOffsetPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraOffsetRequest {
        /**
        Index of the current camera offset in the offsets string array.
        */
        #[serde(rename = "offset")]
        offset: i32,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraOffsetmaxPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraOffsetmaxQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraOffsetminPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraOffsetminQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraOffsetsPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraOffsetsQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraPercentcompletedPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraPercentcompletedQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraPixelsizexPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraPixelsizexQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraPixelsizeyPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraPixelsizeyQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraReadoutmodePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraReadoutmodeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraReadoutmodePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraReadoutmodeRequest {
        /**
        Index into the ReadoutModes array of string readout mode names indicating the camera's current readout mode.
        */
        #[serde(rename = "ReadoutMode")]
        readout_mode: i32,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraReadoutmodesPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraReadoutmodesQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraSensornamePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraSensornameQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraSensortypePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraSensortypeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraSetccdtemperaturePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraSetccdtemperatureQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraSetccdtemperaturePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraSetccdtemperatureRequest {
        /**
        Temperature set point (degrees Celsius).
        */
        #[serde(rename = "SetCCDTemperature")]
        set_ccdtemperature: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraStartxPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraStartxQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraStartxPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraStartxRequest {
        /**
        The subframe X axis start position in binned pixels.
        */
        #[serde(rename = "StartX")]
        start_x: i32,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraStartyPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraStartyQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraStartyPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraStartyRequest {
        /**
        The subframe Y axis start position in binned pixels.
        */
        #[serde(rename = "StartY")]
        start_y: i32,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCameraSubexposuredurationPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCameraSubexposuredurationQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraSubexposuredurationPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraSubexposuredurationRequest {
        /**
        The request sub exposure duration in seconds
        */
        #[serde(rename = "SubExposureDuration")]
        sub_exposure_duration: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraAbortexposurePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraAbortexposureRequest {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraPulseguidePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraPulseguideRequest {
        /**
        Direction of movement (0 = North, 1 = South, 2 = East, 3 = West)
        */
        #[serde(rename = "Direction")]
        direction: i32,

        /**
        Duration of movement in milli-seconds
        */
        #[serde(rename = "Duration")]
        duration: i32,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraStartexposurePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCameraStartexposureRequest {
        /**
        Duration of exposure in seconds
        */
        #[serde(rename = "Duration")]
        duration: f64,

        /**
        True if light frame, false if dark frame.
        */
        #[serde(rename = "Light")]
        light: bool,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCameraStopexposurePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCovercalibratorBrightnessPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCovercalibratorBrightnessQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCovercalibratorCalibratorstatePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCovercalibratorCalibratorstateQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCovercalibratorCoverstatePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCovercalibratorCoverstateQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetCovercalibratorMaxbrightnessPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetCovercalibratorMaxbrightnessQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCovercalibratorCalibratoroffPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCovercalibratorCalibratoronPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutCovercalibratorCalibratoronRequest {
        /**
        The required brightness in the range 0 to MaxBrightness
        */
        #[serde(rename = "Brightness")]
        brightness: Option<i32>,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCovercalibratorClosecoverPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCovercalibratorHaltcoverPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutCovercalibratorOpencoverPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeAltitudePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeAltitudeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeAthomePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeAthomeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeAtparkPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeAtparkQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeAzimuthPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeAzimuthQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeCanfindhomePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeCanfindhomeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeCanparkPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeCanparkQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeCansetaltitudePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeCansetaltitudeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeCansetazimuthPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeCansetazimuthQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeCansetparkPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeCansetparkQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeCansetshutterPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeCansetshutterQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeCanslavePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeCanslaveQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeCansyncazimuthPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeCansyncazimuthQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeShutterstatusPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeShutterstatusQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeSlavedPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeSlavedQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeSlavedPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutDomeSlavedRequest {
        /**
        True if telescope is slaved to dome, otherwise false
        */
        #[serde(rename = "Slaved")]
        slaved: bool,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetDomeSlewingPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetDomeSlewingQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeAbortslewPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeCloseshutterPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeFindhomePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeOpenshutterPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeParkPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeSetparkPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeSlewtoaltitudePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutDomeSlewtoaltitudeRequest {
        /**
        Target dome altitude (degrees, horizon zero and increasing positive to 90 zenith)
        */
        #[serde(rename = "Altitude")]
        altitude: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeSlewtoazimuthPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutDomeSlewtoazimuthRequest {
        /**
        Target dome azimuth (degrees, North zero and increasing clockwise. i.e., 90 East, 180 South, 270 West)
        */
        #[serde(rename = "Azimuth")]
        azimuth: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutDomeSynctoazimuthPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFilterwheelFocusoffsetsPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFilterwheelFocusoffsetsQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFilterwheelNamesPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFilterwheelNamesQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFilterwheelPositionPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFilterwheelPositionQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutFilterwheelPositionPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutFilterwheelPositionRequest {
        /**
        The number of the filter wheel position to select
        */
        #[serde(rename = "Position")]
        position: i32,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFocuserAbsolutePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFocuserAbsoluteQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFocuserIsmovingPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFocuserIsmovingQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFocuserMaxincrementPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFocuserMaxincrementQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFocuserMaxstepPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFocuserMaxstepQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFocuserPositionPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFocuserPositionQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFocuserStepsizePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFocuserStepsizeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFocuserTempcompPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFocuserTempcompQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutFocuserTempcompPath {
        /**
        Zero based device number as set on the server
        */
        #[serde(rename = "device_number")]
        device_number: Option<i32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutFocuserTempcompRequest {
        /**
        Set true to enable the focuser's temperature compensation mode, otherwise false for normal operation.
        */
        #[serde(rename = "TempComp")]
        temp_comp: bool,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "Client")]
        client: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionIDForm")]
        client_transaction_idform: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFocuserTempcompavailablePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFocuserTempcompavailableQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetFocuserTemperaturePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetFocuserTemperatureQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutFocuserHaltPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutFocuserMovePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutFocuserMoveRequest {
        /**
        Step distance or absolute position, depending on the value of the Absolute property
        */
        #[serde(rename = "Position")]
        position: i32,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsAverageperiodPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsAverageperiodQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutObservingconditionsAverageperiodPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutObservingconditionsAverageperiodRequest {
        /**
        Time period (hours) over which to average sensor readings
        */
        #[serde(rename = "AveragePeriod")]
        average_period: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsCloudcoverPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsCloudcoverQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsDewpointPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsDewpointQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsHumidityPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsHumidityQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsPressurePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsPressureQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsRainratePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsRainrateQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsSkybrightnessPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsSkybrightnessQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsSkyqualityPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsSkyqualityQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsSkytemperaturePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsSkytemperatureQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsStarfwhmPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsStarfwhmQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsTemperaturePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsTemperatureQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsWinddirectionPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsWinddirectionQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsWindgustPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsWindgustQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsWindspeedPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsWindspeedQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutObservingconditionsRefreshPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsSensordescriptionPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsSensordescriptionQuery {
        /**
        Name of the sensor whose description is required
        */
        #[serde(rename = "SensorName")]
        sensor_name: Option<String>,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetObservingconditionsTimesincelastupdatePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetObservingconditionsTimesincelastupdateQuery {
        /**
        Name of the sensor whose last update time is required
        */
        #[serde(rename = "SensorName")]
        sensor_name: Option<String>,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetRotatorCanreversePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetRotatorCanreverseQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetRotatorIsmovingPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetRotatorIsmovingQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetRotatorMechanicalpositionPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetRotatorMechanicalpositionQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetRotatorPositionPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetRotatorPositionQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetRotatorReversePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetRotatorReverseQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutRotatorReversePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutRotatorReverseRequest {
        /**
        True if the rotation and angular direction must be reversed to match the optical characteristcs
        */
        #[serde(rename = "Reverse")]
        reverse: bool,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetRotatorStepsizePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetRotatorStepsizeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetRotatorTargetpositionPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetRotatorTargetpositionQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutRotatorHaltPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutRotatorMovePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutRotatorMoveRequest {
        /**
        Relative position to move in degrees from current Position.
        */
        #[serde(rename = "Position")]
        position: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutRotatorMoveabsolutePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutRotatorMoveabsoluteRequest {
        /**
        Absolute position in degrees.
        */
        #[serde(rename = "Position")]
        position: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutRotatorMovemechanicalPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutRotatorMovemechanicalRequest {
        /**
        Absolute position in degrees.
        */
        #[serde(rename = "Position")]
        position: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutRotatorSyncPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutRotatorSyncRequest {
        /**
        Absolute position in degrees.
        */
        #[serde(rename = "Position")]
        position: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSafetymonitorIssafePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSafetymonitorIssafeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSwitchMaxswitchPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSwitchMaxswitchQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSwitchCanwritePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSwitchCanwriteQuery {
        /**
        The device number (0 to MaxSwitch - 1)
        */
        #[serde(rename = "Id")]
        id: Option<i32>,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSwitchGetswitchPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSwitchGetswitchQuery {
        /**
        The device number (0 to MaxSwitch - 1)
        */
        #[serde(rename = "Id")]
        id: Option<i32>,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSwitchGetswitchdescriptionPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSwitchGetswitchdescriptionQuery {
        /**
        The device number (0 to MaxSwitch - 1)
        */
        #[serde(rename = "Id")]
        id: Option<i32>,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSwitchGetswitchnamePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSwitchGetswitchnameQuery {
        /**
        The device number (0 to MaxSwitch - 1)
        */
        #[serde(rename = "Id")]
        id: Option<i32>,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSwitchGetswitchvaluePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSwitchGetswitchvalueQuery {
        /**
        The device number (0 to MaxSwitch - 1)
        */
        #[serde(rename = "Id")]
        id: Option<i32>,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSwitchMinswitchvaluePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSwitchMinswitchvalueQuery {
        /**
        The device number (0 to MaxSwitch - 1)
        */
        #[serde(rename = "Id")]
        id: Option<i32>,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSwitchMaxswitchvaluePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSwitchMaxswitchvalueQuery {
        /**
        The device number (0 to MaxSwitch - 1)
        */
        #[serde(rename = "Id")]
        id: Option<i32>,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutSwitchSetswitchPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutSwitchSetswitchRequest {
        /**
        The device number (0 to MaxSwitch - 1)
        */
        #[serde(rename = "Id")]
        id: i32,

        /**
        The required control state (True or False)
        */
        #[serde(rename = "State")]
        state: bool,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutSwitchSetswitchnamePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutSwitchSetswitchnameRequest {
        /**
        The device number (0 to MaxSwitch - 1)
        */
        #[serde(rename = "Id")]
        id: i32,

        /**
        The name of the device
        */
        #[serde(rename = "Name")]
        name: String,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutSwitchSetswitchvaluePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutSwitchSetswitchvalueRequest {
        /**
        The device number (0 to MaxSwitch - 1)
        */
        #[serde(rename = "Id")]
        id: i32,

        /**
        The value to be set, between MinSwitchValue and MaxSwitchValue
        */
        #[serde(rename = "Value")]
        value: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetSwitchSwitchstepPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetSwitchSwitchstepQuery {
        /**
        The device number (0 to MaxSwitch - 1)
        */
        #[serde(rename = "Id")]
        id: Option<i32>,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeAlignmentmodePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeAlignmentmodeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeAltitudePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeAltitudeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeApertureareaPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeApertureareaQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeAperturediameterPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeAperturediameterQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeAthomePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeAthomeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeAtparkPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeAtparkQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeAzimuthPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeAzimuthQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCanfindhomePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCanfindhomeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCanparkPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCanparkQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCanpulseguidePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCanpulseguideQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCansetdeclinationratePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCansetdeclinationrateQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCansetguideratesPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCansetguideratesQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCansetparkPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCansetparkQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCansetpiersidePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCansetpiersideQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCansetrightascensionratePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCansetrightascensionrateQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCansettrackingPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCansettrackingQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCanslewPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCanslewQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCanslewaltazPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCanslewaltazQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCanslewaltazasyncPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCanslewaltazasyncQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCanslewasyncPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCanslewasyncQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCansyncPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCansyncQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCansyncaltazPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCansyncaltazQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCanunparkPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCanunparkQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeDeclinationPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeDeclinationQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeDeclinationratePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeDeclinationrateQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeDeclinationratePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeDeclinationrateRequest {
        /**
        Declination tracking rate (arcseconds per second)
        */
        #[serde(rename = "DeclinationRate")]
        declination_rate: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeDoesrefractionPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeDoesrefractionQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeDoesrefractionPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeDoesrefractionRequest {
        /**
        Set True to make the telescope or driver applie atmospheric refraction to coordinates.
        */
        #[serde(rename = "DoesRefraction")]
        does_refraction: bool,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeEquatorialsystemPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeEquatorialsystemQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeFocallengthPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeFocallengthQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeGuideratedeclinationPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeGuideratedeclinationQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeGuideratedeclinationPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeGuideratedeclinationRequest {
        /**
        Declination movement rate offset degrees/sec).
        */
        #[serde(rename = "GuideRateDeclination")]
        guide_rate_declination: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeGuideraterightascensionPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeGuideraterightascensionQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeGuideraterightascensionPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeGuideraterightascensionRequest {
        /**
        RightAscension movement rate offset degrees/sec).
        */
        #[serde(rename = "GuideRateRightAscension")]
        guide_rate_right_ascension: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeIspulseguidingPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeIspulseguidingQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeRightascensionPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeRightascensionQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeRightascensionratePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeRightascensionrateQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeRightascensionratePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeRightascensionrateRequest {
        /**
        Right ascension tracking rate (arcseconds per second)
        */
        #[serde(rename = "RightAscensionRate")]
        right_ascension_rate: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeSideofpierPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeSideofpierQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSideofpierPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeSideofpierRequest {
        /**
        New pointing state.
        */
        #[serde(rename = "SideOfPier")]
        side_of_pier: i32,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeSiderealtimePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeSiderealtimeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeSiteelevationPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeSiteelevationQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSiteelevationPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeSiteelevationRequest {
        /**
        Elevation above mean sea level (metres).
        */
        #[serde(rename = "SiteElevation")]
        site_elevation: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeSitelatitudePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeSitelatitudeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSitelatitudePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeSitelatitudeRequest {
        /**
        Site latitude (degrees)
        */
        #[serde(rename = "SiteLatitude")]
        site_latitude: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeSitelongitudePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeSitelongitudeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSitelongitudePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeSitelongitudeRequest {
        /**
        Site longitude (degrees, positive East, WGS84)
        */
        #[serde(rename = "SiteLongitude")]
        site_longitude: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeSlewingPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeSlewingQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeSlewsettletimePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeSlewsettletimeQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSlewsettletimePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeSlewsettletimeRequest {
        /**
        Settling time (integer sec.).
        */
        #[serde(rename = "SlewSettleTime")]
        slew_settle_time: i32,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeTargetdeclinationPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeTargetdeclinationQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeTargetdeclinationPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeTargetdeclinationRequest {
        /**
        Target declination(degrees)
        */
        #[serde(rename = "TargetDeclination")]
        target_declination: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeTargetrightascensionPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeTargetrightascensionQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeTargetrightascensionPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeTargetrightascensionRequest {
        /**
        Target right ascension(hours)
        */
        #[serde(rename = "TargetRightAscension")]
        target_right_ascension: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeTrackingPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeTrackingQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeTrackingPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeTrackingRequest {
        /**
        Tracking enabled / disabled
        */
        #[serde(rename = "Tracking")]
        tracking: bool,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeTrackingratePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeTrackingrateQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeTrackingratePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeTrackingrateRequest {
        /**
        New tracking rate
        */
        #[serde(rename = "TrackingRate")]
        tracking_rate: i32,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeTrackingratesPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeTrackingratesQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeUtcdatePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeUtcdateQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeUtcdatePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeUtcdateRequest {
        /**
        UTC date/time in ISO 8601 format.
        */
        #[serde(rename = "UTCDate")]
        utcdate: String,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeAbortslewPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeAxisratesPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeAxisratesQuery {
        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,

        /**
        The axis about which rate information is desired. 0 = axisPrimary, 1 = axisSecondary, 2 = axisTertiary.
        */
        #[serde(rename = "Axis")]
        axis: Option<i32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeCanmoveaxisPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeCanmoveaxisQuery {
        /**
        The axis about which rate information is desired. 0 = axisPrimary, 1 = axisSecondary, 2 = axisTertiary.
        */
        #[serde(rename = "Axis")]
        axis: Option<i32>,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct GetTelescopeDestinationsideofpierPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Query))]

    struct GetTelescopeDestinationsideofpierQuery {
        /**
        Right Ascension coordinate (0.0 to 23.99999999 hours)
        */
        #[serde(rename = "RightAscension")]
        right_ascension: Option<f64>,

        /**
        Declination coordinate (-90.0 to +90.0 degrees)
        */
        #[serde(rename = "Declination")]
        declination: Option<f64>,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeFindhomePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeMoveaxisPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeMoveaxisRequest {
        /**
        The axis about which rate information is desired. 0 = axisPrimary, 1 = axisSecondary, 2 = axisTertiary.
        */
        #[serde(rename = "Axis")]
        axis: i32,

        /**
        The rate of motion (deg/sec) about the specified axis
        */
        #[serde(rename = "Rate")]
        rate: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeParkPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopePulseguidePath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopePulseguideRequest {
        /**
        The direction in which the guide-rate motion is to be made. 0 = guideNorth, 1 = guideSouth, 2 = guideEast, 3 = guideWest
        */
        #[serde(rename = "Direction")]
        direction: i32,

        /**
        The duration of the guide-rate motion (milliseconds)
        */
        #[serde(rename = "Duration")]
        duration: i32,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSetparkPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSlewtoaltazPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeSlewtoaltazRequest {
        /**
        Azimuth coordinate (degrees, North-referenced, positive East/clockwise)
        */
        #[serde(rename = "Azimuth")]
        azimuth: f64,

        /**
        Altitude coordinate (degrees, positive up)
        */
        #[serde(rename = "Altitude")]
        altitude: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSlewtoaltazasyncPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSlewtocoordinatesPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Form))]

    struct PutTelescopeSlewtocoordinatesRequest {
        /**
        Right Ascension coordinate (hours)
        */
        #[serde(rename = "RightAscension")]
        right_ascension: f64,

        /**
        Declination coordinate (degrees)
        */
        #[serde(rename = "Declination")]
        declination: f64,

        /**
        Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
        */
        #[serde(rename = "ClientID")]
        client_id: Option<u32>,

        /**
        Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSlewtocoordinatesasyncPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSlewtotargetPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSlewtotargetasyncPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSynctoaltazPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSynctocoordinatesPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeSynctotargetPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
        #[serde(rename = "device_number")]
        device_number: Option<u32>,
    }

    #[derive(Deserialize, FromRequest)]
    #[from_request(via(Path))]

    struct PutTelescopeUnparkPath {
        /**
        Zero based device number as set on the server (0 to 4294967295)
        */
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

    schemas::PutActionRequest {
        action,

        parameters,

        client_id,

        client_transaction_id,
    }: schemas::PutActionRequest,
) -> schemas::StringResponse {
}

/**
Transmits an arbitrary string to the device

Transmits an arbitrary string to the device and does not wait for a response. Optionally, protocol framing characters may be added to the string before transmission.
*/
#[put("/<device_type>/<device_number>/commandblind")]
fn put_commandblind(
    schemas::PutCommandblindPath { device_type, device_number }: schemas::PutCommandblindPath,

    schemas::PutCommandblindRequest {
        command,

        raw,

        client_id,

        client_transaction_id,
    }: schemas::PutCommandblindRequest,
) -> schemas::MethodResponse {
}

/**
Transmits an arbitrary string to the device and returns a boolean value from the device.

Transmits an arbitrary string to the device and waits for a boolean response. Optionally, protocol framing characters may be added to the string before transmission.
*/
#[put("/<device_type>/<device_number>/commandbool")]
fn put_commandbool(
    schemas::PutCommandboolPath { device_type, device_number }: schemas::PutCommandboolPath,

    schemas::PutCommandblindRequest {
        command,

        raw,

        client_id,

        client_transaction_id,
    }: schemas::PutCommandblindRequest,
) -> schemas::BoolResponse {
}

/**
Transmits an arbitrary string to the device and returns a string value from the device.

Transmits an arbitrary string to the device and waits for a string response. Optionally, protocol framing characters may be added to the string before transmission.
*/
#[put("/<device_type>/<device_number>/commandstring")]
fn put_commandstring(
    schemas::PutCommandstringPath { device_type, device_number }: schemas::PutCommandstringPath,

    schemas::PutCommandblindRequest {
        command,

        raw,

        client_id,

        client_transaction_id,
    }: schemas::PutCommandblindRequest,
) -> schemas::StringResponse {
}

/**
Retrieves the connected state of the device

Retrieves the connected state of the device
*/
#[get("/<device_type>/<device_number>/connected")]
fn get_connected(
    schemas::GetConnectedPath { device_type, device_number }: schemas::GetConnectedPath,

    schemas::GetConnectedQuery { client_id, client_transaction_id }: schemas::GetConnectedQuery,
) -> schemas::BoolResponse {
}

/**
Sets the connected state of the device

Sets the connected state of the device
*/
#[put("/<device_type>/<device_number>/connected")]
fn put_connected(
    schemas::PutConnectedPath { device_type, device_number }: schemas::PutConnectedPath,

    schemas::PutConnectedRequest {
        connected,

        client_id,

        client_transaction_id,
    }: schemas::PutConnectedRequest,
) -> schemas::MethodResponse {
}

/**
Device description

The description of the device
*/
#[get("/<device_type>/<device_number>/description")]
fn get_description(
    schemas::GetDescriptionPath { device_type, device_number }: schemas::GetDescriptionPath,

    schemas::GetDescriptionQuery { client_id, client_transaction_id }: schemas::GetDescriptionQuery,
) -> schemas::StringResponse {
}

/**
Device driver description

The description of the driver
*/
#[get("/<device_type>/<device_number>/driverinfo")]
fn get_driverinfo(
    schemas::GetDriverinfoPath { device_type, device_number }: schemas::GetDriverinfoPath,

    schemas::GetDriverinfoQuery { client_id, client_transaction_id }: schemas::GetDriverinfoQuery,
) -> schemas::StringResponse {
}

/**
Driver Version

A string containing only the major and minor version of the driver.
*/
#[get("/<device_type>/<device_number>/driverversion")]
fn get_driverversion(
    schemas::GetDriverversionPath { device_type, device_number }: schemas::GetDriverversionPath,

    schemas::GetDriverversionQuery { client_id, client_transaction_id }: schemas::GetDriverversionQuery,
) -> schemas::StringResponse {
}

/**
The ASCOM Device interface version number that this device supports.

This method returns the version of the ASCOM device interface contract to which this device complies. Only one interface version is current at a moment in time and all new devices should be built to the latest interface version. Applications can choose which device interface versions they support and it is in their interest to support  previous versions as well as the current version to ensure thay can use the largest number of devices.
*/
#[get("/<device_type>/<device_number>/interfaceversion")]
fn get_interfaceversion(
    schemas::GetInterfaceversionPath { device_type, device_number }: schemas::GetInterfaceversionPath,

    schemas::GetInterfaceversionQuery { client_id, client_transaction_id }: schemas::GetInterfaceversionQuery,
) -> schemas::IntResponse {
}

/**
Device name

The name of the device
*/
#[get("/<device_type>/<device_number>/name")]
fn get_name(schemas::GetNamePath { device_type, device_number }: schemas::GetNamePath, schemas::GetNameQuery { client_id, client_transaction_id }: schemas::GetNameQuery) -> schemas::StringResponse {}

/**
Returns the list of action names supported by this driver.

Returns the list of action names supported by this driver.
*/
#[get("/<device_type>/<device_number>/supportedactions")]
fn get_supportedactions(
    schemas::GetSupportedactionsPath { device_type, device_number }: schemas::GetSupportedactionsPath,

    schemas::GetSupportedactionsQuery { client_id, client_transaction_id }: schemas::GetSupportedactionsQuery,
) -> schemas::StringArrayResponse {
}

/**
Returns the X offset of the Bayer matrix.

Returns the X offset of the Bayer matrix, as defined in SensorType.
*/
#[get("/camera/<device_number>/bayeroffsetx")]
fn get_camera_bayeroffsetx(
    schemas::GetCameraBayeroffsetxPath { device_number }: schemas::GetCameraBayeroffsetxPath,

    schemas::GetCameraBayeroffsetxQuery { client_id, client_transaction_id }: schemas::GetCameraBayeroffsetxQuery,
) -> schemas::IntResponse {
}

/**
Returns the Y offset of the Bayer matrix.

Returns the Y offset of the Bayer matrix, as defined in SensorType.
*/
#[get("/camera/<device_number>/bayeroffsety")]
fn get_camera_bayeroffsety(
    schemas::GetCameraBayeroffsetyPath { device_number }: schemas::GetCameraBayeroffsetyPath,

    schemas::GetCameraBayeroffsetyQuery { client_id, client_transaction_id }: schemas::GetCameraBayeroffsetyQuery,
) -> schemas::IntResponse {
}

/**
Returns the binning factor for the X axis.

Returns the binning factor for the X axis.
*/
#[get("/camera/<device_number>/binx")]
fn get_camera_binx(
    schemas::GetCameraBinxPath { device_number }: schemas::GetCameraBinxPath,

    schemas::GetCameraBinxQuery { client_id, client_transaction_id }: schemas::GetCameraBinxQuery,
) -> schemas::IntResponse {
}

/**
Sets the binning factor for the X axis.

Sets the binning factor for the X axis.
*/
#[put("/camera/<device_number>/binx")]
fn put_camera_binx(
    schemas::PutCameraBinxPath { device_number }: schemas::PutCameraBinxPath,

    schemas::PutCameraBinxRequest {
        bin_x,

        client_id,

        client_transaction_id,
    }: schemas::PutCameraBinxRequest,
) -> schemas::MethodResponse {
}

/**
Returns the binning factor for the Y axis.

Returns the binning factor for the Y axis.
*/
#[get("/camera/<device_number>/biny")]
fn get_camera_biny(
    schemas::GetCameraBinyPath { device_number }: schemas::GetCameraBinyPath,

    schemas::GetCameraBinyQuery { client_id, client_transaction_id }: schemas::GetCameraBinyQuery,
) -> schemas::IntResponse {
}

/**
Sets the binning factor for the Y axis.

Sets the binning factor for the Y axis.
*/
#[put("/camera/<device_number>/biny")]
fn put_camera_biny(
    schemas::PutCameraBinyPath { device_number }: schemas::PutCameraBinyPath,

    schemas::PutCameraBinyRequest {
        bin_y,

        client_id,

        client_transaction_id,
    }: schemas::PutCameraBinyRequest,
) -> schemas::MethodResponse {
}

/**
Returns the camera operational state.

Returns the current camera operational state as an integer. 0 = CameraIdle , 1 = CameraWaiting , 2 = CameraExposing , 3 = CameraReading , 4 = CameraDownload , 5 = CameraError
*/
#[get("/camera/<device_number>/camerastate")]
fn get_camera_camerastate(
    schemas::GetCameraCamerastatePath { device_number }: schemas::GetCameraCamerastatePath,

    schemas::GetCameraCamerastateQuery { client_id, client_transaction_id }: schemas::GetCameraCamerastateQuery,
) -> schemas::IntResponse {
}

/**
Returns the width of the CCD camera chip.

Returns the width of the CCD camera chip in unbinned pixels.
*/
#[get("/camera/<device_number>/cameraxsize")]
fn get_camera_cameraxsize(
    schemas::GetCameraCameraxsizePath { device_number }: schemas::GetCameraCameraxsizePath,

    schemas::GetCameraCameraxsizeQuery { client_id, client_transaction_id }: schemas::GetCameraCameraxsizeQuery,
) -> schemas::IntResponse {
}

/**
Returns the height of the CCD camera chip.

Returns the height of the CCD camera chip in unbinned pixels.
*/
#[get("/camera/<device_number>/cameraysize")]
fn get_camera_cameraysize(
    schemas::GetCameraCameraysizePath { device_number }: schemas::GetCameraCameraysizePath,

    schemas::GetCameraCameraysizeQuery { client_id, client_transaction_id }: schemas::GetCameraCameraysizeQuery,
) -> schemas::IntResponse {
}

/**
Indicates whether the camera can abort exposures.

Returns true if the camera can abort exposures; false if not.
*/
#[get("/camera/<device_number>/canabortexposure")]
fn get_camera_canabortexposure(
    schemas::GetCameraCanabortexposurePath { device_number }: schemas::GetCameraCanabortexposurePath,

    schemas::GetCameraCanabortexposureQuery { client_id, client_transaction_id }: schemas::GetCameraCanabortexposureQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the camera supports asymmetric binning

Returns a flag showing whether this camera supports asymmetric binning
*/
#[get("/camera/<device_number>/canasymmetricbin")]
fn get_camera_canasymmetricbin(
    schemas::GetCameraCanasymmetricbinPath { device_number }: schemas::GetCameraCanasymmetricbinPath,

    schemas::GetCameraCanasymmetricbinQuery { client_id, client_transaction_id }: schemas::GetCameraCanasymmetricbinQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the camera has a fast readout mode.

Indicates whether the camera has a fast readout mode.
*/
#[get("/camera/<device_number>/canfastreadout")]
fn get_camera_canfastreadout(
    schemas::GetCameraCanfastreadoutPath { device_number }: schemas::GetCameraCanfastreadoutPath,

    schemas::GetCameraCanfastreadoutQuery { client_id, client_transaction_id }: schemas::GetCameraCanfastreadoutQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the camera's cooler power setting can be read.

If true, the camera's cooler power setting can be read.
*/
#[get("/camera/<device_number>/cangetcoolerpower")]
fn get_camera_cangetcoolerpower(
    schemas::GetCameraCangetcoolerpowerPath { device_number }: schemas::GetCameraCangetcoolerpowerPath,

    schemas::GetCameraCangetcoolerpowerQuery { client_id, client_transaction_id }: schemas::GetCameraCangetcoolerpowerQuery,
) -> schemas::BoolResponse {
}

/**
Returns a flag indicating whether this camera supports pulse guiding

Returns a flag indicating whether this camera supports pulse guiding.
*/
#[get("/camera/<device_number>/canpulseguide")]
fn get_camera_canpulseguide(
    schemas::GetCameraCanpulseguidePath { device_number }: schemas::GetCameraCanpulseguidePath,

    schemas::GetCameraCanpulseguideQuery { client_id, client_transaction_id }: schemas::GetCameraCanpulseguideQuery,
) -> schemas::BoolResponse {
}

/**
Returns a flag indicating whether this camera supports setting the CCD temperature

Returns a flag indicatig whether this camera supports setting the CCD temperature
*/
#[get("/camera/<device_number>/cansetccdtemperature")]
fn get_camera_cansetccdtemperature(
    schemas::GetCameraCansetccdtemperaturePath { device_number }: schemas::GetCameraCansetccdtemperaturePath,

    schemas::GetCameraCansetccdtemperatureQuery { client_id, client_transaction_id }: schemas::GetCameraCansetccdtemperatureQuery,
) -> schemas::BoolResponse {
}

/**
Returns a flag indicating whether this camera can stop an exposure that is in progress

Returns a flag indicating whether this camera can stop an exposure that is in progress
*/
#[get("/camera/<device_number>/canstopexposure")]
fn get_camera_canstopexposure(
    schemas::GetCameraCanstopexposurePath { device_number }: schemas::GetCameraCanstopexposurePath,

    schemas::GetCameraCanstopexposureQuery { client_id, client_transaction_id }: schemas::GetCameraCanstopexposureQuery,
) -> schemas::BoolResponse {
}

/**
Returns the current CCD temperature

Returns the current CCD temperature in degrees Celsius.
*/
#[get("/camera/<device_number>/ccdtemperature")]
fn get_camera_ccdtemperature(
    schemas::GetCameraCcdtemperaturePath { device_number }: schemas::GetCameraCcdtemperaturePath,

    schemas::GetCameraCcdtemperatureQuery { client_id, client_transaction_id }: schemas::GetCameraCcdtemperatureQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the current cooler on/off state.

Returns the current cooler on/off state.
*/
#[get("/camera/<device_number>/cooleron")]
fn get_camera_cooleron(
    schemas::GetCameraCooleronPath { device_number }: schemas::GetCameraCooleronPath,

    schemas::GetCameraCooleronQuery { client_id, client_transaction_id }: schemas::GetCameraCooleronQuery,
) -> schemas::BoolResponse {
}

/**
Turns the camera cooler on and off

Turns on and off the camera cooler. True = cooler on, False = cooler off
*/
#[put("/camera/<device_number>/cooleron")]
fn put_camera_cooleron(
    schemas::PutCameraCooleronPath { device_number }: schemas::PutCameraCooleronPath,

    schemas::PutCameraCooleronRequest {
        cooler_on,

        client_id,

        client_transaction_id,
    }: schemas::PutCameraCooleronRequest,
) -> schemas::MethodResponse {
}

/**
Returns the present cooler power level

Returns the present cooler power level, in percent.
*/
#[get("/camera/<device_number>/coolerpower")]
fn get_camera_coolerpower(
    schemas::GetCameraCoolerpowerPath { device_number }: schemas::GetCameraCoolerpowerPath,

    schemas::GetCameraCoolerpowerQuery { client_id, client_transaction_id }: schemas::GetCameraCoolerpowerQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the gain of the camera

Returns the gain of the camera in photoelectrons per A/D unit.
*/
#[get("/camera/<device_number>/electronsperadu")]
fn get_camera_electronsperadu(
    schemas::GetCameraElectronsperaduPath { device_number }: schemas::GetCameraElectronsperaduPath,

    schemas::GetCameraElectronsperaduQuery { client_id, client_transaction_id }: schemas::GetCameraElectronsperaduQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the maximum exposure time supported by StartExposure.

Returns the maximum exposure time supported by StartExposure.
*/
#[get("/camera/<device_number>/exposuremax")]
fn get_camera_exposuremax(
    schemas::GetCameraExposuremaxPath { device_number }: schemas::GetCameraExposuremaxPath,

    schemas::GetCameraExposuremaxQuery { client_id, client_transaction_id }: schemas::GetCameraExposuremaxQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the Minimium exposure time

Returns the Minimium exposure time in seconds that the camera supports through StartExposure.
*/
#[get("/camera/<device_number>/exposuremin")]
fn get_camera_exposuremin(
    schemas::GetCameraExposureminPath { device_number }: schemas::GetCameraExposureminPath,

    schemas::GetCameraExposureminQuery { client_id, client_transaction_id }: schemas::GetCameraExposureminQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the smallest increment in exposure time supported by StartExposure.

Returns the smallest increment in exposure time supported by StartExposure.
*/
#[get("/camera/<device_number>/exposureresolution")]
fn get_camera_exposureresolution(
    schemas::GetCameraExposureresolutionPath { device_number }: schemas::GetCameraExposureresolutionPath,

    schemas::GetCameraExposureresolutionQuery { client_id, client_transaction_id }: schemas::GetCameraExposureresolutionQuery,
) -> schemas::DoubleResponse {
}

/**
Returns whenther Fast Readout Mode is enabled.

Returns whenther Fast Readout Mode is enabled.
*/
#[get("/camera/<device_number>/fastreadout")]
fn get_camera_fastreadout(
    schemas::GetCameraFastreadoutPath { device_number }: schemas::GetCameraFastreadoutPath,

    schemas::GetCameraFastreadoutQuery { client_id, client_transaction_id }: schemas::GetCameraFastreadoutQuery,
) -> schemas::BoolResponse {
}

/**
Sets whether Fast Readout Mode is enabled.

Sets whether Fast Readout Mode is enabled.
*/
#[put("/camera/<device_number>/fastreadout")]
fn put_camera_fastreadout(
    schemas::PutCameraFastreadoutPath { device_number }: schemas::PutCameraFastreadoutPath,

    schemas::PutCameraFastreadoutRequest {
        fast_readout,

        client_id,

        client_transaction_id,
    }: schemas::PutCameraFastreadoutRequest,
) -> schemas::MethodResponse {
}

/**
Reports the full well capacity of the camera

Reports the full well capacity of the camera in electrons, at the current camera settings (binning, SetupDialog settings, etc.).
*/
#[get("/camera/<device_number>/fullwellcapacity")]
fn get_camera_fullwellcapacity(
    schemas::GetCameraFullwellcapacityPath { device_number }: schemas::GetCameraFullwellcapacityPath,

    schemas::GetCameraFullwellcapacityQuery { client_id, client_transaction_id }: schemas::GetCameraFullwellcapacityQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the camera's gain

The camera's gain (GAIN VALUE MODE) OR the index of the selected camera gain description in the Gains array (GAINS INDEX MODE).
*/
#[get("/camera/<device_number>/gain")]
fn get_camera_gain(
    schemas::GetCameraGainPath { device_number }: schemas::GetCameraGainPath,

    schemas::GetCameraGainQuery { client_id, client_transaction_id }: schemas::GetCameraGainQuery,
) -> schemas::IntResponse {
}

/**
Sets the camera's gain.

The camera's gain (GAIN VALUE MODE) OR the index of the selected camera gain description in the Gains array (GAINS INDEX MODE).
*/
#[put("/camera/<device_number>/gain")]
fn put_camera_gain(
    schemas::PutCameraGainPath { device_number }: schemas::PutCameraGainPath,

    schemas::PutCameraGainRequest {
        gain,

        client_id,

        client_transaction_id,
    }: schemas::PutCameraGainRequest,
) -> schemas::MethodResponse {
}

/**
Maximum Gain value of that this camera supports

Returns the maximum value of Gain.
*/
#[get("/camera/<device_number>/gainmax")]
fn get_camera_gainmax(
    schemas::GetCameraGainmaxPath { device_number }: schemas::GetCameraGainmaxPath,

    schemas::GetCameraGainmaxQuery { client_id, client_transaction_id }: schemas::GetCameraGainmaxQuery,
) -> schemas::IntResponse {
}

/**
Minimum Gain value of that this camera supports

Returns the Minimum value of Gain.
*/
#[get("/camera/<device_number>/gainmin")]
fn get_camera_gainmin(
    schemas::GetCameraGainminPath { device_number }: schemas::GetCameraGainminPath,

    schemas::GetCameraGainminQuery { client_id, client_transaction_id }: schemas::GetCameraGainminQuery,
) -> schemas::IntResponse {
}

/**
List of Gain names supported by the camera

Returns the Gains supported by the camera.
*/
#[get("/camera/<device_number>/gains")]
fn get_camera_gains(
    schemas::GetCameraGainsPath { device_number }: schemas::GetCameraGainsPath,

    schemas::GetCameraGainsQuery { client_id, client_transaction_id }: schemas::GetCameraGainsQuery,
) -> schemas::StringArrayResponse {
}

/**
Indicates whether the camera has a mechanical shutter

Returns a flag indicating whether this camera has a mechanical shutter.
*/
#[get("/camera/<device_number>/hasshutter")]
fn get_camera_hasshutter(
    schemas::GetCameraHasshutterPath { device_number }: schemas::GetCameraHasshutterPath,

    schemas::GetCameraHasshutterQuery { client_id, client_transaction_id }: schemas::GetCameraHasshutterQuery,
) -> schemas::BoolResponse {
}

/**
Returns the current heat sink temperature.

Returns the current heat sink temperature (called "ambient temperature" by some manufacturers) in degrees Celsius.
*/
#[get("/camera/<device_number>/heatsinktemperature")]
fn get_camera_heatsinktemperature(
    schemas::GetCameraHeatsinktemperaturePath { device_number }: schemas::GetCameraHeatsinktemperaturePath,

    schemas::GetCameraHeatsinktemperatureQuery { client_id, client_transaction_id }: schemas::GetCameraHeatsinktemperatureQuery,
) -> schemas::DoubleResponse {
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

    schemas::GetCameraImagearrayQuery { client_id, client_transaction_id }: schemas::GetCameraImagearrayQuery,
) -> schemas::ImageArrayResponse {
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

    schemas::GetCameraImagearrayvariantQuery { client_id, client_transaction_id }: schemas::GetCameraImagearrayvariantQuery,
) -> schemas::ImageArrayResponse {
}

/**
Indicates that an image is ready to be downloaded

Returns a flag indicating whether the image is ready to be downloaded from the camera.
*/
#[get("/camera/<device_number>/imageready")]
fn get_camera_imageready(
    schemas::GetCameraImagereadyPath { device_number }: schemas::GetCameraImagereadyPath,

    schemas::GetCameraImagereadyQuery { client_id, client_transaction_id }: schemas::GetCameraImagereadyQuery,
) -> schemas::BoolResponse {
}

/**
Indicates that the camera is pulse guideing.

Returns a flag indicating whether the camera is currrently in a PulseGuide operation.
*/
#[get("/camera/<device_number>/ispulseguiding")]
fn get_camera_ispulseguiding(
    schemas::GetCameraIspulseguidingPath { device_number }: schemas::GetCameraIspulseguidingPath,

    schemas::GetCameraIspulseguidingQuery { client_id, client_transaction_id }: schemas::GetCameraIspulseguidingQuery,
) -> schemas::BoolResponse {
}

/**
Duration of the last exposure

Reports the actual exposure duration in seconds (i.e. shutter open time).
*/
#[get("/camera/<device_number>/lastexposureduration")]
fn get_camera_lastexposureduration(
    schemas::GetCameraLastexposuredurationPath { device_number }: schemas::GetCameraLastexposuredurationPath,

    schemas::GetCameraLastexposuredurationQuery { client_id, client_transaction_id }: schemas::GetCameraLastexposuredurationQuery,
) -> schemas::DoubleResponse {
}

/**
Start time of the last exposure in FITS standard format.

Reports the actual exposure start in the FITS-standard CCYY-MM-DDThh:mm:ss[.sss...] format.
*/
#[get("/camera/<device_number>/lastexposurestarttime")]
fn get_camera_lastexposurestarttime(
    schemas::GetCameraLastexposurestarttimePath { device_number }: schemas::GetCameraLastexposurestarttimePath,

    schemas::GetCameraLastexposurestarttimeQuery { client_id, client_transaction_id }: schemas::GetCameraLastexposurestarttimeQuery,
) -> schemas::StringResponse {
}

/**
Camera's maximum ADU value

Reports the maximum ADU value the camera can produce.
*/
#[get("/camera/<device_number>/maxadu")]
fn get_camera_maxadu(
    schemas::GetCameraMaxaduPath { device_number }: schemas::GetCameraMaxaduPath,

    schemas::GetCameraMaxaduQuery { client_id, client_transaction_id }: schemas::GetCameraMaxaduQuery,
) -> schemas::IntResponse {
}

/**
Maximum  binning for the camera X axis

Returns the maximum allowed binning for the X camera axis
*/
#[get("/camera/<device_number>/maxbinx")]
fn get_camera_maxbinx(
    schemas::GetCameraMaxbinxPath { device_number }: schemas::GetCameraMaxbinxPath,

    schemas::GetCameraMaxbinxQuery { client_id, client_transaction_id }: schemas::GetCameraMaxbinxQuery,
) -> schemas::IntResponse {
}

/**
Maximum  binning for the camera Y axis

Returns the maximum allowed binning for the Y camera axis
*/
#[get("/camera/<device_number>/maxbiny")]
fn get_camera_maxbiny(
    schemas::GetCameraMaxbinyPath { device_number }: schemas::GetCameraMaxbinyPath,

    schemas::GetCameraMaxbinyQuery { client_id, client_transaction_id }: schemas::GetCameraMaxbinyQuery,
) -> schemas::IntResponse {
}

/**
Returns the current subframe width

Returns the current subframe width, if binning is active, value is in binned pixels.
*/
#[get("/camera/<device_number>/numx")]
fn get_camera_numx(
    schemas::GetCameraNumxPath { device_number }: schemas::GetCameraNumxPath,

    schemas::GetCameraNumxQuery { client_id, client_transaction_id }: schemas::GetCameraNumxQuery,
) -> schemas::IntResponse {
}

/**
Sets the current subframe width

Sets the current subframe width.
*/
#[put("/camera/<device_number>/numx")]
fn put_camera_numx(
    schemas::PutCameraNumxPath { device_number }: schemas::PutCameraNumxPath,

    schemas::PutCameraNumxRequest {
        num_x,

        client_id,

        client_transaction_id,
    }: schemas::PutCameraNumxRequest,
) -> schemas::MethodResponse {
}

/**
Returns the current subframe height

Returns the current subframe height, if binning is active, value is in binned pixels.
*/
#[get("/camera/<device_number>/numy")]
fn get_camera_numy(
    schemas::GetCameraNumyPath { device_number }: schemas::GetCameraNumyPath,

    schemas::GetCameraNumyQuery { client_id, client_transaction_id }: schemas::GetCameraNumyQuery,
) -> schemas::IntResponse {
}

/**
Sets the current subframe height

Sets the current subframe height.
*/
#[put("/camera/<device_number>/numy")]
fn put_camera_numy(
    schemas::PutCameraNumyPath { device_number }: schemas::PutCameraNumyPath,

    schemas::PutCameraNumyRequest {
        num_y,

        client_id,

        client_transaction_id,
    }: schemas::PutCameraNumyRequest,
) -> schemas::MethodResponse {
}

/**
Returns the camera's offset

Returns the camera's offset (OFFSET VALUE MODE) OR the index of the selected camera offset description in the offsets array (OFFSETS INDEX MODE).
*/
#[get("/camera/<device_number>/offset")]
fn get_camera_offset(
    schemas::GetCameraOffsetPath { device_number }: schemas::GetCameraOffsetPath,

    schemas::GetCameraOffsetQuery { client_id, client_transaction_id }: schemas::GetCameraOffsetQuery,
) -> schemas::IntResponse {
}

/**
Sets the camera's offset.

Sets the camera's offset (OFFSET VALUE MODE) OR the index of the selected camera offset description in the offsets array (OFFSETS INDEX MODE).
*/
#[put("/camera/<device_number>/offset")]
fn put_camera_offset(
    schemas::PutCameraOffsetPath { device_number }: schemas::PutCameraOffsetPath,

    schemas::PutCameraOffsetRequest {
        offset,

        client_id,

        client_transaction_id,
    }: schemas::PutCameraOffsetRequest,
) -> schemas::MethodResponse {
}

/**
Maximum offset value of that this camera supports

Returns the maximum value of offset.
*/
#[get("/camera/<device_number>/offsetmax")]
fn get_camera_offsetmax(
    schemas::GetCameraOffsetmaxPath { device_number }: schemas::GetCameraOffsetmaxPath,

    schemas::GetCameraOffsetmaxQuery { client_id, client_transaction_id }: schemas::GetCameraOffsetmaxQuery,
) -> schemas::IntResponse {
}

/**
Minimum offset value of that this camera supports

Returns the Minimum value of offset.
*/
#[get("/camera/<device_number>/offsetmin")]
fn get_camera_offsetmin(
    schemas::GetCameraOffsetminPath { device_number }: schemas::GetCameraOffsetminPath,

    schemas::GetCameraOffsetminQuery { client_id, client_transaction_id }: schemas::GetCameraOffsetminQuery,
) -> schemas::IntResponse {
}

/**
List of offset names supported by the camera

Returns the offsets supported by the camera.
*/
#[get("/camera/<device_number>/offsets")]
fn get_camera_offsets(
    schemas::GetCameraOffsetsPath { device_number }: schemas::GetCameraOffsetsPath,

    schemas::GetCameraOffsetsQuery { client_id, client_transaction_id }: schemas::GetCameraOffsetsQuery,
) -> schemas::StringArrayResponse {
}

/**
Indicates percentage completeness of the current operation

Returns the percentage of the current operation that is complete. If valid, returns an integer between 0 and 100, where 0 indicates 0% progress (function just started) and 100 indicates 100% progress (i.e. completion).
*/
#[get("/camera/<device_number>/percentcompleted")]
fn get_camera_percentcompleted(
    schemas::GetCameraPercentcompletedPath { device_number }: schemas::GetCameraPercentcompletedPath,

    schemas::GetCameraPercentcompletedQuery { client_id, client_transaction_id }: schemas::GetCameraPercentcompletedQuery,
) -> schemas::IntResponse {
}

/**
Width of CCD chip pixels (microns)

Returns the width of the CCD chip pixels in microns.
*/
#[get("/camera/<device_number>/pixelsizex")]
fn get_camera_pixelsizex(
    schemas::GetCameraPixelsizexPath { device_number }: schemas::GetCameraPixelsizexPath,

    schemas::GetCameraPixelsizexQuery { client_id, client_transaction_id }: schemas::GetCameraPixelsizexQuery,
) -> schemas::DoubleResponse {
}

/**
Height of CCD chip pixels (microns)

Returns the Height of the CCD chip pixels in microns.
*/
#[get("/camera/<device_number>/pixelsizey")]
fn get_camera_pixelsizey(
    schemas::GetCameraPixelsizeyPath { device_number }: schemas::GetCameraPixelsizeyPath,

    schemas::GetCameraPixelsizeyQuery { client_id, client_transaction_id }: schemas::GetCameraPixelsizeyQuery,
) -> schemas::DoubleResponse {
}

/**
Indicates the canera's readout mode as an index into the array ReadoutModes

ReadoutMode is an index into the array ReadoutModes and returns the desired readout mode for the camera. Defaults to 0 if not set.
*/
#[get("/camera/<device_number>/readoutmode")]
fn get_camera_readoutmode(
    schemas::GetCameraReadoutmodePath { device_number }: schemas::GetCameraReadoutmodePath,

    schemas::GetCameraReadoutmodeQuery { client_id, client_transaction_id }: schemas::GetCameraReadoutmodeQuery,
) -> schemas::IntResponse {
}

/**
Set the camera's readout mode

Sets the ReadoutMode as an index into the array ReadoutModes.
*/
#[put("/camera/<device_number>/readoutmode")]
fn put_camera_readoutmode(
    schemas::PutCameraReadoutmodePath { device_number }: schemas::PutCameraReadoutmodePath,

    schemas::PutCameraReadoutmodeRequest {
        readout_mode,

        client_id,

        client_transaction_id,
    }: schemas::PutCameraReadoutmodeRequest,
) -> schemas::MethodResponse {
}

/**
List of available readout modes

This property provides an array of strings, each of which describes an available readout mode of the camera. At least one string must be present in the list.
*/
#[get("/camera/<device_number>/readoutmodes")]
fn get_camera_readoutmodes(
    schemas::GetCameraReadoutmodesPath { device_number }: schemas::GetCameraReadoutmodesPath,

    schemas::GetCameraReadoutmodesQuery { client_id, client_transaction_id }: schemas::GetCameraReadoutmodesQuery,
) -> schemas::StringArrayResponse {
}

/**
Sensor name

The name of the sensor used within the camera.
*/
#[get("/camera/<device_number>/sensorname")]
fn get_camera_sensorname(
    schemas::GetCameraSensornamePath { device_number }: schemas::GetCameraSensornamePath,

    schemas::GetCameraSensornameQuery { client_id, client_transaction_id }: schemas::GetCameraSensornameQuery,
) -> schemas::StringResponse {
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

    schemas::GetCameraSensortypeQuery { client_id, client_transaction_id }: schemas::GetCameraSensortypeQuery,
) -> schemas::IntResponse {
}

/**
Returns the current camera cooler setpoint in degrees Celsius.

Returns the current camera cooler setpoint in degrees Celsius.
*/
#[get("/camera/<device_number>/setccdtemperature")]
fn get_camera_setccdtemperature(
    schemas::GetCameraSetccdtemperaturePath { device_number }: schemas::GetCameraSetccdtemperaturePath,

    schemas::GetCameraSetccdtemperatureQuery { client_id, client_transaction_id }: schemas::GetCameraSetccdtemperatureQuery,
) -> schemas::DoubleResponse {
}

/**
Set the camera's cooler setpoint (degrees Celsius).

Set's the camera's cooler setpoint in degrees Celsius.
*/
#[put("/camera/<device_number>/setccdtemperature")]
fn put_camera_setccdtemperature(
    schemas::PutCameraSetccdtemperaturePath { device_number }: schemas::PutCameraSetccdtemperaturePath,

    schemas::PutCameraSetccdtemperatureRequest {
        set_ccdtemperature,

        client_id,

        client_transaction_id,
    }: schemas::PutCameraSetccdtemperatureRequest,
) -> schemas::MethodResponse {
}

/**
Return the current subframe X axis start position

Sets the subframe start position for the X axis (0 based) and returns the current value. If binning is active, value is in binned pixels.
*/
#[get("/camera/<device_number>/startx")]
fn get_camera_startx(
    schemas::GetCameraStartxPath { device_number }: schemas::GetCameraStartxPath,

    schemas::GetCameraStartxQuery { client_id, client_transaction_id }: schemas::GetCameraStartxQuery,
) -> schemas::IntResponse {
}

/**
Sets the current subframe X axis start position

Sets the current subframe X axis start position in binned pixels.
*/
#[put("/camera/<device_number>/startx")]
fn put_camera_startx(
    schemas::PutCameraStartxPath { device_number }: schemas::PutCameraStartxPath,

    schemas::PutCameraStartxRequest {
        start_x,

        client_id,

        client_transaction_id,
    }: schemas::PutCameraStartxRequest,
) -> schemas::MethodResponse {
}

/**
Return the current subframe Y axis start position

Sets the subframe start position for the Y axis (0 based) and returns the current value. If binning is active, value is in binned pixels.
*/
#[get("/camera/<device_number>/starty")]
fn get_camera_starty(
    schemas::GetCameraStartyPath { device_number }: schemas::GetCameraStartyPath,

    schemas::GetCameraStartyQuery { client_id, client_transaction_id }: schemas::GetCameraStartyQuery,
) -> schemas::IntResponse {
}

/**
Sets the current subframe Y axis start position

Sets the current subframe Y axis start position in binned pixels.
*/
#[put("/camera/<device_number>/starty")]
fn put_camera_starty(
    schemas::PutCameraStartyPath { device_number }: schemas::PutCameraStartyPath,

    schemas::PutCameraStartyRequest {
        start_y,

        client_id,

        client_transaction_id,
    }: schemas::PutCameraStartyRequest,
) -> schemas::MethodResponse {
}

/**
Camera's sub-exposure interval

The Camera's sub exposure duration in seconds. Only available in Camera Interface Version 3 and later.
*/
#[get("/camera/<device_number>/subexposureduration")]
fn get_camera_subexposureduration(
    schemas::GetCameraSubexposuredurationPath { device_number }: schemas::GetCameraSubexposuredurationPath,

    schemas::GetCameraSubexposuredurationQuery { client_id, client_transaction_id }: schemas::GetCameraSubexposuredurationQuery,
) -> schemas::DoubleResponse {
}

/**
Sets the current Sub Exposure Duration

Sets image sub exposure duration in seconds. Only available in Camera Interface Version 3 and later.
*/
#[put("/camera/<device_number>/subexposureduration")]
fn put_camera_subexposureduration(
    schemas::PutCameraSubexposuredurationPath { device_number }: schemas::PutCameraSubexposuredurationPath,

    schemas::PutCameraSubexposuredurationRequest {
        sub_exposure_duration,

        client_id,

        client_transaction_id,
    }: schemas::PutCameraSubexposuredurationRequest,
) -> schemas::MethodResponse {
}

/**
Aborts the current exposure

Aborts the current exposure, if any, and returns the camera to Idle state.
*/
#[put("/camera/<device_number>/abortexposure")]
fn put_camera_abortexposure(
    schemas::PutCameraAbortexposurePath { device_number }: schemas::PutCameraAbortexposurePath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Pulse guide in the specified direction for the specified time.

Activates the Camera's mount control sytem to instruct the mount to move in a particular direction for a given period of time
*/
#[put("/camera/<device_number>/pulseguide")]
fn put_camera_pulseguide(
    schemas::PutCameraPulseguidePath { device_number }: schemas::PutCameraPulseguidePath,

    schemas::PutCameraPulseguideRequest {
        direction,

        duration,

        client_id,

        client_transaction_id,
    }: schemas::PutCameraPulseguideRequest,
) -> schemas::MethodResponse {
}

/**
Starts an exposure

Starts an exposure. Use ImageReady to check when the exposure is complete.
*/
#[put("/camera/<device_number>/startexposure")]
fn put_camera_startexposure(
    schemas::PutCameraStartexposurePath { device_number }: schemas::PutCameraStartexposurePath,

    schemas::PutCameraStartexposureRequest {
        duration,

        light,

        client_id,

        client_transaction_id,
    }: schemas::PutCameraStartexposureRequest,
) -> schemas::MethodResponse {
}

/**
Stops the current exposure

Stops the current exposure, if any. If an exposure is in progress, the readout process is initiated. Ignored if readout is already in process.
*/
#[put("/camera/<device_number>/stopexposure")]
fn put_camera_stopexposure(
    schemas::PutCameraStopexposurePath { device_number }: schemas::PutCameraStopexposurePath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Returns the current calibrator brightness

Returns the current calibrator brightness in the range 0 (completely off) to MaxBrightness (fully on)
*/
#[get("/covercalibrator/<device_number>/brightness")]
fn get_covercalibrator_brightness(
    schemas::GetCovercalibratorBrightnessPath { device_number }: schemas::GetCovercalibratorBrightnessPath,

    schemas::GetCovercalibratorBrightnessQuery { client_id, client_transaction_id }: schemas::GetCovercalibratorBrightnessQuery,
) -> schemas::IntResponse {
}

/**
Returns the state of the calibration device

Returns the state of the calibration device, if present, otherwise returns "NotPresent".  The calibrator state mode is specified as an integer value from the CalibratorStatus Enum.
*/
#[get("/covercalibrator/<device_number>/calibratorstate")]
fn get_covercalibrator_calibratorstate(
    schemas::GetCovercalibratorCalibratorstatePath { device_number }: schemas::GetCovercalibratorCalibratorstatePath,

    schemas::GetCovercalibratorCalibratorstateQuery { client_id, client_transaction_id }: schemas::GetCovercalibratorCalibratorstateQuery,
) -> schemas::IntResponse {
}

/**
Returns the state of the device cover"

Returns the state of the device cover, if present, otherwise returns "NotPresent".  The cover state mode is specified as an integer value from the CoverStatus Enum.
*/
#[get("/covercalibrator/<device_number>/coverstate")]
fn get_covercalibrator_coverstate(
    schemas::GetCovercalibratorCoverstatePath { device_number }: schemas::GetCovercalibratorCoverstatePath,

    schemas::GetCovercalibratorCoverstateQuery { client_id, client_transaction_id }: schemas::GetCovercalibratorCoverstateQuery,
) -> schemas::IntResponse {
}

/**
Returns the calibrator's maximum Brightness value.

The Brightness value that makes the calibrator deliver its maximum illumination.
*/
#[get("/covercalibrator/<device_number>/maxbrightness")]
fn get_covercalibrator_maxbrightness(
    schemas::GetCovercalibratorMaxbrightnessPath { device_number }: schemas::GetCovercalibratorMaxbrightnessPath,

    schemas::GetCovercalibratorMaxbrightnessQuery { client_id, client_transaction_id }: schemas::GetCovercalibratorMaxbrightnessQuery,
) -> schemas::IntResponse {
}

/**
Turns the calibrator off

Turns the calibrator off if the device has calibration capability.
*/
#[put("/covercalibrator/<device_number>/calibratoroff")]
fn put_covercalibrator_calibratoroff(
    schemas::PutCovercalibratorCalibratoroffPath { device_number }: schemas::PutCovercalibratorCalibratoroffPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Turns the calibrator on at the specified brightness

Turns the calibrator on at the specified brightness if the device has calibration capability.
*/
#[put("/covercalibrator/<device_number>/calibratoron")]
fn put_covercalibrator_calibratoron(
    schemas::PutCovercalibratorCalibratoronPath { device_number }: schemas::PutCovercalibratorCalibratoronPath,

    schemas::PutCovercalibratorCalibratoronRequest {
        brightness,

        client_id,

        client_transaction_id,
    }: schemas::PutCovercalibratorCalibratoronRequest,
) -> schemas::MethodResponse {
}

/**
Initiates cover closing

Initiates cover closing if a cover is present.
*/
#[put("/covercalibrator/<device_number>/closecover")]
fn put_covercalibrator_closecover(
    schemas::PutCovercalibratorClosecoverPath { device_number }: schemas::PutCovercalibratorClosecoverPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Stops any cover movement that may be in progress

Stops any cover movement that may be in progress if a cover is present and cover movement can be interrupted.
*/
#[put("/covercalibrator/<device_number>/haltcover")]
fn put_covercalibrator_haltcover(
    schemas::PutCovercalibratorHaltcoverPath { device_number }: schemas::PutCovercalibratorHaltcoverPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Initiates cover opening

Initiates cover opening if a cover is present.
*/
#[put("/covercalibrator/<device_number>/opencover")]
fn put_covercalibrator_opencover(
    schemas::PutCovercalibratorOpencoverPath { device_number }: schemas::PutCovercalibratorOpencoverPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
The dome altitude

The dome altitude (degrees, horizon zero and increasing positive to 90 zenith).
*/
#[get("/dome/<device_number>/altitude")]
fn get_dome_altitude(
    schemas::GetDomeAltitudePath { device_number }: schemas::GetDomeAltitudePath,

    schemas::GetDomeAltitudeQuery { client_id, client_transaction_id }: schemas::GetDomeAltitudeQuery,
) -> schemas::DoubleResponse {
}

/**
Indicates whether the dome is in the home position.

Indicates whether the dome is in the home position. This is normally used following a FindHome()  operation. The value is reset with any azimuth slew operation that moves the dome away from the home position. AtHome may also become true durng normal slew operations, if the dome passes through the home position and the dome controller hardware is capable of detecting that; or at the end of a slew operation if the dome comes to rest at the home position.
*/
#[get("/dome/<device_number>/athome")]
fn get_dome_athome(
    schemas::GetDomeAthomePath { device_number }: schemas::GetDomeAthomePath,

    schemas::GetDomeAthomeQuery { client_id, client_transaction_id }: schemas::GetDomeAthomeQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the telescope is at the park position

True if the dome is in the programmed park position. Set only following a Park() operation and reset with any slew operation.
*/
#[get("/dome/<device_number>/atpark")]
fn get_dome_atpark(
    schemas::GetDomeAtparkPath { device_number }: schemas::GetDomeAtparkPath,

    schemas::GetDomeAtparkQuery { client_id, client_transaction_id }: schemas::GetDomeAtparkQuery,
) -> schemas::BoolResponse {
}

/**
The dome azimuth

Returns the dome azimuth (degrees, North zero and increasing clockwise, i.e., 90 East, 180 South, 270 West)
*/
#[get("/dome/<device_number>/azimuth")]
fn get_dome_azimuth(
    schemas::GetDomeAzimuthPath { device_number }: schemas::GetDomeAzimuthPath,

    schemas::GetDomeAzimuthQuery { client_id, client_transaction_id }: schemas::GetDomeAzimuthQuery,
) -> schemas::DoubleResponse {
}

/**
Indicates whether the dome can find the home position.

True if the dome can move to the home position.
*/
#[get("/dome/<device_number>/canfindhome")]
fn get_dome_canfindhome(
    schemas::GetDomeCanfindhomePath { device_number }: schemas::GetDomeCanfindhomePath,

    schemas::GetDomeCanfindhomeQuery { client_id, client_transaction_id }: schemas::GetDomeCanfindhomeQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the dome can be parked.

True if the dome is capable of programmed parking (Park() method)
*/
#[get("/dome/<device_number>/canpark")]
fn get_dome_canpark(
    schemas::GetDomeCanparkPath { device_number }: schemas::GetDomeCanparkPath,

    schemas::GetDomeCanparkQuery { client_id, client_transaction_id }: schemas::GetDomeCanparkQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the dome altitude can be set

True if driver is capable of setting the dome altitude.
*/
#[get("/dome/<device_number>/cansetaltitude")]
fn get_dome_cansetaltitude(
    schemas::GetDomeCansetaltitudePath { device_number }: schemas::GetDomeCansetaltitudePath,

    schemas::GetDomeCansetaltitudeQuery { client_id, client_transaction_id }: schemas::GetDomeCansetaltitudeQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the dome azimuth can be set

True if driver is capable of setting the dome azimuth.
*/
#[get("/dome/<device_number>/cansetazimuth")]
fn get_dome_cansetazimuth(
    schemas::GetDomeCansetazimuthPath { device_number }: schemas::GetDomeCansetazimuthPath,

    schemas::GetDomeCansetazimuthQuery { client_id, client_transaction_id }: schemas::GetDomeCansetazimuthQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the dome park position can be set

True if driver is capable of setting the dome park position.
*/
#[get("/dome/<device_number>/cansetpark")]
fn get_dome_cansetpark(
    schemas::GetDomeCansetparkPath { device_number }: schemas::GetDomeCansetparkPath,

    schemas::GetDomeCansetparkQuery { client_id, client_transaction_id }: schemas::GetDomeCansetparkQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the dome shutter can be opened

True if driver is capable of automatically operating shutter
*/
#[get("/dome/<device_number>/cansetshutter")]
fn get_dome_cansetshutter(
    schemas::GetDomeCansetshutterPath { device_number }: schemas::GetDomeCansetshutterPath,

    schemas::GetDomeCansetshutterQuery { client_id, client_transaction_id }: schemas::GetDomeCansetshutterQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the dome supports slaving to a telescope

True if driver is capable of slaving to a telescope.
*/
#[get("/dome/<device_number>/canslave")]
fn get_dome_canslave(
    schemas::GetDomeCanslavePath { device_number }: schemas::GetDomeCanslavePath,

    schemas::GetDomeCanslaveQuery { client_id, client_transaction_id }: schemas::GetDomeCanslaveQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the dome azimuth position can be synched

True if driver is capable of synchronizing the dome azimuth position using the SyncToAzimuth(Double) method.
*/
#[get("/dome/<device_number>/cansyncazimuth")]
fn get_dome_cansyncazimuth(
    schemas::GetDomeCansyncazimuthPath { device_number }: schemas::GetDomeCansyncazimuthPath,

    schemas::GetDomeCansyncazimuthQuery { client_id, client_transaction_id }: schemas::GetDomeCansyncazimuthQuery,
) -> schemas::BoolResponse {
}

/**
Status of the dome shutter or roll-off roof

Returns the status of the dome shutter or roll-off roof. 0 = Open, 1 = Closed, 2 = Opening, 3 = Closing, 4 = Shutter status error
*/
#[get("/dome/<device_number>/shutterstatus")]
fn get_dome_shutterstatus(
    schemas::GetDomeShutterstatusPath { device_number }: schemas::GetDomeShutterstatusPath,

    schemas::GetDomeShutterstatusQuery { client_id, client_transaction_id }: schemas::GetDomeShutterstatusQuery,
) -> schemas::IntResponse {
}

/**
Indicates whether the dome is slaved to the telescope

True if the dome is slaved to the telescope in its hardware, else False.
*/
#[get("/dome/<device_number>/slaved")]
fn get_dome_slaved(
    schemas::GetDomeSlavedPath { device_number }: schemas::GetDomeSlavedPath,

    schemas::GetDomeSlavedQuery { client_id, client_transaction_id }: schemas::GetDomeSlavedQuery,
) -> schemas::BoolResponse {
}

/**
Sets whether the dome is slaved to the telescope

Sets the current subframe height.
*/
#[put("/dome/<device_number>/slaved")]
fn put_dome_slaved(
    schemas::PutDomeSlavedPath { device_number }: schemas::PutDomeSlavedPath,

    schemas::PutDomeSlavedRequest {
        slaved,

        client_id,

        client_transaction_id,
    }: schemas::PutDomeSlavedRequest,
) -> schemas::MethodResponse {
}

/**
Indicates whether the any part of the dome is moving

True if any part of the dome is currently moving, False if all dome components are steady.
*/
#[get("/dome/<device_number>/slewing")]
fn get_dome_slewing(
    schemas::GetDomeSlewingPath { device_number }: schemas::GetDomeSlewingPath,

    schemas::GetDomeSlewingQuery { client_id, client_transaction_id }: schemas::GetDomeSlewingQuery,
) -> schemas::BoolResponse {
}

/**
Immediately cancel current dome operation.

Calling this method will immediately disable hardware slewing (Slaved will become False).
*/
#[put("/dome/<device_number>/abortslew")]
fn put_dome_abortslew(
    schemas::PutDomeAbortslewPath { device_number }: schemas::PutDomeAbortslewPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Close the shutter or otherwise shield telescope from the sky.

Close the shutter or otherwise shield telescope from the sky.
*/
#[put("/dome/<device_number>/closeshutter")]
fn put_dome_closeshutter(
    schemas::PutDomeCloseshutterPath { device_number }: schemas::PutDomeCloseshutterPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Start operation to search for the dome home position.

After Home position is established initializes Azimuth to the default value and sets the AtHome flag.
*/
#[put("/dome/<device_number>/findhome")]
fn put_dome_findhome(
    schemas::PutDomeFindhomePath { device_number }: schemas::PutDomeFindhomePath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Open shutter or otherwise expose telescope to the sky.

Open shutter or otherwise expose telescope to the sky.
*/
#[put("/dome/<device_number>/openshutter")]
fn put_dome_openshutter(
    schemas::PutDomeOpenshutterPath { device_number }: schemas::PutDomeOpenshutterPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Rotate dome in azimuth to park position.

After assuming programmed park position, sets AtPark flag.
*/
#[put("/dome/<device_number>/park")]
fn put_dome_park(
    schemas::PutDomeParkPath { device_number }: schemas::PutDomeParkPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Set the current azimuth, altitude position of dome to be the park position

Set the current azimuth, altitude position of dome to be the park position.
*/
#[put("/dome/<device_number>/setpark")]
fn put_dome_setpark(
    schemas::PutDomeSetparkPath { device_number }: schemas::PutDomeSetparkPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Slew the dome to the given altitude position.

Slew the dome to the given altitude position.
*/
#[put("/dome/<device_number>/slewtoaltitude")]
fn put_dome_slewtoaltitude(
    schemas::PutDomeSlewtoaltitudePath { device_number }: schemas::PutDomeSlewtoaltitudePath,

    schemas::PutDomeSlewtoaltitudeRequest {
        altitude,

        client_id,

        client_transaction_id,
    }: schemas::PutDomeSlewtoaltitudeRequest,
) -> schemas::MethodResponse {
}

/**
Slew the dome to the given azimuth position.

Slew the dome to the given azimuth position.
*/
#[put("/dome/<device_number>/slewtoazimuth")]
fn put_dome_slewtoazimuth(
    schemas::PutDomeSlewtoazimuthPath { device_number }: schemas::PutDomeSlewtoazimuthPath,

    schemas::PutDomeSlewtoazimuthRequest {
        azimuth,

        client_id,

        client_transaction_id,
    }: schemas::PutDomeSlewtoazimuthRequest,
) -> schemas::MethodResponse {
}

/**
Synchronize the current position of the dome to the given azimuth.

Synchronize the current position of the dome to the given azimuth.
*/
#[put("/dome/<device_number>/synctoazimuth")]
fn put_dome_synctoazimuth(
    schemas::PutDomeSynctoazimuthPath { device_number }: schemas::PutDomeSynctoazimuthPath,

    schemas::PutDomeSlewtoazimuthRequest {
        azimuth,

        client_id,

        client_transaction_id,
    }: schemas::PutDomeSlewtoazimuthRequest,
) -> schemas::MethodResponse {
}

/**
Filter focus offsets

An integer array of filter focus offsets.
*/
#[get("/filterwheel/<device_number>/focusoffsets")]
fn get_filterwheel_focusoffsets(
    schemas::GetFilterwheelFocusoffsetsPath { device_number }: schemas::GetFilterwheelFocusoffsetsPath,

    schemas::GetFilterwheelFocusoffsetsQuery { client_id, client_transaction_id }: schemas::GetFilterwheelFocusoffsetsQuery,
) -> schemas::IntArrayResponse {
}

/**
Filter wheel filter names

The names of the filters
*/
#[get("/filterwheel/<device_number>/names")]
fn get_filterwheel_names(
    schemas::GetFilterwheelNamesPath { device_number }: schemas::GetFilterwheelNamesPath,

    schemas::GetFilterwheelNamesQuery { client_id, client_transaction_id }: schemas::GetFilterwheelNamesQuery,
) -> schemas::StringArrayResponse {
}

/**
Returns the current filter wheel position

Returns the current filter wheel position
*/
#[get("/filterwheel/<device_number>/position")]
fn get_filterwheel_position(
    schemas::GetFilterwheelPositionPath { device_number }: schemas::GetFilterwheelPositionPath,

    schemas::GetFilterwheelPositionQuery { client_id, client_transaction_id }: schemas::GetFilterwheelPositionQuery,
) -> schemas::IntResponse {
}

/**
Sets the filter wheel position

Sets the filter wheel position
*/
#[put("/filterwheel/<device_number>/position")]
fn put_filterwheel_position(
    schemas::PutFilterwheelPositionPath { device_number }: schemas::PutFilterwheelPositionPath,

    schemas::PutFilterwheelPositionRequest {
        position,

        client_id,

        client_transaction_id,
    }: schemas::PutFilterwheelPositionRequest,
) -> schemas::MethodResponse {
}

/**
Indicates whether the focuser is capable of absolute position.

True if the focuser is capable of absolute position; that is, being commanded to a specific step location.
*/
#[get("/focuser/<device_number>/absolute")]
fn get_focuser_absolute(
    schemas::GetFocuserAbsolutePath { device_number }: schemas::GetFocuserAbsolutePath,

    schemas::GetFocuserAbsoluteQuery { client_id, client_transaction_id }: schemas::GetFocuserAbsoluteQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the focuser is currently moving.

True if the focuser is currently moving to a new position. False if the focuser is stationary.
*/
#[get("/focuser/<device_number>/ismoving")]
fn get_focuser_ismoving(
    schemas::GetFocuserIsmovingPath { device_number }: schemas::GetFocuserIsmovingPath,

    schemas::GetFocuserIsmovingQuery { client_id, client_transaction_id }: schemas::GetFocuserIsmovingQuery,
) -> schemas::BoolResponse {
}

/**
Returns the focuser's maximum increment size.

Maximum increment size allowed by the focuser; i.e. the maximum number of steps allowed in one move operation.
*/
#[get("/focuser/<device_number>/maxincrement")]
fn get_focuser_maxincrement(
    schemas::GetFocuserMaxincrementPath { device_number }: schemas::GetFocuserMaxincrementPath,

    schemas::GetFocuserMaxincrementQuery { client_id, client_transaction_id }: schemas::GetFocuserMaxincrementQuery,
) -> schemas::IntResponse {
}

/**
Returns the focuser's maximum step size.

Maximum step position permitted.
*/
#[get("/focuser/<device_number>/maxstep")]
fn get_focuser_maxstep(
    schemas::GetFocuserMaxstepPath { device_number }: schemas::GetFocuserMaxstepPath,

    schemas::GetFocuserMaxstepQuery { client_id, client_transaction_id }: schemas::GetFocuserMaxstepQuery,
) -> schemas::IntResponse {
}

/**
Returns the focuser's current position.

Current focuser position, in steps.
*/
#[get("/focuser/<device_number>/position")]
fn get_focuser_position(
    schemas::GetFocuserPositionPath { device_number }: schemas::GetFocuserPositionPath,

    schemas::GetFocuserPositionQuery { client_id, client_transaction_id }: schemas::GetFocuserPositionQuery,
) -> schemas::IntResponse {
}

/**
Returns the focuser's step size.

Step size (microns) for the focuser.
*/
#[get("/focuser/<device_number>/stepsize")]
fn get_focuser_stepsize(
    schemas::GetFocuserStepsizePath { device_number }: schemas::GetFocuserStepsizePath,

    schemas::GetFocuserStepsizeQuery { client_id, client_transaction_id }: schemas::GetFocuserStepsizeQuery,
) -> schemas::DoubleResponse {
}

/**
Retrieves the state of temperature compensation mode

Gets the state of temperature compensation mode (if available), else always False.
*/
#[get("/focuser/<device_number>/tempcomp")]
fn get_focuser_tempcomp(
    schemas::GetFocuserTempcompPath { device_number }: schemas::GetFocuserTempcompPath,

    schemas::GetFocuserTempcompQuery { client_id, client_transaction_id }: schemas::GetFocuserTempcompQuery,
) -> schemas::BoolResponse {
}

/**
Sets the device's temperature compensation mode.

Sets the state of temperature compensation mode.
*/
#[put("/focuser/<device_number>/tempcomp")]
fn put_focuser_tempcomp(
    schemas::PutFocuserTempcompPath { device_number }: schemas::PutFocuserTempcompPath,

    schemas::PutFocuserTempcompRequest {
        temp_comp,

        client,

        client_transaction_idform,
    }: schemas::PutFocuserTempcompRequest,
) -> schemas::MethodResponse {
}

/**
Indicates whether the focuser has temperature compensation.

True if focuser has temperature compensation available.
*/
#[get("/focuser/<device_number>/tempcompavailable")]
fn get_focuser_tempcompavailable(
    schemas::GetFocuserTempcompavailablePath { device_number }: schemas::GetFocuserTempcompavailablePath,

    schemas::GetFocuserTempcompavailableQuery { client_id, client_transaction_id }: schemas::GetFocuserTempcompavailableQuery,
) -> schemas::BoolResponse {
}

/**
Returns the focuser's current temperature.

Current ambient temperature as measured by the focuser.
*/
#[get("/focuser/<device_number>/temperature")]
fn get_focuser_temperature(
    schemas::GetFocuserTemperaturePath { device_number }: schemas::GetFocuserTemperaturePath,

    schemas::GetFocuserTemperatureQuery { client_id, client_transaction_id }: schemas::GetFocuserTemperatureQuery,
) -> schemas::DoubleResponse {
}

/**
Immediatley stops focuser motion.

Immediately stop any focuser motion due to a previous Move(Int32) method call.
*/
#[put("/focuser/<device_number>/halt")]
fn put_focuser_halt(
    schemas::PutFocuserHaltPath { device_number }: schemas::PutFocuserHaltPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Moves the focuser to a new position.

Moves the focuser by the specified amount or to the specified position depending on the value of the Absolute property.
*/
#[put("/focuser/<device_number>/move")]
fn put_focuser_move(
    schemas::PutFocuserMovePath { device_number }: schemas::PutFocuserMovePath,

    schemas::PutFocuserMoveRequest {
        position,

        client_id,

        client_transaction_id,
    }: schemas::PutFocuserMoveRequest,
) -> schemas::MethodResponse {
}

/**
Returns the time period over which observations will be averaged

Gets the time period over which observations will be averaged
*/
#[get("/observingconditions/<device_number>/averageperiod")]
fn get_observingconditions_averageperiod(
    schemas::GetObservingconditionsAverageperiodPath { device_number }: schemas::GetObservingconditionsAverageperiodPath,

    schemas::GetObservingconditionsAverageperiodQuery { client_id, client_transaction_id }: schemas::GetObservingconditionsAverageperiodQuery,
) -> schemas::DoubleResponse {
}

/**
Sets the time period over which observations will be averaged

Sets the time period over which observations will be averaged
*/
#[put("/observingconditions/<device_number>/averageperiod")]
fn put_observingconditions_averageperiod(
    schemas::PutObservingconditionsAverageperiodPath { device_number }: schemas::PutObservingconditionsAverageperiodPath,

    schemas::PutObservingconditionsAverageperiodRequest {
        average_period,

        client_id,

        client_transaction_id,
    }: schemas::PutObservingconditionsAverageperiodRequest,
) -> schemas::MethodResponse {
}

/**
Returns the amount of sky obscured by cloud

Gets the percentage of the sky obscured by cloud
*/
#[get("/observingconditions/<device_number>/cloudcover")]
fn get_observingconditions_cloudcover(
    schemas::GetObservingconditionsCloudcoverPath { device_number }: schemas::GetObservingconditionsCloudcoverPath,

    schemas::GetObservingconditionsCloudcoverQuery { client_id, client_transaction_id }: schemas::GetObservingconditionsCloudcoverQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the atmospheric dew point at the observatory

Gets the atmospheric dew point at the observatory reported in °C.
*/
#[get("/observingconditions/<device_number>/dewpoint")]
fn get_observingconditions_dewpoint(
    schemas::GetObservingconditionsDewpointPath { device_number }: schemas::GetObservingconditionsDewpointPath,

    schemas::GetObservingconditionsDewpointQuery { client_id, client_transaction_id }: schemas::GetObservingconditionsDewpointQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the atmospheric humidity at the observatory

Gets the atmospheric  humidity (%) at the observatory
*/
#[get("/observingconditions/<device_number>/humidity")]
fn get_observingconditions_humidity(
    schemas::GetObservingconditionsHumidityPath { device_number }: schemas::GetObservingconditionsHumidityPath,

    schemas::GetObservingconditionsHumidityQuery { client_id, client_transaction_id }: schemas::GetObservingconditionsHumidityQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the atmospheric pressure at the observatory.

Gets the atmospheric pressure in hectoPascals at the observatory's altitude - NOT reduced to sea level.
*/
#[get("/observingconditions/<device_number>/pressure")]
fn get_observingconditions_pressure(
    schemas::GetObservingconditionsPressurePath { device_number }: schemas::GetObservingconditionsPressurePath,

    schemas::GetObservingconditionsPressureQuery { client_id, client_transaction_id }: schemas::GetObservingconditionsPressureQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the rain rate at the observatory.

Gets the rain rate (mm/hour) at the observatory.
*/
#[get("/observingconditions/<device_number>/rainrate")]
fn get_observingconditions_rainrate(
    schemas::GetObservingconditionsRainratePath { device_number }: schemas::GetObservingconditionsRainratePath,

    schemas::GetObservingconditionsRainrateQuery { client_id, client_transaction_id }: schemas::GetObservingconditionsRainrateQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the sky brightness at the observatory

Gets the sky brightness at the observatory (Lux)
*/
#[get("/observingconditions/<device_number>/skybrightness")]
fn get_observingconditions_skybrightness(
    schemas::GetObservingconditionsSkybrightnessPath { device_number }: schemas::GetObservingconditionsSkybrightnessPath,

    schemas::GetObservingconditionsSkybrightnessQuery { client_id, client_transaction_id }: schemas::GetObservingconditionsSkybrightnessQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the sky quality at the observatory

Gets the sky quality at the observatory (magnitudes per square arc second)
*/
#[get("/observingconditions/<device_number>/skyquality")]
fn get_observingconditions_skyquality(
    schemas::GetObservingconditionsSkyqualityPath { device_number }: schemas::GetObservingconditionsSkyqualityPath,

    schemas::GetObservingconditionsSkyqualityQuery { client_id, client_transaction_id }: schemas::GetObservingconditionsSkyqualityQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the sky temperature at the observatory

Gets the sky temperature(°C) at the observatory.
*/
#[get("/observingconditions/<device_number>/skytemperature")]
fn get_observingconditions_skytemperature(
    schemas::GetObservingconditionsSkytemperaturePath { device_number }: schemas::GetObservingconditionsSkytemperaturePath,

    schemas::GetObservingconditionsSkytemperatureQuery { client_id, client_transaction_id }: schemas::GetObservingconditionsSkytemperatureQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the seeing at the observatory

Gets the seeing at the observatory measured as star full width half maximum (FWHM) in arc secs.
*/
#[get("/observingconditions/<device_number>/starfwhm")]
fn get_observingconditions_starfwhm(
    schemas::GetObservingconditionsStarfwhmPath { device_number }: schemas::GetObservingconditionsStarfwhmPath,

    schemas::GetObservingconditionsStarfwhmQuery { client_id, client_transaction_id }: schemas::GetObservingconditionsStarfwhmQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the temperature at the observatory

Gets the temperature(°C) at the observatory.
*/
#[get("/observingconditions/<device_number>/temperature")]
fn get_observingconditions_temperature(
    schemas::GetObservingconditionsTemperaturePath { device_number }: schemas::GetObservingconditionsTemperaturePath,

    schemas::GetObservingconditionsTemperatureQuery { client_id, client_transaction_id }: schemas::GetObservingconditionsTemperatureQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the wind direction at the observatory

Gets the wind direction. The returned value must be between 0.0 and 360.0, interpreted according to the metereological standard, where a special value of 0.0 is returned when the wind speed is 0.0. Wind direction is measured clockwise from north, through east, where East=90.0, South=180.0, West=270.0 and North=360.0.
*/
#[get("/observingconditions/<device_number>/winddirection")]
fn get_observingconditions_winddirection(
    schemas::GetObservingconditionsWinddirectionPath { device_number }: schemas::GetObservingconditionsWinddirectionPath,

    schemas::GetObservingconditionsWinddirectionQuery { client_id, client_transaction_id }: schemas::GetObservingconditionsWinddirectionQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the peak 3 second wind gust at the observatory over the last 2 minutes

Gets the peak 3 second wind gust(m/s) at the observatory over the last 2 minutes.
*/
#[get("/observingconditions/<device_number>/windgust")]
fn get_observingconditions_windgust(
    schemas::GetObservingconditionsWindgustPath { device_number }: schemas::GetObservingconditionsWindgustPath,

    schemas::GetObservingconditionsWindgustQuery { client_id, client_transaction_id }: schemas::GetObservingconditionsWindgustQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the wind speed at the observatory.

Gets the wind speed(m/s) at the observatory.
*/
#[get("/observingconditions/<device_number>/windspeed")]
fn get_observingconditions_windspeed(
    schemas::GetObservingconditionsWindspeedPath { device_number }: schemas::GetObservingconditionsWindspeedPath,

    schemas::GetObservingconditionsWindspeedQuery { client_id, client_transaction_id }: schemas::GetObservingconditionsWindspeedQuery,
) -> schemas::DoubleResponse {
}

/**
Refreshes sensor values from hardware.

Forces the driver to immediately query its attached hardware to refresh sensor values.
*/
#[put("/observingconditions/<device_number>/refresh")]
fn put_observingconditions_refresh(
    schemas::PutObservingconditionsRefreshPath { device_number }: schemas::PutObservingconditionsRefreshPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Return a sensor description

Gets a description of the sensor with the name specified in the SensorName parameter
*/
#[get("/observingconditions/<device_number>/sensordescription")]
fn get_observingconditions_sensordescription(
    schemas::GetObservingconditionsSensordescriptionPath { device_number }: schemas::GetObservingconditionsSensordescriptionPath,

    schemas::GetObservingconditionsSensordescriptionQuery {
        sensor_name,

        client_id,

        client_transaction_id,
    }: schemas::GetObservingconditionsSensordescriptionQuery,
) -> schemas::StringResponse {
}

/**
Return the time since the sensor value was last updated

Gets the time since the sensor specified in the SensorName parameter was last updated
*/
#[get("/observingconditions/<device_number>/timesincelastupdate")]
fn get_observingconditions_timesincelastupdate(
    schemas::GetObservingconditionsTimesincelastupdatePath { device_number }: schemas::GetObservingconditionsTimesincelastupdatePath,

    schemas::GetObservingconditionsTimesincelastupdateQuery {
        sensor_name,

        client_id,

        client_transaction_id,
    }: schemas::GetObservingconditionsTimesincelastupdateQuery,
) -> schemas::DoubleResponse {
}

/**
IIndicates whether the Rotator supports the Reverse method.

True if the Rotator supports the Reverse method.
*/
#[get("/rotator/<device_number>/canreverse")]
fn get_rotator_canreverse(
    schemas::GetRotatorCanreversePath { device_number }: schemas::GetRotatorCanreversePath,

    schemas::GetRotatorCanreverseQuery { client_id, client_transaction_id }: schemas::GetRotatorCanreverseQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the rotator is currently moving.

True if the rotator is currently moving to a new position. False if the focuser is stationary.
*/
#[get("/rotator/<device_number>/ismoving")]
fn get_rotator_ismoving(
    schemas::GetRotatorIsmovingPath { device_number }: schemas::GetRotatorIsmovingPath,

    schemas::GetRotatorIsmovingQuery { client_id, client_transaction_id }: schemas::GetRotatorIsmovingQuery,
) -> schemas::BoolResponse {
}

/**
Returns the rotator's mechanical current position.

Returns the raw mechanical position of the rotator in degrees.
*/
#[get("/rotator/<device_number>/mechanicalposition")]
fn get_rotator_mechanicalposition(
    schemas::GetRotatorMechanicalpositionPath { device_number }: schemas::GetRotatorMechanicalpositionPath,

    schemas::GetRotatorMechanicalpositionQuery { client_id, client_transaction_id }: schemas::GetRotatorMechanicalpositionQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the rotator's current position.

Current instantaneous Rotator position, in degrees.
*/
#[get("/rotator/<device_number>/position")]
fn get_rotator_position(
    schemas::GetRotatorPositionPath { device_number }: schemas::GetRotatorPositionPath,

    schemas::GetRotatorPositionQuery { client_id, client_transaction_id }: schemas::GetRotatorPositionQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the rotator’s Reverse state.

Returns the rotator’s Reverse state.
*/
#[get("/rotator/<device_number>/reverse")]
fn get_rotator_reverse(
    schemas::GetRotatorReversePath { device_number }: schemas::GetRotatorReversePath,

    schemas::GetRotatorReverseQuery { client_id, client_transaction_id }: schemas::GetRotatorReverseQuery,
) -> schemas::BoolResponse {
}

/**
Sets the rotator’s Reverse state.

Sets the rotator’s Reverse state.
*/
#[put("/rotator/<device_number>/reverse")]
fn put_rotator_reverse(
    schemas::PutRotatorReversePath { device_number }: schemas::PutRotatorReversePath,

    schemas::PutRotatorReverseRequest {
        reverse,

        client_id,

        client_transaction_id,
    }: schemas::PutRotatorReverseRequest,
) -> schemas::MethodResponse {
}

/**
Returns the minimum StepSize

The minimum StepSize, in degrees.
*/
#[get("/rotator/<device_number>/stepsize")]
fn get_rotator_stepsize(
    schemas::GetRotatorStepsizePath { device_number }: schemas::GetRotatorStepsizePath,

    schemas::GetRotatorStepsizeQuery { client_id, client_transaction_id }: schemas::GetRotatorStepsizeQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the destination position angle.

The destination position angle for Move() and MoveAbsolute().
*/
#[get("/rotator/<device_number>/targetposition")]
fn get_rotator_targetposition(
    schemas::GetRotatorTargetpositionPath { device_number }: schemas::GetRotatorTargetpositionPath,

    schemas::GetRotatorTargetpositionQuery { client_id, client_transaction_id }: schemas::GetRotatorTargetpositionQuery,
) -> schemas::DoubleResponse {
}

/**
Immediatley stops rotator motion.

Immediately stop any Rotator motion due to a previous Move or MoveAbsolute method call.
*/
#[put("/rotator/<device_number>/halt")]
fn put_rotator_halt(
    schemas::PutRotatorHaltPath { device_number }: schemas::PutRotatorHaltPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Moves the rotator to a new relative position.

Causes the rotator to move Position degrees relative to the current Position value.
*/
#[put("/rotator/<device_number>/move")]
fn put_rotator_move(
    schemas::PutRotatorMovePath { device_number }: schemas::PutRotatorMovePath,

    schemas::PutRotatorMoveRequest {
        position,

        client_id,

        client_transaction_id,
    }: schemas::PutRotatorMoveRequest,
) -> schemas::MethodResponse {
}

/**
Moves the rotator to a new absolute position.

Causes the rotator to move the absolute position of Position degrees.
*/
#[put("/rotator/<device_number>/moveabsolute")]
fn put_rotator_moveabsolute(
    schemas::PutRotatorMoveabsolutePath { device_number }: schemas::PutRotatorMoveabsolutePath,

    schemas::PutRotatorMoveabsoluteRequest {
        position,

        client_id,

        client_transaction_id,
    }: schemas::PutRotatorMoveabsoluteRequest,
) -> schemas::MethodResponse {
}

/**
Moves the rotator to a new raw mechanical position.

Causes the rotator to move the mechanical position of Position degrees.
*/
#[put("/rotator/<device_number>/movemechanical")]
fn put_rotator_movemechanical(
    schemas::PutRotatorMovemechanicalPath { device_number }: schemas::PutRotatorMovemechanicalPath,

    schemas::PutRotatorMovemechanicalRequest {
        position,

        client_id,

        client_transaction_id,
    }: schemas::PutRotatorMovemechanicalRequest,
) -> schemas::MethodResponse {
}

/**
Syncs the rotator to the specified position angle without moving it.

Causes the rotator to sync to the position of Position degrees.
*/
#[put("/rotator/<device_number>/sync")]
fn put_rotator_sync(
    schemas::PutRotatorSyncPath { device_number }: schemas::PutRotatorSyncPath,

    schemas::PutRotatorSyncRequest {
        position,

        client_id,

        client_transaction_id,
    }: schemas::PutRotatorSyncRequest,
) -> schemas::MethodResponse {
}

/**
Indicates whether the monitored state is safe for use.

Indicates whether the monitored state is safe for use. True if the state is safe, False if it is unsafe.
*/
#[get("/safetymonitor/<device_number>/issafe")]
fn get_safetymonitor_issafe(
    schemas::GetSafetymonitorIssafePath { device_number }: schemas::GetSafetymonitorIssafePath,

    schemas::GetSafetymonitorIssafeQuery { client_id, client_transaction_id }: schemas::GetSafetymonitorIssafeQuery,
) -> schemas::BoolResponse {
}

/**
The number of switch devices managed by this driver

Returns the number of switch devices managed by this driver. Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/maxswitch")]
fn get_switch_maxswitch(
    schemas::GetSwitchMaxswitchPath { device_number }: schemas::GetSwitchMaxswitchPath,

    schemas::GetSwitchMaxswitchQuery { client_id, client_transaction_id }: schemas::GetSwitchMaxswitchQuery,
) -> schemas::IntResponse {
}

/**
Indicates whether the specified switch device can be written to

Reports if the specified switch device can be written to, default true. This is false if the device cannot be written to, for example a limit switch or a sensor.  Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/canwrite")]
fn get_switch_canwrite(
    schemas::GetSwitchCanwritePath { device_number }: schemas::GetSwitchCanwritePath,

    schemas::GetSwitchCanwriteQuery { id, client_id, client_transaction_id }: schemas::GetSwitchCanwriteQuery,
) -> schemas::BoolResponse {
}

/**
Return the state of switch device id as a boolean

Return the state of switch device id as a boolean.  Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/getswitch")]
fn get_switch_getswitch(
    schemas::GetSwitchGetswitchPath { device_number }: schemas::GetSwitchGetswitchPath,

    schemas::GetSwitchGetswitchQuery { id, client_id, client_transaction_id }: schemas::GetSwitchGetswitchQuery,
) -> schemas::BoolResponse {
}

/**
Gets the description of the specified switch device

Gets the description of the specified switch device. This is to allow a fuller description of the device to be returned, for example for a tool tip. Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/getswitchdescription")]
fn get_switch_getswitchdescription(
    schemas::GetSwitchGetswitchdescriptionPath { device_number }: schemas::GetSwitchGetswitchdescriptionPath,

    schemas::GetSwitchGetswitchdescriptionQuery { id, client_id, client_transaction_id }: schemas::GetSwitchGetswitchdescriptionQuery,
) -> schemas::StringResponse {
}

/**
Gets the name of the specified switch device

Gets the name of the specified switch device. Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/getswitchname")]
fn get_switch_getswitchname(
    schemas::GetSwitchGetswitchnamePath { device_number }: schemas::GetSwitchGetswitchnamePath,

    schemas::GetSwitchGetswitchnameQuery { id, client_id, client_transaction_id }: schemas::GetSwitchGetswitchnameQuery,
) -> schemas::StringResponse {
}

/**
Gets the value of the specified switch device as a double

Gets the value of the specified switch device as a double. Devices are numbered from 0 to MaxSwitch - 1, The value of this switch is expected to be between MinSwitchValue and MaxSwitchValue.
*/
#[get("/switch/<device_number>/getswitchvalue")]
fn get_switch_getswitchvalue(
    schemas::GetSwitchGetswitchvaluePath { device_number }: schemas::GetSwitchGetswitchvaluePath,

    schemas::GetSwitchGetswitchvalueQuery { id, client_id, client_transaction_id }: schemas::GetSwitchGetswitchvalueQuery,
) -> schemas::DoubleResponse {
}

/**
Gets the minimum value of the specified switch device as a double

Gets the minimum value of the specified switch device as a double. Devices are numbered from 0 to MaxSwitch - 1.
*/
#[get("/switch/<device_number>/minswitchvalue")]
fn get_switch_minswitchvalue(
    schemas::GetSwitchMinswitchvaluePath { device_number }: schemas::GetSwitchMinswitchvaluePath,

    schemas::GetSwitchMinswitchvalueQuery { id, client_id, client_transaction_id }: schemas::GetSwitchMinswitchvalueQuery,
) -> schemas::DoubleResponse {
}

/**
Gets the maximum value of the specified switch device as a double

Gets the maximum value of the specified switch device as a double. Devices are numbered from 0 to MaxSwitch - 1.
*/
#[get("/switch/<device_number>/maxswitchvalue")]
fn get_switch_maxswitchvalue(
    schemas::GetSwitchMaxswitchvaluePath { device_number }: schemas::GetSwitchMaxswitchvaluePath,

    schemas::GetSwitchMaxswitchvalueQuery { id, client_id, client_transaction_id }: schemas::GetSwitchMaxswitchvalueQuery,
) -> schemas::DoubleResponse {
}

/**
Sets a switch controller device to the specified state, true or false

Sets a switch controller device to the specified state, true or false.
*/
#[put("/switch/<device_number>/setswitch")]
fn put_switch_setswitch(
    schemas::PutSwitchSetswitchPath { device_number }: schemas::PutSwitchSetswitchPath,

    schemas::PutSwitchSetswitchRequest {
        id,

        state,

        client_id,

        client_transaction_id,
    }: schemas::PutSwitchSetswitchRequest,
) -> schemas::MethodResponse {
}

/**
Sets a switch device name to the specified value

Sets a switch device name to the specified value.
*/
#[put("/switch/<device_number>/setswitchname")]
fn put_switch_setswitchname(
    schemas::PutSwitchSetswitchnamePath { device_number }: schemas::PutSwitchSetswitchnamePath,

    schemas::PutSwitchSetswitchnameRequest {
        id,

        name,

        client_id,

        client_transaction_id,
    }: schemas::PutSwitchSetswitchnameRequest,
) -> schemas::MethodResponse {
}

/**
Sets a switch device value to the specified value

Sets a switch device value to the specified value.
*/
#[put("/switch/<device_number>/setswitchvalue")]
fn put_switch_setswitchvalue(
    schemas::PutSwitchSetswitchvaluePath { device_number }: schemas::PutSwitchSetswitchvaluePath,

    schemas::PutSwitchSetswitchvalueRequest {
        id,

        value,

        client_id,

        client_transaction_id,
    }: schemas::PutSwitchSetswitchvalueRequest,
) -> schemas::MethodResponse {
}

/**
Returns the step size that this device supports (the difference between successive values of the device).

Returns the step size that this device supports (the difference between successive values of the device). Devices are numbered from 0 to MaxSwitch - 1.
*/
#[get("/switch/<device_number>/switchstep")]
fn get_switch_switchstep(
    schemas::GetSwitchSwitchstepPath { device_number }: schemas::GetSwitchSwitchstepPath,

    schemas::GetSwitchSwitchstepQuery { id, client_id, client_transaction_id }: schemas::GetSwitchSwitchstepQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the current mount alignment mode

Returns the alignment mode of the mount (Alt/Az, Polar, German Polar).  The alignment mode is specified as an integer value from the AlignmentModes Enum.
*/
#[get("/telescope/<device_number>/alignmentmode")]
fn get_telescope_alignmentmode(
    schemas::GetTelescopeAlignmentmodePath { device_number }: schemas::GetTelescopeAlignmentmodePath,

    schemas::GetTelescopeAlignmentmodeQuery { client_id, client_transaction_id }: schemas::GetTelescopeAlignmentmodeQuery,
) -> schemas::IntResponse {
}

/**
Returns the mount's altitude above the horizon.

The altitude above the local horizon of the mount's current position (degrees, positive up)
*/
#[get("/telescope/<device_number>/altitude")]
fn get_telescope_altitude(
    schemas::GetTelescopeAltitudePath { device_number }: schemas::GetTelescopeAltitudePath,

    schemas::GetTelescopeAltitudeQuery { client_id, client_transaction_id }: schemas::GetTelescopeAltitudeQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the telescope's aperture.

The area of the telescope's aperture, taking into account any obstructions (square meters)
*/
#[get("/telescope/<device_number>/aperturearea")]
fn get_telescope_aperturearea(
    schemas::GetTelescopeApertureareaPath { device_number }: schemas::GetTelescopeApertureareaPath,

    schemas::GetTelescopeApertureareaQuery { client_id, client_transaction_id }: schemas::GetTelescopeApertureareaQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the telescope's effective aperture.

The telescope's effective aperture diameter (meters)
*/
#[get("/telescope/<device_number>/aperturediameter")]
fn get_telescope_aperturediameter(
    schemas::GetTelescopeAperturediameterPath { device_number }: schemas::GetTelescopeAperturediameterPath,

    schemas::GetTelescopeAperturediameterQuery { client_id, client_transaction_id }: schemas::GetTelescopeAperturediameterQuery,
) -> schemas::DoubleResponse {
}

/**
Indicates whether the mount is at the home position.

True if the mount is stopped in the Home position. Set only following a FindHome()  operation, and reset with any slew operation. This property must be False if the telescope does not support homing.
*/
#[get("/telescope/<device_number>/athome")]
fn get_telescope_athome(
    schemas::GetTelescopeAthomePath { device_number }: schemas::GetTelescopeAthomePath,

    schemas::GetTelescopeAthomeQuery { client_id, client_transaction_id }: schemas::GetTelescopeAthomeQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the telescope is at the park position.

True if the telescope has been put into the parked state by the seee Park()  method. Set False by calling the Unpark() method.
*/
#[get("/telescope/<device_number>/atpark")]
fn get_telescope_atpark(
    schemas::GetTelescopeAtparkPath { device_number }: schemas::GetTelescopeAtparkPath,

    schemas::GetTelescopeAtparkQuery { client_id, client_transaction_id }: schemas::GetTelescopeAtparkQuery,
) -> schemas::BoolResponse {
}

/**
Returns the mount's azimuth.

The azimuth at the local horizon of the mount's current position (degrees, North-referenced, positive East/clockwise).
*/
#[get("/telescope/<device_number>/azimuth")]
fn get_telescope_azimuth(
    schemas::GetTelescopeAzimuthPath { device_number }: schemas::GetTelescopeAzimuthPath,

    schemas::GetTelescopeAzimuthQuery { client_id, client_transaction_id }: schemas::GetTelescopeAzimuthQuery,
) -> schemas::DoubleResponse {
}

/**
Indicates whether the mount can find the home position.

True if this telescope is capable of programmed finding its home position (FindHome()  method).
*/
#[get("/telescope/<device_number>/canfindhome")]
fn get_telescope_canfindhome(
    schemas::GetTelescopeCanfindhomePath { device_number }: schemas::GetTelescopeCanfindhomePath,

    schemas::GetTelescopeCanfindhomeQuery { client_id, client_transaction_id }: schemas::GetTelescopeCanfindhomeQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the telescope can be parked.

True if this telescope is capable of programmed parking (Park() method)
*/
#[get("/telescope/<device_number>/canpark")]
fn get_telescope_canpark(
    schemas::GetTelescopeCanparkPath { device_number }: schemas::GetTelescopeCanparkPath,

    schemas::GetTelescopeCanparkQuery { client_id, client_transaction_id }: schemas::GetTelescopeCanparkQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the telescope can be pulse guided.

True if this telescope is capable of software-pulsed guiding (via the PulseGuide(GuideDirections, Int32) method)
*/
#[get("/telescope/<device_number>/canpulseguide")]
fn get_telescope_canpulseguide(
    schemas::GetTelescopeCanpulseguidePath { device_number }: schemas::GetTelescopeCanpulseguidePath,

    schemas::GetTelescopeCanpulseguideQuery { client_id, client_transaction_id }: schemas::GetTelescopeCanpulseguideQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the DeclinationRate property can be changed.

True if the DeclinationRate property can be changed to provide offset tracking in the declination axis.
*/
#[get("/telescope/<device_number>/cansetdeclinationrate")]
fn get_telescope_cansetdeclinationrate(
    schemas::GetTelescopeCansetdeclinationratePath { device_number }: schemas::GetTelescopeCansetdeclinationratePath,

    schemas::GetTelescopeCansetdeclinationrateQuery { client_id, client_transaction_id }: schemas::GetTelescopeCansetdeclinationrateQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the DeclinationRate property can be changed.

True if the guide rate properties used for PulseGuide(GuideDirections, Int32) can ba adjusted.
*/
#[get("/telescope/<device_number>/cansetguiderates")]
fn get_telescope_cansetguiderates(
    schemas::GetTelescopeCansetguideratesPath { device_number }: schemas::GetTelescopeCansetguideratesPath,

    schemas::GetTelescopeCansetguideratesQuery { client_id, client_transaction_id }: schemas::GetTelescopeCansetguideratesQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the telescope park position can be set.

True if this telescope is capable of programmed setting of its park position (SetPark() method)
*/
#[get("/telescope/<device_number>/cansetpark")]
fn get_telescope_cansetpark(
    schemas::GetTelescopeCansetparkPath { device_number }: schemas::GetTelescopeCansetparkPath,

    schemas::GetTelescopeCansetparkQuery { client_id, client_transaction_id }: schemas::GetTelescopeCansetparkQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the telescope SideOfPier can be set.

True if the SideOfPier property can be set, meaning that the mount can be forced to flip.
*/
#[get("/telescope/<device_number>/cansetpierside")]
fn get_telescope_cansetpierside(
    schemas::GetTelescopeCansetpiersidePath { device_number }: schemas::GetTelescopeCansetpiersidePath,

    schemas::GetTelescopeCansetpiersideQuery { client_id, client_transaction_id }: schemas::GetTelescopeCansetpiersideQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the RightAscensionRate property can be changed.

True if the RightAscensionRate property can be changed to provide offset tracking in the right ascension axis. .
*/
#[get("/telescope/<device_number>/cansetrightascensionrate")]
fn get_telescope_cansetrightascensionrate(
    schemas::GetTelescopeCansetrightascensionratePath { device_number }: schemas::GetTelescopeCansetrightascensionratePath,

    schemas::GetTelescopeCansetrightascensionrateQuery { client_id, client_transaction_id }: schemas::GetTelescopeCansetrightascensionrateQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the Tracking property can be changed.

True if the Tracking property can be changed, turning telescope sidereal tracking on and off.
*/
#[get("/telescope/<device_number>/cansettracking")]
fn get_telescope_cansettracking(
    schemas::GetTelescopeCansettrackingPath { device_number }: schemas::GetTelescopeCansettrackingPath,

    schemas::GetTelescopeCansettrackingQuery { client_id, client_transaction_id }: schemas::GetTelescopeCansettrackingQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the telescope can slew synchronously.

True if this telescope is capable of programmed slewing (synchronous or asynchronous) to equatorial coordinates
*/
#[get("/telescope/<device_number>/canslew")]
fn get_telescope_canslew(
    schemas::GetTelescopeCanslewPath { device_number }: schemas::GetTelescopeCanslewPath,

    schemas::GetTelescopeCanslewQuery { client_id, client_transaction_id }: schemas::GetTelescopeCanslewQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the telescope can slew synchronously to AltAz coordinates.

True if this telescope is capable of programmed slewing (synchronous or asynchronous) to local horizontal coordinates
*/
#[get("/telescope/<device_number>/canslewaltaz")]
fn get_telescope_canslewaltaz(
    schemas::GetTelescopeCanslewaltazPath { device_number }: schemas::GetTelescopeCanslewaltazPath,

    schemas::GetTelescopeCanslewaltazQuery { client_id, client_transaction_id }: schemas::GetTelescopeCanslewaltazQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the telescope can slew asynchronously to AltAz coordinates.

True if this telescope is capable of programmed asynchronous slewing to local horizontal coordinates
*/
#[get("/telescope/<device_number>/canslewaltazasync")]
fn get_telescope_canslewaltazasync(
    schemas::GetTelescopeCanslewaltazasyncPath { device_number }: schemas::GetTelescopeCanslewaltazasyncPath,

    schemas::GetTelescopeCanslewaltazasyncQuery { client_id, client_transaction_id }: schemas::GetTelescopeCanslewaltazasyncQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the telescope can slew asynchronously.

True if this telescope is capable of programmed asynchronous slewing to equatorial coordinates.
*/
#[get("/telescope/<device_number>/canslewasync")]
fn get_telescope_canslewasync(
    schemas::GetTelescopeCanslewasyncPath { device_number }: schemas::GetTelescopeCanslewasyncPath,

    schemas::GetTelescopeCanslewasyncQuery { client_id, client_transaction_id }: schemas::GetTelescopeCanslewasyncQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the telescope can sync to equatorial coordinates.

True if this telescope is capable of programmed synching to equatorial coordinates.
*/
#[get("/telescope/<device_number>/cansync")]
fn get_telescope_cansync(
    schemas::GetTelescopeCansyncPath { device_number }: schemas::GetTelescopeCansyncPath,

    schemas::GetTelescopeCansyncQuery { client_id, client_transaction_id }: schemas::GetTelescopeCansyncQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the telescope can sync to local horizontal coordinates.

True if this telescope is capable of programmed synching to local horizontal coordinates
*/
#[get("/telescope/<device_number>/cansyncaltaz")]
fn get_telescope_cansyncaltaz(
    schemas::GetTelescopeCansyncaltazPath { device_number }: schemas::GetTelescopeCansyncaltazPath,

    schemas::GetTelescopeCansyncaltazQuery { client_id, client_transaction_id }: schemas::GetTelescopeCansyncaltazQuery,
) -> schemas::BoolResponse {
}

/**
Indicates whether the telescope can be unparked.

True if this telescope is capable of programmed unparking (UnPark() method)
*/
#[get("/telescope/<device_number>/canunpark")]
fn get_telescope_canunpark(
    schemas::GetTelescopeCanunparkPath { device_number }: schemas::GetTelescopeCanunparkPath,

    schemas::GetTelescopeCanunparkQuery { client_id, client_transaction_id }: schemas::GetTelescopeCanunparkQuery,
) -> schemas::BoolResponse {
}

/**
Returns the mount's declination.

The declination (degrees) of the mount's current equatorial coordinates, in the coordinate system given by the EquatorialSystem property. Reading the property will raise an error if the value is unavailable.
*/
#[get("/telescope/<device_number>/declination")]
fn get_telescope_declination(
    schemas::GetTelescopeDeclinationPath { device_number }: schemas::GetTelescopeDeclinationPath,

    schemas::GetTelescopeDeclinationQuery { client_id, client_transaction_id }: schemas::GetTelescopeDeclinationQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the telescope's declination tracking rate.

The declination tracking rate (arcseconds per second, default = 0.0)
*/
#[get("/telescope/<device_number>/declinationrate")]
fn get_telescope_declinationrate(
    schemas::GetTelescopeDeclinationratePath { device_number }: schemas::GetTelescopeDeclinationratePath,

    schemas::GetTelescopeDeclinationrateQuery { client_id, client_transaction_id }: schemas::GetTelescopeDeclinationrateQuery,
) -> schemas::DoubleResponse {
}

/**
Sets the telescope's declination tracking rate.

Sets the declination tracking rate (arcseconds per second)
*/
#[put("/telescope/<device_number>/declinationrate")]
fn put_telescope_declinationrate(
    schemas::PutTelescopeDeclinationratePath { device_number }: schemas::PutTelescopeDeclinationratePath,

    schemas::PutTelescopeDeclinationrateRequest {
        declination_rate,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeDeclinationrateRequest,
) -> schemas::MethodResponse {
}

/**
Indicates whether atmospheric refraction is applied to coordinates.

True if the telescope or driver applies atmospheric refraction to coordinates.
*/
#[get("/telescope/<device_number>/doesrefraction")]
fn get_telescope_doesrefraction(
    schemas::GetTelescopeDoesrefractionPath { device_number }: schemas::GetTelescopeDoesrefractionPath,

    schemas::GetTelescopeDoesrefractionQuery { client_id, client_transaction_id }: schemas::GetTelescopeDoesrefractionQuery,
) -> schemas::BoolResponse {
}

/**
Determines whether atmospheric refraction is applied to coordinates.

Causes the rotator to move Position degrees relative to the current Position value.
*/
#[put("/telescope/<device_number>/doesrefraction")]
fn put_telescope_doesrefraction(
    schemas::PutTelescopeDoesrefractionPath { device_number }: schemas::PutTelescopeDoesrefractionPath,

    schemas::PutTelescopeDoesrefractionRequest {
        does_refraction,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeDoesrefractionRequest,
) -> schemas::MethodResponse {
}

/**
Returns the current equatorial coordinate system used by this telescope.

Returns the current equatorial coordinate system used by this telescope (e.g. Topocentric or J2000).
*/
#[get("/telescope/<device_number>/equatorialsystem")]
fn get_telescope_equatorialsystem(
    schemas::GetTelescopeEquatorialsystemPath { device_number }: schemas::GetTelescopeEquatorialsystemPath,

    schemas::GetTelescopeEquatorialsystemQuery { client_id, client_transaction_id }: schemas::GetTelescopeEquatorialsystemQuery,
) -> schemas::IntResponse {
}

/**
Returns the telescope's focal length in meters.

The telescope's focal length in meters
*/
#[get("/telescope/<device_number>/focallength")]
fn get_telescope_focallength(
    schemas::GetTelescopeFocallengthPath { device_number }: schemas::GetTelescopeFocallengthPath,

    schemas::GetTelescopeFocallengthQuery { client_id, client_transaction_id }: schemas::GetTelescopeFocallengthQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the  current Declination rate offset for telescope guiding

The current Declination movement rate offset for telescope guiding (degrees/sec)
*/
#[get("/telescope/<device_number>/guideratedeclination")]
fn get_telescope_guideratedeclination(
    schemas::GetTelescopeGuideratedeclinationPath { device_number }: schemas::GetTelescopeGuideratedeclinationPath,

    schemas::GetTelescopeGuideratedeclinationQuery { client_id, client_transaction_id }: schemas::GetTelescopeGuideratedeclinationQuery,
) -> schemas::DoubleResponse {
}

/**
Sets the  current Declination rate offset for telescope guiding.

Sets the current Declination movement rate offset for telescope guiding (degrees/sec).
*/
#[put("/telescope/<device_number>/guideratedeclination")]
fn put_telescope_guideratedeclination(
    schemas::PutTelescopeGuideratedeclinationPath { device_number }: schemas::PutTelescopeGuideratedeclinationPath,

    schemas::PutTelescopeGuideratedeclinationRequest {
        guide_rate_declination,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeGuideratedeclinationRequest,
) -> schemas::MethodResponse {
}

/**
Returns the  current RightAscension rate offset for telescope guiding

The current RightAscension movement rate offset for telescope guiding (degrees/sec)
*/
#[get("/telescope/<device_number>/guideraterightascension")]
fn get_telescope_guideraterightascension(
    schemas::GetTelescopeGuideraterightascensionPath { device_number }: schemas::GetTelescopeGuideraterightascensionPath,

    schemas::GetTelescopeGuideraterightascensionQuery { client_id, client_transaction_id }: schemas::GetTelescopeGuideraterightascensionQuery,
) -> schemas::DoubleResponse {
}

/**
Sets the  current RightAscension rate offset for telescope guiding.

Sets the current RightAscension movement rate offset for telescope guiding (degrees/sec).
*/
#[put("/telescope/<device_number>/guideraterightascension")]
fn put_telescope_guideraterightascension(
    schemas::PutTelescopeGuideraterightascensionPath { device_number }: schemas::PutTelescopeGuideraterightascensionPath,

    schemas::PutTelescopeGuideraterightascensionRequest {
        guide_rate_right_ascension,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeGuideraterightascensionRequest,
) -> schemas::MethodResponse {
}

/**
Indicates whether the telescope is currently executing a PulseGuide command

True if a PulseGuide(GuideDirections, Int32) command is in progress, False otherwise
*/
#[get("/telescope/<device_number>/ispulseguiding")]
fn get_telescope_ispulseguiding(
    schemas::GetTelescopeIspulseguidingPath { device_number }: schemas::GetTelescopeIspulseguidingPath,

    schemas::GetTelescopeIspulseguidingQuery { client_id, client_transaction_id }: schemas::GetTelescopeIspulseguidingQuery,
) -> schemas::BoolResponse {
}

/**
Returns the mount's right ascension coordinate.

The right ascension (hours) of the mount's current equatorial coordinates, in the coordinate system given by the EquatorialSystem property
*/
#[get("/telescope/<device_number>/rightascension")]
fn get_telescope_rightascension(
    schemas::GetTelescopeRightascensionPath { device_number }: schemas::GetTelescopeRightascensionPath,

    schemas::GetTelescopeRightascensionQuery { client_id, client_transaction_id }: schemas::GetTelescopeRightascensionQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the telescope's right ascension tracking rate.

The right ascension tracking rate (arcseconds per second, default = 0.0)
*/
#[get("/telescope/<device_number>/rightascensionrate")]
fn get_telescope_rightascensionrate(
    schemas::GetTelescopeRightascensionratePath { device_number }: schemas::GetTelescopeRightascensionratePath,

    schemas::GetTelescopeRightascensionrateQuery { client_id, client_transaction_id }: schemas::GetTelescopeRightascensionrateQuery,
) -> schemas::DoubleResponse {
}

/**
Sets the telescope's right ascension tracking rate.

Sets the right ascension tracking rate (arcseconds per second)
*/
#[put("/telescope/<device_number>/rightascensionrate")]
fn put_telescope_rightascensionrate(
    schemas::PutTelescopeRightascensionratePath { device_number }: schemas::PutTelescopeRightascensionratePath,

    schemas::PutTelescopeRightascensionrateRequest {
        right_ascension_rate,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeRightascensionrateRequest,
) -> schemas::MethodResponse {
}

/**
Returns the mount's pointing state.

Indicates the pointing state of the mount. 0 = pierEast, 1 = pierWest, -1= pierUnknown
*/
#[get("/telescope/<device_number>/sideofpier")]
fn get_telescope_sideofpier(
    schemas::GetTelescopeSideofpierPath { device_number }: schemas::GetTelescopeSideofpierPath,

    schemas::GetTelescopeSideofpierQuery { client_id, client_transaction_id }: schemas::GetTelescopeSideofpierQuery,
) -> schemas::IntResponse {
}

/**
Sets the mount's pointing state.

Sets the pointing state of the mount. 0 = pierEast, 1 = pierWest
*/
#[put("/telescope/<device_number>/sideofpier")]
fn put_telescope_sideofpier(
    schemas::PutTelescopeSideofpierPath { device_number }: schemas::PutTelescopeSideofpierPath,

    schemas::PutTelescopeSideofpierRequest {
        side_of_pier,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeSideofpierRequest,
) -> schemas::MethodResponse {
}

/**
Returns the local apparent sidereal time.

The local apparent sidereal time from the telescope's internal clock (hours, sidereal).
*/
#[get("/telescope/<device_number>/siderealtime")]
fn get_telescope_siderealtime(
    schemas::GetTelescopeSiderealtimePath { device_number }: schemas::GetTelescopeSiderealtimePath,

    schemas::GetTelescopeSiderealtimeQuery { client_id, client_transaction_id }: schemas::GetTelescopeSiderealtimeQuery,
) -> schemas::DoubleResponse {
}

/**
Returns the observing site's elevation above mean sea level.

The elevation above mean sea level (meters) of the site at which the telescope is located.
*/
#[get("/telescope/<device_number>/siteelevation")]
fn get_telescope_siteelevation(
    schemas::GetTelescopeSiteelevationPath { device_number }: schemas::GetTelescopeSiteelevationPath,

    schemas::GetTelescopeSiteelevationQuery { client_id, client_transaction_id }: schemas::GetTelescopeSiteelevationQuery,
) -> schemas::DoubleResponse {
}

/**
Sets the observing site's elevation above mean sea level.

Sets the elevation above mean sea level (metres) of the site at which the telescope is located.
*/
#[put("/telescope/<device_number>/siteelevation")]
fn put_telescope_siteelevation(
    schemas::PutTelescopeSiteelevationPath { device_number }: schemas::PutTelescopeSiteelevationPath,

    schemas::PutTelescopeSiteelevationRequest {
        site_elevation,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeSiteelevationRequest,
) -> schemas::MethodResponse {
}

/**
Returns the observing site's latitude.

The geodetic(map) latitude (degrees, positive North, WGS84) of the site at which the telescope is located.
*/
#[get("/telescope/<device_number>/sitelatitude")]
fn get_telescope_sitelatitude(
    schemas::GetTelescopeSitelatitudePath { device_number }: schemas::GetTelescopeSitelatitudePath,

    schemas::GetTelescopeSitelatitudeQuery { client_id, client_transaction_id }: schemas::GetTelescopeSitelatitudeQuery,
) -> schemas::DoubleResponse {
}

/**
Sets the observing site's latitude.

Sets the observing site's latitude (degrees).
*/
#[put("/telescope/<device_number>/sitelatitude")]
fn put_telescope_sitelatitude(
    schemas::PutTelescopeSitelatitudePath { device_number }: schemas::PutTelescopeSitelatitudePath,

    schemas::PutTelescopeSitelatitudeRequest {
        site_latitude,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeSitelatitudeRequest,
) -> schemas::MethodResponse {
}

/**
Returns the observing site's longitude.

The longitude (degrees, positive East, WGS84) of the site at which the telescope is located.
*/
#[get("/telescope/<device_number>/sitelongitude")]
fn get_telescope_sitelongitude(
    schemas::GetTelescopeSitelongitudePath { device_number }: schemas::GetTelescopeSitelongitudePath,

    schemas::GetTelescopeSitelongitudeQuery { client_id, client_transaction_id }: schemas::GetTelescopeSitelongitudeQuery,
) -> schemas::DoubleResponse {
}

/**
Sets the observing site's longitude.

Sets the observing site's longitude (degrees, positive East, WGS84).
*/
#[put("/telescope/<device_number>/sitelongitude")]
fn put_telescope_sitelongitude(
    schemas::PutTelescopeSitelongitudePath { device_number }: schemas::PutTelescopeSitelongitudePath,

    schemas::PutTelescopeSitelongitudeRequest {
        site_longitude,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeSitelongitudeRequest,
) -> schemas::MethodResponse {
}

/**
Indicates whether the telescope is currently slewing.

True if telescope is currently moving in response to one of the Slew methods or the MoveAxis(TelescopeAxes, Double) method, False at all other times.
*/
#[get("/telescope/<device_number>/slewing")]
fn get_telescope_slewing(
    schemas::GetTelescopeSlewingPath { device_number }: schemas::GetTelescopeSlewingPath,

    schemas::GetTelescopeSlewingQuery { client_id, client_transaction_id }: schemas::GetTelescopeSlewingQuery,
) -> schemas::BoolResponse {
}

/**
Returns the post-slew settling time.

Returns the post-slew settling time (sec.).
*/
#[get("/telescope/<device_number>/slewsettletime")]
fn get_telescope_slewsettletime(
    schemas::GetTelescopeSlewsettletimePath { device_number }: schemas::GetTelescopeSlewsettletimePath,

    schemas::GetTelescopeSlewsettletimeQuery { client_id, client_transaction_id }: schemas::GetTelescopeSlewsettletimeQuery,
) -> schemas::IntResponse {
}

/**
Sets the post-slew settling time.

Sets the  post-slew settling time (integer sec.).
*/
#[put("/telescope/<device_number>/slewsettletime")]
fn put_telescope_slewsettletime(
    schemas::PutTelescopeSlewsettletimePath { device_number }: schemas::PutTelescopeSlewsettletimePath,

    schemas::PutTelescopeSlewsettletimeRequest {
        slew_settle_time,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeSlewsettletimeRequest,
) -> schemas::MethodResponse {
}

/**
Returns the current target declination.

The declination (degrees, positive North) for the target of an equatorial slew or sync operation
*/
#[get("/telescope/<device_number>/targetdeclination")]
fn get_telescope_targetdeclination(
    schemas::GetTelescopeTargetdeclinationPath { device_number }: schemas::GetTelescopeTargetdeclinationPath,

    schemas::GetTelescopeTargetdeclinationQuery { client_id, client_transaction_id }: schemas::GetTelescopeTargetdeclinationQuery,
) -> schemas::DoubleResponse {
}

/**
Sets the target declination of a slew or sync.

Sets the declination (degrees, positive North) for the target of an equatorial slew or sync operation
*/
#[put("/telescope/<device_number>/targetdeclination")]
fn put_telescope_targetdeclination(
    schemas::PutTelescopeTargetdeclinationPath { device_number }: schemas::PutTelescopeTargetdeclinationPath,

    schemas::PutTelescopeTargetdeclinationRequest {
        target_declination,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeTargetdeclinationRequest,
) -> schemas::MethodResponse {
}

/**
Returns the current target right ascension.

The right ascension (hours) for the target of an equatorial slew or sync operation
*/
#[get("/telescope/<device_number>/targetrightascension")]
fn get_telescope_targetrightascension(
    schemas::GetTelescopeTargetrightascensionPath { device_number }: schemas::GetTelescopeTargetrightascensionPath,

    schemas::GetTelescopeTargetrightascensionQuery { client_id, client_transaction_id }: schemas::GetTelescopeTargetrightascensionQuery,
) -> schemas::DoubleResponse {
}

/**
Sets the target right ascension of a slew or sync.

Sets the right ascension (hours) for the target of an equatorial slew or sync operation
*/
#[put("/telescope/<device_number>/targetrightascension")]
fn put_telescope_targetrightascension(
    schemas::PutTelescopeTargetrightascensionPath { device_number }: schemas::PutTelescopeTargetrightascensionPath,

    schemas::PutTelescopeTargetrightascensionRequest {
        target_right_ascension,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeTargetrightascensionRequest,
) -> schemas::MethodResponse {
}

/**
Indicates whether the telescope is tracking.

Returns the state of the telescope's sidereal tracking drive.
*/
#[get("/telescope/<device_number>/tracking")]
fn get_telescope_tracking(
    schemas::GetTelescopeTrackingPath { device_number }: schemas::GetTelescopeTrackingPath,

    schemas::GetTelescopeTrackingQuery { client_id, client_transaction_id }: schemas::GetTelescopeTrackingQuery,
) -> schemas::BoolResponse {
}

/**
Enables or disables telescope tracking.

Sets the state of the telescope's sidereal tracking drive.
*/
#[put("/telescope/<device_number>/tracking")]
fn put_telescope_tracking(
    schemas::PutTelescopeTrackingPath { device_number }: schemas::PutTelescopeTrackingPath,

    schemas::PutTelescopeTrackingRequest {
        tracking,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeTrackingRequest,
) -> schemas::MethodResponse {
}

/**
Returns the current tracking rate.

The current tracking rate of the telescope's sidereal drive.
*/
#[get("/telescope/<device_number>/trackingrate")]
fn get_telescope_trackingrate(
    schemas::GetTelescopeTrackingratePath { device_number }: schemas::GetTelescopeTrackingratePath,

    schemas::GetTelescopeTrackingrateQuery { client_id, client_transaction_id }: schemas::GetTelescopeTrackingrateQuery,
) -> schemas::IntResponse {
}

/**
Sets the mount's tracking rate.

Sets the tracking rate of the telescope's sidereal drive. 0 = driveSidereal, 1 = driveLunar, 2 = driveSolar, 3 = driveKing
*/
#[put("/telescope/<device_number>/trackingrate")]
fn put_telescope_trackingrate(
    schemas::PutTelescopeTrackingratePath { device_number }: schemas::PutTelescopeTrackingratePath,

    schemas::PutTelescopeTrackingrateRequest {
        tracking_rate,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeTrackingrateRequest,
) -> schemas::MethodResponse {
}

/**
Returns a collection of supported DriveRates values.

Returns an array of supported DriveRates values that describe the permissible values of the TrackingRate property for this telescope type.
*/
#[get("/telescope/<device_number>/trackingrates")]
fn get_telescope_trackingrates(
    schemas::GetTelescopeTrackingratesPath { device_number }: schemas::GetTelescopeTrackingratesPath,

    schemas::GetTelescopeTrackingratesQuery { client_id, client_transaction_id }: schemas::GetTelescopeTrackingratesQuery,
) -> schemas::DriveRatesResponse {
}

/**
Returns the UTC date/time of the telescope's internal clock.

The UTC date/time of the telescope's internal clock in ISO 8601 format including fractional seconds. The general format (in Microsoft custom date format style) is yyyy-MM-ddTHH:mm:ss.fffffffZ E.g. 2016-03-04T17:45:31.1234567Z or 2016-11-14T07:03:08.1234567Z Please note the compulsary trailing Z indicating the 'Zulu', UTC time zone.
*/
#[get("/telescope/<device_number>/utcdate")]
fn get_telescope_utcdate(
    schemas::GetTelescopeUtcdatePath { device_number }: schemas::GetTelescopeUtcdatePath,

    schemas::GetTelescopeUtcdateQuery { client_id, client_transaction_id }: schemas::GetTelescopeUtcdateQuery,
) -> schemas::StringResponse {
}

/**
Sets the UTC date/time of the telescope's internal clock.

The UTC date/time of the telescope's internal clock in ISO 8601 format including fractional seconds. The general format (in Microsoft custom date format style) is yyyy-MM-ddTHH:mm:ss.fffffffZ E.g. 2016-03-04T17:45:31.1234567Z or 2016-11-14T07:03:08.1234567Z Please note the compulsary trailing Z indicating the 'Zulu', UTC time zone.
*/
#[put("/telescope/<device_number>/utcdate")]
fn put_telescope_utcdate(
    schemas::PutTelescopeUtcdatePath { device_number }: schemas::PutTelescopeUtcdatePath,

    schemas::PutTelescopeUtcdateRequest {
        utcdate,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeUtcdateRequest,
) -> schemas::MethodResponse {
}

/**
Immediatley stops a slew in progress.

Immediately Stops a slew in progress.
*/
#[put("/telescope/<device_number>/abortslew")]
fn put_telescope_abortslew(
    schemas::PutTelescopeAbortslewPath { device_number }: schemas::PutTelescopeAbortslewPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Returns the rates at which the telescope may be moved about the specified axis.

The rates at which the telescope may be moved about the specified axis by the MoveAxis(TelescopeAxes, Double) method.
*/
#[get("/telescope/<device_number>/axisrates")]
fn get_telescope_axisrates(
    schemas::GetTelescopeAxisratesPath { device_number }: schemas::GetTelescopeAxisratesPath,

    schemas::GetTelescopeAxisratesQuery {
        client_id,

        client_transaction_id,

        axis,
    }: schemas::GetTelescopeAxisratesQuery,
) -> schemas::AxisRatesResponse {
}

/**
Indicates whether the telescope can move the requested axis.

True if this telescope can move the requested axis.
*/
#[get("/telescope/<device_number>/canmoveaxis")]
fn get_telescope_canmoveaxis(
    schemas::GetTelescopeCanmoveaxisPath { device_number }: schemas::GetTelescopeCanmoveaxisPath,

    schemas::GetTelescopeCanmoveaxisQuery {
        axis,

        client_id,

        client_transaction_id,
    }: schemas::GetTelescopeCanmoveaxisQuery,
) -> schemas::BoolResponse {
}

/**
Predicts the pointing state after a German equatorial mount slews to given coordinates.

Predicts the pointing state that a German equatorial mount will be in if it slews to the given coordinates. The  return value will be one of - 0 = pierEast, 1 = pierWest, -1 = pierUnknown
*/
#[get("/telescope/<device_number>/destinationsideofpier")]
fn get_telescope_destinationsideofpier(
    schemas::GetTelescopeDestinationsideofpierPath { device_number }: schemas::GetTelescopeDestinationsideofpierPath,

    schemas::GetTelescopeDestinationsideofpierQuery {
        right_ascension,

        declination,

        client_id,

        client_transaction_id,
    }: schemas::GetTelescopeDestinationsideofpierQuery,
) -> schemas::IntResponse {
}

/**
Moves the mount to the "home" position.

Locates the telescope's "home" position (synchronous)
*/
#[put("/telescope/<device_number>/findhome")]
fn put_telescope_findhome(
    schemas::PutTelescopeFindhomePath { device_number }: schemas::PutTelescopeFindhomePath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Moves a telescope axis at the given rate.

Move the telescope in one axis at the given rate.
*/
#[put("/telescope/<device_number>/moveaxis")]
fn put_telescope_moveaxis(
    schemas::PutTelescopeMoveaxisPath { device_number }: schemas::PutTelescopeMoveaxisPath,

    schemas::PutTelescopeMoveaxisRequest {
        axis,

        rate,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeMoveaxisRequest,
) -> schemas::MethodResponse {
}

/**
Park the mount

Move the telescope to its park position, stop all motion (or restrict to a small safe range), and set AtPark to True. )
*/
#[put("/telescope/<device_number>/park")]
fn put_telescope_park(
    schemas::PutTelescopeParkPath { device_number }: schemas::PutTelescopeParkPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Moves the scope in the given direction for the given time.

Moves the scope in the given direction for the given interval or time at the rate given by the corresponding guide rate property
*/
#[put("/telescope/<device_number>/pulseguide")]
fn put_telescope_pulseguide(
    schemas::PutTelescopePulseguidePath { device_number }: schemas::PutTelescopePulseguidePath,

    schemas::PutTelescopePulseguideRequest {
        direction,

        duration,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopePulseguideRequest,
) -> schemas::MethodResponse {
}

/**
Sets the telescope's park position

Sets the telescope's park position to be its current position.
*/
#[put("/telescope/<device_number>/setpark")]
fn put_telescope_setpark(
    schemas::PutTelescopeSetparkPath { device_number }: schemas::PutTelescopeSetparkPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Synchronously slew to the given local horizontal coordinates.

Move the telescope to the given local horizontal coordinates, return when slew is complete
*/
#[put("/telescope/<device_number>/slewtoaltaz")]
fn put_telescope_slewtoaltaz(
    schemas::PutTelescopeSlewtoaltazPath { device_number }: schemas::PutTelescopeSlewtoaltazPath,

    schemas::PutTelescopeSlewtoaltazRequest {
        azimuth,

        altitude,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeSlewtoaltazRequest,
) -> schemas::MethodResponse {
}

/**
Asynchronously slew to the given local horizontal coordinates.

Move the telescope to the given local horizontal coordinates, return immediatley after the slew starts. The client can poll the Slewing method to determine when the mount reaches the intended coordinates.
*/
#[put("/telescope/<device_number>/slewtoaltazasync")]
fn put_telescope_slewtoaltazasync(
    schemas::PutTelescopeSlewtoaltazasyncPath { device_number }: schemas::PutTelescopeSlewtoaltazasyncPath,

    schemas::PutTelescopeSlewtoaltazRequest {
        azimuth,

        altitude,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeSlewtoaltazRequest,
) -> schemas::MethodResponse {
}

/**
Synchronously slew to the given equatorial coordinates.

Move the telescope to the given equatorial coordinates, return when slew is complete
*/
#[put("/telescope/<device_number>/slewtocoordinates")]
fn put_telescope_slewtocoordinates(
    schemas::PutTelescopeSlewtocoordinatesPath { device_number }: schemas::PutTelescopeSlewtocoordinatesPath,

    schemas::PutTelescopeSlewtocoordinatesRequest {
        right_ascension,

        declination,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeSlewtocoordinatesRequest,
) -> schemas::MethodResponse {
}

/**
Asynchronously slew to the given equatorial coordinates.

Move the telescope to the given equatorial coordinates, return immediatley after the slew starts. The client can poll the Slewing method to determine when the mount reaches the intended coordinates.
*/
#[put("/telescope/<device_number>/slewtocoordinatesasync")]
fn put_telescope_slewtocoordinatesasync(
    schemas::PutTelescopeSlewtocoordinatesasyncPath { device_number }: schemas::PutTelescopeSlewtocoordinatesasyncPath,

    schemas::PutTelescopeSlewtocoordinatesRequest {
        right_ascension,

        declination,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeSlewtocoordinatesRequest,
) -> schemas::MethodResponse {
}

/**
Synchronously slew to the TargetRightAscension and TargetDeclination coordinates.

Move the telescope to the TargetRightAscension and TargetDeclination equatorial coordinates, return when slew is complete
*/
#[put("/telescope/<device_number>/slewtotarget")]
fn put_telescope_slewtotarget(
    schemas::PutTelescopeSlewtotargetPath { device_number }: schemas::PutTelescopeSlewtotargetPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Asynchronously slew to the TargetRightAscension and TargetDeclination coordinates.

Move the telescope to the TargetRightAscension and TargetDeclination equatorial coordinates, return immediatley after the slew starts. The client can poll the Slewing method to determine when the mount reaches the intended coordinates.
*/
#[put("/telescope/<device_number>/slewtotargetasync")]
fn put_telescope_slewtotargetasync(
    schemas::PutTelescopeSlewtotargetasyncPath { device_number }: schemas::PutTelescopeSlewtotargetasyncPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Syncs to the given local horizontal coordinates.

Matches the scope's local horizontal coordinates to the given local horizontal coordinates.
*/
#[put("/telescope/<device_number>/synctoaltaz")]
fn put_telescope_synctoaltaz(
    schemas::PutTelescopeSynctoaltazPath { device_number }: schemas::PutTelescopeSynctoaltazPath,

    schemas::PutTelescopeSlewtoaltazRequest {
        azimuth,

        altitude,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeSlewtoaltazRequest,
) -> schemas::MethodResponse {
}

/**
Syncs to the given equatorial coordinates.

Matches the scope's equatorial coordinates to the given equatorial coordinates.
*/
#[put("/telescope/<device_number>/synctocoordinates")]
fn put_telescope_synctocoordinates(
    schemas::PutTelescopeSynctocoordinatesPath { device_number }: schemas::PutTelescopeSynctocoordinatesPath,

    schemas::PutTelescopeSlewtocoordinatesRequest {
        right_ascension,

        declination,

        client_id,

        client_transaction_id,
    }: schemas::PutTelescopeSlewtocoordinatesRequest,
) -> schemas::MethodResponse {
}

/**
Syncs to the TargetRightAscension and TargetDeclination coordinates.

Matches the scope's equatorial coordinates to the TargetRightAscension and TargetDeclination equatorial coordinates.
*/
#[put("/telescope/<device_number>/synctotarget")]
fn put_telescope_synctotarget(
    schemas::PutTelescopeSynctotargetPath { device_number }: schemas::PutTelescopeSynctotargetPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
}

/**
Unparks the mount.

Takes telescope out of the Parked state. )
*/
#[put("/telescope/<device_number>/unpark")]
fn put_telescope_unpark(
    schemas::PutTelescopeUnparkPath { device_number }: schemas::PutTelescopeUnparkPath,

    schemas::PutCameraAbortexposureRequest { client_id, client_transaction_id }: schemas::PutCameraAbortexposureRequest,
) -> schemas::MethodResponse {
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
