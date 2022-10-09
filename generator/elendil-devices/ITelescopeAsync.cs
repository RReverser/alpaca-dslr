using System;
using System.Collections.Generic;
using System.Threading.Tasks;

namespace ES.Ascom.Alpaca.Devices
{
    /// <summary>
    /// Defines the capabilities supported by an ASCOM Alpaca Telescope device
    /// </summary>
    /// <seealso cref="ITelescope"/>
    public interface ITelescopeAsync : IDeviceAsync
    {
        /// <summary>
        /// Returns the alignment mode of the mount (Alt/Az, Polar, German Polar).
        /// </summary>
        /// <returns></returns>
        Task<AlignmentMode> GetAlignmentModeAsync();

        /// <summary>
        /// Returns the Altitude above the local horizon of the telescope's current position (degrees, positive up)
        /// </summary>
        /// <returns></returns>
        Task<double> GetAltitudeAsync();

        /// <summary>
        /// Returns thhe area of the telescope's aperture, taking into account any obstructions (square meters)
        /// </summary>
        /// <returns></returns>
        Task<double> GetApertureAreaAsync();

        /// <summary>
        /// Returns the telescope's effective aperture diameter (meters)
        /// </summary>
        /// <returns></returns>
        Task<double> GetApertureDiameterAsync();

        /// <summary>
        /// Indicates whether the mount is at the home position.
        /// </summary>
        /// <returns>True if the mount is stopped in the Home position.</returns>
        Task<bool> IsAtHomeAsync();

        /// <summary>
        /// Indicates whether the telescope is at the park position.
        /// </summary>
        /// <returns>True if the telescope has been put into the parked state</returns>
        Task<bool> IsAtParkAsync();

        /// <summary>
        /// Returns the azimuth at the local horizon of the telescope's current position (degrees, North-referenced,
        /// positive East/clockwise).
        /// </summary>
        /// <returns></returns>
        Task<double> GetAzimuthAsync();

        /// <summary>
        /// Indicates whether the mount can find the home position.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanFindHomeAsync();

        /// <summary>
        /// Indicates whether the telescope can be parked.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanParkAsync();

        /// <summary>
        /// ndicates whether the telescope can be pulse guided.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanPulseGuideAsync();

        /// <summary>
        /// Indicates whether the DeclinationRate property can be changed.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanSetDeclinationRateAsync();

        /// <summary>
        /// Indicates whether the DeclinationRate property can be changed.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanSetGuideRatesAsync();

        /// <summary>
        /// Indicates whether the telescope park position can be set.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanSetParkAsync();

        /// <summary>
        /// Indicates whether the telescope SideOfPier can be set.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanSetPierSideAsync();

        /// <summary>
        /// Indicates whether the RightAscensionRate property can be changed.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanSetRightAscensionRateAsync();

        /// <summary>
        /// Indicates whether the Tracking property can be changed.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanSetTrackingAsync();

        /// <summary>
        /// Indicates whether the telescope can slew synchronously.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanSlewAsync();

        /// <summary>
        /// Indicates whether the telescope can slew synchronously to AltAz coordinates.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanSlewAltAzAsync();

        /// <summary>
        /// Indicates whether the telescope can sync to equatorial coordinates.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanSyncAsync();

        /// <summary>
        /// Indicates whether the telescope can sync to local horizontal coordinates.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanSyncAltAzAsync();

        /// <summary>
        /// The declination (degrees) of the telescope's current equatorial coordinates, in the coordinate system
        /// given by <see cref="GetEquatorialSystemAsync"/>. 
        /// </summary>
        /// <returns></returns>
        Task<double> GetDeclinationAsync();

        /// <summary>
        /// Returns the telescope's declination tracking rate.
        /// </summary>
        /// <returns>The declination tracking rate (arcseconds per second, default = 0.0)</returns>
        Task<double> GetDeclinationRateAsync();

        /// <summary>
        /// Sets the telescope's declination tracking rate.
        /// </summary>
        /// <param name="declinationRate">Declination tracking rate (arcseconds per second)</param>
        Task SetDeclinationRateAsync(double declinationRate);

        /// <summary>
        /// Indicates whether atmospheric refraction is applied to coordinates.
        /// </summary>
        /// <returns></returns>
        Task<bool> DoesRefractionAsync();

        /// <summary>
        /// Determines whether atmospheric refraction is applied to coordinates.
        /// </summary>
        /// <param name="doesRefraction">Set True to make the telescope or driver applie atmospheric refraction to coordinates.</param>
        Task SetDoesRefractionAsync(bool doesRefraction);

        /// <summary>
        /// Returns the current equatorial coordinate system used by this telescope.
        /// </summary>
        /// <returns></returns>
        Task<EquatorialCoordinateType> GetEquatorialSystemAsync();

        /// <summary>
        /// Returns the telescope's focal length in meters.
        /// </summary>
        /// <returns></returns>
        Task<double> GetFocalLengthAsync();

        /// <summary>
        /// Returns the  current Declination rate offset for telescope guiding (degrees/sec)
        /// </summary>
        /// <returns></returns>
        Task<double> GetGuideRateDeclinationAsync();

        /// <summary>
        /// Sets the current Declination movement rate offset for telescope guiding (degrees/sec).
        /// </summary>
        /// <param name="guideRate">Declination movement rate offset degrees/sec).</param>
        Task SetGuideRateDeclinationAsync(double guideRate);

        /// <summary>
        /// Returns the  current Right ascension rate offset for telescope guiding (degrees/sec)
        /// </summary>
        /// <returns></returns>
        Task<double> GetGuideRateRightAscensionAsync();

        /// <summary>
        /// Sets the current Right ascension movement rate offset for telescope guiding (degrees/sec).
        /// </summary>
        /// <param name="guideRate">Right ascension movement rate offset degrees/sec).</param>
        Task SetGuideRateRightAscensionAsync(double guideRate);

        /// <summary>
        /// Indicates whether the telescope is currently executing a PulseGuide command
        /// </summary>
        /// <returns></returns>
        Task<bool> IsPulseGuidingAsync();

        /// <summary>
        /// Returns the right ascension (hours) of the telescope's current equatorial coordinates,
        /// in the coordinate system given by the <see cref="GetEquatorialSystemAsync"/>
        /// </summary>
        /// <returns></returns>
        Task<double> GetRightAscensionAsync();

        /// <summary>
        /// Returns the right ascension tracking rate (arcseconds per second, default = 0.0)
        /// </summary>
        /// <returns></returns>
        Task<double> GetRightAscensionRateAsync();

        /// <summary>
        /// Sets the right ascension tracking rate (arcseconds per second)
        /// </summary>
        /// <param name="rightAscensionRate">Right ascension tracking rate (arcseconds per second)</param>
        Task SetRightAscensionRateAsync(double rightAscensionRate);

        /// <summary>
        /// Indicates the pointing state of the mount.
        /// </summary>
        /// <returns></returns>
        Task<PierSide> GetSideOfPierAsync();

        /// <summary>
        /// Sets the pointing state of the mount.
        /// </summary>
        /// <param name="sideOfPier">New pointing state.</param>
        Task SetSideOfPierAsync(PierSide sideOfPier);

        /// <summary>
        /// Returns the local apparent sidereal time from the telescope's internal clock (hours, sidereal).
        /// </summary>
        /// <returns></returns>
        Task<double> GetSiderealTimeAsync();

        /// <summary>
        /// Returns the elevation above mean sea level (meters) of the site at which the telescope is located.
        /// </summary>
        /// <returns></returns>
        Task<double> GetSiteElevationAsync();

        /// <summary>
        /// Sets the elevation above mean sea level (metres) of the site at which the telescope is located.
        /// </summary>
        /// <param name="siteElevation">Elevation above mean sea level (metres).</param>
        Task SetSiteElevationAsync(double siteElevation);

        /// <summary>
        /// Returns the latitude (degrees, positive East, WGS84) of the site at which the telescope is located.
        /// </summary>
        /// <returns></returns>
        Task<double> GetSiteLatitudeAsync();

        /// <summary>
        /// Sets the observing site's latitude (degrees).
        /// </summary>
        /// <param name="latitude">Site latitude (degrees)</param>
        Task SetSiteLatitudeAsync(double latitude);

        /// <summary>
        /// Returns the longitude (degrees, positive East, WGS84) of the site at which the telescope is located.
        /// </summary>
        /// <returns></returns>
        Task<double> GetSiteLongitudeAsync();

        /// <summary>
        /// Sets the observing site's longitude (degrees).
        /// </summary>
        /// <param name="longitude">Site longitude (degrees)</param>
        Task SetSiteLongitudeAsync(double longitude);

        /// <summary>
        /// Indicates whether the telescope is currently slewing.
        /// </summary>
        /// <returns></returns>
        Task<bool> IsSlewingAsync();

        /// <summary>
        /// Returns the post-slew settling time.
        /// </summary>
        /// <returns></returns>
        Task<int> GetSlewSettleTimeAsync();

        /// <summary>
        /// Sets the post-slew settling time.
        /// </summary>
        /// <param name="settleTime">Settle time in seconds</param>
        Task SetSlewSettleTimeAsync(int settleTime);

        /// <summary>
        /// Returns the declination (degrees, positive North) for the target of an equatorial slew or sync operation
        /// </summary>
        /// <returns></returns>
        Task<double> GetTargetDeclinationAsync();

        /// <summary>
        /// Sets the declination (degrees, positive North) for the target of an equatorial slew or sync operation
        /// </summary>
        /// <param name="declination">Target declination (degrees, positive North)</param>
        Task SetTargetDeclinationAsync(double declination);

        /// <summary>
        /// Returns the right ascension (hours) for the target of an equatorial slew or sync operation
        /// </summary>
        /// <returns></returns>
        Task<double> GetTargetRightAscensionAsync();

        /// <summary>
        /// Sets the right ascension (hours) for the target of an equatorial slew or sync operation
        /// </summary>
        /// <param name="rightAscension">Target right ascension(hours)</param>
        Task SetTargetRightAscensionAsync(double rightAscension);

        /// <summary>
        /// Indicates whether the telescope is tracking.
        /// </summary>
        /// <returns></returns>
        Task<bool> IsTrackingAsync();

        /// <summary>
        /// Enables or disables telescope tracking.
        /// </summary>
        /// <param name="tracking">Tracking enabled / disabled</param>
        Task SetTrackingAsync(bool tracking);

        /// <summary>
        /// Returns the current tracking rate of the telescope's sidereal drive.
        /// </summary>
        /// <returns></returns>
        Task<DriveRate> GetTrackingRateAsync();

        /// <summary>
        /// Sets the mount's tracking rate.
        /// </summary>
        /// <param name="trackingRate"></param>
        Task SetTrackingRateAsync(DriveRate trackingRate);

        /// <summary>
        /// Returns a collection of supported DriveRates values.
        /// </summary>
        /// <returns></returns>
        Task<IList<DriveRate>> GetTrackingRatesAsync();

        /// <summary>
        /// Returns the UTC date/time of the telescope's internal clock.
        /// </summary>
        /// <returns></returns>
        Task<DateTime> GetUtcDateAsync();

        /// <summary>
        /// Sets the UTC date/time of the telescope's internal clock.
        /// </summary>
        /// <param name="utcDate"></param>
        Task SetUtcDateAsync(DateTime utcDate);

        /// <summary>
        /// Immediately stops a slew in progress.
        /// </summary>
        Task AbortSlewAsync();

        /// <summary>
        /// Returns the rates at which the telescope may be moved about the specified axis.
        /// </summary>
        /// <param name="axis">The axis about which rate information is desired</param>
        /// <returns></returns>
        Task<IList<AxisRate>> GetAxisRatesAsync(TelescopeAxis axis);

        /// <summary>
        /// Indicates whether the telescope can move the requested axis.
        /// </summary>
        /// <param name="axis">The axis about which rate information is desired.</param>
        /// <returns></returns>
        Task<bool> CanMoveAxisAsync(TelescopeAxis axis);

        /// <summary>
        /// Predicts the pointing state after a German equatorial mount slews to given coordinates.
        /// </summary>
        /// <param name="rightAscension">Right Ascension coordinate (0.0 to 23.99999999 hours</param>
        /// <param name="declination">Declination coordinate (-90.0 to +90.0 degrees)</param>
        /// <returns></returns>
        Task<PierSide> GetDestinationSideOfPierAsync(double rightAscension, double declination);

        /// <summary>
        /// Moves the mount to the "home" position.
        /// </summary>
        Task FindHomeAsync();

        /// <summary>
        /// Move the telescope in one axis at the given rate.
        /// </summary>
        /// <param name="axis">The axis about which rate information is desired</param>
        /// <param name="rate">The rate of motion (deg/sec) about the specified axis</param>
        Task MoveAxisAsync(TelescopeAxis axis, double rate);

        /// <summary>
        /// Move the telescope to its park position, stop all motion (or restrict to a small safe range), and set AtPark to True. )
        /// </summary>
        Task ParkAsync();

        /// <summary>
        /// Moves the scope in the given direction for the given time.
        /// </summary>
        /// <param name="direction">The direction in which the guide-rate motion is to be made</param>
        /// <param name="duration">The duration of the guide-rate motion (milliseconds)</param>
        Task PulseGuideAsync(GuideDirection direction, int duration);

        /// <summary>
        /// Sets the telescope's park position to be its current position.
        /// </summary>
        Task SetParkAsync();

        /// <summary>
        /// Move the telescope to the given local horizontal coordinates, return when slew is complete
        /// </summary>
        /// <param name="altitude">Altitude coordinate (degrees, positive up)</param>
        /// <param name="azimuth">Azimuth coordinate (degrees, North-referenced, positive East/clockwise)</param>
        Task SlewToAltAzAsync(double altitude, double azimuth);

        /// <summary>
        /// Synchronously slew to the given equatorial coordinates.
        /// </summary>
        /// <param name="rightAscension">Right Ascension coordinate (hours)</param>
        /// <param name="declination">Declination coordinate (degrees)</param>
        Task SlewToCoordinatesAsync(double rightAscension, double declination);

        /// <summary>
        /// Move the telescope to the TargetRightAscension and TargetDeclination equatorial coordinates, return
        /// when slew is complete
        /// </summary>
        Task SlewToTargetAsync();

        /// <summary>
        /// Matches the scope's local horizontal coordinates to the given local horizontal coordinates.
        /// </summary>
        /// <param name="altitude">Altitude coordinate (degrees, positive up)</param>
        /// <param name="azimuth">Azimuth coordinate (degrees, North-referenced, positive East/clockwise)</param>
        Task SyncToAltAzAsync(double altitude, double azimuth);

        /// <summary>
        /// Matches the scope's equatorial coordinates to the given equatorial coordinates.
        /// </summary>
        /// <param name="rightAscension">Right Ascension coordinate (hours)</param>
        /// <param name="declination">Declination coordinate (degrees)</param>
        Task SyncToCoordinatesAsync(double rightAscension, double declination);

        /// <summary>
        /// Matches the scope's equatorial coordinates to the TargetRightAscension and
        /// TargetDeclination equatorial coordinates.
        /// </summary>
        Task SyncToTargetAsync();

        /// <summary>
        /// Unparks the mount.
        /// </summary>
        Task UnparkAsync();
    }
}