using System.Collections.Generic;
using System.Threading.Tasks;

namespace ES.Ascom.Alpaca.Devices
{
    /// <summary>
    /// Base interface of every client implementation of ASCOM Alpaca device 
    /// </summary>
    /// <seealso cref="IDevice"/>
    public interface IDeviceAsync
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
        Task<string> InvokeActionAsync(string actionName, string actionParameters);

        /// <summary>
        /// Transmits an arbitrary string to the device and does not wait for a response. Optionally, protocol framing characters may be added to the string before transmission.
        /// </summary>
        /// <param name="command">The literal command string to be transmitted</param>
        /// <param name="raw">If set to true the string is transmitted 'as-is', if set to false then protocol framing characters may be added prior to transmission</param>
        Task SendCommandBlindAsync(string command, bool raw = false);

        /// <summary>
        /// Transmits an arbitrary string to the device and waits for a boolean response. Optionally, protocol framing characters may be added to the string before transmission.
        /// </summary>
        /// <param name="command">The literal command string to be transmitted</param>
        /// <param name="raw">If set to true the string is transmitted 'as-is', if set to false then protocol framing characters may be added prior to transmission</param>
        /// <returns>Boolean response from the device.</returns>
        Task<bool> SendCommandBoolAsync(string command, bool raw = false);

        /// <summary>
        /// Transmits an arbitrary string to the device and waits for a string response. Optionally, protocol framing characters may be added to the string before transmission.
        /// </summary>
        /// <param name="command">The literal command string to be transmitted.</param>
        /// <param name="raw">If set to true the string is transmitted 'as-is', if set to false then protocol framing characters may be added prior to transmission</param>
        /// <returns>String response from the device.</returns>
        Task<string> SendCommandStringAsync(string command, bool raw = false);

        /// <summary>
        /// Retrieves the connected state of the device
        /// </summary>
        /// <returns><c>True</c> if the device is connected, <c>false</c> if not</returns>
        Task<bool> IsConnectedAsync();

        /// <summary>
        /// Sets the connected state of the device
        /// </summary>
        /// <param name="connected">Set True to connect to the device hardware, set False to disconnect from the device hardware</param>
        Task SetConnectedAsync(bool connected);

        /// <summary>
        /// Retrieves a description of the device (manufacturer, model number, ...)
        /// </summary>
        /// <returns>Device description</returns>
        Task<string> GetDescriptionAsync();

        /// <summary>
        /// Retrieves descriptive and version information about the device. 
        /// </summary>
        /// <returns>Device information</returns>
        Task<string> GetDriverInfoAsync();

        /// <summary>
        /// Retrieves the version of the driver
        /// </summary>
        /// <returns>A string containing only the major and minor version of the driver.</returns>
        Task<string> GetDriverVersionAsync();

        /// <summary>
        /// The ASCOM Device interface version number that this device supports.
        /// </summary>
        /// <returns></returns>

        Task<int> GetInterfaceVersionAsync();

        /// <summary>
        /// Retrieves the name of the device
        /// </summary>
        /// <returns>Name of the device</returns>
        Task<string> GetNameAsync();

        /// <summary>
        /// Returns the list of action names supported by this driver. 
        /// </summary>
        /// <returns></returns>
        Task<IList<string>> GetSupportedActionsAsync();
    }
}