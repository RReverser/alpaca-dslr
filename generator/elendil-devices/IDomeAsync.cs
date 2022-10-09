using System.Threading.Tasks;

namespace ES.Ascom.Alpaca.Devices
{
    /// <summary>
    /// Defines the capabilities supported by an ASCOM Alpaca Dome device
    /// </summary>
    /// <seealso cref="IDome"/>
    public interface IDomeAsync : IDeviceAsync
    {


        /// <summary>
        /// The dome altitude (degrees, horizon zero and increasing positive to 90 zenith).
        /// </summary>
        /// <returns>The dome altitude</returns>
        Task<double> GetAltitudeAsync();

        /// <summary>
        /// Indicates whether the dome is in the home position. This is normally used following a FindHome() operation.
        /// The value is reset with any azimuth slew operation that moves the dome away from the home position.
        /// AtHome may also become true during normal slew operations, if the dome passes through the home position
        /// and the dome controller hardware is capable of detecting that; or at the end of a slew operation if the
        /// dome comes to rest at the home position.
        /// </summary>
        /// <returns>True if the dome is in the home position, false if not</returns>
        Task<bool> IsAtHomeAsync();

        /// <summary>
        /// True if the dome is in the programmed park position.
        /// Set only following a Park() operation and reset with any slew operation.
        /// </summary>
        /// <returns>True if the dome is in the park position, false if not</returns>
        Task<bool> IsAtParkAsync();

        /// <summary>
        /// Returns the dome azimuth (degrees, North zero and increasing clockwise, i.e., 90 East, 180 South, 270 West)
        /// </summary>
        /// <returns>The dome azimuth in degrees</returns>
        Task<double> GetAzimuthAsync();

        /// <summary>
        /// True if the dome can move to the home position.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanFindHomeAsync();

        /// <summary>
        /// True if the dome is capable of programmed parking (Park() method)
        /// </summary>
        /// <returns></returns>
        Task<bool> CanParkAsync();

        /// <summary>
        /// True if driver is capable of setting the dome altitude.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanSetAltitudeAsync();

        /// <summary>
        /// True if driver is capable of setting the dome azimuth.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanSetAzimuthAsync();

        /// <summary>
        /// True if driver is capable of setting the dome park position.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanSetParkAsync();

        /// <summary>
        /// True if driver is capable of automatically operating shutter
        /// </summary>
        /// <returns></returns>
        Task<bool> CanSetShutterAsync();

        /// <summary>
        /// True if driver is capable of slaving to a telescope.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanSlaveAsync();

        /// <summary>
        /// True if driver is capable of synchronizing the dome azimuth position using the SyncToAzimuth(Double) method.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanSyncAzimuthAsync();

        /// <summary>
        /// Returns the status of the dome shutter or roll-off roof.
        /// </summary>
        /// <returns></returns>
        Task<ShutterState> GetShutterStatusAsync();

        /// <summary>
        /// True if the dome is slaved to the telescope in its hardware, else False.
        /// </summary>
        /// <returns></returns>
        Task<bool> IsSlavedAsync();

        /// <summary>
        /// Sets whether the dome is slaved to the telescope
        /// </summary>
        /// <param name="slaved"></param>
        Task SetSlavedAsync(bool slaved);

        /// <summary>
        /// Indicates whether the any part of the dome is moving
        /// </summary>
        /// <returns>True if any part of the dome is currently moving, False if all dome components are steady.</returns>
        Task<bool> IsSlewingAsync();

        /// <summary>
        /// Immediately cancel current dome operation.
        /// Calling this method will immediately disable hardware slewing (Slaved will become False).
        /// </summary>
        Task AbortSlewAsync();

        /// <summary>
        /// Close the shutter or otherwise shield telescope from the sky.
        /// </summary>
        Task CloseShutterAsync();

        /// <summary>
        /// Start operation to search for the dome home position.
        /// After Home position is established initializes Azimuth to the default value and sets the AtHome flag.
        /// </summary>
        Task FindHomeAsync();

        /// <summary>
        /// Open shutter or otherwise expose telescope to the sky.
        /// </summary>
        Task OpenShutterAsync();

        /// <summary>
        /// Rotate dome in azimuth to park position.
        /// After assuming programmed park position, sets AtPark flag.
        /// </summary>
        Task ParkAsync();

        /// <summary>
        /// Set the current azimuth, altitude position of dome to be the park position
        /// </summary>
        Task SetParkAsync();

        /// <summary>
        /// Slew the dome to the given altitude position.
        /// </summary>
        /// <param name="altitude">Target dome altitude (degrees, horizon zero and increasing positive to 90 zenith)</param>
        Task SlewToAltitudeAsync(double altitude);

        /// <summary>
        /// Slew the dome to the given azimuth position.
        /// </summary>
        /// <param name="azimuth">Target dome azimuth (degrees, North zero and increasing clockwise. i.e., 90 East, 180 South, 270 West)</param>
        Task SlewToAzimuthAsync(double azimuth);

        /// <summary>
        /// Synchronize the current position of the dome to the given azimuth.
        /// </summary>
        Task SyncToAzimuthAsync(double azimuth);
    }
}