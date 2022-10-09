using System;
using System.Collections.Generic;
using System.Threading.Tasks;

namespace ES.Ascom.Alpaca.Devices
{
    /// <summary>
    /// Defines the capabilities supported by an ASCOM Alpaca Camera device
    /// </summary>
    /// <seealso cref="ICamera"/>
    public interface ICameraAsync : IDeviceAsync
    {
        /// <summary>
        /// Returns the X offset of the Bayer matrix, as defined in SensorType.
        /// </summary>
        /// <returns></returns>
        Task<int> GetBayerOffsetXAsync();

        /// <summary>
        /// Returns the Y offset of the Bayer matrix, as defined in SensorType.
        /// </summary>
        /// <returns></returns>
        Task<int> GetBayerOffsetYAsync();

        /// <summary>
        /// Sets the binning factor for the X axis.
        /// </summary>
        /// <returns></returns>
        Task<int> GetBinXAsync();

        /// <summary>
        /// Returns the binning factor for the X axis.
        /// </summary>
        Task SetBinXAsync(int binX);

        /// <summary>
        /// Returns the binning factor for the Y axis.
        /// </summary>
        /// <returns></returns>
        Task<int> GetBinYAsync();

        /// <summary>
        /// Returns the binning factor for the Y axis.
        /// </summary>
        Task SetBinYAsync(int binY);

        /// <summary>
        /// Returns the current camera operational state
        /// </summary>
        /// <returns></returns>
        Task<CameraState> GetCameraStateAsync();

        /// <summary>
        /// Returns the width of the CCD camera chip in unbinned pixels.
        /// </summary>
        /// <returns></returns>
        Task<int> GetCameraXSizeAsync();

        /// <summary>
        /// Returns the height of the CCD camera chip in unbinned pixels.
        /// </summary>
        /// <returns></returns>
        Task<int> GetCameraYSizeAsync();

        /// <summary>
        /// Indicates whether the camera can abort exposures.
        /// </summary>
        /// <returns>Returns true if the camera can abort exposures; false if not.</returns>
        Task<bool> CanAbortExposureAsync();

        /// <summary>
        /// Indicates whether the camera supports asymmetric binning
        /// </summary>
        /// <returns>Returns true if the camera supports asymmetric binning; false if not.</returns>
        Task<bool> CanAsymmetricBinAsync();

        /// <summary>
        /// Indicates whether the camera has a fast readout mode.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanFastReadoutAsync();

        /// <summary>
        /// Indicates whether the camera's cooler power setting can be read.
        /// </summary>
        /// <returns></returns>
        Task<bool> CanGetCoolerPowerAsync();

        /// <summary>
        /// Returns a flag indicating whether this camera supports pulse guiding
        /// </summary>
        /// <returns></returns>
        Task<bool> CanPulseGuideAsync();

        /// <summary>
        /// Returns a flag indicating whether this camera supports setting the CCD temperature
        /// </summary>
        /// <returns></returns>
        Task<bool> CanSetCCDTemperatureAsync();

        /// <summary>
        /// Returns a flag indicating whether this camera can stop an exposure that is in progress
        /// </summary>
        /// <returns></returns>
        Task<bool> CanStopExposureAsync();

        /// <summary>
        /// Returns the current CCD temperature in degrees Celsius.
        /// </summary>
        /// <returns></returns>
        Task<double> GetCCDTemperatureAsync();

        /// <summary>
        /// Returns the current cooler on/off state.
        /// </summary>
        /// <returns></returns>
        Task<bool> IsCoolerOnAsync();

        /// <summary>
        /// Turns on and off the camera cooler
        /// </summary>
        /// <param name="coolerOn"></param>
        Task SetCoolerOnAsync(bool coolerOn);

        /// <summary>
        /// Returns the present cooler power level, in percent.
        /// </summary>
        /// <returns></returns>
        Task<double> GetCoolerPowerAsync();

        /// <summary>
        /// Returns the gain of the camera in photoelectrons per A/D unit.
        /// </summary>
        /// <returns></returns>
        Task<double> GetElectronPerADUAsync();

        /// <summary>
        /// Returns the maximum exposure time supported by StartExposure.
        /// </summary>
        /// <returns></returns>
        Task<double> GetExposureMaxAsync();

        /// <summary>
        /// Returns the Minimium exposure time in seconds that the camera supports through StartExposure.
        /// </summary>
        /// <returns></returns>
        Task<double> GetExposureMinAsync();

        /// <summary>
        /// Returns the smallest increment in exposure time supported by StartExposure.
        /// </summary>
        /// <returns></returns>
        Task<double> GetExposureResolutionAsync();

        /// <summary>
        /// Returns whether Fast Readout Mode is enabled.
        /// </summary>
        /// <returns></returns>
        Task<bool> IsFastReadoutAsync();

        /// <summary>
        /// Sets whether Fast Readout Mode is enabled.
        /// </summary>
        /// <param name="fastReadout">True to enable fast readout mode</param>
        Task SetFastReadoutAsync(bool fastReadout);

        /// <summary>
        /// Reports the full well capacity of the camera in electrons, at the
        /// current camera settings (binning, SetupDialog settings, etc.).
        /// </summary>
        /// <returns></returns>
        Task<double> GetFullWellCapacityAsync();

        /// <summary>
        /// Returns an index into the Gains array for the selected camera gain.
        /// </summary>
        /// <returns></returns>
        Task<int> GetGainAsync();

        /// <summary>
        /// Set the camera gain
        /// </summary>
        /// <param name="gain">Index of the current camera gain in the Gains string array.</param>
        Task SetGainAsync(int gain);

        /// <summary>
        /// Returns the maximum value of Gain.
        /// </summary>
        /// <returns></returns>
        Task<int> GetGainMaxAsync();

        /// <summary>
        /// Returns the minimum value of Gain.
        /// </summary>
        /// <returns></returns>
        Task<int> GetGainMinAsync();

        /// <summary>
        /// Returns the Gains supported by the camera.
        /// </summary>
        /// <returns></returns>
        Task<IList<string>> GetGainsAsync();

        /// <summary>
        /// Indicates whether the camera has a mechanical shutter
        /// </summary>
        /// <returns></returns>
        Task<bool> HasShutterAsync();

        /// <summary>
        /// Returns the current heat sink temperature (called "ambient temperature"
        /// by some manufacturers) in degrees Celsius.
        /// </summary>
        /// <returns></returns>
        Task<double> GetHeatSinkTemperatureAsync();

        /// <summary>
        /// Returns an array of integers containing the exposure pixel values",
        /// "description": "Returns an array of 32bit integers containing the pixel values from the last 
        /// exposure. This call can return either a 2 dimension (monochrome images) or 3 dimension (colour or 
        /// multi-plane images) array of size NumX * NumY or NumX * NumY * NumPlanes. Where applicable, the 
        /// size of NumPlanes has to be determined by inspection of the returned Array.
        /// 
        /// Since 32bit integers are always returned by this call, the returned JSON Type value (0 = Unknown, 
        /// 1 = short(16bit), 2 = int(32bit), 3 = Double) is always 2. The number of planes is given in the 
        /// returned Rank value.
        /// 
        /// When de-serialising to an object it helps enormously to know the array Rank beforehand so that 
        /// the correct data class can be used. This can be achieved through a regular expression or by 
        /// direct parsing of the returned JSON string to extract the Type and Rank values before 
        /// de-serialising.
        /// 
        /// This regular expression accomplishes the etraction into two named groups Type and Rank
        /// 
        /// ^*"Type":(?&lt;Type&gt;\\d*),"Rank":(?&lt;Rank&gt;\\d*)
        /// 
        /// which can then be used to select the correct de-serialisation data class.
        /// </summary>
        /// <returns></returns>
        Task<Array> GetImageArrayAsync();

        /// <summary>
        /// Returns an array containing the pixel values from the last exposure. This call can return either 
        /// a 2 dimension (monochrome images) or 3 dimension (colour or multi-plane images) array of size 
        /// NumX * NumY  or NumX * NumY * NumPlanes. Where applicable, the size of NumPlanes has to be 
        /// determined by inspection of the returned Array.
        /// 
        /// This call can return values as short(16bit) integers, int(32bit) integers or double floating 
        /// point values. The nature of the returned values is given in the Type parameter: 0 = Unknown, 1 = 
        /// short(16bit), 2 = int(32bit), 3 = Double. The number of planes is given in the returned Rank value.
        /// 
        /// When deserialising to an object it helps enormously to know the Type and Rank beforehand so that 
        /// the correct data class can be used. This can be achieved through a regular expression or by 
        /// direct parsing of the returned JSON string to extract the Type and Rank values before 
        /// deserialising.
        /// 
        /// This regular expression accomplishes the extraction into two named groups: Type and Rank:
        /// 
        /// __`^*"Type":(?&lt;Type&gt;\\d*),"Rank":(?&lt;Rank&gt;\\d*)`__
        /// 
        /// which can then be used to select the correct deserialisation data class.
        /// </summary>
        /// <returns></returns>
        Task<Array> GetImageArrayVariantAsync();

        /// <summary>
        /// Indicates that an image is ready to be downloaded
        /// </summary>
        /// <returns></returns>
        Task<bool> IsImageReadyAsync();

        /// <summary>
        /// Indicates that the camera is pulse guideing.
        /// </summary>
        /// <returns></returns>
        Task<bool> IsPulseGuidingAsync();

        /// <summary>
        /// Get the duration of the last exposure in seconds (i.e. shutter open time).
        /// </summary>
        /// <returns></returns>
        Task<double> GetLastExposureDurationAsync();

        /// <summary>
        /// Start time of the last exposure in FITS standard format.
        /// (CCYY-MM-DDThh:mm:ss[.sss...])
        /// </summary>
        /// <returns></returns>
        Task<DateTime> GetLastExposureStartTimeAsync();

        /// <summary>
        /// Reports the maximum ADU value the camera can produce.
        /// </summary>
        /// <returns></returns>
        Task<int> GetMaxADUAsync();

        /// <summary>
        /// Returns the maximum allowed binning for the X camera axis
        /// </summary>
        /// <returns></returns>
        Task<int> GetMaxBinXAsync();

        /// <summary>
        /// Returns the maximum allowed binning for the Y camera axis
        /// </summary>
        /// <returns></returns>
        Task<int> GetMaxBinYAsync();

        /// <summary>
        /// Returns the current subframe width, if binning is active, value is in binned pixels.
        /// </summary>
        /// <returns></returns>
        Task<int> GetNumXAsync();

        /// <summary>
        /// Sets the current subframe width
        /// </summary>
        /// <param name="numX">Sets the subframe width, if binning is active, value is in binned pixels.</param>
        Task SetNumXAsync(int numX);

        /// <summary>
        /// Returns the current subframe height, if binning is active, value is in binned pixels.
        /// </summary>
        /// <returns></returns>
        Task<int> GetNumYAsync();

        /// <summary>
        /// Sets the current subframe width
        /// </summary>
        /// <param name="numY">Sets the subframe height, if binning is active, value is in binned pixels.</param>
        Task SetNumYAsync(int numY);

        /// <summary>
        /// Returns the percentage of the current operation that is complete.
        /// If valid, returns an integer between 0 and 100, where 0 indicates 0%
        /// progress (function just started) and 100 indicates 100% progress (i.e. completion).
        /// </summary>
        /// <returns></returns>
        Task<int> GetPercentCompletedAsync();

        /// <summary>
        /// Returns the width of the CCD chip pixels in microns.
        /// </summary>
        /// <returns></returns>
        Task<double> GetPixelSizeXAsync();

        /// <summary>
        /// Returns the height of the CCD chip pixels in microns.
        /// </summary>
        /// <returns></returns>
        Task<double> GetPixelSizeYAsync();

        /// <summary>
        /// Indicates the canera's readout mode as an index into the array ReadoutModes.
        /// ReadoutMode is an index into the array ReadoutModes and returns the
        /// desired readout mode for the camera. Defaults to 0 if not set.
        /// </summary>
        /// <returns></returns>
        Task<int> GetReadoutModeAsync();

        /// <summary>
        /// Sets the ReadoutMode as an index into the array ReadoutModes.
        /// </summary>
        /// <returns></returns>
        Task SetReadoutModeAsync(int readoutMode);

        /// <summary>
        /// Get the list of available readout modes.
        /// This property provides an array of strings, each of which describes
        /// an available readout mode of the camera.
        /// At least one string must be present in the list.
        /// </summary>
        /// <returns></returns>
        Task<IList<string>> GetReadoutModesAsync();

        /// <summary>
        /// Gets the name of the sensor used within the camera.
        /// </summary>
        /// <returns></returns>
        Task<string> GetSensorNameAsync();

        /// <summary>
        /// Returns a value indicating whether the sensor is monochrome, or what Bayer matrix it encodes.
        /// Please see the ASCOM Help file for more informaiton on the SensorType.
        /// </summary>
        /// <returns></returns>
        Task<SensorType> GetSensorTypeAsync();

        /// <summary>
        /// Returns the current camera cooler setpoint in degrees Celsius.
        /// </summary>
        /// <returns></returns>
        Task<double> GetCCDTemperatureSetPointAsync();

        /// <summary>
        /// Set the camera's cooler setpoint (degrees Celsius).
        /// </summary>
        /// <param name="temperature">Temperature set point in degrees Celsius</param>
        Task SetCCDTemperatureSetPointAsync(double temperature);

        /// <summary>
        /// Gets the subframe start position for the X axis (0 based).
        /// If binning is active, value is in binned pixels.
        /// </summary>
        /// <returns></returns>
        Task<int> GetStartXAsync();

        /// <summary>
        /// Sets the current subframe X axis start position in binned pixels.
        /// </summary>
        /// <param name="startX">The subframe X axis start position in binned pixels.</param>
        Task SetStartXAsync(int startX);

        /// <summary>
        /// Gets the subframe start position for the Y axis (0 based).
        /// If binning is active, value is in binned pixels.
        /// </summary>
        /// <returns></returns>
        Task<int> GetStartYAsync();

        /// <summary>
        /// Sets the current subframe Y axis start position in binned pixels.
        /// </summary>
        /// <param name="startY">The subframe Y axis start position in binned pixels.</param>
        Task SetStartYAsync(int startY);

        /// <summary>
        /// Aborts the current exposure, if any, and returns the camera to Idle state.
        /// </summary>
        Task AbortExposureAsync();

        /// <summary>
        /// Activates the Camera's mount control sytem to instruct the mount to move in a particular direction for a given period of time
        /// </summary>
        /// <param name="direction">Direction of movement</param>
        /// <param name="duration">Duration of movement in milli-seconds</param>
        Task PulseGuideAsync(GuideDirection direction, int duration);

        /// <summary>
        /// Starts an exposure. Use ImageReady to check when the exposure is complete.
        /// </summary>
        /// <param name="duration">Duration of exposure in seconds</param>
        /// <param name="isLight">True if light frame, false if dark frame.</param>
        Task StartExposureAsync(double duration, bool isLight);

        /// <summary>
        /// Stops the current exposure, if any. If an exposure is in progress, the readout process is initiated. Ignored if readout is already in process.
        /// </summary>
        Task StopExposureAsync();
    }
}