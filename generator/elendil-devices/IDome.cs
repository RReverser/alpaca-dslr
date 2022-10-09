namespace ES.Ascom.Alpaca.Devices
{
    /// <summary>
    /// Defines the capabilities supported by an ASCOM Alpaca Dome device
    /// </summary>
    /// <seealso cref="IDomeAsync"/>
    public interface IDome : IDevice
    {
        /// <summary>
        /// The dome altitude (degrees, horizon zero and increasing positive to 90 zenith).
        /// </summary>
        /// <returns>The dome altitude</returns>
        double GetAltitude();

        /// <summary>
        /// Indicates whether the dome is in the home position. This is normally used following a FindHome() operation.
        /// The value is reset with any azimuth slew operation that moves the dome away from the home position.
        /// AtHome may also become true during normal slew operations, if the dome passes through the home position
        /// and the dome controller hardware is capable of detecting that; or at the end of a slew operation if the
        /// dome comes to rest at the home position.
        /// </summary>
        /// <returns>True if the dome is in the home position, false if not</returns>
        bool IsAtHome();

        /// <summary>
        /// True if the dome is in the programmed park position.
        /// Set only following a Park() operation and reset with any slew operation.
        /// </summary>
        /// <returns>True if the dome is in the park position, false if not</returns>
        bool IsAtPark();

        /// <summary>
        /// Returns the dome azimuth (degrees, North zero and increasing clockwise, i.e., 90 East, 180 South, 270 West)
        /// </summary>
        /// <returns>The dome azimuth in degrees</returns>
        double GetAzimuth();

        /// <summary>
        /// True if the dome can move to the home position.
        /// </summary>
        /// <returns></returns>
        bool CanFindHome();

        /// <summary>
        /// True if the dome is capable of programmed parking (Park() method)
        /// </summary>
        /// <returns></returns>
        bool CanPark();

        /// <summary>
        /// True if driver is capable of setting the dome altitude.
        /// </summary>
        /// <returns></returns>
        bool CanSetAltitude();

        /// <summary>
        /// True if driver is capable of setting the dome azimuth.
        /// </summary>
        /// <returns></returns>
        bool CanSetAzimuth();

        /// <summary>
        /// True if driver is capable of setting the dome park position.
        /// </summary>
        /// <returns></returns>
        bool CanSetPark();

        /// <summary>
        /// True if driver is capable of automatically operating shutter
        /// </summary>
        /// <returns></returns>
        bool CanSetShutter();

        /// <summary>
        /// True if driver is capable of slaving to a telescope.
        /// </summary>
        /// <returns></returns>
        bool CanSlave();

        /// <summary>
        /// True if driver is capable of synchronizing the dome azimuth position using the SyncToAzimuth(Double) method.
        /// </summary>
        /// <returns></returns>
        bool CanSyncAzimuth();

        /// <summary>
        /// Returns the status of the dome shutter or roll-off roof.
        /// </summary>
        /// <returns></returns>
        ShutterState GetShutterStatus();

        /// <summary>
        /// True if the dome is slaved to the telescope in its hardware, else False.
        /// </summary>
        /// <returns></returns>
        bool IsSlaved();

        /// <summary>
        /// Sets whether the dome is slaved to the telescope
        /// </summary>
        /// <param name="slaved">True if telescope is slaved to dome, otherwise false</param>
        void SetSlaved(bool slaved);

        /// <summary>
        /// Indicates whether the any part of the dome is moving
        /// </summary>
        /// <returns>True if any part of the dome is currently moving, False if all dome components are steady.</returns>
        bool IsSlewing();

        /// <summary>
        /// Immediately cancel current dome operation.
        /// Calling this method will immediately disable hardware slewing (Slaved will become False).
        /// </summary>
        void AbortSlew();

        /// <summary>
        /// Close the shutter or otherwise shield telescope from the sky.
        /// </summary>
        void CloseShutter();

        /// <summary>
        /// Start operation to search for the dome home position.
        /// After Home position is established initializes Azimuth to the default value and sets the AtHome flag.
        /// </summary>
        void FindHome();

        /// <summary>
        /// Open shutter or otherwise expose telescope to the sky.
        /// </summary>
        void OpenShutter();

        /// <summary>
        /// Rotate dome in azimuth to park position.
        /// After assuming programmed park position, sets AtPark flag.
        /// </summary>
        void Park();

        /// <summary>
        /// Set the current azimuth, altitude position of dome to be the park position
        /// </summary>
        void SetPark();
        
        /// <summary>
        /// Slew the dome to the given altitude position.
        /// </summary>
        /// <param name="altitude">Target dome altitude (degrees, horizon zero and increasing positive to 90 zenith)</param>
        void SlewToAltitude(double altitude);

        /// <summary>
        /// Slew the dome to the given azimuth position.
        /// </summary>
        /// <param name="azimuth">Target dome azimuth (degrees, North zero and increasing clockwise. i.e., 90 East, 180 South, 270 West)</param>
        void SlewToAzimuth(double azimuth);

        /// <summary>
        /// Synchronize the current position of the dome to the given azimuth.
        /// </summary>
        void SyncToAzimuth(double azimuth);
    }
}