using System.Collections.Generic;

namespace ES.Ascom.Alpaca.Devices
{
    /// <summary>
    /// Base interface of every client implementation of ASCOM Alpaca device 
    /// </summary>
    /// <seealso cref="IDeviceAsync"/>
    public interface IDevice
    {
        /// <summary>
        /// Device number configured in the device instance
        /// </summary>
        int DeviceNumber { get; }
        
        /// <summary>
        /// Invokes the specified device-specific action.
        /// </summary>
        /// <param name="actionName">A well known name that represents the action to be carried out.</param>
        /// <param name="actionParameters">List of required parameters or an Empty String if none are required</param>
        /// <returns>String response from the device</returns>
        string InvokeAction(string actionName, string actionParameters);
        
        /// <summary>
        /// Transmits an arbitrary string to the device and does not wait for a response. Optionally, protocol framing characters may be added to the string before transmission.
        /// </summary>
        /// <param name="command">The literal command string to be transmitted</param>
        /// <param name="raw">If set to true the string is transmitted 'as-is', if set to false then protocol framing characters may be added prior to transmission</param>
        void SendCommandBlind(string command, bool raw = false);
        
        /// <summary>
        /// Transmits an arbitrary string to the device and waits for a boolean response. Optionally, protocol framing characters may be added to the string before transmission.
        /// </summary>
        /// <param name="command">The literal command string to be transmitted</param>
        /// <param name="raw">If set to true the string is transmitted 'as-is', if set to false then protocol framing characters may be added prior to transmission</param>
        /// <returns>Boolean response from the device.</returns>
        bool SendCommandBool(string command, bool raw = false);
        
        /// <summary>
        /// Transmits an arbitrary string to the device and waits for a string response. Optionally, protocol framing characters may be added to the string before transmission.
        /// </summary>
        /// <param name="command">The literal command string to be transmitted.</param>
        /// <param name="raw">If set to true the string is transmitted 'as-is', if set to false then protocol framing characters may be added prior to transmission</param>
        /// <returns>String response from the device.</returns>
        string SendCommandString(string command, bool raw = false);

        /// <summary>
        /// Retrieves the connected state of the device
        /// </summary>
        /// <returns><c>True</c> if the device is connected, <c>false</c> if not</returns>
        bool IsConnected();

        /// <summary>
        /// Sets the connected state of the device
        /// </summary>
        /// <param name="connected">Set True to connect to the device hardware, set False to disconnect from the device hardware</param>
        void SetConnected(bool connected);

        /// <summary>
        /// Retrieves a description of the device (manufacturer, model number, ...)
        /// </summary>
        /// <returns>Device description</returns>
        string GetDescription();

        /// <summary>
        /// Retrieves descriptive and version information about the device. 
        /// </summary>
        /// <returns>Device information</returns>
        string GetDriverInfo();

        /// <summary>
        /// Retrieves the version of the driver
        /// </summary>
        /// <returns>A string containing only the major and minor version of the driver.</returns>
        string GetDriverVersion();

        /// <summary>
        /// The ASCOM Device interface version number that this device supports.
        /// </summary>
        /// <returns></returns>
        int GetInterfaceVersion();

        /// <summary>
        /// Retrieves the name of the device
        /// </summary>
        /// <returns>Name of the device</returns>
        string GetName();

        /// <summary>
        /// Returns the list of action names supported by this driver. 
        /// </summary>
        /// <returns></returns>
        IList<string> GetSupportedActions();
    }
}