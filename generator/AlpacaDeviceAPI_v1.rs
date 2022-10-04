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

mod schemas {

    #[derive(Serialize)]

    struct ImageArrayResponse {
        /**
        0 = Unknown, 1 = Short(int16), 2 = Integer (int32), 3 = Double (Double precision real number).
        */
        #[serde(rename = "Type")]
        type_: i32,

        /**
        The array's rank, will be 2 (single plane image (monochrome)) or 3 (multi-plane image).
        */
        #[serde(rename = "Rank")]
        rank: i32,

        /**
        Returned integer or double value
        */
        #[serde(rename = "Value")]
        value: Vec<Vec<f64>>,

        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: u32,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: u32,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: i32,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: String,
    }

    #[derive(Serialize)]

    struct BoolResponse {
        /**
        True or False value
        */
        #[serde(rename = "Value")]
        value: bool,

        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: u32,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: u32,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: i32,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered. If an error message is returned, a non zero error number must also be returned.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: String,
    }

    #[derive(Serialize)]

    struct DoubleResponse {
        /**
        Returned double value
        */
        #[serde(rename = "Value")]
        value: f64,

        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: u32,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: u32,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: i32,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered. If an error message is returned, a non zero error number must also be returned.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: String,
    }

    #[derive(Serialize)]

    struct IntResponse {
        /**
        Returned integer value
        */
        #[serde(rename = "Value")]
        value: i32,

        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: u32,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: u32,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: i32,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered. If an error message is returned, a non zero error number must also be returned.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: String,
    }

    #[derive(Serialize)]

    struct IntArrayResponse {
        /**
        Array of integer values.
        */
        #[serde(rename = "Value")]
        value: Vec<i32>,

        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: u32,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: u32,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: i32,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered. If an error message is returned, a non zero error number must also be returned.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: String,
    }

    #[derive(Serialize)]

    struct MethodResponse {
        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: u32,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: u32,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: i32,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered. If an error message is returned, a non zero error number must also be returned.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: String,
    }

    #[derive(Serialize)]

    struct StringResponse {
        /**
        String response from the device.
        */
        #[serde(rename = "Value")]
        value: String,

        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: u32,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: u32,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: i32,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered. If an error message is returned, a non zero error number must also be returned.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: String,
    }

    #[derive(Serialize)]

    struct StringArrayResponse {
        /**
        Array of string values.
        */
        #[serde(rename = "Value")]
        value: Vec<String>,

        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: u32,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: u32,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: i32,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered. If an error message is returned, a non zero error number must also be returned.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: String,
    }

    #[derive(Serialize)]

    struct AxisRatesResponse {
        /**
        Array of AxisRate objects
        */
        #[serde(rename = "Value")]
        value: Vec<()>,

        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: u32,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: u32,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: i32,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered. If an error message is returned, a non zero error number must also be returned.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: String,
    }

    #[derive(Serialize)]

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
        value: Vec<f64>,

        /**
        Client's transaction ID (0 to 4294967295), as supplied by the client in the command request.
        */
        #[serde(rename = "ClientTransactionID")]
        client_transaction_id: u32,

        /**
        Server's transaction ID (0 to 4294967295), should be unique for each client transaction so that log messages on the client can be associated with logs on the device.
        */
        #[serde(rename = "ServerTransactionID")]
        server_transaction_id: u32,

        /**
        Zero for a successful transaction, or a non-zero integer (-2147483648 to 2147483647) if the device encountered an issue. Devices must use ASCOM reserved error numbers whenever appropriate so that clients can take informed actions. E.g. returning 0x401 (1025) to indicate that an invalid value was received (see Alpaca API definition and developer documentation for further information).
        */
        #[serde(rename = "ErrorNumber")]
        error_number: i32,

        /**
        Empty string for a successful transaction, or a message describing the issue that was encountered. If an error message is returned, a non zero error number must also be returned.
        */
        #[serde(rename = "ErrorMessage")]
        error_message: String,
    }

    #[derive(Serialize)]

    struct DriveRate(f64);
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutActionPathParams {
    /**
    One of the recognised ASCOM device types e.g. telescope (must be lower case)
    */
    #[serde(rename = "device_type")]
    device_type: String,

    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutActionBodyParams {
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
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
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
    PutActionPathParams { device_type, device_number }: PutActionPathParams,

    PutActionBodyParams {
        action,

        parameters,

        client_id,

        client_transaction_id,
    }: PutActionBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCommandblindPathParams {
    /**
    One of the recognised ASCOM device types e.g. telescope (must be lower case)
    */
    #[serde(rename = "device_type")]
    device_type: String,

    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCommandblindBodyParams {
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
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Transmits an arbitrary string to the device

Transmits an arbitrary string to the device and does not wait for a response. Optionally, protocol framing characters may be added to the string before transmission.
*/
#[put("/<device_type>/<device_number>/commandblind")]
fn put_commandblind(
    PutCommandblindPathParams { device_type, device_number }: PutCommandblindPathParams,

    PutCommandblindBodyParams {
        command,

        raw,

        client_id,

        client_transaction_id,
    }: PutCommandblindBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCommandboolPathParams {
    /**
    One of the recognised ASCOM device types e.g. telescope (must be lower case)
    */
    #[serde(rename = "device_type")]
    device_type: String,

    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCommandboolBodyParams {
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
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Transmits an arbitrary string to the device and returns a boolean value from the device.

Transmits an arbitrary string to the device and waits for a boolean response. Optionally, protocol framing characters may be added to the string before transmission.
*/
#[put("/<device_type>/<device_number>/commandbool")]
fn put_commandbool(
    PutCommandboolPathParams { device_type, device_number }: PutCommandboolPathParams,

    PutCommandboolBodyParams {
        command,

        raw,

        client_id,

        client_transaction_id,
    }: PutCommandboolBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCommandstringPathParams {
    /**
    One of the recognised ASCOM device types e.g. telescope (must be lower case)
    */
    #[serde(rename = "device_type")]
    device_type: String,

    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCommandstringBodyParams {
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
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Transmits an arbitrary string to the device and returns a string value from the device.

Transmits an arbitrary string to the device and waits for a string response. Optionally, protocol framing characters may be added to the string before transmission.
*/
#[put("/<device_type>/<device_number>/commandstring")]
fn put_commandstring(
    PutCommandstringPathParams { device_type, device_number }: PutCommandstringPathParams,

    PutCommandstringBodyParams {
        command,

        raw,

        client_id,

        client_transaction_id,
    }: PutCommandstringBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetConnectedPathParams {
    /**
    One of the recognised ASCOM device types e.g. telescope (must be lower case)
    */
    #[serde(rename = "device_type")]
    device_type: String,

    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetConnectedQueryParams {
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

/**
Retrieves the connected state of the device

Retrieves the connected state of the device
*/
#[get("/<device_type>/<device_number>/connected")]
fn get_connected(GetConnectedPathParams { device_type, device_number }: GetConnectedPathParams, GetConnectedQueryParams { client_id, client_transaction_id }: GetConnectedQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutConnectedPathParams {
    /**
    One of the recognised ASCOM device types e.g. telescope (must be lower case)
    */
    #[serde(rename = "device_type")]
    device_type: String,

    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutConnectedBodyParams {
    /**
    Set True to connect to the device hardware, set False to disconnect from the device hardware
    */
    #[serde(rename = "Connected")]
    connected: bool,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the connected state of the device

Sets the connected state of the device
*/
#[put("/<device_type>/<device_number>/connected")]
fn put_connected(
    PutConnectedPathParams { device_type, device_number }: PutConnectedPathParams,

    PutConnectedBodyParams {
        connected,

        client_id,

        client_transaction_id,
    }: PutConnectedBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetDescriptionPathParams {
    /**
    One of the recognised ASCOM device types e.g. telescope (must be lower case)
    */
    #[serde(rename = "device_type")]
    device_type: String,

    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetDescriptionQueryParams {
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

/**
Device description

The description of the device
*/
#[get("/<device_type>/<device_number>/description")]
fn get_description(GetDescriptionPathParams { device_type, device_number }: GetDescriptionPathParams, GetDescriptionQueryParams { client_id, client_transaction_id }: GetDescriptionQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetDriverinfoPathParams {
    /**
    One of the recognised ASCOM device types e.g. telescope (must be lower case)
    */
    #[serde(rename = "device_type")]
    device_type: String,

    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetDriverinfoQueryParams {
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

/**
Device driver description

The description of the driver
*/
#[get("/<device_type>/<device_number>/driverinfo")]
fn get_driverinfo(GetDriverinfoPathParams { device_type, device_number }: GetDriverinfoPathParams, GetDriverinfoQueryParams { client_id, client_transaction_id }: GetDriverinfoQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetDriverversionPathParams {
    /**
    One of the recognised ASCOM device types e.g. telescope (must be lower case)
    */
    #[serde(rename = "device_type")]
    device_type: String,

    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetDriverversionQueryParams {
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

/**
Driver Version

A string containing only the major and minor version of the driver.
*/
#[get("/<device_type>/<device_number>/driverversion")]
fn get_driverversion(
    GetDriverversionPathParams { device_type, device_number }: GetDriverversionPathParams,

    GetDriverversionQueryParams { client_id, client_transaction_id }: GetDriverversionQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetInterfaceversionPathParams {
    /**
    One of the recognised ASCOM device types e.g. telescope (must be lower case)
    */
    #[serde(rename = "device_type")]
    device_type: String,

    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetInterfaceversionQueryParams {
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

/**
The ASCOM Device interface version number that this device supports.

This method returns the version of the ASCOM device interface contract to which this device complies. Only one interface version is current at a moment in time and all new devices should be built to the latest interface version. Applications can choose which device interface versions they support and it is in their interest to support  previous versions as well as the current version to ensure thay can use the largest number of devices.
*/
#[get("/<device_type>/<device_number>/interfaceversion")]
fn get_interfaceversion(
    GetInterfaceversionPathParams { device_type, device_number }: GetInterfaceversionPathParams,

    GetInterfaceversionQueryParams { client_id, client_transaction_id }: GetInterfaceversionQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetNamePathParams {
    /**
    One of the recognised ASCOM device types e.g. telescope (must be lower case)
    */
    #[serde(rename = "device_type")]
    device_type: String,

    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetNameQueryParams {
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

/**
Device name

The name of the device
*/
#[get("/<device_type>/<device_number>/name")]
fn get_name(GetNamePathParams { device_type, device_number }: GetNamePathParams, GetNameQueryParams { client_id, client_transaction_id }: GetNameQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetSupportedactionsPathParams {
    /**
    One of the recognised ASCOM device types e.g. telescope (must be lower case)
    */
    #[serde(rename = "device_type")]
    device_type: String,

    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetSupportedactionsQueryParams {
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

/**
Returns the list of action names supported by this driver.

Returns the list of action names supported by this driver.
*/
#[get("/<device_type>/<device_number>/supportedactions")]
fn get_supportedactions(
    GetSupportedactionsPathParams { device_type, device_number }: GetSupportedactionsPathParams,

    GetSupportedactionsQueryParams { client_id, client_transaction_id }: GetSupportedactionsQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraBayeroffsetxPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraBayeroffsetxQueryParams {
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

/**
Returns the X offset of the Bayer matrix.

Returns the X offset of the Bayer matrix, as defined in SensorType.
*/
#[get("/camera/<device_number>/bayeroffsetx")]
fn get_camera_bayeroffsetx(
    GetCameraBayeroffsetxPathParams { device_number }: GetCameraBayeroffsetxPathParams,

    GetCameraBayeroffsetxQueryParams { client_id, client_transaction_id }: GetCameraBayeroffsetxQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraBayeroffsetyPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraBayeroffsetyQueryParams {
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

/**
Returns the Y offset of the Bayer matrix.

Returns the Y offset of the Bayer matrix, as defined in SensorType.
*/
#[get("/camera/<device_number>/bayeroffsety")]
fn get_camera_bayeroffsety(
    GetCameraBayeroffsetyPathParams { device_number }: GetCameraBayeroffsetyPathParams,

    GetCameraBayeroffsetyQueryParams { client_id, client_transaction_id }: GetCameraBayeroffsetyQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraBinxPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraBinxQueryParams {
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

/**
Returns the binning factor for the X axis.

Returns the binning factor for the X axis.
*/
#[get("/camera/<device_number>/binx")]
fn get_camera_binx(GetCameraBinxPathParams { device_number }: GetCameraBinxPathParams, GetCameraBinxQueryParams { client_id, client_transaction_id }: GetCameraBinxQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCameraBinxPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCameraBinxBodyParams {
    /**
    The X binning value
    */
    #[serde(rename = "BinX")]
    bin_x: i32,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the binning factor for the X axis.

Sets the binning factor for the X axis.
*/
#[put("/camera/<device_number>/binx")]
fn put_camera_binx(
    PutCameraBinxPathParams { device_number }: PutCameraBinxPathParams,

    PutCameraBinxBodyParams {
        bin_x,

        client_id,

        client_transaction_id,
    }: PutCameraBinxBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraBinyPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraBinyQueryParams {
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

/**
Returns the binning factor for the Y axis.

Returns the binning factor for the Y axis.
*/
#[get("/camera/<device_number>/biny")]
fn get_camera_biny(GetCameraBinyPathParams { device_number }: GetCameraBinyPathParams, GetCameraBinyQueryParams { client_id, client_transaction_id }: GetCameraBinyQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCameraBinyPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCameraBinyBodyParams {
    /**
    The Y binning value
    */
    #[serde(rename = "BinY")]
    bin_y: i32,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the binning factor for the Y axis.

Sets the binning factor for the Y axis.
*/
#[put("/camera/<device_number>/biny")]
fn put_camera_biny(
    PutCameraBinyPathParams { device_number }: PutCameraBinyPathParams,

    PutCameraBinyBodyParams {
        bin_y,

        client_id,

        client_transaction_id,
    }: PutCameraBinyBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraCamerastatePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraCamerastateQueryParams {
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

/**
Returns the camera operational state.

Returns the current camera operational state as an integer. 0 = CameraIdle , 1 = CameraWaiting , 2 = CameraExposing , 3 = CameraReading , 4 = CameraDownload , 5 = CameraError
*/
#[get("/camera/<device_number>/camerastate")]
fn get_camera_camerastate(
    GetCameraCamerastatePathParams { device_number }: GetCameraCamerastatePathParams,

    GetCameraCamerastateQueryParams { client_id, client_transaction_id }: GetCameraCamerastateQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraCameraxsizePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraCameraxsizeQueryParams {
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

/**
Returns the width of the CCD camera chip.

Returns the width of the CCD camera chip in unbinned pixels.
*/
#[get("/camera/<device_number>/cameraxsize")]
fn get_camera_cameraxsize(
    GetCameraCameraxsizePathParams { device_number }: GetCameraCameraxsizePathParams,

    GetCameraCameraxsizeQueryParams { client_id, client_transaction_id }: GetCameraCameraxsizeQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraCameraysizePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraCameraysizeQueryParams {
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

/**
Returns the height of the CCD camera chip.

Returns the height of the CCD camera chip in unbinned pixels.
*/
#[get("/camera/<device_number>/cameraysize")]
fn get_camera_cameraysize(
    GetCameraCameraysizePathParams { device_number }: GetCameraCameraysizePathParams,

    GetCameraCameraysizeQueryParams { client_id, client_transaction_id }: GetCameraCameraysizeQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraCanabortexposurePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraCanabortexposureQueryParams {
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

/**
Indicates whether the camera can abort exposures.

Returns true if the camera can abort exposures; false if not.
*/
#[get("/camera/<device_number>/canabortexposure")]
fn get_camera_canabortexposure(
    GetCameraCanabortexposurePathParams { device_number }: GetCameraCanabortexposurePathParams,

    GetCameraCanabortexposureQueryParams { client_id, client_transaction_id }: GetCameraCanabortexposureQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraCanasymmetricbinPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraCanasymmetricbinQueryParams {
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

/**
Indicates whether the camera supports asymmetric binning

Returns a flag showing whether this camera supports asymmetric binning
*/
#[get("/camera/<device_number>/canasymmetricbin")]
fn get_camera_canasymmetricbin(
    GetCameraCanasymmetricbinPathParams { device_number }: GetCameraCanasymmetricbinPathParams,

    GetCameraCanasymmetricbinQueryParams { client_id, client_transaction_id }: GetCameraCanasymmetricbinQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraCanfastreadoutPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraCanfastreadoutQueryParams {
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

/**
Indicates whether the camera has a fast readout mode.

Indicates whether the camera has a fast readout mode.
*/
#[get("/camera/<device_number>/canfastreadout")]
fn get_camera_canfastreadout(
    GetCameraCanfastreadoutPathParams { device_number }: GetCameraCanfastreadoutPathParams,

    GetCameraCanfastreadoutQueryParams { client_id, client_transaction_id }: GetCameraCanfastreadoutQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraCangetcoolerpowerPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraCangetcoolerpowerQueryParams {
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

/**
Indicates whether the camera's cooler power setting can be read.

If true, the camera's cooler power setting can be read.
*/
#[get("/camera/<device_number>/cangetcoolerpower")]
fn get_camera_cangetcoolerpower(
    GetCameraCangetcoolerpowerPathParams { device_number }: GetCameraCangetcoolerpowerPathParams,

    GetCameraCangetcoolerpowerQueryParams { client_id, client_transaction_id }: GetCameraCangetcoolerpowerQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraCanpulseguidePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraCanpulseguideQueryParams {
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

/**
Returns a flag indicating whether this camera supports pulse guiding

Returns a flag indicating whether this camera supports pulse guiding.
*/
#[get("/camera/<device_number>/canpulseguide")]
fn get_camera_canpulseguide(
    GetCameraCanpulseguidePathParams { device_number }: GetCameraCanpulseguidePathParams,

    GetCameraCanpulseguideQueryParams { client_id, client_transaction_id }: GetCameraCanpulseguideQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraCansetccdtemperaturePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraCansetccdtemperatureQueryParams {
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

/**
Returns a flag indicating whether this camera supports setting the CCD temperature

Returns a flag indicatig whether this camera supports setting the CCD temperature
*/
#[get("/camera/<device_number>/cansetccdtemperature")]
fn get_camera_cansetccdtemperature(
    GetCameraCansetccdtemperaturePathParams { device_number }: GetCameraCansetccdtemperaturePathParams,

    GetCameraCansetccdtemperatureQueryParams { client_id, client_transaction_id }: GetCameraCansetccdtemperatureQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraCanstopexposurePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraCanstopexposureQueryParams {
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

/**
Returns a flag indicating whether this camera can stop an exposure that is in progress

Returns a flag indicating whether this camera can stop an exposure that is in progress
*/
#[get("/camera/<device_number>/canstopexposure")]
fn get_camera_canstopexposure(
    GetCameraCanstopexposurePathParams { device_number }: GetCameraCanstopexposurePathParams,

    GetCameraCanstopexposureQueryParams { client_id, client_transaction_id }: GetCameraCanstopexposureQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraCcdtemperaturePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraCcdtemperatureQueryParams {
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

/**
Returns the current CCD temperature

Returns the current CCD temperature in degrees Celsius.
*/
#[get("/camera/<device_number>/ccdtemperature")]
fn get_camera_ccdtemperature(
    GetCameraCcdtemperaturePathParams { device_number }: GetCameraCcdtemperaturePathParams,

    GetCameraCcdtemperatureQueryParams { client_id, client_transaction_id }: GetCameraCcdtemperatureQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraCooleronPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraCooleronQueryParams {
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

/**
Returns the current cooler on/off state.

Returns the current cooler on/off state.
*/
#[get("/camera/<device_number>/cooleron")]
fn get_camera_cooleron(GetCameraCooleronPathParams { device_number }: GetCameraCooleronPathParams, GetCameraCooleronQueryParams { client_id, client_transaction_id }: GetCameraCooleronQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCameraCooleronPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCameraCooleronBodyParams {
    /**
    Cooler state
    */
    #[serde(rename = "CoolerOn")]
    cooler_on: bool,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Turns the camera cooler on and off

Turns on and off the camera cooler. True = cooler on, False = cooler off
*/
#[put("/camera/<device_number>/cooleron")]
fn put_camera_cooleron(
    PutCameraCooleronPathParams { device_number }: PutCameraCooleronPathParams,

    PutCameraCooleronBodyParams {
        cooler_on,

        client_id,

        client_transaction_id,
    }: PutCameraCooleronBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraCoolerpowerPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraCoolerpowerQueryParams {
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

/**
Returns the present cooler power level

Returns the present cooler power level, in percent.
*/
#[get("/camera/<device_number>/coolerpower")]
fn get_camera_coolerpower(
    GetCameraCoolerpowerPathParams { device_number }: GetCameraCoolerpowerPathParams,

    GetCameraCoolerpowerQueryParams { client_id, client_transaction_id }: GetCameraCoolerpowerQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraElectronsperaduPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraElectronsperaduQueryParams {
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

/**
Returns the gain of the camera

Returns the gain of the camera in photoelectrons per A/D unit.
*/
#[get("/camera/<device_number>/electronsperadu")]
fn get_camera_electronsperadu(
    GetCameraElectronsperaduPathParams { device_number }: GetCameraElectronsperaduPathParams,

    GetCameraElectronsperaduQueryParams { client_id, client_transaction_id }: GetCameraElectronsperaduQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraExposuremaxPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraExposuremaxQueryParams {
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

/**
Returns the maximum exposure time supported by StartExposure.

Returns the maximum exposure time supported by StartExposure.
*/
#[get("/camera/<device_number>/exposuremax")]
fn get_camera_exposuremax(
    GetCameraExposuremaxPathParams { device_number }: GetCameraExposuremaxPathParams,

    GetCameraExposuremaxQueryParams { client_id, client_transaction_id }: GetCameraExposuremaxQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraExposureminPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraExposureminQueryParams {
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

/**
Returns the Minimium exposure time

Returns the Minimium exposure time in seconds that the camera supports through StartExposure.
*/
#[get("/camera/<device_number>/exposuremin")]
fn get_camera_exposuremin(
    GetCameraExposureminPathParams { device_number }: GetCameraExposureminPathParams,

    GetCameraExposureminQueryParams { client_id, client_transaction_id }: GetCameraExposureminQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraExposureresolutionPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraExposureresolutionQueryParams {
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

/**
Returns the smallest increment in exposure time supported by StartExposure.

Returns the smallest increment in exposure time supported by StartExposure.
*/
#[get("/camera/<device_number>/exposureresolution")]
fn get_camera_exposureresolution(
    GetCameraExposureresolutionPathParams { device_number }: GetCameraExposureresolutionPathParams,

    GetCameraExposureresolutionQueryParams { client_id, client_transaction_id }: GetCameraExposureresolutionQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraFastreadoutPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraFastreadoutQueryParams {
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

/**
Returns whenther Fast Readout Mode is enabled.

Returns whenther Fast Readout Mode is enabled.
*/
#[get("/camera/<device_number>/fastreadout")]
fn get_camera_fastreadout(
    GetCameraFastreadoutPathParams { device_number }: GetCameraFastreadoutPathParams,

    GetCameraFastreadoutQueryParams { client_id, client_transaction_id }: GetCameraFastreadoutQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCameraFastreadoutPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCameraFastreadoutBodyParams {
    /**
    True to enable fast readout mode
    */
    #[serde(rename = "FastReadout")]
    fast_readout: bool,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets whether Fast Readout Mode is enabled.

Sets whether Fast Readout Mode is enabled.
*/
#[put("/camera/<device_number>/fastreadout")]
fn put_camera_fastreadout(
    PutCameraFastreadoutPathParams { device_number }: PutCameraFastreadoutPathParams,

    PutCameraFastreadoutBodyParams {
        fast_readout,

        client_id,

        client_transaction_id,
    }: PutCameraFastreadoutBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraFullwellcapacityPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraFullwellcapacityQueryParams {
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

/**
Reports the full well capacity of the camera

Reports the full well capacity of the camera in electrons, at the current camera settings (binning, SetupDialog settings, etc.).
*/
#[get("/camera/<device_number>/fullwellcapacity")]
fn get_camera_fullwellcapacity(
    GetCameraFullwellcapacityPathParams { device_number }: GetCameraFullwellcapacityPathParams,

    GetCameraFullwellcapacityQueryParams { client_id, client_transaction_id }: GetCameraFullwellcapacityQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraGainPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraGainQueryParams {
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

/**
Returns the camera's gain

The camera's gain (GAIN VALUE MODE) OR the index of the selected camera gain description in the Gains array (GAINS INDEX MODE).
*/
#[get("/camera/<device_number>/gain")]
fn get_camera_gain(GetCameraGainPathParams { device_number }: GetCameraGainPathParams, GetCameraGainQueryParams { client_id, client_transaction_id }: GetCameraGainQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCameraGainPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCameraGainBodyParams {
    /**
    Index of the current camera gain in the Gains string array.
    */
    #[serde(rename = "Gain")]
    gain: i32,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the camera's gain.

The camera's gain (GAIN VALUE MODE) OR the index of the selected camera gain description in the Gains array (GAINS INDEX MODE).
*/
#[put("/camera/<device_number>/gain")]
fn put_camera_gain(
    PutCameraGainPathParams { device_number }: PutCameraGainPathParams,

    PutCameraGainBodyParams {
        gain,

        client_id,

        client_transaction_id,
    }: PutCameraGainBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraGainmaxPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraGainmaxQueryParams {
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

/**
Maximum Gain value of that this camera supports

Returns the maximum value of Gain.
*/
#[get("/camera/<device_number>/gainmax")]
fn get_camera_gainmax(GetCameraGainmaxPathParams { device_number }: GetCameraGainmaxPathParams, GetCameraGainmaxQueryParams { client_id, client_transaction_id }: GetCameraGainmaxQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraGainminPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraGainminQueryParams {
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

/**
Minimum Gain value of that this camera supports

Returns the Minimum value of Gain.
*/
#[get("/camera/<device_number>/gainmin")]
fn get_camera_gainmin(GetCameraGainminPathParams { device_number }: GetCameraGainminPathParams, GetCameraGainminQueryParams { client_id, client_transaction_id }: GetCameraGainminQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraGainsPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraGainsQueryParams {
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

/**
List of Gain names supported by the camera

Returns the Gains supported by the camera.
*/
#[get("/camera/<device_number>/gains")]
fn get_camera_gains(GetCameraGainsPathParams { device_number }: GetCameraGainsPathParams, GetCameraGainsQueryParams { client_id, client_transaction_id }: GetCameraGainsQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraHasshutterPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraHasshutterQueryParams {
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

/**
Indicates whether the camera has a mechanical shutter

Returns a flag indicating whether this camera has a mechanical shutter.
*/
#[get("/camera/<device_number>/hasshutter")]
fn get_camera_hasshutter(
    GetCameraHasshutterPathParams { device_number }: GetCameraHasshutterPathParams,

    GetCameraHasshutterQueryParams { client_id, client_transaction_id }: GetCameraHasshutterQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraHeatsinktemperaturePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraHeatsinktemperatureQueryParams {
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

/**
Returns the current heat sink temperature.

Returns the current heat sink temperature (called "ambient temperature" by some manufacturers) in degrees Celsius.
*/
#[get("/camera/<device_number>/heatsinktemperature")]
fn get_camera_heatsinktemperature(
    GetCameraHeatsinktemperaturePathParams { device_number }: GetCameraHeatsinktemperaturePathParams,

    GetCameraHeatsinktemperatureQueryParams { client_id, client_transaction_id }: GetCameraHeatsinktemperatureQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraImagearrayPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraImagearrayQueryParams {
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
    GetCameraImagearrayPathParams { device_number }: GetCameraImagearrayPathParams,

    GetCameraImagearrayQueryParams { client_id, client_transaction_id }: GetCameraImagearrayQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraImagearrayvariantPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraImagearrayvariantQueryParams {
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
    GetCameraImagearrayvariantPathParams { device_number }: GetCameraImagearrayvariantPathParams,

    GetCameraImagearrayvariantQueryParams { client_id, client_transaction_id }: GetCameraImagearrayvariantQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraImagereadyPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraImagereadyQueryParams {
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

/**
Indicates that an image is ready to be downloaded

Returns a flag indicating whether the image is ready to be downloaded from the camera.
*/
#[get("/camera/<device_number>/imageready")]
fn get_camera_imageready(
    GetCameraImagereadyPathParams { device_number }: GetCameraImagereadyPathParams,

    GetCameraImagereadyQueryParams { client_id, client_transaction_id }: GetCameraImagereadyQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraIspulseguidingPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraIspulseguidingQueryParams {
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

/**
Indicates that the camera is pulse guideing.

Returns a flag indicating whether the camera is currrently in a PulseGuide operation.
*/
#[get("/camera/<device_number>/ispulseguiding")]
fn get_camera_ispulseguiding(
    GetCameraIspulseguidingPathParams { device_number }: GetCameraIspulseguidingPathParams,

    GetCameraIspulseguidingQueryParams { client_id, client_transaction_id }: GetCameraIspulseguidingQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraLastexposuredurationPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraLastexposuredurationQueryParams {
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

/**
Duration of the last exposure

Reports the actual exposure duration in seconds (i.e. shutter open time).
*/
#[get("/camera/<device_number>/lastexposureduration")]
fn get_camera_lastexposureduration(
    GetCameraLastexposuredurationPathParams { device_number }: GetCameraLastexposuredurationPathParams,

    GetCameraLastexposuredurationQueryParams { client_id, client_transaction_id }: GetCameraLastexposuredurationQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraLastexposurestarttimePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraLastexposurestarttimeQueryParams {
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

/**
Start time of the last exposure in FITS standard format.

Reports the actual exposure start in the FITS-standard CCYY-MM-DDThh:mm:ss[.sss...] format.
*/
#[get("/camera/<device_number>/lastexposurestarttime")]
fn get_camera_lastexposurestarttime(
    GetCameraLastexposurestarttimePathParams { device_number }: GetCameraLastexposurestarttimePathParams,

    GetCameraLastexposurestarttimeQueryParams { client_id, client_transaction_id }: GetCameraLastexposurestarttimeQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraMaxaduPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraMaxaduQueryParams {
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

/**
Camera's maximum ADU value

Reports the maximum ADU value the camera can produce.
*/
#[get("/camera/<device_number>/maxadu")]
fn get_camera_maxadu(GetCameraMaxaduPathParams { device_number }: GetCameraMaxaduPathParams, GetCameraMaxaduQueryParams { client_id, client_transaction_id }: GetCameraMaxaduQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraMaxbinxPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraMaxbinxQueryParams {
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

/**
Maximum  binning for the camera X axis

Returns the maximum allowed binning for the X camera axis
*/
#[get("/camera/<device_number>/maxbinx")]
fn get_camera_maxbinx(GetCameraMaxbinxPathParams { device_number }: GetCameraMaxbinxPathParams, GetCameraMaxbinxQueryParams { client_id, client_transaction_id }: GetCameraMaxbinxQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraMaxbinyPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraMaxbinyQueryParams {
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

/**
Maximum  binning for the camera Y axis

Returns the maximum allowed binning for the Y camera axis
*/
#[get("/camera/<device_number>/maxbiny")]
fn get_camera_maxbiny(GetCameraMaxbinyPathParams { device_number }: GetCameraMaxbinyPathParams, GetCameraMaxbinyQueryParams { client_id, client_transaction_id }: GetCameraMaxbinyQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraNumxPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraNumxQueryParams {
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

/**
Returns the current subframe width

Returns the current subframe width, if binning is active, value is in binned pixels.
*/
#[get("/camera/<device_number>/numx")]
fn get_camera_numx(GetCameraNumxPathParams { device_number }: GetCameraNumxPathParams, GetCameraNumxQueryParams { client_id, client_transaction_id }: GetCameraNumxQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCameraNumxPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCameraNumxBodyParams {
    /**
    Sets the subframe width, if binning is active, value is in binned pixels.
    */
    #[serde(rename = "NumX")]
    num_x: i32,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the current subframe width

Sets the current subframe width.
*/
#[put("/camera/<device_number>/numx")]
fn put_camera_numx(
    PutCameraNumxPathParams { device_number }: PutCameraNumxPathParams,

    PutCameraNumxBodyParams {
        num_x,

        client_id,

        client_transaction_id,
    }: PutCameraNumxBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraNumyPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraNumyQueryParams {
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

/**
Returns the current subframe height

Returns the current subframe height, if binning is active, value is in binned pixels.
*/
#[get("/camera/<device_number>/numy")]
fn get_camera_numy(GetCameraNumyPathParams { device_number }: GetCameraNumyPathParams, GetCameraNumyQueryParams { client_id, client_transaction_id }: GetCameraNumyQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCameraNumyPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCameraNumyBodyParams {
    /**
    Sets the subframe height, if binning is active, value is in binned pixels.
    */
    #[serde(rename = "NumY")]
    num_y: i32,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the current subframe height

Sets the current subframe height.
*/
#[put("/camera/<device_number>/numy")]
fn put_camera_numy(
    PutCameraNumyPathParams { device_number }: PutCameraNumyPathParams,

    PutCameraNumyBodyParams {
        num_y,

        client_id,

        client_transaction_id,
    }: PutCameraNumyBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraOffsetPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraOffsetQueryParams {
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

/**
Returns the camera's offset

Returns the camera's offset (OFFSET VALUE MODE) OR the index of the selected camera offset description in the offsets array (OFFSETS INDEX MODE).
*/
#[get("/camera/<device_number>/offset")]
fn get_camera_offset(GetCameraOffsetPathParams { device_number }: GetCameraOffsetPathParams, GetCameraOffsetQueryParams { client_id, client_transaction_id }: GetCameraOffsetQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCameraOffsetPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCameraOffsetBodyParams {
    /**
    Index of the current camera offset in the offsets string array.
    */
    #[serde(rename = "offset")]
    offset: i32,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the camera's offset.

Sets the camera's offset (OFFSET VALUE MODE) OR the index of the selected camera offset description in the offsets array (OFFSETS INDEX MODE).
*/
#[put("/camera/<device_number>/offset")]
fn put_camera_offset(
    PutCameraOffsetPathParams { device_number }: PutCameraOffsetPathParams,

    PutCameraOffsetBodyParams {
        offset,

        client_id,

        client_transaction_id,
    }: PutCameraOffsetBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraOffsetmaxPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraOffsetmaxQueryParams {
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

/**
Maximum offset value of that this camera supports

Returns the maximum value of offset.
*/
#[get("/camera/<device_number>/offsetmax")]
fn get_camera_offsetmax(
    GetCameraOffsetmaxPathParams { device_number }: GetCameraOffsetmaxPathParams,

    GetCameraOffsetmaxQueryParams { client_id, client_transaction_id }: GetCameraOffsetmaxQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraOffsetminPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraOffsetminQueryParams {
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

/**
Minimum offset value of that this camera supports

Returns the Minimum value of offset.
*/
#[get("/camera/<device_number>/offsetmin")]
fn get_camera_offsetmin(
    GetCameraOffsetminPathParams { device_number }: GetCameraOffsetminPathParams,

    GetCameraOffsetminQueryParams { client_id, client_transaction_id }: GetCameraOffsetminQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraOffsetsPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraOffsetsQueryParams {
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

/**
List of offset names supported by the camera

Returns the offsets supported by the camera.
*/
#[get("/camera/<device_number>/offsets")]
fn get_camera_offsets(GetCameraOffsetsPathParams { device_number }: GetCameraOffsetsPathParams, GetCameraOffsetsQueryParams { client_id, client_transaction_id }: GetCameraOffsetsQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraPercentcompletedPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraPercentcompletedQueryParams {
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

/**
Indicates percentage completeness of the current operation

Returns the percentage of the current operation that is complete. If valid, returns an integer between 0 and 100, where 0 indicates 0% progress (function just started) and 100 indicates 100% progress (i.e. completion).
*/
#[get("/camera/<device_number>/percentcompleted")]
fn get_camera_percentcompleted(
    GetCameraPercentcompletedPathParams { device_number }: GetCameraPercentcompletedPathParams,

    GetCameraPercentcompletedQueryParams { client_id, client_transaction_id }: GetCameraPercentcompletedQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraPixelsizexPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraPixelsizexQueryParams {
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

/**
Width of CCD chip pixels (microns)

Returns the width of the CCD chip pixels in microns.
*/
#[get("/camera/<device_number>/pixelsizex")]
fn get_camera_pixelsizex(
    GetCameraPixelsizexPathParams { device_number }: GetCameraPixelsizexPathParams,

    GetCameraPixelsizexQueryParams { client_id, client_transaction_id }: GetCameraPixelsizexQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraPixelsizeyPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraPixelsizeyQueryParams {
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

/**
Height of CCD chip pixels (microns)

Returns the Height of the CCD chip pixels in microns.
*/
#[get("/camera/<device_number>/pixelsizey")]
fn get_camera_pixelsizey(
    GetCameraPixelsizeyPathParams { device_number }: GetCameraPixelsizeyPathParams,

    GetCameraPixelsizeyQueryParams { client_id, client_transaction_id }: GetCameraPixelsizeyQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraReadoutmodePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraReadoutmodeQueryParams {
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

/**
Indicates the canera's readout mode as an index into the array ReadoutModes

ReadoutMode is an index into the array ReadoutModes and returns the desired readout mode for the camera. Defaults to 0 if not set.
*/
#[get("/camera/<device_number>/readoutmode")]
fn get_camera_readoutmode(
    GetCameraReadoutmodePathParams { device_number }: GetCameraReadoutmodePathParams,

    GetCameraReadoutmodeQueryParams { client_id, client_transaction_id }: GetCameraReadoutmodeQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCameraReadoutmodePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCameraReadoutmodeBodyParams {
    /**
    Index into the ReadoutModes array of string readout mode names indicating the camera's current readout mode.
    */
    #[serde(rename = "ReadoutMode")]
    readout_mode: i32,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Set the camera's readout mode

Sets the ReadoutMode as an index into the array ReadoutModes.
*/
#[put("/camera/<device_number>/readoutmode")]
fn put_camera_readoutmode(
    PutCameraReadoutmodePathParams { device_number }: PutCameraReadoutmodePathParams,

    PutCameraReadoutmodeBodyParams {
        readout_mode,

        client_id,

        client_transaction_id,
    }: PutCameraReadoutmodeBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraReadoutmodesPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraReadoutmodesQueryParams {
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

/**
List of available readout modes

This property provides an array of strings, each of which describes an available readout mode of the camera. At least one string must be present in the list.
*/
#[get("/camera/<device_number>/readoutmodes")]
fn get_camera_readoutmodes(
    GetCameraReadoutmodesPathParams { device_number }: GetCameraReadoutmodesPathParams,

    GetCameraReadoutmodesQueryParams { client_id, client_transaction_id }: GetCameraReadoutmodesQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraSensornamePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraSensornameQueryParams {
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

/**
Sensor name

The name of the sensor used within the camera.
*/
#[get("/camera/<device_number>/sensorname")]
fn get_camera_sensorname(
    GetCameraSensornamePathParams { device_number }: GetCameraSensornamePathParams,

    GetCameraSensornameQueryParams { client_id, client_transaction_id }: GetCameraSensornameQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraSensortypePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraSensortypeQueryParams {
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
    GetCameraSensortypePathParams { device_number }: GetCameraSensortypePathParams,

    GetCameraSensortypeQueryParams { client_id, client_transaction_id }: GetCameraSensortypeQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraSetccdtemperaturePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraSetccdtemperatureQueryParams {
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

/**
Returns the current camera cooler setpoint in degrees Celsius.

Returns the current camera cooler setpoint in degrees Celsius.
*/
#[get("/camera/<device_number>/setccdtemperature")]
fn get_camera_setccdtemperature(
    GetCameraSetccdtemperaturePathParams { device_number }: GetCameraSetccdtemperaturePathParams,

    GetCameraSetccdtemperatureQueryParams { client_id, client_transaction_id }: GetCameraSetccdtemperatureQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCameraSetccdtemperaturePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCameraSetccdtemperatureBodyParams {
    /**
    Temperature set point (degrees Celsius).
    */
    #[serde(rename = "SetCCDTemperature")]
    set_ccdtemperature: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Set the camera's cooler setpoint (degrees Celsius).

Set's the camera's cooler setpoint in degrees Celsius.
*/
#[put("/camera/<device_number>/setccdtemperature")]
fn put_camera_setccdtemperature(
    PutCameraSetccdtemperaturePathParams { device_number }: PutCameraSetccdtemperaturePathParams,

    PutCameraSetccdtemperatureBodyParams {
        set_ccdtemperature,

        client_id,

        client_transaction_id,
    }: PutCameraSetccdtemperatureBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraStartxPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraStartxQueryParams {
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

/**
Return the current subframe X axis start position

Sets the subframe start position for the X axis (0 based) and returns the current value. If binning is active, value is in binned pixels.
*/
#[get("/camera/<device_number>/startx")]
fn get_camera_startx(GetCameraStartxPathParams { device_number }: GetCameraStartxPathParams, GetCameraStartxQueryParams { client_id, client_transaction_id }: GetCameraStartxQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCameraStartxPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCameraStartxBodyParams {
    /**
    The subframe X axis start position in binned pixels.
    */
    #[serde(rename = "StartX")]
    start_x: i32,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the current subframe X axis start position

Sets the current subframe X axis start position in binned pixels.
*/
#[put("/camera/<device_number>/startx")]
fn put_camera_startx(
    PutCameraStartxPathParams { device_number }: PutCameraStartxPathParams,

    PutCameraStartxBodyParams {
        start_x,

        client_id,

        client_transaction_id,
    }: PutCameraStartxBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraStartyPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraStartyQueryParams {
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

/**
Return the current subframe Y axis start position

Sets the subframe start position for the Y axis (0 based) and returns the current value. If binning is active, value is in binned pixels.
*/
#[get("/camera/<device_number>/starty")]
fn get_camera_starty(GetCameraStartyPathParams { device_number }: GetCameraStartyPathParams, GetCameraStartyQueryParams { client_id, client_transaction_id }: GetCameraStartyQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCameraStartyPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCameraStartyBodyParams {
    /**
    The subframe Y axis start position in binned pixels.
    */
    #[serde(rename = "StartY")]
    start_y: i32,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the current subframe Y axis start position

Sets the current subframe Y axis start position in binned pixels.
*/
#[put("/camera/<device_number>/starty")]
fn put_camera_starty(
    PutCameraStartyPathParams { device_number }: PutCameraStartyPathParams,

    PutCameraStartyBodyParams {
        start_y,

        client_id,

        client_transaction_id,
    }: PutCameraStartyBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCameraSubexposuredurationPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCameraSubexposuredurationQueryParams {
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

/**
Camera's sub-exposure interval

The Camera's sub exposure duration in seconds. Only available in Camera Interface Version 3 and later.
*/
#[get("/camera/<device_number>/subexposureduration")]
fn get_camera_subexposureduration(
    GetCameraSubexposuredurationPathParams { device_number }: GetCameraSubexposuredurationPathParams,

    GetCameraSubexposuredurationQueryParams { client_id, client_transaction_id }: GetCameraSubexposuredurationQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCameraSubexposuredurationPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCameraSubexposuredurationBodyParams {
    /**
    The request sub exposure duration in seconds
    */
    #[serde(rename = "SubExposureDuration")]
    sub_exposure_duration: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the current Sub Exposure Duration

Sets image sub exposure duration in seconds. Only available in Camera Interface Version 3 and later.
*/
#[put("/camera/<device_number>/subexposureduration")]
fn put_camera_subexposureduration(
    PutCameraSubexposuredurationPathParams { device_number }: PutCameraSubexposuredurationPathParams,

    PutCameraSubexposuredurationBodyParams {
        sub_exposure_duration,

        client_id,

        client_transaction_id,
    }: PutCameraSubexposuredurationBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCameraAbortexposurePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCameraAbortexposureBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Aborts the current exposure

Aborts the current exposure, if any, and returns the camera to Idle state.
*/
#[put("/camera/<device_number>/abortexposure")]
fn put_camera_abortexposure(
    PutCameraAbortexposurePathParams { device_number }: PutCameraAbortexposurePathParams,

    PutCameraAbortexposureBodyParams { client_id, client_transaction_id }: PutCameraAbortexposureBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCameraPulseguidePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCameraPulseguideBodyParams {
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
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Pulse guide in the specified direction for the specified time.

Activates the Camera's mount control sytem to instruct the mount to move in a particular direction for a given period of time
*/
#[put("/camera/<device_number>/pulseguide")]
fn put_camera_pulseguide(
    PutCameraPulseguidePathParams { device_number }: PutCameraPulseguidePathParams,

    PutCameraPulseguideBodyParams {
        direction,

        duration,

        client_id,

        client_transaction_id,
    }: PutCameraPulseguideBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCameraStartexposurePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCameraStartexposureBodyParams {
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
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Starts an exposure

Starts an exposure. Use ImageReady to check when the exposure is complete.
*/
#[put("/camera/<device_number>/startexposure")]
fn put_camera_startexposure(
    PutCameraStartexposurePathParams { device_number }: PutCameraStartexposurePathParams,

    PutCameraStartexposureBodyParams {
        duration,

        light,

        client_id,

        client_transaction_id,
    }: PutCameraStartexposureBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCameraStopexposurePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCameraStopexposureBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Stops the current exposure

Stops the current exposure, if any. If an exposure is in progress, the readout process is initiated. Ignored if readout is already in process.
*/
#[put("/camera/<device_number>/stopexposure")]
fn put_camera_stopexposure(
    PutCameraStopexposurePathParams { device_number }: PutCameraStopexposurePathParams,

    PutCameraStopexposureBodyParams { client_id, client_transaction_id }: PutCameraStopexposureBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCovercalibratorBrightnessPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCovercalibratorBrightnessQueryParams {
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

/**
Returns the current calibrator brightness

Returns the current calibrator brightness in the range 0 (completely off) to MaxBrightness (fully on)
*/
#[get("/covercalibrator/<device_number>/brightness")]
fn get_covercalibrator_brightness(
    GetCovercalibratorBrightnessPathParams { device_number }: GetCovercalibratorBrightnessPathParams,

    GetCovercalibratorBrightnessQueryParams { client_id, client_transaction_id }: GetCovercalibratorBrightnessQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCovercalibratorCalibratorstatePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCovercalibratorCalibratorstateQueryParams {
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

/**
Returns the state of the calibration device

Returns the state of the calibration device, if present, otherwise returns "NotPresent".  The calibrator state mode is specified as an integer value from the CalibratorStatus Enum.
*/
#[get("/covercalibrator/<device_number>/calibratorstate")]
fn get_covercalibrator_calibratorstate(
    GetCovercalibratorCalibratorstatePathParams { device_number }: GetCovercalibratorCalibratorstatePathParams,

    GetCovercalibratorCalibratorstateQueryParams { client_id, client_transaction_id }: GetCovercalibratorCalibratorstateQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCovercalibratorCoverstatePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCovercalibratorCoverstateQueryParams {
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

/**
Returns the state of the device cover"

Returns the state of the device cover, if present, otherwise returns "NotPresent".  The cover state mode is specified as an integer value from the CoverStatus Enum.
*/
#[get("/covercalibrator/<device_number>/coverstate")]
fn get_covercalibrator_coverstate(
    GetCovercalibratorCoverstatePathParams { device_number }: GetCovercalibratorCoverstatePathParams,

    GetCovercalibratorCoverstateQueryParams { client_id, client_transaction_id }: GetCovercalibratorCoverstateQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetCovercalibratorMaxbrightnessPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetCovercalibratorMaxbrightnessQueryParams {
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

/**
Returns the calibrator's maximum Brightness value.

The Brightness value that makes the calibrator deliver its maximum illumination.
*/
#[get("/covercalibrator/<device_number>/maxbrightness")]
fn get_covercalibrator_maxbrightness(
    GetCovercalibratorMaxbrightnessPathParams { device_number }: GetCovercalibratorMaxbrightnessPathParams,

    GetCovercalibratorMaxbrightnessQueryParams { client_id, client_transaction_id }: GetCovercalibratorMaxbrightnessQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCovercalibratorCalibratoroffPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCovercalibratorCalibratoroffBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Turns the calibrator off

Turns the calibrator off if the device has calibration capability.
*/
#[put("/covercalibrator/<device_number>/calibratoroff")]
fn put_covercalibrator_calibratoroff(
    PutCovercalibratorCalibratoroffPathParams { device_number }: PutCovercalibratorCalibratoroffPathParams,

    PutCovercalibratorCalibratoroffBodyParams { client_id, client_transaction_id }: PutCovercalibratorCalibratoroffBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCovercalibratorCalibratoronPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCovercalibratorCalibratoronBodyParams {
    /**
    The required brightness in the range 0 to MaxBrightness
    */
    #[serde(rename = "Brightness")]
    brightness: i32,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Turns the calibrator on at the specified brightness

Turns the calibrator on at the specified brightness if the device has calibration capability.
*/
#[put("/covercalibrator/<device_number>/calibratoron")]
fn put_covercalibrator_calibratoron(
    PutCovercalibratorCalibratoronPathParams { device_number }: PutCovercalibratorCalibratoronPathParams,

    PutCovercalibratorCalibratoronBodyParams {
        brightness,

        client_id,

        client_transaction_id,
    }: PutCovercalibratorCalibratoronBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCovercalibratorClosecoverPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCovercalibratorClosecoverBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Initiates cover closing

Initiates cover closing if a cover is present.
*/
#[put("/covercalibrator/<device_number>/closecover")]
fn put_covercalibrator_closecover(
    PutCovercalibratorClosecoverPathParams { device_number }: PutCovercalibratorClosecoverPathParams,

    PutCovercalibratorClosecoverBodyParams { client_id, client_transaction_id }: PutCovercalibratorClosecoverBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCovercalibratorHaltcoverPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCovercalibratorHaltcoverBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Stops any cover movement that may be in progress

Stops any cover movement that may be in progress if a cover is present and cover movement can be interrupted.
*/
#[put("/covercalibrator/<device_number>/haltcover")]
fn put_covercalibrator_haltcover(
    PutCovercalibratorHaltcoverPathParams { device_number }: PutCovercalibratorHaltcoverPathParams,

    PutCovercalibratorHaltcoverBodyParams { client_id, client_transaction_id }: PutCovercalibratorHaltcoverBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutCovercalibratorOpencoverPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutCovercalibratorOpencoverBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Initiates cover opening

Initiates cover opening if a cover is present.
*/
#[put("/covercalibrator/<device_number>/opencover")]
fn put_covercalibrator_opencover(
    PutCovercalibratorOpencoverPathParams { device_number }: PutCovercalibratorOpencoverPathParams,

    PutCovercalibratorOpencoverBodyParams { client_id, client_transaction_id }: PutCovercalibratorOpencoverBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetDomeAltitudePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetDomeAltitudeQueryParams {
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

/**
The dome altitude

The dome altitude (degrees, horizon zero and increasing positive to 90 zenith).
*/
#[get("/dome/<device_number>/altitude")]
fn get_dome_altitude(GetDomeAltitudePathParams { device_number }: GetDomeAltitudePathParams, GetDomeAltitudeQueryParams { client_id, client_transaction_id }: GetDomeAltitudeQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetDomeAthomePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetDomeAthomeQueryParams {
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

/**
Indicates whether the dome is in the home position.

Indicates whether the dome is in the home position. This is normally used following a FindHome()  operation. The value is reset with any azimuth slew operation that moves the dome away from the home position. AtHome may also become true durng normal slew operations, if the dome passes through the home position and the dome controller hardware is capable of detecting that; or at the end of a slew operation if the dome comes to rest at the home position.
*/
#[get("/dome/<device_number>/athome")]
fn get_dome_athome(GetDomeAthomePathParams { device_number }: GetDomeAthomePathParams, GetDomeAthomeQueryParams { client_id, client_transaction_id }: GetDomeAthomeQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetDomeAtparkPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetDomeAtparkQueryParams {
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

/**
Indicates whether the telescope is at the park position

True if the dome is in the programmed park position. Set only following a Park() operation and reset with any slew operation.
*/
#[get("/dome/<device_number>/atpark")]
fn get_dome_atpark(GetDomeAtparkPathParams { device_number }: GetDomeAtparkPathParams, GetDomeAtparkQueryParams { client_id, client_transaction_id }: GetDomeAtparkQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetDomeAzimuthPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetDomeAzimuthQueryParams {
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

/**
The dome azimuth

Returns the dome azimuth (degrees, North zero and increasing clockwise, i.e., 90 East, 180 South, 270 West)
*/
#[get("/dome/<device_number>/azimuth")]
fn get_dome_azimuth(GetDomeAzimuthPathParams { device_number }: GetDomeAzimuthPathParams, GetDomeAzimuthQueryParams { client_id, client_transaction_id }: GetDomeAzimuthQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetDomeCanfindhomePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetDomeCanfindhomeQueryParams {
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

/**
Indicates whether the dome can find the home position.

True if the dome can move to the home position.
*/
#[get("/dome/<device_number>/canfindhome")]
fn get_dome_canfindhome(
    GetDomeCanfindhomePathParams { device_number }: GetDomeCanfindhomePathParams,

    GetDomeCanfindhomeQueryParams { client_id, client_transaction_id }: GetDomeCanfindhomeQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetDomeCanparkPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetDomeCanparkQueryParams {
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

/**
Indicates whether the dome can be parked.

True if the dome is capable of programmed parking (Park() method)
*/
#[get("/dome/<device_number>/canpark")]
fn get_dome_canpark(GetDomeCanparkPathParams { device_number }: GetDomeCanparkPathParams, GetDomeCanparkQueryParams { client_id, client_transaction_id }: GetDomeCanparkQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetDomeCansetaltitudePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetDomeCansetaltitudeQueryParams {
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

/**
Indicates whether the dome altitude can be set

True if driver is capable of setting the dome altitude.
*/
#[get("/dome/<device_number>/cansetaltitude")]
fn get_dome_cansetaltitude(
    GetDomeCansetaltitudePathParams { device_number }: GetDomeCansetaltitudePathParams,

    GetDomeCansetaltitudeQueryParams { client_id, client_transaction_id }: GetDomeCansetaltitudeQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetDomeCansetazimuthPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetDomeCansetazimuthQueryParams {
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

/**
Indicates whether the dome azimuth can be set

True if driver is capable of setting the dome azimuth.
*/
#[get("/dome/<device_number>/cansetazimuth")]
fn get_dome_cansetazimuth(
    GetDomeCansetazimuthPathParams { device_number }: GetDomeCansetazimuthPathParams,

    GetDomeCansetazimuthQueryParams { client_id, client_transaction_id }: GetDomeCansetazimuthQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetDomeCansetparkPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetDomeCansetparkQueryParams {
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

/**
Indicates whether the dome park position can be set

True if driver is capable of setting the dome park position.
*/
#[get("/dome/<device_number>/cansetpark")]
fn get_dome_cansetpark(GetDomeCansetparkPathParams { device_number }: GetDomeCansetparkPathParams, GetDomeCansetparkQueryParams { client_id, client_transaction_id }: GetDomeCansetparkQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetDomeCansetshutterPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetDomeCansetshutterQueryParams {
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

/**
Indicates whether the dome shutter can be opened

True if driver is capable of automatically operating shutter
*/
#[get("/dome/<device_number>/cansetshutter")]
fn get_dome_cansetshutter(
    GetDomeCansetshutterPathParams { device_number }: GetDomeCansetshutterPathParams,

    GetDomeCansetshutterQueryParams { client_id, client_transaction_id }: GetDomeCansetshutterQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetDomeCanslavePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetDomeCanslaveQueryParams {
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

/**
Indicates whether the dome supports slaving to a telescope

True if driver is capable of slaving to a telescope.
*/
#[get("/dome/<device_number>/canslave")]
fn get_dome_canslave(GetDomeCanslavePathParams { device_number }: GetDomeCanslavePathParams, GetDomeCanslaveQueryParams { client_id, client_transaction_id }: GetDomeCanslaveQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetDomeCansyncazimuthPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetDomeCansyncazimuthQueryParams {
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

/**
Indicates whether the dome azimuth position can be synched

True if driver is capable of synchronizing the dome azimuth position using the SyncToAzimuth(Double) method.
*/
#[get("/dome/<device_number>/cansyncazimuth")]
fn get_dome_cansyncazimuth(
    GetDomeCansyncazimuthPathParams { device_number }: GetDomeCansyncazimuthPathParams,

    GetDomeCansyncazimuthQueryParams { client_id, client_transaction_id }: GetDomeCansyncazimuthQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetDomeShutterstatusPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetDomeShutterstatusQueryParams {
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

/**
Status of the dome shutter or roll-off roof

Returns the status of the dome shutter or roll-off roof. 0 = Open, 1 = Closed, 2 = Opening, 3 = Closing, 4 = Shutter status error
*/
#[get("/dome/<device_number>/shutterstatus")]
fn get_dome_shutterstatus(
    GetDomeShutterstatusPathParams { device_number }: GetDomeShutterstatusPathParams,

    GetDomeShutterstatusQueryParams { client_id, client_transaction_id }: GetDomeShutterstatusQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetDomeSlavedPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetDomeSlavedQueryParams {
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

/**
Indicates whether the dome is slaved to the telescope

True if the dome is slaved to the telescope in its hardware, else False.
*/
#[get("/dome/<device_number>/slaved")]
fn get_dome_slaved(GetDomeSlavedPathParams { device_number }: GetDomeSlavedPathParams, GetDomeSlavedQueryParams { client_id, client_transaction_id }: GetDomeSlavedQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutDomeSlavedPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutDomeSlavedBodyParams {
    /**
    True if telescope is slaved to dome, otherwise false
    */
    #[serde(rename = "Slaved")]
    slaved: bool,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets whether the dome is slaved to the telescope

Sets the current subframe height.
*/
#[put("/dome/<device_number>/slaved")]
fn put_dome_slaved(
    PutDomeSlavedPathParams { device_number }: PutDomeSlavedPathParams,

    PutDomeSlavedBodyParams {
        slaved,

        client_id,

        client_transaction_id,
    }: PutDomeSlavedBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetDomeSlewingPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetDomeSlewingQueryParams {
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

/**
Indicates whether the any part of the dome is moving

True if any part of the dome is currently moving, False if all dome components are steady.
*/
#[get("/dome/<device_number>/slewing")]
fn get_dome_slewing(GetDomeSlewingPathParams { device_number }: GetDomeSlewingPathParams, GetDomeSlewingQueryParams { client_id, client_transaction_id }: GetDomeSlewingQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutDomeAbortslewPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutDomeAbortslewBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Immediately cancel current dome operation.

Calling this method will immediately disable hardware slewing (Slaved will become False).
*/
#[put("/dome/<device_number>/abortslew")]
fn put_dome_abortslew(PutDomeAbortslewPathParams { device_number }: PutDomeAbortslewPathParams, PutDomeAbortslewBodyParams { client_id, client_transaction_id }: PutDomeAbortslewBodyParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutDomeCloseshutterPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutDomeCloseshutterBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Close the shutter or otherwise shield telescope from the sky.

Close the shutter or otherwise shield telescope from the sky.
*/
#[put("/dome/<device_number>/closeshutter")]
fn put_dome_closeshutter(
    PutDomeCloseshutterPathParams { device_number }: PutDomeCloseshutterPathParams,

    PutDomeCloseshutterBodyParams { client_id, client_transaction_id }: PutDomeCloseshutterBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutDomeFindhomePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutDomeFindhomeBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Start operation to search for the dome home position.

After Home position is established initializes Azimuth to the default value and sets the AtHome flag.
*/
#[put("/dome/<device_number>/findhome")]
fn put_dome_findhome(PutDomeFindhomePathParams { device_number }: PutDomeFindhomePathParams, PutDomeFindhomeBodyParams { client_id, client_transaction_id }: PutDomeFindhomeBodyParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutDomeOpenshutterPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutDomeOpenshutterBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Open shutter or otherwise expose telescope to the sky.

Open shutter or otherwise expose telescope to the sky.
*/
#[put("/dome/<device_number>/openshutter")]
fn put_dome_openshutter(PutDomeOpenshutterPathParams { device_number }: PutDomeOpenshutterPathParams, PutDomeOpenshutterBodyParams { client_id, client_transaction_id }: PutDomeOpenshutterBodyParams) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutDomeParkPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutDomeParkBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Rotate dome in azimuth to park position.

After assuming programmed park position, sets AtPark flag.
*/
#[put("/dome/<device_number>/park")]
fn put_dome_park(PutDomeParkPathParams { device_number }: PutDomeParkPathParams, PutDomeParkBodyParams { client_id, client_transaction_id }: PutDomeParkBodyParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutDomeSetparkPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutDomeSetparkBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Set the current azimuth, altitude position of dome to be the park position

Set the current azimuth, altitude position of dome to be the park position.
*/
#[put("/dome/<device_number>/setpark")]
fn put_dome_setpark(PutDomeSetparkPathParams { device_number }: PutDomeSetparkPathParams, PutDomeSetparkBodyParams { client_id, client_transaction_id }: PutDomeSetparkBodyParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutDomeSlewtoaltitudePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutDomeSlewtoaltitudeBodyParams {
    /**
    Target dome altitude (degrees, horizon zero and increasing positive to 90 zenith)
    */
    #[serde(rename = "Altitude")]
    altitude: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Slew the dome to the given altitude position.

Slew the dome to the given altitude position.
*/
#[put("/dome/<device_number>/slewtoaltitude")]
fn put_dome_slewtoaltitude(
    PutDomeSlewtoaltitudePathParams { device_number }: PutDomeSlewtoaltitudePathParams,

    PutDomeSlewtoaltitudeBodyParams {
        altitude,

        client_id,

        client_transaction_id,
    }: PutDomeSlewtoaltitudeBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutDomeSlewtoazimuthPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutDomeSlewtoazimuthBodyParams {
    /**
    Target dome azimuth (degrees, North zero and increasing clockwise. i.e., 90 East, 180 South, 270 West)
    */
    #[serde(rename = "Azimuth")]
    azimuth: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Slew the dome to the given azimuth position.

Slew the dome to the given azimuth position.
*/
#[put("/dome/<device_number>/slewtoazimuth")]
fn put_dome_slewtoazimuth(
    PutDomeSlewtoazimuthPathParams { device_number }: PutDomeSlewtoazimuthPathParams,

    PutDomeSlewtoazimuthBodyParams {
        azimuth,

        client_id,

        client_transaction_id,
    }: PutDomeSlewtoazimuthBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutDomeSynctoazimuthPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutDomeSynctoazimuthBodyParams {
    /**
    Target dome azimuth (degrees, North zero and increasing clockwise. i.e., 90 East, 180 South, 270 West)
    */
    #[serde(rename = "Azimuth")]
    azimuth: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Synchronize the current position of the dome to the given azimuth.

Synchronize the current position of the dome to the given azimuth.
*/
#[put("/dome/<device_number>/synctoazimuth")]
fn put_dome_synctoazimuth(
    PutDomeSynctoazimuthPathParams { device_number }: PutDomeSynctoazimuthPathParams,

    PutDomeSynctoazimuthBodyParams {
        azimuth,

        client_id,

        client_transaction_id,
    }: PutDomeSynctoazimuthBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetFilterwheelFocusoffsetsPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetFilterwheelFocusoffsetsQueryParams {
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

/**
Filter focus offsets

An integer array of filter focus offsets.
*/
#[get("/filterwheel/<device_number>/focusoffsets")]
fn get_filterwheel_focusoffsets(
    GetFilterwheelFocusoffsetsPathParams { device_number }: GetFilterwheelFocusoffsetsPathParams,

    GetFilterwheelFocusoffsetsQueryParams { client_id, client_transaction_id }: GetFilterwheelFocusoffsetsQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetFilterwheelNamesPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetFilterwheelNamesQueryParams {
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

/**
Filter wheel filter names

The names of the filters
*/
#[get("/filterwheel/<device_number>/names")]
fn get_filterwheel_names(
    GetFilterwheelNamesPathParams { device_number }: GetFilterwheelNamesPathParams,

    GetFilterwheelNamesQueryParams { client_id, client_transaction_id }: GetFilterwheelNamesQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetFilterwheelPositionPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetFilterwheelPositionQueryParams {
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

/**
Returns the current filter wheel position

Returns the current filter wheel position
*/
#[get("/filterwheel/<device_number>/position")]
fn get_filterwheel_position(
    GetFilterwheelPositionPathParams { device_number }: GetFilterwheelPositionPathParams,

    GetFilterwheelPositionQueryParams { client_id, client_transaction_id }: GetFilterwheelPositionQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutFilterwheelPositionPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutFilterwheelPositionBodyParams {
    /**
    The number of the filter wheel position to select
    */
    #[serde(rename = "Position")]
    position: i32,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the filter wheel position

Sets the filter wheel position
*/
#[put("/filterwheel/<device_number>/position")]
fn put_filterwheel_position(
    PutFilterwheelPositionPathParams { device_number }: PutFilterwheelPositionPathParams,

    PutFilterwheelPositionBodyParams {
        position,

        client_id,

        client_transaction_id,
    }: PutFilterwheelPositionBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetFocuserAbsolutePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetFocuserAbsoluteQueryParams {
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

/**
Indicates whether the focuser is capable of absolute position.

True if the focuser is capable of absolute position; that is, being commanded to a specific step location.
*/
#[get("/focuser/<device_number>/absolute")]
fn get_focuser_absolute(
    GetFocuserAbsolutePathParams { device_number }: GetFocuserAbsolutePathParams,

    GetFocuserAbsoluteQueryParams { client_id, client_transaction_id }: GetFocuserAbsoluteQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetFocuserIsmovingPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetFocuserIsmovingQueryParams {
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

/**
Indicates whether the focuser is currently moving.

True if the focuser is currently moving to a new position. False if the focuser is stationary.
*/
#[get("/focuser/<device_number>/ismoving")]
fn get_focuser_ismoving(
    GetFocuserIsmovingPathParams { device_number }: GetFocuserIsmovingPathParams,

    GetFocuserIsmovingQueryParams { client_id, client_transaction_id }: GetFocuserIsmovingQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetFocuserMaxincrementPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetFocuserMaxincrementQueryParams {
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

/**
Returns the focuser's maximum increment size.

Maximum increment size allowed by the focuser; i.e. the maximum number of steps allowed in one move operation.
*/
#[get("/focuser/<device_number>/maxincrement")]
fn get_focuser_maxincrement(
    GetFocuserMaxincrementPathParams { device_number }: GetFocuserMaxincrementPathParams,

    GetFocuserMaxincrementQueryParams { client_id, client_transaction_id }: GetFocuserMaxincrementQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetFocuserMaxstepPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetFocuserMaxstepQueryParams {
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

/**
Returns the focuser's maximum step size.

Maximum step position permitted.
*/
#[get("/focuser/<device_number>/maxstep")]
fn get_focuser_maxstep(GetFocuserMaxstepPathParams { device_number }: GetFocuserMaxstepPathParams, GetFocuserMaxstepQueryParams { client_id, client_transaction_id }: GetFocuserMaxstepQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetFocuserPositionPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetFocuserPositionQueryParams {
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

/**
Returns the focuser's current position.

Current focuser position, in steps.
*/
#[get("/focuser/<device_number>/position")]
fn get_focuser_position(
    GetFocuserPositionPathParams { device_number }: GetFocuserPositionPathParams,

    GetFocuserPositionQueryParams { client_id, client_transaction_id }: GetFocuserPositionQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetFocuserStepsizePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetFocuserStepsizeQueryParams {
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

/**
Returns the focuser's step size.

Step size (microns) for the focuser.
*/
#[get("/focuser/<device_number>/stepsize")]
fn get_focuser_stepsize(
    GetFocuserStepsizePathParams { device_number }: GetFocuserStepsizePathParams,

    GetFocuserStepsizeQueryParams { client_id, client_transaction_id }: GetFocuserStepsizeQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetFocuserTempcompPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetFocuserTempcompQueryParams {
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

/**
Retrieves the state of temperature compensation mode

Gets the state of temperature compensation mode (if available), else always False.
*/
#[get("/focuser/<device_number>/tempcomp")]
fn get_focuser_tempcomp(
    GetFocuserTempcompPathParams { device_number }: GetFocuserTempcompPathParams,

    GetFocuserTempcompQueryParams { client_id, client_transaction_id }: GetFocuserTempcompQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutFocuserTempcompPathParams {
    /**
    Zero based device number as set on the server
    */
    #[serde(rename = "device_number")]
    device_number: i32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutFocuserTempcompBodyParams {
    /**
    Set true to enable the focuser's temperature compensation mode, otherwise false for normal operation.
    */
    #[serde(rename = "TempComp")]
    temp_comp: bool,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "Client")]
    client: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionIDForm")]
    client_transaction_idform: u32,
}

/**
Sets the device's temperature compensation mode.

Sets the state of temperature compensation mode.
*/
#[put("/focuser/<device_number>/tempcomp")]
fn put_focuser_tempcomp(
    PutFocuserTempcompPathParams { device_number }: PutFocuserTempcompPathParams,

    PutFocuserTempcompBodyParams {
        temp_comp,

        client,

        client_transaction_idform,
    }: PutFocuserTempcompBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetFocuserTempcompavailablePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetFocuserTempcompavailableQueryParams {
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

/**
Indicates whether the focuser has temperature compensation.

True if focuser has temperature compensation available.
*/
#[get("/focuser/<device_number>/tempcompavailable")]
fn get_focuser_tempcompavailable(
    GetFocuserTempcompavailablePathParams { device_number }: GetFocuserTempcompavailablePathParams,

    GetFocuserTempcompavailableQueryParams { client_id, client_transaction_id }: GetFocuserTempcompavailableQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetFocuserTemperaturePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetFocuserTemperatureQueryParams {
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

/**
Returns the focuser's current temperature.

Current ambient temperature as measured by the focuser.
*/
#[get("/focuser/<device_number>/temperature")]
fn get_focuser_temperature(
    GetFocuserTemperaturePathParams { device_number }: GetFocuserTemperaturePathParams,

    GetFocuserTemperatureQueryParams { client_id, client_transaction_id }: GetFocuserTemperatureQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutFocuserHaltPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutFocuserHaltBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Immediatley stops focuser motion.

Immediately stop any focuser motion due to a previous Move(Int32) method call.
*/
#[put("/focuser/<device_number>/halt")]
fn put_focuser_halt(PutFocuserHaltPathParams { device_number }: PutFocuserHaltPathParams, PutFocuserHaltBodyParams { client_id, client_transaction_id }: PutFocuserHaltBodyParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutFocuserMovePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutFocuserMoveBodyParams {
    /**
    Step distance or absolute position, depending on the value of the Absolute property
    */
    #[serde(rename = "Position")]
    position: i32,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Moves the focuser to a new position.

Moves the focuser by the specified amount or to the specified position depending on the value of the Absolute property.
*/
#[put("/focuser/<device_number>/move")]
fn put_focuser_move(
    PutFocuserMovePathParams { device_number }: PutFocuserMovePathParams,

    PutFocuserMoveBodyParams {
        position,

        client_id,

        client_transaction_id,
    }: PutFocuserMoveBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetObservingconditionsAverageperiodPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetObservingconditionsAverageperiodQueryParams {
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

/**
Returns the time period over which observations will be averaged

Gets the time period over which observations will be averaged
*/
#[get("/observingconditions/<device_number>/averageperiod")]
fn get_observingconditions_averageperiod(
    GetObservingconditionsAverageperiodPathParams { device_number }: GetObservingconditionsAverageperiodPathParams,

    GetObservingconditionsAverageperiodQueryParams { client_id, client_transaction_id }: GetObservingconditionsAverageperiodQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutObservingconditionsAverageperiodPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutObservingconditionsAverageperiodBodyParams {
    /**
    Time period (hours) over which to average sensor readings
    */
    #[serde(rename = "AveragePeriod")]
    average_period: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the time period over which observations will be averaged

Sets the time period over which observations will be averaged
*/
#[put("/observingconditions/<device_number>/averageperiod")]
fn put_observingconditions_averageperiod(
    PutObservingconditionsAverageperiodPathParams { device_number }: PutObservingconditionsAverageperiodPathParams,

    PutObservingconditionsAverageperiodBodyParams {
        average_period,

        client_id,

        client_transaction_id,
    }: PutObservingconditionsAverageperiodBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetObservingconditionsCloudcoverPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetObservingconditionsCloudcoverQueryParams {
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

/**
Returns the amount of sky obscured by cloud

Gets the percentage of the sky obscured by cloud
*/
#[get("/observingconditions/<device_number>/cloudcover")]
fn get_observingconditions_cloudcover(
    GetObservingconditionsCloudcoverPathParams { device_number }: GetObservingconditionsCloudcoverPathParams,

    GetObservingconditionsCloudcoverQueryParams { client_id, client_transaction_id }: GetObservingconditionsCloudcoverQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetObservingconditionsDewpointPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetObservingconditionsDewpointQueryParams {
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

/**
Returns the atmospheric dew point at the observatory

Gets the atmospheric dew point at the observatory reported in °C.
*/
#[get("/observingconditions/<device_number>/dewpoint")]
fn get_observingconditions_dewpoint(
    GetObservingconditionsDewpointPathParams { device_number }: GetObservingconditionsDewpointPathParams,

    GetObservingconditionsDewpointQueryParams { client_id, client_transaction_id }: GetObservingconditionsDewpointQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetObservingconditionsHumidityPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetObservingconditionsHumidityQueryParams {
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

/**
Returns the atmospheric humidity at the observatory

Gets the atmospheric  humidity (%) at the observatory
*/
#[get("/observingconditions/<device_number>/humidity")]
fn get_observingconditions_humidity(
    GetObservingconditionsHumidityPathParams { device_number }: GetObservingconditionsHumidityPathParams,

    GetObservingconditionsHumidityQueryParams { client_id, client_transaction_id }: GetObservingconditionsHumidityQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetObservingconditionsPressurePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetObservingconditionsPressureQueryParams {
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

/**
Returns the atmospheric pressure at the observatory.

Gets the atmospheric pressure in hectoPascals at the observatory's altitude - NOT reduced to sea level.
*/
#[get("/observingconditions/<device_number>/pressure")]
fn get_observingconditions_pressure(
    GetObservingconditionsPressurePathParams { device_number }: GetObservingconditionsPressurePathParams,

    GetObservingconditionsPressureQueryParams { client_id, client_transaction_id }: GetObservingconditionsPressureQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetObservingconditionsRainratePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetObservingconditionsRainrateQueryParams {
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

/**
Returns the rain rate at the observatory.

Gets the rain rate (mm/hour) at the observatory.
*/
#[get("/observingconditions/<device_number>/rainrate")]
fn get_observingconditions_rainrate(
    GetObservingconditionsRainratePathParams { device_number }: GetObservingconditionsRainratePathParams,

    GetObservingconditionsRainrateQueryParams { client_id, client_transaction_id }: GetObservingconditionsRainrateQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetObservingconditionsSkybrightnessPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetObservingconditionsSkybrightnessQueryParams {
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

/**
Returns the sky brightness at the observatory

Gets the sky brightness at the observatory (Lux)
*/
#[get("/observingconditions/<device_number>/skybrightness")]
fn get_observingconditions_skybrightness(
    GetObservingconditionsSkybrightnessPathParams { device_number }: GetObservingconditionsSkybrightnessPathParams,

    GetObservingconditionsSkybrightnessQueryParams { client_id, client_transaction_id }: GetObservingconditionsSkybrightnessQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetObservingconditionsSkyqualityPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetObservingconditionsSkyqualityQueryParams {
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

/**
Returns the sky quality at the observatory

Gets the sky quality at the observatory (magnitudes per square arc second)
*/
#[get("/observingconditions/<device_number>/skyquality")]
fn get_observingconditions_skyquality(
    GetObservingconditionsSkyqualityPathParams { device_number }: GetObservingconditionsSkyqualityPathParams,

    GetObservingconditionsSkyqualityQueryParams { client_id, client_transaction_id }: GetObservingconditionsSkyqualityQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetObservingconditionsSkytemperaturePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetObservingconditionsSkytemperatureQueryParams {
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

/**
Returns the sky temperature at the observatory

Gets the sky temperature(°C) at the observatory.
*/
#[get("/observingconditions/<device_number>/skytemperature")]
fn get_observingconditions_skytemperature(
    GetObservingconditionsSkytemperaturePathParams { device_number }: GetObservingconditionsSkytemperaturePathParams,

    GetObservingconditionsSkytemperatureQueryParams { client_id, client_transaction_id }: GetObservingconditionsSkytemperatureQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetObservingconditionsStarfwhmPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetObservingconditionsStarfwhmQueryParams {
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

/**
Returns the seeing at the observatory

Gets the seeing at the observatory measured as star full width half maximum (FWHM) in arc secs.
*/
#[get("/observingconditions/<device_number>/starfwhm")]
fn get_observingconditions_starfwhm(
    GetObservingconditionsStarfwhmPathParams { device_number }: GetObservingconditionsStarfwhmPathParams,

    GetObservingconditionsStarfwhmQueryParams { client_id, client_transaction_id }: GetObservingconditionsStarfwhmQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetObservingconditionsTemperaturePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetObservingconditionsTemperatureQueryParams {
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

/**
Returns the temperature at the observatory

Gets the temperature(°C) at the observatory.
*/
#[get("/observingconditions/<device_number>/temperature")]
fn get_observingconditions_temperature(
    GetObservingconditionsTemperaturePathParams { device_number }: GetObservingconditionsTemperaturePathParams,

    GetObservingconditionsTemperatureQueryParams { client_id, client_transaction_id }: GetObservingconditionsTemperatureQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetObservingconditionsWinddirectionPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetObservingconditionsWinddirectionQueryParams {
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

/**
Returns the wind direction at the observatory

Gets the wind direction. The returned value must be between 0.0 and 360.0, interpreted according to the metereological standard, where a special value of 0.0 is returned when the wind speed is 0.0. Wind direction is measured clockwise from north, through east, where East=90.0, South=180.0, West=270.0 and North=360.0.
*/
#[get("/observingconditions/<device_number>/winddirection")]
fn get_observingconditions_winddirection(
    GetObservingconditionsWinddirectionPathParams { device_number }: GetObservingconditionsWinddirectionPathParams,

    GetObservingconditionsWinddirectionQueryParams { client_id, client_transaction_id }: GetObservingconditionsWinddirectionQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetObservingconditionsWindgustPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetObservingconditionsWindgustQueryParams {
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

/**
Returns the peak 3 second wind gust at the observatory over the last 2 minutes

Gets the peak 3 second wind gust(m/s) at the observatory over the last 2 minutes.
*/
#[get("/observingconditions/<device_number>/windgust")]
fn get_observingconditions_windgust(
    GetObservingconditionsWindgustPathParams { device_number }: GetObservingconditionsWindgustPathParams,

    GetObservingconditionsWindgustQueryParams { client_id, client_transaction_id }: GetObservingconditionsWindgustQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetObservingconditionsWindspeedPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetObservingconditionsWindspeedQueryParams {
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

/**
Returns the wind speed at the observatory.

Gets the wind speed(m/s) at the observatory.
*/
#[get("/observingconditions/<device_number>/windspeed")]
fn get_observingconditions_windspeed(
    GetObservingconditionsWindspeedPathParams { device_number }: GetObservingconditionsWindspeedPathParams,

    GetObservingconditionsWindspeedQueryParams { client_id, client_transaction_id }: GetObservingconditionsWindspeedQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutObservingconditionsRefreshPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutObservingconditionsRefreshBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Refreshes sensor values from hardware.

Forces the driver to immediately query its attached hardware to refresh sensor values.
*/
#[put("/observingconditions/<device_number>/refresh")]
fn put_observingconditions_refresh(
    PutObservingconditionsRefreshPathParams { device_number }: PutObservingconditionsRefreshPathParams,

    PutObservingconditionsRefreshBodyParams { client_id, client_transaction_id }: PutObservingconditionsRefreshBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetObservingconditionsSensordescriptionPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetObservingconditionsSensordescriptionQueryParams {
    /**
    Name of the sensor whose description is required
    */
    #[serde(rename = "SensorName")]
    sensor_name: String,

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

/**
Return a sensor description

Gets a description of the sensor with the name specified in the SensorName parameter
*/
#[get("/observingconditions/<device_number>/sensordescription")]
fn get_observingconditions_sensordescription(
    GetObservingconditionsSensordescriptionPathParams { device_number }: GetObservingconditionsSensordescriptionPathParams,

    GetObservingconditionsSensordescriptionQueryParams {
        sensor_name,

        client_id,

        client_transaction_id,
    }: GetObservingconditionsSensordescriptionQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetObservingconditionsTimesincelastupdatePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetObservingconditionsTimesincelastupdateQueryParams {
    /**
    Name of the sensor whose last update time is required
    */
    #[serde(rename = "SensorName")]
    sensor_name: String,

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

/**
Return the time since the sensor value was last updated

Gets the time since the sensor specified in the SensorName parameter was last updated
*/
#[get("/observingconditions/<device_number>/timesincelastupdate")]
fn get_observingconditions_timesincelastupdate(
    GetObservingconditionsTimesincelastupdatePathParams { device_number }: GetObservingconditionsTimesincelastupdatePathParams,

    GetObservingconditionsTimesincelastupdateQueryParams {
        sensor_name,

        client_id,

        client_transaction_id,
    }: GetObservingconditionsTimesincelastupdateQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetRotatorCanreversePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetRotatorCanreverseQueryParams {
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

/**
IIndicates whether the Rotator supports the Reverse method.

True if the Rotator supports the Reverse method.
*/
#[get("/rotator/<device_number>/canreverse")]
fn get_rotator_canreverse(
    GetRotatorCanreversePathParams { device_number }: GetRotatorCanreversePathParams,

    GetRotatorCanreverseQueryParams { client_id, client_transaction_id }: GetRotatorCanreverseQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetRotatorIsmovingPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetRotatorIsmovingQueryParams {
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

/**
Indicates whether the rotator is currently moving.

True if the rotator is currently moving to a new position. False if the focuser is stationary.
*/
#[get("/rotator/<device_number>/ismoving")]
fn get_rotator_ismoving(
    GetRotatorIsmovingPathParams { device_number }: GetRotatorIsmovingPathParams,

    GetRotatorIsmovingQueryParams { client_id, client_transaction_id }: GetRotatorIsmovingQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetRotatorMechanicalpositionPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetRotatorMechanicalpositionQueryParams {
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

/**
Returns the rotator's mechanical current position.

Returns the raw mechanical position of the rotator in degrees.
*/
#[get("/rotator/<device_number>/mechanicalposition")]
fn get_rotator_mechanicalposition(
    GetRotatorMechanicalpositionPathParams { device_number }: GetRotatorMechanicalpositionPathParams,

    GetRotatorMechanicalpositionQueryParams { client_id, client_transaction_id }: GetRotatorMechanicalpositionQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetRotatorPositionPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetRotatorPositionQueryParams {
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

/**
Returns the rotator's current position.

Current instantaneous Rotator position, in degrees.
*/
#[get("/rotator/<device_number>/position")]
fn get_rotator_position(
    GetRotatorPositionPathParams { device_number }: GetRotatorPositionPathParams,

    GetRotatorPositionQueryParams { client_id, client_transaction_id }: GetRotatorPositionQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetRotatorReversePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetRotatorReverseQueryParams {
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

/**
Returns the rotator’s Reverse state.

Returns the rotator’s Reverse state.
*/
#[get("/rotator/<device_number>/reverse")]
fn get_rotator_reverse(GetRotatorReversePathParams { device_number }: GetRotatorReversePathParams, GetRotatorReverseQueryParams { client_id, client_transaction_id }: GetRotatorReverseQueryParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutRotatorReversePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutRotatorReverseBodyParams {
    /**
    True if the rotation and angular direction must be reversed to match the optical characteristcs
    */
    #[serde(rename = "Reverse")]
    reverse: bool,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the rotator’s Reverse state.

Sets the rotator’s Reverse state.
*/
#[put("/rotator/<device_number>/reverse")]
fn put_rotator_reverse(
    PutRotatorReversePathParams { device_number }: PutRotatorReversePathParams,

    PutRotatorReverseBodyParams {
        reverse,

        client_id,

        client_transaction_id,
    }: PutRotatorReverseBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetRotatorStepsizePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetRotatorStepsizeQueryParams {
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

/**
Returns the minimum StepSize

The minimum StepSize, in degrees.
*/
#[get("/rotator/<device_number>/stepsize")]
fn get_rotator_stepsize(
    GetRotatorStepsizePathParams { device_number }: GetRotatorStepsizePathParams,

    GetRotatorStepsizeQueryParams { client_id, client_transaction_id }: GetRotatorStepsizeQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetRotatorTargetpositionPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetRotatorTargetpositionQueryParams {
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

/**
Returns the destination position angle.

The destination position angle for Move() and MoveAbsolute().
*/
#[get("/rotator/<device_number>/targetposition")]
fn get_rotator_targetposition(
    GetRotatorTargetpositionPathParams { device_number }: GetRotatorTargetpositionPathParams,

    GetRotatorTargetpositionQueryParams { client_id, client_transaction_id }: GetRotatorTargetpositionQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutRotatorHaltPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutRotatorHaltBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Immediatley stops rotator motion.

Immediately stop any Rotator motion due to a previous Move or MoveAbsolute method call.
*/
#[put("/rotator/<device_number>/halt")]
fn put_rotator_halt(PutRotatorHaltPathParams { device_number }: PutRotatorHaltPathParams, PutRotatorHaltBodyParams { client_id, client_transaction_id }: PutRotatorHaltBodyParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutRotatorMovePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutRotatorMoveBodyParams {
    /**
    Relative position to move in degrees from current Position.
    */
    #[serde(rename = "Position")]
    position: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Moves the rotator to a new relative position.

Causes the rotator to move Position degrees relative to the current Position value.
*/
#[put("/rotator/<device_number>/move")]
fn put_rotator_move(
    PutRotatorMovePathParams { device_number }: PutRotatorMovePathParams,

    PutRotatorMoveBodyParams {
        position,

        client_id,

        client_transaction_id,
    }: PutRotatorMoveBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutRotatorMoveabsolutePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutRotatorMoveabsoluteBodyParams {
    /**
    Absolute position in degrees.
    */
    #[serde(rename = "Position")]
    position: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Moves the rotator to a new absolute position.

Causes the rotator to move the absolute position of Position degrees.
*/
#[put("/rotator/<device_number>/moveabsolute")]
fn put_rotator_moveabsolute(
    PutRotatorMoveabsolutePathParams { device_number }: PutRotatorMoveabsolutePathParams,

    PutRotatorMoveabsoluteBodyParams {
        position,

        client_id,

        client_transaction_id,
    }: PutRotatorMoveabsoluteBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutRotatorMovemechanicalPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutRotatorMovemechanicalBodyParams {
    /**
    Absolute position in degrees.
    */
    #[serde(rename = "Position")]
    position: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Moves the rotator to a new raw mechanical position.

Causes the rotator to move the mechanical position of Position degrees.
*/
#[put("/rotator/<device_number>/movemechanical")]
fn put_rotator_movemechanical(
    PutRotatorMovemechanicalPathParams { device_number }: PutRotatorMovemechanicalPathParams,

    PutRotatorMovemechanicalBodyParams {
        position,

        client_id,

        client_transaction_id,
    }: PutRotatorMovemechanicalBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutRotatorSyncPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutRotatorSyncBodyParams {
    /**
    Absolute position in degrees.
    */
    #[serde(rename = "Position")]
    position: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Syncs the rotator to the specified position angle without moving it.

Causes the rotator to sync to the position of Position degrees.
*/
#[put("/rotator/<device_number>/sync")]
fn put_rotator_sync(
    PutRotatorSyncPathParams { device_number }: PutRotatorSyncPathParams,

    PutRotatorSyncBodyParams {
        position,

        client_id,

        client_transaction_id,
    }: PutRotatorSyncBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetSafetymonitorIssafePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetSafetymonitorIssafeQueryParams {
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

/**
Indicates whether the monitored state is safe for use.

Indicates whether the monitored state is safe for use. True if the state is safe, False if it is unsafe.
*/
#[get("/safetymonitor/<device_number>/issafe")]
fn get_safetymonitor_issafe(
    GetSafetymonitorIssafePathParams { device_number }: GetSafetymonitorIssafePathParams,

    GetSafetymonitorIssafeQueryParams { client_id, client_transaction_id }: GetSafetymonitorIssafeQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetSwitchMaxswitchPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetSwitchMaxswitchQueryParams {
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

/**
The number of switch devices managed by this driver

Returns the number of switch devices managed by this driver. Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/maxswitch")]
fn get_switch_maxswitch(
    GetSwitchMaxswitchPathParams { device_number }: GetSwitchMaxswitchPathParams,

    GetSwitchMaxswitchQueryParams { client_id, client_transaction_id }: GetSwitchMaxswitchQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetSwitchCanwritePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetSwitchCanwriteQueryParams {
    /**
    The device number (0 to MaxSwitch - 1)
    */
    #[serde(rename = "Id")]
    id: i32,

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

/**
Indicates whether the specified switch device can be written to

Reports if the specified switch device can be written to, default true. This is false if the device cannot be written to, for example a limit switch or a sensor.  Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/canwrite")]
fn get_switch_canwrite(
    GetSwitchCanwritePathParams { device_number }: GetSwitchCanwritePathParams,

    GetSwitchCanwriteQueryParams { id, client_id, client_transaction_id }: GetSwitchCanwriteQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetSwitchGetswitchPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetSwitchGetswitchQueryParams {
    /**
    The device number (0 to MaxSwitch - 1)
    */
    #[serde(rename = "Id")]
    id: i32,

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

/**
Return the state of switch device id as a boolean

Return the state of switch device id as a boolean.  Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/getswitch")]
fn get_switch_getswitch(
    GetSwitchGetswitchPathParams { device_number }: GetSwitchGetswitchPathParams,

    GetSwitchGetswitchQueryParams { id, client_id, client_transaction_id }: GetSwitchGetswitchQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetSwitchGetswitchdescriptionPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetSwitchGetswitchdescriptionQueryParams {
    /**
    The device number (0 to MaxSwitch - 1)
    */
    #[serde(rename = "Id")]
    id: i32,

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

/**
Gets the description of the specified switch device

Gets the description of the specified switch device. This is to allow a fuller description of the device to be returned, for example for a tool tip. Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/getswitchdescription")]
fn get_switch_getswitchdescription(
    GetSwitchGetswitchdescriptionPathParams { device_number }: GetSwitchGetswitchdescriptionPathParams,

    GetSwitchGetswitchdescriptionQueryParams { id, client_id, client_transaction_id }: GetSwitchGetswitchdescriptionQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetSwitchGetswitchnamePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetSwitchGetswitchnameQueryParams {
    /**
    The device number (0 to MaxSwitch - 1)
    */
    #[serde(rename = "Id")]
    id: i32,

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

/**
Gets the name of the specified switch device

Gets the name of the specified switch device. Devices are numbered from 0 to MaxSwitch - 1
*/
#[get("/switch/<device_number>/getswitchname")]
fn get_switch_getswitchname(
    GetSwitchGetswitchnamePathParams { device_number }: GetSwitchGetswitchnamePathParams,

    GetSwitchGetswitchnameQueryParams { id, client_id, client_transaction_id }: GetSwitchGetswitchnameQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetSwitchGetswitchvaluePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetSwitchGetswitchvalueQueryParams {
    /**
    The device number (0 to MaxSwitch - 1)
    */
    #[serde(rename = "Id")]
    id: i32,

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

/**
Gets the value of the specified switch device as a double

Gets the value of the specified switch device as a double. Devices are numbered from 0 to MaxSwitch - 1, The value of this switch is expected to be between MinSwitchValue and MaxSwitchValue.
*/
#[get("/switch/<device_number>/getswitchvalue")]
fn get_switch_getswitchvalue(
    GetSwitchGetswitchvaluePathParams { device_number }: GetSwitchGetswitchvaluePathParams,

    GetSwitchGetswitchvalueQueryParams { id, client_id, client_transaction_id }: GetSwitchGetswitchvalueQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetSwitchMinswitchvaluePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetSwitchMinswitchvalueQueryParams {
    /**
    The device number (0 to MaxSwitch - 1)
    */
    #[serde(rename = "Id")]
    id: i32,

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

/**
Gets the minimum value of the specified switch device as a double

Gets the minimum value of the specified switch device as a double. Devices are numbered from 0 to MaxSwitch - 1.
*/
#[get("/switch/<device_number>/minswitchvalue")]
fn get_switch_minswitchvalue(
    GetSwitchMinswitchvaluePathParams { device_number }: GetSwitchMinswitchvaluePathParams,

    GetSwitchMinswitchvalueQueryParams { id, client_id, client_transaction_id }: GetSwitchMinswitchvalueQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetSwitchMaxswitchvaluePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetSwitchMaxswitchvalueQueryParams {
    /**
    The device number (0 to MaxSwitch - 1)
    */
    #[serde(rename = "Id")]
    id: i32,

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

/**
Gets the maximum value of the specified switch device as a double

Gets the maximum value of the specified switch device as a double. Devices are numbered from 0 to MaxSwitch - 1.
*/
#[get("/switch/<device_number>/maxswitchvalue")]
fn get_switch_maxswitchvalue(
    GetSwitchMaxswitchvaluePathParams { device_number }: GetSwitchMaxswitchvaluePathParams,

    GetSwitchMaxswitchvalueQueryParams { id, client_id, client_transaction_id }: GetSwitchMaxswitchvalueQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutSwitchSetswitchPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutSwitchSetswitchBodyParams {
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
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets a switch controller device to the specified state, true or false

Sets a switch controller device to the specified state, true or false.
*/
#[put("/switch/<device_number>/setswitch")]
fn put_switch_setswitch(
    PutSwitchSetswitchPathParams { device_number }: PutSwitchSetswitchPathParams,

    PutSwitchSetswitchBodyParams {
        id,

        state,

        client_id,

        client_transaction_id,
    }: PutSwitchSetswitchBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutSwitchSetswitchnamePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutSwitchSetswitchnameBodyParams {
    /**
    The device number (0 to MaxSwitch - 1)
    */
    #[serde(rename = "Id")]
    id: (),

    /**
    The name of the device
    */
    #[serde(rename = "Name")]
    name: String,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets a switch device name to the specified value

Sets a switch device name to the specified value.
*/
#[put("/switch/<device_number>/setswitchname")]
fn put_switch_setswitchname(
    PutSwitchSetswitchnamePathParams { device_number }: PutSwitchSetswitchnamePathParams,

    PutSwitchSetswitchnameBodyParams {
        id,

        name,

        client_id,

        client_transaction_id,
    }: PutSwitchSetswitchnameBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutSwitchSetswitchvaluePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutSwitchSetswitchvalueBodyParams {
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
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets a switch device value to the specified value

Sets a switch device value to the specified value.
*/
#[put("/switch/<device_number>/setswitchvalue")]
fn put_switch_setswitchvalue(
    PutSwitchSetswitchvaluePathParams { device_number }: PutSwitchSetswitchvaluePathParams,

    PutSwitchSetswitchvalueBodyParams {
        id,

        value,

        client_id,

        client_transaction_id,
    }: PutSwitchSetswitchvalueBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetSwitchSwitchstepPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetSwitchSwitchstepQueryParams {
    /**
    The device number (0 to MaxSwitch - 1)
    */
    #[serde(rename = "Id")]
    id: i32,

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

/**
Returns the step size that this device supports (the difference between successive values of the device).

Returns the step size that this device supports (the difference between successive values of the device). Devices are numbered from 0 to MaxSwitch - 1.
*/
#[get("/switch/<device_number>/switchstep")]
fn get_switch_switchstep(
    GetSwitchSwitchstepPathParams { device_number }: GetSwitchSwitchstepPathParams,

    GetSwitchSwitchstepQueryParams { id, client_id, client_transaction_id }: GetSwitchSwitchstepQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeAlignmentmodePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeAlignmentmodeQueryParams {
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

/**
Returns the current mount alignment mode

Returns the alignment mode of the mount (Alt/Az, Polar, German Polar).  The alignment mode is specified as an integer value from the AlignmentModes Enum.
*/
#[get("/telescope/<device_number>/alignmentmode")]
fn get_telescope_alignmentmode(
    GetTelescopeAlignmentmodePathParams { device_number }: GetTelescopeAlignmentmodePathParams,

    GetTelescopeAlignmentmodeQueryParams { client_id, client_transaction_id }: GetTelescopeAlignmentmodeQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeAltitudePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeAltitudeQueryParams {
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

/**
Returns the mount's altitude above the horizon.

The altitude above the local horizon of the mount's current position (degrees, positive up)
*/
#[get("/telescope/<device_number>/altitude")]
fn get_telescope_altitude(
    GetTelescopeAltitudePathParams { device_number }: GetTelescopeAltitudePathParams,

    GetTelescopeAltitudeQueryParams { client_id, client_transaction_id }: GetTelescopeAltitudeQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeApertureareaPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeApertureareaQueryParams {
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

/**
Returns the telescope's aperture.

The area of the telescope's aperture, taking into account any obstructions (square meters)
*/
#[get("/telescope/<device_number>/aperturearea")]
fn get_telescope_aperturearea(
    GetTelescopeApertureareaPathParams { device_number }: GetTelescopeApertureareaPathParams,

    GetTelescopeApertureareaQueryParams { client_id, client_transaction_id }: GetTelescopeApertureareaQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeAperturediameterPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeAperturediameterQueryParams {
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

/**
Returns the telescope's effective aperture.

The telescope's effective aperture diameter (meters)
*/
#[get("/telescope/<device_number>/aperturediameter")]
fn get_telescope_aperturediameter(
    GetTelescopeAperturediameterPathParams { device_number }: GetTelescopeAperturediameterPathParams,

    GetTelescopeAperturediameterQueryParams { client_id, client_transaction_id }: GetTelescopeAperturediameterQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeAthomePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeAthomeQueryParams {
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

/**
Indicates whether the mount is at the home position.

True if the mount is stopped in the Home position. Set only following a FindHome()  operation, and reset with any slew operation. This property must be False if the telescope does not support homing.
*/
#[get("/telescope/<device_number>/athome")]
fn get_telescope_athome(
    GetTelescopeAthomePathParams { device_number }: GetTelescopeAthomePathParams,

    GetTelescopeAthomeQueryParams { client_id, client_transaction_id }: GetTelescopeAthomeQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeAtparkPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeAtparkQueryParams {
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

/**
Indicates whether the telescope is at the park position.

True if the telescope has been put into the parked state by the seee Park()  method. Set False by calling the Unpark() method.
*/
#[get("/telescope/<device_number>/atpark")]
fn get_telescope_atpark(
    GetTelescopeAtparkPathParams { device_number }: GetTelescopeAtparkPathParams,

    GetTelescopeAtparkQueryParams { client_id, client_transaction_id }: GetTelescopeAtparkQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeAzimuthPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeAzimuthQueryParams {
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

/**
Returns the mount's azimuth.

The azimuth at the local horizon of the mount's current position (degrees, North-referenced, positive East/clockwise).
*/
#[get("/telescope/<device_number>/azimuth")]
fn get_telescope_azimuth(
    GetTelescopeAzimuthPathParams { device_number }: GetTelescopeAzimuthPathParams,

    GetTelescopeAzimuthQueryParams { client_id, client_transaction_id }: GetTelescopeAzimuthQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeCanfindhomePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeCanfindhomeQueryParams {
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

/**
Indicates whether the mount can find the home position.

True if this telescope is capable of programmed finding its home position (FindHome()  method).
*/
#[get("/telescope/<device_number>/canfindhome")]
fn get_telescope_canfindhome(
    GetTelescopeCanfindhomePathParams { device_number }: GetTelescopeCanfindhomePathParams,

    GetTelescopeCanfindhomeQueryParams { client_id, client_transaction_id }: GetTelescopeCanfindhomeQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeCanparkPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeCanparkQueryParams {
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

/**
Indicates whether the telescope can be parked.

True if this telescope is capable of programmed parking (Park() method)
*/
#[get("/telescope/<device_number>/canpark")]
fn get_telescope_canpark(
    GetTelescopeCanparkPathParams { device_number }: GetTelescopeCanparkPathParams,

    GetTelescopeCanparkQueryParams { client_id, client_transaction_id }: GetTelescopeCanparkQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeCanpulseguidePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeCanpulseguideQueryParams {
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

/**
Indicates whether the telescope can be pulse guided.

True if this telescope is capable of software-pulsed guiding (via the PulseGuide(GuideDirections, Int32) method)
*/
#[get("/telescope/<device_number>/canpulseguide")]
fn get_telescope_canpulseguide(
    GetTelescopeCanpulseguidePathParams { device_number }: GetTelescopeCanpulseguidePathParams,

    GetTelescopeCanpulseguideQueryParams { client_id, client_transaction_id }: GetTelescopeCanpulseguideQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeCansetdeclinationratePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeCansetdeclinationrateQueryParams {
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

/**
Indicates whether the DeclinationRate property can be changed.

True if the DeclinationRate property can be changed to provide offset tracking in the declination axis.
*/
#[get("/telescope/<device_number>/cansetdeclinationrate")]
fn get_telescope_cansetdeclinationrate(
    GetTelescopeCansetdeclinationratePathParams { device_number }: GetTelescopeCansetdeclinationratePathParams,

    GetTelescopeCansetdeclinationrateQueryParams { client_id, client_transaction_id }: GetTelescopeCansetdeclinationrateQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeCansetguideratesPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeCansetguideratesQueryParams {
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

/**
Indicates whether the DeclinationRate property can be changed.

True if the guide rate properties used for PulseGuide(GuideDirections, Int32) can ba adjusted.
*/
#[get("/telescope/<device_number>/cansetguiderates")]
fn get_telescope_cansetguiderates(
    GetTelescopeCansetguideratesPathParams { device_number }: GetTelescopeCansetguideratesPathParams,

    GetTelescopeCansetguideratesQueryParams { client_id, client_transaction_id }: GetTelescopeCansetguideratesQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeCansetparkPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeCansetparkQueryParams {
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

/**
Indicates whether the telescope park position can be set.

True if this telescope is capable of programmed setting of its park position (SetPark() method)
*/
#[get("/telescope/<device_number>/cansetpark")]
fn get_telescope_cansetpark(
    GetTelescopeCansetparkPathParams { device_number }: GetTelescopeCansetparkPathParams,

    GetTelescopeCansetparkQueryParams { client_id, client_transaction_id }: GetTelescopeCansetparkQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeCansetpiersidePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeCansetpiersideQueryParams {
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

/**
Indicates whether the telescope SideOfPier can be set.

True if the SideOfPier property can be set, meaning that the mount can be forced to flip.
*/
#[get("/telescope/<device_number>/cansetpierside")]
fn get_telescope_cansetpierside(
    GetTelescopeCansetpiersidePathParams { device_number }: GetTelescopeCansetpiersidePathParams,

    GetTelescopeCansetpiersideQueryParams { client_id, client_transaction_id }: GetTelescopeCansetpiersideQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeCansetrightascensionratePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeCansetrightascensionrateQueryParams {
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

/**
Indicates whether the RightAscensionRate property can be changed.

True if the RightAscensionRate property can be changed to provide offset tracking in the right ascension axis. .
*/
#[get("/telescope/<device_number>/cansetrightascensionrate")]
fn get_telescope_cansetrightascensionrate(
    GetTelescopeCansetrightascensionratePathParams { device_number }: GetTelescopeCansetrightascensionratePathParams,

    GetTelescopeCansetrightascensionrateQueryParams { client_id, client_transaction_id }: GetTelescopeCansetrightascensionrateQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeCansettrackingPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeCansettrackingQueryParams {
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

/**
Indicates whether the Tracking property can be changed.

True if the Tracking property can be changed, turning telescope sidereal tracking on and off.
*/
#[get("/telescope/<device_number>/cansettracking")]
fn get_telescope_cansettracking(
    GetTelescopeCansettrackingPathParams { device_number }: GetTelescopeCansettrackingPathParams,

    GetTelescopeCansettrackingQueryParams { client_id, client_transaction_id }: GetTelescopeCansettrackingQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeCanslewPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeCanslewQueryParams {
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

/**
Indicates whether the telescope can slew synchronously.

True if this telescope is capable of programmed slewing (synchronous or asynchronous) to equatorial coordinates
*/
#[get("/telescope/<device_number>/canslew")]
fn get_telescope_canslew(
    GetTelescopeCanslewPathParams { device_number }: GetTelescopeCanslewPathParams,

    GetTelescopeCanslewQueryParams { client_id, client_transaction_id }: GetTelescopeCanslewQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeCanslewaltazPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeCanslewaltazQueryParams {
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

/**
Indicates whether the telescope can slew synchronously to AltAz coordinates.

True if this telescope is capable of programmed slewing (synchronous or asynchronous) to local horizontal coordinates
*/
#[get("/telescope/<device_number>/canslewaltaz")]
fn get_telescope_canslewaltaz(
    GetTelescopeCanslewaltazPathParams { device_number }: GetTelescopeCanslewaltazPathParams,

    GetTelescopeCanslewaltazQueryParams { client_id, client_transaction_id }: GetTelescopeCanslewaltazQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeCanslewaltazasyncPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeCanslewaltazasyncQueryParams {
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

/**
Indicates whether the telescope can slew asynchronously to AltAz coordinates.

True if this telescope is capable of programmed asynchronous slewing to local horizontal coordinates
*/
#[get("/telescope/<device_number>/canslewaltazasync")]
fn get_telescope_canslewaltazasync(
    GetTelescopeCanslewaltazasyncPathParams { device_number }: GetTelescopeCanslewaltazasyncPathParams,

    GetTelescopeCanslewaltazasyncQueryParams { client_id, client_transaction_id }: GetTelescopeCanslewaltazasyncQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeCanslewasyncPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeCanslewasyncQueryParams {
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

/**
Indicates whether the telescope can slew asynchronously.

True if this telescope is capable of programmed asynchronous slewing to equatorial coordinates.
*/
#[get("/telescope/<device_number>/canslewasync")]
fn get_telescope_canslewasync(
    GetTelescopeCanslewasyncPathParams { device_number }: GetTelescopeCanslewasyncPathParams,

    GetTelescopeCanslewasyncQueryParams { client_id, client_transaction_id }: GetTelescopeCanslewasyncQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeCansyncPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeCansyncQueryParams {
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

/**
Indicates whether the telescope can sync to equatorial coordinates.

True if this telescope is capable of programmed synching to equatorial coordinates.
*/
#[get("/telescope/<device_number>/cansync")]
fn get_telescope_cansync(
    GetTelescopeCansyncPathParams { device_number }: GetTelescopeCansyncPathParams,

    GetTelescopeCansyncQueryParams { client_id, client_transaction_id }: GetTelescopeCansyncQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeCansyncaltazPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeCansyncaltazQueryParams {
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

/**
Indicates whether the telescope can sync to local horizontal coordinates.

True if this telescope is capable of programmed synching to local horizontal coordinates
*/
#[get("/telescope/<device_number>/cansyncaltaz")]
fn get_telescope_cansyncaltaz(
    GetTelescopeCansyncaltazPathParams { device_number }: GetTelescopeCansyncaltazPathParams,

    GetTelescopeCansyncaltazQueryParams { client_id, client_transaction_id }: GetTelescopeCansyncaltazQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeCanunparkPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeCanunparkQueryParams {
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

/**
Indicates whether the telescope can be unparked.

True if this telescope is capable of programmed unparking (UnPark() method)
*/
#[get("/telescope/<device_number>/canunpark")]
fn get_telescope_canunpark(
    GetTelescopeCanunparkPathParams { device_number }: GetTelescopeCanunparkPathParams,

    GetTelescopeCanunparkQueryParams { client_id, client_transaction_id }: GetTelescopeCanunparkQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeDeclinationPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeDeclinationQueryParams {
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

/**
Returns the mount's declination.

The declination (degrees) of the mount's current equatorial coordinates, in the coordinate system given by the EquatorialSystem property. Reading the property will raise an error if the value is unavailable.
*/
#[get("/telescope/<device_number>/declination")]
fn get_telescope_declination(
    GetTelescopeDeclinationPathParams { device_number }: GetTelescopeDeclinationPathParams,

    GetTelescopeDeclinationQueryParams { client_id, client_transaction_id }: GetTelescopeDeclinationQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeDeclinationratePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeDeclinationrateQueryParams {
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

/**
Returns the telescope's declination tracking rate.

The declination tracking rate (arcseconds per second, default = 0.0)
*/
#[get("/telescope/<device_number>/declinationrate")]
fn get_telescope_declinationrate(
    GetTelescopeDeclinationratePathParams { device_number }: GetTelescopeDeclinationratePathParams,

    GetTelescopeDeclinationrateQueryParams { client_id, client_transaction_id }: GetTelescopeDeclinationrateQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeDeclinationratePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeDeclinationrateBodyParams {
    /**
    Declination tracking rate (arcseconds per second)
    */
    #[serde(rename = "DeclinationRate")]
    declination_rate: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the telescope's declination tracking rate.

Sets the declination tracking rate (arcseconds per second)
*/
#[put("/telescope/<device_number>/declinationrate")]
fn put_telescope_declinationrate(
    PutTelescopeDeclinationratePathParams { device_number }: PutTelescopeDeclinationratePathParams,

    PutTelescopeDeclinationrateBodyParams {
        declination_rate,

        client_id,

        client_transaction_id,
    }: PutTelescopeDeclinationrateBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeDoesrefractionPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeDoesrefractionQueryParams {
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

/**
Indicates whether atmospheric refraction is applied to coordinates.

True if the telescope or driver applies atmospheric refraction to coordinates.
*/
#[get("/telescope/<device_number>/doesrefraction")]
fn get_telescope_doesrefraction(
    GetTelescopeDoesrefractionPathParams { device_number }: GetTelescopeDoesrefractionPathParams,

    GetTelescopeDoesrefractionQueryParams { client_id, client_transaction_id }: GetTelescopeDoesrefractionQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeDoesrefractionPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeDoesrefractionBodyParams {
    /**
    Set True to make the telescope or driver applie atmospheric refraction to coordinates.
    */
    #[serde(rename = "DoesRefraction")]
    does_refraction: bool,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Determines whether atmospheric refraction is applied to coordinates.

Causes the rotator to move Position degrees relative to the current Position value.
*/
#[put("/telescope/<device_number>/doesrefraction")]
fn put_telescope_doesrefraction(
    PutTelescopeDoesrefractionPathParams { device_number }: PutTelescopeDoesrefractionPathParams,

    PutTelescopeDoesrefractionBodyParams {
        does_refraction,

        client_id,

        client_transaction_id,
    }: PutTelescopeDoesrefractionBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeEquatorialsystemPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeEquatorialsystemQueryParams {
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

/**
Returns the current equatorial coordinate system used by this telescope.

Returns the current equatorial coordinate system used by this telescope (e.g. Topocentric or J2000).
*/
#[get("/telescope/<device_number>/equatorialsystem")]
fn get_telescope_equatorialsystem(
    GetTelescopeEquatorialsystemPathParams { device_number }: GetTelescopeEquatorialsystemPathParams,

    GetTelescopeEquatorialsystemQueryParams { client_id, client_transaction_id }: GetTelescopeEquatorialsystemQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeFocallengthPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeFocallengthQueryParams {
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

/**
Returns the telescope's focal length in meters.

The telescope's focal length in meters
*/
#[get("/telescope/<device_number>/focallength")]
fn get_telescope_focallength(
    GetTelescopeFocallengthPathParams { device_number }: GetTelescopeFocallengthPathParams,

    GetTelescopeFocallengthQueryParams { client_id, client_transaction_id }: GetTelescopeFocallengthQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeGuideratedeclinationPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeGuideratedeclinationQueryParams {
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

/**
Returns the  current Declination rate offset for telescope guiding

The current Declination movement rate offset for telescope guiding (degrees/sec)
*/
#[get("/telescope/<device_number>/guideratedeclination")]
fn get_telescope_guideratedeclination(
    GetTelescopeGuideratedeclinationPathParams { device_number }: GetTelescopeGuideratedeclinationPathParams,

    GetTelescopeGuideratedeclinationQueryParams { client_id, client_transaction_id }: GetTelescopeGuideratedeclinationQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeGuideratedeclinationPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeGuideratedeclinationBodyParams {
    /**
    Declination movement rate offset degrees/sec).
    */
    #[serde(rename = "GuideRateDeclination")]
    guide_rate_declination: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the  current Declination rate offset for telescope guiding.

Sets the current Declination movement rate offset for telescope guiding (degrees/sec).
*/
#[put("/telescope/<device_number>/guideratedeclination")]
fn put_telescope_guideratedeclination(
    PutTelescopeGuideratedeclinationPathParams { device_number }: PutTelescopeGuideratedeclinationPathParams,

    PutTelescopeGuideratedeclinationBodyParams {
        guide_rate_declination,

        client_id,

        client_transaction_id,
    }: PutTelescopeGuideratedeclinationBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeGuideraterightascensionPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeGuideraterightascensionQueryParams {
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

/**
Returns the  current RightAscension rate offset for telescope guiding

The current RightAscension movement rate offset for telescope guiding (degrees/sec)
*/
#[get("/telescope/<device_number>/guideraterightascension")]
fn get_telescope_guideraterightascension(
    GetTelescopeGuideraterightascensionPathParams { device_number }: GetTelescopeGuideraterightascensionPathParams,

    GetTelescopeGuideraterightascensionQueryParams { client_id, client_transaction_id }: GetTelescopeGuideraterightascensionQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeGuideraterightascensionPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeGuideraterightascensionBodyParams {
    /**
    RightAscension movement rate offset degrees/sec).
    */
    #[serde(rename = "GuideRateRightAscension")]
    guide_rate_right_ascension: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the  current RightAscension rate offset for telescope guiding.

Sets the current RightAscension movement rate offset for telescope guiding (degrees/sec).
*/
#[put("/telescope/<device_number>/guideraterightascension")]
fn put_telescope_guideraterightascension(
    PutTelescopeGuideraterightascensionPathParams { device_number }: PutTelescopeGuideraterightascensionPathParams,

    PutTelescopeGuideraterightascensionBodyParams {
        guide_rate_right_ascension,

        client_id,

        client_transaction_id,
    }: PutTelescopeGuideraterightascensionBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeIspulseguidingPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeIspulseguidingQueryParams {
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

/**
Indicates whether the telescope is currently executing a PulseGuide command

True if a PulseGuide(GuideDirections, Int32) command is in progress, False otherwise
*/
#[get("/telescope/<device_number>/ispulseguiding")]
fn get_telescope_ispulseguiding(
    GetTelescopeIspulseguidingPathParams { device_number }: GetTelescopeIspulseguidingPathParams,

    GetTelescopeIspulseguidingQueryParams { client_id, client_transaction_id }: GetTelescopeIspulseguidingQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeRightascensionPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeRightascensionQueryParams {
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

/**
Returns the mount's right ascension coordinate.

The right ascension (hours) of the mount's current equatorial coordinates, in the coordinate system given by the EquatorialSystem property
*/
#[get("/telescope/<device_number>/rightascension")]
fn get_telescope_rightascension(
    GetTelescopeRightascensionPathParams { device_number }: GetTelescopeRightascensionPathParams,

    GetTelescopeRightascensionQueryParams { client_id, client_transaction_id }: GetTelescopeRightascensionQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeRightascensionratePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeRightascensionrateQueryParams {
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

/**
Returns the telescope's right ascension tracking rate.

The right ascension tracking rate (arcseconds per second, default = 0.0)
*/
#[get("/telescope/<device_number>/rightascensionrate")]
fn get_telescope_rightascensionrate(
    GetTelescopeRightascensionratePathParams { device_number }: GetTelescopeRightascensionratePathParams,

    GetTelescopeRightascensionrateQueryParams { client_id, client_transaction_id }: GetTelescopeRightascensionrateQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeRightascensionratePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeRightascensionrateBodyParams {
    /**
    Right ascension tracking rate (arcseconds per second)
    */
    #[serde(rename = "RightAscensionRate")]
    right_ascension_rate: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the telescope's right ascension tracking rate.

Sets the right ascension tracking rate (arcseconds per second)
*/
#[put("/telescope/<device_number>/rightascensionrate")]
fn put_telescope_rightascensionrate(
    PutTelescopeRightascensionratePathParams { device_number }: PutTelescopeRightascensionratePathParams,

    PutTelescopeRightascensionrateBodyParams {
        right_ascension_rate,

        client_id,

        client_transaction_id,
    }: PutTelescopeRightascensionrateBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeSideofpierPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeSideofpierQueryParams {
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

/**
Returns the mount's pointing state.

Indicates the pointing state of the mount. 0 = pierEast, 1 = pierWest, -1= pierUnknown
*/
#[get("/telescope/<device_number>/sideofpier")]
fn get_telescope_sideofpier(
    GetTelescopeSideofpierPathParams { device_number }: GetTelescopeSideofpierPathParams,

    GetTelescopeSideofpierQueryParams { client_id, client_transaction_id }: GetTelescopeSideofpierQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeSideofpierPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeSideofpierBodyParams {
    /**
    New pointing state.
    */
    #[serde(rename = "SideOfPier")]
    side_of_pier: i32,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the mount's pointing state.

Sets the pointing state of the mount. 0 = pierEast, 1 = pierWest
*/
#[put("/telescope/<device_number>/sideofpier")]
fn put_telescope_sideofpier(
    PutTelescopeSideofpierPathParams { device_number }: PutTelescopeSideofpierPathParams,

    PutTelescopeSideofpierBodyParams {
        side_of_pier,

        client_id,

        client_transaction_id,
    }: PutTelescopeSideofpierBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeSiderealtimePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeSiderealtimeQueryParams {
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

/**
Returns the local apparent sidereal time.

The local apparent sidereal time from the telescope's internal clock (hours, sidereal).
*/
#[get("/telescope/<device_number>/siderealtime")]
fn get_telescope_siderealtime(
    GetTelescopeSiderealtimePathParams { device_number }: GetTelescopeSiderealtimePathParams,

    GetTelescopeSiderealtimeQueryParams { client_id, client_transaction_id }: GetTelescopeSiderealtimeQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeSiteelevationPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeSiteelevationQueryParams {
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

/**
Returns the observing site's elevation above mean sea level.

The elevation above mean sea level (meters) of the site at which the telescope is located.
*/
#[get("/telescope/<device_number>/siteelevation")]
fn get_telescope_siteelevation(
    GetTelescopeSiteelevationPathParams { device_number }: GetTelescopeSiteelevationPathParams,

    GetTelescopeSiteelevationQueryParams { client_id, client_transaction_id }: GetTelescopeSiteelevationQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeSiteelevationPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeSiteelevationBodyParams {
    /**
    Elevation above mean sea level (metres).
    */
    #[serde(rename = "SiteElevation")]
    site_elevation: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the observing site's elevation above mean sea level.

Sets the elevation above mean sea level (metres) of the site at which the telescope is located.
*/
#[put("/telescope/<device_number>/siteelevation")]
fn put_telescope_siteelevation(
    PutTelescopeSiteelevationPathParams { device_number }: PutTelescopeSiteelevationPathParams,

    PutTelescopeSiteelevationBodyParams {
        site_elevation,

        client_id,

        client_transaction_id,
    }: PutTelescopeSiteelevationBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeSitelatitudePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeSitelatitudeQueryParams {
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

/**
Returns the observing site's latitude.

The geodetic(map) latitude (degrees, positive North, WGS84) of the site at which the telescope is located.
*/
#[get("/telescope/<device_number>/sitelatitude")]
fn get_telescope_sitelatitude(
    GetTelescopeSitelatitudePathParams { device_number }: GetTelescopeSitelatitudePathParams,

    GetTelescopeSitelatitudeQueryParams { client_id, client_transaction_id }: GetTelescopeSitelatitudeQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeSitelatitudePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeSitelatitudeBodyParams {
    /**
    Site latitude (degrees)
    */
    #[serde(rename = "SiteLatitude")]
    site_latitude: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the observing site's latitude.

Sets the observing site's latitude (degrees).
*/
#[put("/telescope/<device_number>/sitelatitude")]
fn put_telescope_sitelatitude(
    PutTelescopeSitelatitudePathParams { device_number }: PutTelescopeSitelatitudePathParams,

    PutTelescopeSitelatitudeBodyParams {
        site_latitude,

        client_id,

        client_transaction_id,
    }: PutTelescopeSitelatitudeBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeSitelongitudePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeSitelongitudeQueryParams {
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

/**
Returns the observing site's longitude.

The longitude (degrees, positive East, WGS84) of the site at which the telescope is located.
*/
#[get("/telescope/<device_number>/sitelongitude")]
fn get_telescope_sitelongitude(
    GetTelescopeSitelongitudePathParams { device_number }: GetTelescopeSitelongitudePathParams,

    GetTelescopeSitelongitudeQueryParams { client_id, client_transaction_id }: GetTelescopeSitelongitudeQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeSitelongitudePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeSitelongitudeBodyParams {
    /**
    Site longitude (degrees, positive East, WGS84)
    */
    #[serde(rename = "SiteLongitude")]
    site_longitude: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the observing site's longitude.

Sets the observing site's longitude (degrees, positive East, WGS84).
*/
#[put("/telescope/<device_number>/sitelongitude")]
fn put_telescope_sitelongitude(
    PutTelescopeSitelongitudePathParams { device_number }: PutTelescopeSitelongitudePathParams,

    PutTelescopeSitelongitudeBodyParams {
        site_longitude,

        client_id,

        client_transaction_id,
    }: PutTelescopeSitelongitudeBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeSlewingPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeSlewingQueryParams {
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

/**
Indicates whether the telescope is currently slewing.

True if telescope is currently moving in response to one of the Slew methods or the MoveAxis(TelescopeAxes, Double) method, False at all other times.
*/
#[get("/telescope/<device_number>/slewing")]
fn get_telescope_slewing(
    GetTelescopeSlewingPathParams { device_number }: GetTelescopeSlewingPathParams,

    GetTelescopeSlewingQueryParams { client_id, client_transaction_id }: GetTelescopeSlewingQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeSlewsettletimePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeSlewsettletimeQueryParams {
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

/**
Returns the post-slew settling time.

Returns the post-slew settling time (sec.).
*/
#[get("/telescope/<device_number>/slewsettletime")]
fn get_telescope_slewsettletime(
    GetTelescopeSlewsettletimePathParams { device_number }: GetTelescopeSlewsettletimePathParams,

    GetTelescopeSlewsettletimeQueryParams { client_id, client_transaction_id }: GetTelescopeSlewsettletimeQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeSlewsettletimePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeSlewsettletimeBodyParams {
    /**
    Settling time (integer sec.).
    */
    #[serde(rename = "SlewSettleTime")]
    slew_settle_time: i32,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the post-slew settling time.

Sets the  post-slew settling time (integer sec.).
*/
#[put("/telescope/<device_number>/slewsettletime")]
fn put_telescope_slewsettletime(
    PutTelescopeSlewsettletimePathParams { device_number }: PutTelescopeSlewsettletimePathParams,

    PutTelescopeSlewsettletimeBodyParams {
        slew_settle_time,

        client_id,

        client_transaction_id,
    }: PutTelescopeSlewsettletimeBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeTargetdeclinationPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeTargetdeclinationQueryParams {
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

/**
Returns the current target declination.

The declination (degrees, positive North) for the target of an equatorial slew or sync operation
*/
#[get("/telescope/<device_number>/targetdeclination")]
fn get_telescope_targetdeclination(
    GetTelescopeTargetdeclinationPathParams { device_number }: GetTelescopeTargetdeclinationPathParams,

    GetTelescopeTargetdeclinationQueryParams { client_id, client_transaction_id }: GetTelescopeTargetdeclinationQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeTargetdeclinationPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeTargetdeclinationBodyParams {
    /**
    Target declination(degrees)
    */
    #[serde(rename = "TargetDeclination")]
    target_declination: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the target declination of a slew or sync.

Sets the declination (degrees, positive North) for the target of an equatorial slew or sync operation
*/
#[put("/telescope/<device_number>/targetdeclination")]
fn put_telescope_targetdeclination(
    PutTelescopeTargetdeclinationPathParams { device_number }: PutTelescopeTargetdeclinationPathParams,

    PutTelescopeTargetdeclinationBodyParams {
        target_declination,

        client_id,

        client_transaction_id,
    }: PutTelescopeTargetdeclinationBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeTargetrightascensionPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeTargetrightascensionQueryParams {
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

/**
Returns the current target right ascension.

The right ascension (hours) for the target of an equatorial slew or sync operation
*/
#[get("/telescope/<device_number>/targetrightascension")]
fn get_telescope_targetrightascension(
    GetTelescopeTargetrightascensionPathParams { device_number }: GetTelescopeTargetrightascensionPathParams,

    GetTelescopeTargetrightascensionQueryParams { client_id, client_transaction_id }: GetTelescopeTargetrightascensionQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeTargetrightascensionPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeTargetrightascensionBodyParams {
    /**
    Target right ascension(hours)
    */
    #[serde(rename = "TargetRightAscension")]
    target_right_ascension: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the target right ascension of a slew or sync.

Sets the right ascension (hours) for the target of an equatorial slew or sync operation
*/
#[put("/telescope/<device_number>/targetrightascension")]
fn put_telescope_targetrightascension(
    PutTelescopeTargetrightascensionPathParams { device_number }: PutTelescopeTargetrightascensionPathParams,

    PutTelescopeTargetrightascensionBodyParams {
        target_right_ascension,

        client_id,

        client_transaction_id,
    }: PutTelescopeTargetrightascensionBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeTrackingPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeTrackingQueryParams {
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

/**
Indicates whether the telescope is tracking.

Returns the state of the telescope's sidereal tracking drive.
*/
#[get("/telescope/<device_number>/tracking")]
fn get_telescope_tracking(
    GetTelescopeTrackingPathParams { device_number }: GetTelescopeTrackingPathParams,

    GetTelescopeTrackingQueryParams { client_id, client_transaction_id }: GetTelescopeTrackingQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeTrackingPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeTrackingBodyParams {
    /**
    Tracking enabled / disabled
    */
    #[serde(rename = "Tracking")]
    tracking: bool,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Enables or disables telescope tracking.

Sets the state of the telescope's sidereal tracking drive.
*/
#[put("/telescope/<device_number>/tracking")]
fn put_telescope_tracking(
    PutTelescopeTrackingPathParams { device_number }: PutTelescopeTrackingPathParams,

    PutTelescopeTrackingBodyParams {
        tracking,

        client_id,

        client_transaction_id,
    }: PutTelescopeTrackingBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeTrackingratePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeTrackingrateQueryParams {
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

/**
Returns the current tracking rate.

The current tracking rate of the telescope's sidereal drive.
*/
#[get("/telescope/<device_number>/trackingrate")]
fn get_telescope_trackingrate(
    GetTelescopeTrackingratePathParams { device_number }: GetTelescopeTrackingratePathParams,

    GetTelescopeTrackingrateQueryParams { client_id, client_transaction_id }: GetTelescopeTrackingrateQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeTrackingratePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeTrackingrateBodyParams {
    /**
    New tracking rate
    */
    #[serde(rename = "TrackingRate")]
    tracking_rate: i32,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the mount's tracking rate.

Sets the tracking rate of the telescope's sidereal drive. 0 = driveSidereal, 1 = driveLunar, 2 = driveSolar, 3 = driveKing
*/
#[put("/telescope/<device_number>/trackingrate")]
fn put_telescope_trackingrate(
    PutTelescopeTrackingratePathParams { device_number }: PutTelescopeTrackingratePathParams,

    PutTelescopeTrackingrateBodyParams {
        tracking_rate,

        client_id,

        client_transaction_id,
    }: PutTelescopeTrackingrateBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeTrackingratesPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeTrackingratesQueryParams {
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

/**
Returns a collection of supported DriveRates values.

Returns an array of supported DriveRates values that describe the permissible values of the TrackingRate property for this telescope type.
*/
#[get("/telescope/<device_number>/trackingrates")]
fn get_telescope_trackingrates(
    GetTelescopeTrackingratesPathParams { device_number }: GetTelescopeTrackingratesPathParams,

    GetTelescopeTrackingratesQueryParams { client_id, client_transaction_id }: GetTelescopeTrackingratesQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeUtcdatePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeUtcdateQueryParams {
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

/**
Returns the UTC date/time of the telescope's internal clock.

The UTC date/time of the telescope's internal clock in ISO 8601 format including fractional seconds. The general format (in Microsoft custom date format style) is yyyy-MM-ddTHH:mm:ss.fffffffZ E.g. 2016-03-04T17:45:31.1234567Z or 2016-11-14T07:03:08.1234567Z Please note the compulsary trailing Z indicating the 'Zulu', UTC time zone.
*/
#[get("/telescope/<device_number>/utcdate")]
fn get_telescope_utcdate(
    GetTelescopeUtcdatePathParams { device_number }: GetTelescopeUtcdatePathParams,

    GetTelescopeUtcdateQueryParams { client_id, client_transaction_id }: GetTelescopeUtcdateQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeUtcdatePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeUtcdateBodyParams {
    /**
    UTC date/time in ISO 8601 format.
    */
    #[serde(rename = "UTCDate")]
    utcdate: String,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the UTC date/time of the telescope's internal clock.

The UTC date/time of the telescope's internal clock in ISO 8601 format including fractional seconds. The general format (in Microsoft custom date format style) is yyyy-MM-ddTHH:mm:ss.fffffffZ E.g. 2016-03-04T17:45:31.1234567Z or 2016-11-14T07:03:08.1234567Z Please note the compulsary trailing Z indicating the 'Zulu', UTC time zone.
*/
#[put("/telescope/<device_number>/utcdate")]
fn put_telescope_utcdate(
    PutTelescopeUtcdatePathParams { device_number }: PutTelescopeUtcdatePathParams,

    PutTelescopeUtcdateBodyParams {
        utcdate,

        client_id,

        client_transaction_id,
    }: PutTelescopeUtcdateBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeAbortslewPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeAbortslewBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Immediatley stops a slew in progress.

Immediately Stops a slew in progress.
*/
#[put("/telescope/<device_number>/abortslew")]
fn put_telescope_abortslew(
    PutTelescopeAbortslewPathParams { device_number }: PutTelescopeAbortslewPathParams,

    PutTelescopeAbortslewBodyParams { client_id, client_transaction_id }: PutTelescopeAbortslewBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeAxisratesPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeAxisratesQueryParams {
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
    axis: i32,
}

/**
Returns the rates at which the telescope may be moved about the specified axis.

The rates at which the telescope may be moved about the specified axis by the MoveAxis(TelescopeAxes, Double) method.
*/
#[get("/telescope/<device_number>/axisrates")]
fn get_telescope_axisrates(
    GetTelescopeAxisratesPathParams { device_number }: GetTelescopeAxisratesPathParams,

    GetTelescopeAxisratesQueryParams {
        client_id,

        client_transaction_id,

        axis,
    }: GetTelescopeAxisratesQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeCanmoveaxisPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeCanmoveaxisQueryParams {
    /**
    The axis about which rate information is desired. 0 = axisPrimary, 1 = axisSecondary, 2 = axisTertiary.
    */
    #[serde(rename = "Axis")]
    axis: i32,

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

/**
Indicates whether the telescope can move the requested axis.

True if this telescope can move the requested axis.
*/
#[get("/telescope/<device_number>/canmoveaxis")]
fn get_telescope_canmoveaxis(
    GetTelescopeCanmoveaxisPathParams { device_number }: GetTelescopeCanmoveaxisPathParams,

    GetTelescopeCanmoveaxisQueryParams {
        axis,

        client_id,

        client_transaction_id,
    }: GetTelescopeCanmoveaxisQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct GetTelescopeDestinationsideofpierPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Query))]
struct GetTelescopeDestinationsideofpierQueryParams {
    /**
    Right Ascension coordinate (0.0 to 23.99999999 hours)
    */
    #[serde(rename = "RightAscension")]
    right_ascension: f64,

    /**
    Declination coordinate (-90.0 to +90.0 degrees)
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

/**
Predicts the pointing state after a German equatorial mount slews to given coordinates.

Predicts the pointing state that a German equatorial mount will be in if it slews to the given coordinates. The  return value will be one of - 0 = pierEast, 1 = pierWest, -1 = pierUnknown
*/
#[get("/telescope/<device_number>/destinationsideofpier")]
fn get_telescope_destinationsideofpier(
    GetTelescopeDestinationsideofpierPathParams { device_number }: GetTelescopeDestinationsideofpierPathParams,

    GetTelescopeDestinationsideofpierQueryParams {
        right_ascension,

        declination,

        client_id,

        client_transaction_id,
    }: GetTelescopeDestinationsideofpierQueryParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeFindhomePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeFindhomeBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Moves the mount to the "home" position.

Locates the telescope's "home" position (synchronous)
*/
#[put("/telescope/<device_number>/findhome")]
fn put_telescope_findhome(
    PutTelescopeFindhomePathParams { device_number }: PutTelescopeFindhomePathParams,

    PutTelescopeFindhomeBodyParams { client_id, client_transaction_id }: PutTelescopeFindhomeBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeMoveaxisPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeMoveaxisBodyParams {
    /**
    The axis about which rate information is desired. 0 = axisPrimary, 1 = axisSecondary, 2 = axisTertiary.
    */
    #[serde(rename = "Axis")]
    axis: (),

    /**
    The rate of motion (deg/sec) about the specified axis
    */
    #[serde(rename = "Rate")]
    rate: f64,

    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Moves a telescope axis at the given rate.

Move the telescope in one axis at the given rate.
*/
#[put("/telescope/<device_number>/moveaxis")]
fn put_telescope_moveaxis(
    PutTelescopeMoveaxisPathParams { device_number }: PutTelescopeMoveaxisPathParams,

    PutTelescopeMoveaxisBodyParams {
        axis,

        rate,

        client_id,

        client_transaction_id,
    }: PutTelescopeMoveaxisBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeParkPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeParkBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Park the mount

Move the telescope to its park position, stop all motion (or restrict to a small safe range), and set AtPark to True. )
*/
#[put("/telescope/<device_number>/park")]
fn put_telescope_park(PutTelescopeParkPathParams { device_number }: PutTelescopeParkPathParams, PutTelescopeParkBodyParams { client_id, client_transaction_id }: PutTelescopeParkBodyParams) {}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopePulseguidePathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopePulseguideBodyParams {
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
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Moves the scope in the given direction for the given time.

Moves the scope in the given direction for the given interval or time at the rate given by the corresponding guide rate property
*/
#[put("/telescope/<device_number>/pulseguide")]
fn put_telescope_pulseguide(
    PutTelescopePulseguidePathParams { device_number }: PutTelescopePulseguidePathParams,

    PutTelescopePulseguideBodyParams {
        direction,

        duration,

        client_id,

        client_transaction_id,
    }: PutTelescopePulseguideBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeSetparkPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeSetparkBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Sets the telescope's park position

Sets the telescope's park position to be its current position.
*/
#[put("/telescope/<device_number>/setpark")]
fn put_telescope_setpark(
    PutTelescopeSetparkPathParams { device_number }: PutTelescopeSetparkPathParams,

    PutTelescopeSetparkBodyParams { client_id, client_transaction_id }: PutTelescopeSetparkBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeSlewtoaltazPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeSlewtoaltazBodyParams {
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
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Synchronously slew to the given local horizontal coordinates.

Move the telescope to the given local horizontal coordinates, return when slew is complete
*/
#[put("/telescope/<device_number>/slewtoaltaz")]
fn put_telescope_slewtoaltaz(
    PutTelescopeSlewtoaltazPathParams { device_number }: PutTelescopeSlewtoaltazPathParams,

    PutTelescopeSlewtoaltazBodyParams {
        azimuth,

        altitude,

        client_id,

        client_transaction_id,
    }: PutTelescopeSlewtoaltazBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeSlewtoaltazasyncPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeSlewtoaltazasyncBodyParams {
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
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Asynchronously slew to the given local horizontal coordinates.

Move the telescope to the given local horizontal coordinates, return immediatley after the slew starts. The client can poll the Slewing method to determine when the mount reaches the intended coordinates.
*/
#[put("/telescope/<device_number>/slewtoaltazasync")]
fn put_telescope_slewtoaltazasync(
    PutTelescopeSlewtoaltazasyncPathParams { device_number }: PutTelescopeSlewtoaltazasyncPathParams,

    PutTelescopeSlewtoaltazasyncBodyParams {
        azimuth,

        altitude,

        client_id,

        client_transaction_id,
    }: PutTelescopeSlewtoaltazasyncBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeSlewtocoordinatesPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeSlewtocoordinatesBodyParams {
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
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Synchronously slew to the given equatorial coordinates.

Move the telescope to the given equatorial coordinates, return when slew is complete
*/
#[put("/telescope/<device_number>/slewtocoordinates")]
fn put_telescope_slewtocoordinates(
    PutTelescopeSlewtocoordinatesPathParams { device_number }: PutTelescopeSlewtocoordinatesPathParams,

    PutTelescopeSlewtocoordinatesBodyParams {
        right_ascension,

        declination,

        client_id,

        client_transaction_id,
    }: PutTelescopeSlewtocoordinatesBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeSlewtocoordinatesasyncPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeSlewtocoordinatesasyncBodyParams {
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
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Asynchronously slew to the given equatorial coordinates.

Move the telescope to the given equatorial coordinates, return immediatley after the slew starts. The client can poll the Slewing method to determine when the mount reaches the intended coordinates.
*/
#[put("/telescope/<device_number>/slewtocoordinatesasync")]
fn put_telescope_slewtocoordinatesasync(
    PutTelescopeSlewtocoordinatesasyncPathParams { device_number }: PutTelescopeSlewtocoordinatesasyncPathParams,

    PutTelescopeSlewtocoordinatesasyncBodyParams {
        right_ascension,

        declination,

        client_id,

        client_transaction_id,
    }: PutTelescopeSlewtocoordinatesasyncBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeSlewtotargetPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeSlewtotargetBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Synchronously slew to the TargetRightAscension and TargetDeclination coordinates.

Move the telescope to the TargetRightAscension and TargetDeclination equatorial coordinates, return when slew is complete
*/
#[put("/telescope/<device_number>/slewtotarget")]
fn put_telescope_slewtotarget(
    PutTelescopeSlewtotargetPathParams { device_number }: PutTelescopeSlewtotargetPathParams,

    PutTelescopeSlewtotargetBodyParams { client_id, client_transaction_id }: PutTelescopeSlewtotargetBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeSlewtotargetasyncPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeSlewtotargetasyncBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Asynchronously slew to the TargetRightAscension and TargetDeclination coordinates.

Move the telescope to the TargetRightAscension and TargetDeclination equatorial coordinates, return immediatley after the slew starts. The client can poll the Slewing method to determine when the mount reaches the intended coordinates.
*/
#[put("/telescope/<device_number>/slewtotargetasync")]
fn put_telescope_slewtotargetasync(
    PutTelescopeSlewtotargetasyncPathParams { device_number }: PutTelescopeSlewtotargetasyncPathParams,

    PutTelescopeSlewtotargetasyncBodyParams { client_id, client_transaction_id }: PutTelescopeSlewtotargetasyncBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeSynctoaltazPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeSynctoaltazBodyParams {
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
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Syncs to the given local horizontal coordinates.

Matches the scope's local horizontal coordinates to the given local horizontal coordinates.
*/
#[put("/telescope/<device_number>/synctoaltaz")]
fn put_telescope_synctoaltaz(
    PutTelescopeSynctoaltazPathParams { device_number }: PutTelescopeSynctoaltazPathParams,

    PutTelescopeSynctoaltazBodyParams {
        azimuth,

        altitude,

        client_id,

        client_transaction_id,
    }: PutTelescopeSynctoaltazBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeSynctocoordinatesPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeSynctocoordinatesBodyParams {
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
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Syncs to the given equatorial coordinates.

Matches the scope's equatorial coordinates to the given equatorial coordinates.
*/
#[put("/telescope/<device_number>/synctocoordinates")]
fn put_telescope_synctocoordinates(
    PutTelescopeSynctocoordinatesPathParams { device_number }: PutTelescopeSynctocoordinatesPathParams,

    PutTelescopeSynctocoordinatesBodyParams {
        right_ascension,

        declination,

        client_id,

        client_transaction_id,
    }: PutTelescopeSynctocoordinatesBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeSynctotargetPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeSynctotargetBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Syncs to the TargetRightAscension and TargetDeclination coordinates.

Matches the scope's equatorial coordinates to the TargetRightAscension and TargetDeclination equatorial coordinates.
*/
#[put("/telescope/<device_number>/synctotarget")]
fn put_telescope_synctotarget(
    PutTelescopeSynctotargetPathParams { device_number }: PutTelescopeSynctotargetPathParams,

    PutTelescopeSynctotargetBodyParams { client_id, client_transaction_id }: PutTelescopeSynctotargetBodyParams,
) {
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Path))]
struct PutTelescopeUnparkPathParams {
    /**
    Zero based device number as set on the server (0 to 4294967295)
    */
    #[serde(rename = "device_number")]
    device_number: u32,
}

#[derive(Deserialize, FromRequest)]
#[from_request(via(Form))]

struct PutTelescopeUnparkBodyParams {
    /**
    Client's unique ID. (0 to 4294967295). The client should choose a value at start-up, e.g. a random value between 0 and 65535, and send this value on every transaction to help associate entries in device logs with this particular client.
    */
    #[serde(rename = "ClientID")]
    client_id: u32,

    /**
    Client's transaction ID. (0 to 4294967295). The client should start this count at 1 and increment by one on each successive transaction. This will aid associating entries in device logs with corresponding entries in client side logs.
    */
    #[serde(rename = "ClientTransactionID")]
    client_transaction_id: u32,
}

/**
Unparks the mount.

Takes telescope out of the Parked state. )
*/
#[put("/telescope/<device_number>/unpark")]
fn put_telescope_unpark(PutTelescopeUnparkPathParams { device_number }: PutTelescopeUnparkPathParams, PutTelescopeUnparkBodyParams { client_id, client_transaction_id }: PutTelescopeUnparkBodyParams) {
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
