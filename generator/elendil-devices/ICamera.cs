using System;
using System.Collections.Generic;

namespace ES.Ascom.Alpaca.Devices
{
    /// <summary>
    /// Defines the capabilities supported by an ASCOM Alpaca Camera device
    /// </summary>
    /// <seealso cref="ICameraAsync"/>
    public interface ICamera : IDevice
    {
        /// <summary>
        /// Returns the X offset of the Bayer matrix, as defined in SensorType.
        /// </summary>
        /// <returns></returns>
        int GetBayerOffsetX();

        /// <summary>
        /// Returns the Y offset of the Bayer matrix, as defined in SensorType.
        /// </summary>
        /// <returns></returns>
        int GetBayerOffsetY();

        /// <summary>
        /// Returns the binning factor for the X axis.
        /// </summary>
        /// <returns></returns>
        int GetBinX();

        /// <summary>
        /// Sets the binning factor for the X axis.
        /// </summary>
        void SetBinX(int binX);
        
        /// <summary>
        /// Returns the binning factor for the Y axis.
        /// </summary>
        /// <returns></returns>
        int GetBinY();

        /// <summary>
        /// Sets the binning factor for the Y axis.
        /// </summary>
        void SetBinY(int binY);

        /// <summary>
        /// Returns the current camera operational state
        /// </summary>
        /// <returns></returns>
        CameraState GetCameraState();

        /// <summary>
        /// Returns the width of the CCD camera chip in unbinned pixels.
        /// </summary>
        /// <returns></returns>
        int GetCameraXSize();

        /// <summary>
        /// Returns the height of the CCD camera chip in unbinned pixels.
        /// </summary>
        /// <returns></returns>
        int GetCameraYSize();

        /// <summary>
        /// Indicates whether the camera can abort exposures.
        /// </summary>
        /// <returns>Returns true if the camera can abort exposures; false if not.</returns>
        bool CanAbortExposure();

        /// <summary>
        /// Indicates whether the camera supports asymmetric binning
        /// </summary>
        /// <returns>Returns true if the camera supports asymmetric binning; false if not.</returns>
        bool CanAsymmetricBin();

        /// <summary>
        /// Indicates whether the camera has a fast readout mode.
        /// </summary>
        /// <returns></returns>
        bool CanFastReadout();

        /// <summary>
        /// Indicates whether the camera's cooler power setting can be read.
        /// </summary>
        /// <returns></returns>
        bool CanGetCoolerPower();

        /// <summary>
        /// Returns a flag indicating whether this camera supports pulse guiding
        /// </summary>
        /// <returns></returns>
        bool CanPulseGuide();

        /// <summary>
        /// Returns a flag indicating whether this camera supports setting the CCD temperature
        /// </summary>
        /// <returns></returns>
        bool CanSetCCDTemperature();

        /// <summary>
        /// Returns a flag indicating whether this camera can stop an exposure that is in progress
        /// </summary>
        /// <returns></returns>
        bool CanStopExposure();

        /// <summary>
        /// Returns the current CCD temperature in degrees Celsius.
        /// </summary>
        /// <returns></returns>
        double GetCCDTemperature();

        /// <summary>
        /// Returns the current cooler on/off state.
        /// </summary>
        /// <returns></returns>
        bool IsCoolerOn();

        /// <summary>
        /// Turns on and off the camera cooler
        /// </summary>
        /// <param name="coolerOn"></param>
        void SetCoolerOn(bool coolerOn);

        /// <summary>
        /// Returns the present cooler power level, in percent.
        /// </summary>
        /// <returns></returns>
        double GetCoolerPower();

        /// <summary>
        /// Returns the gain of the camera in photoelectrons per A/D unit.
        /// </summary>
        /// <returns></returns>
        double GetElectronPerADU();

        /// <summary>
        /// Returns the maximum exposure time supported by StartExposure.
        /// </summary>
        /// <returns></returns>
        double GetExposureMax();

        /// <summary>
        /// Returns the Minimium exposure time in seconds that the camera supports through StartExposure.
        /// </summary>
        /// <returns></returns>
        double GetExposureMin();

        /// <summary>
        /// Returns the smallest increment in exposure time supported by StartExposure.
        /// </summary>
        /// <returns></returns>
        double GetExposureResolution();

        /// <summary>
        /// Returns whether Fast Readout Mode is enabled.
        /// </summary>
        /// <returns></returns>
        bool IsFastReadout();

        /// <summary>
        /// Sets whether Fast Readout Mode is enabled.
        /// </summary>
        /// <param name="fastReadout">True to enable fast readout mode</param>
        void SetFastReadout(bool fastReadout);

        /// <summary>
        /// Reports the full well capacity of the camera in electrons, at the
        /// current camera settings (binning, SetupDialog settings, etc.).
        /// </summary>
        /// <returns></returns>
        double GetFullWellCapacity();

        /// <summary>
        /// Returns an index into the Gains array for the selected camera gain.
        /// </summary>
        /// <returns></returns>
        int GetGain();

        /// <summary>
        /// Set the camera gain
        /// </summary>
        /// <param name="gain">Index of the current camera gain in the Gains string array.</param>
        void SetGain(int gain);

        /// <summary>
        /// Returns the maximum value of Gain.
        /// </summary>
        /// <returns></returns>
        int GetGainMax();

        /// <summary>
        /// Returns the minimum value of Gain.
        /// </summary>
        /// <returns></returns>
        int GetGainMin();

        /// <summary>
        /// Returns the Gains supported by the camera.
        /// </summary>
        /// <returns></returns>
        IList<string> GetGains();

        /// <summary>
        /// Indicates whether the camera has a mechanical shutter
        /// </summary>
        /// <returns></returns>
        bool HasShutter();

        /// <summary>
        /// Returns the current heat sink temperature (called "ambient temperature"
        /// by some manufacturers) in degrees Celsius.
        /// </summary>
        /// <returns></returns>
        double GetHeatSinkTemperature();

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
        Array GetImageArray();

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
        Array GetImageArrayVariant();

        /// <summary>
        /// Indicates that an image is ready to be downloaded
        /// </summary>
        /// <returns></returns>
        bool IsImageReady();

        /// <summary>
        /// Indicates that the camera is pulse guideing.
        /// </summary>
        /// <returns></returns>
        bool IsPulseGuiding();

        /// <summary>
        /// Get the duration of the last exposure in seconds (i.e. shutter open time).
        /// </summary>
        /// <returns></returns>
        double GetLastExposureDuration();

        /// <summary>
        /// Start time of the last exposure in FITS standard format.
        /// (CCYY-MM-DDThh:mm:ss[.sss...])
        /// </summary>
        /// <returns></returns>
        DateTime GetLastExposureStartTime();

        /// <summary>
        /// Reports the maximum ADU value the camera can produce.
        /// </summary>
        /// <returns></returns>
        int GetMaxADU();

        /// <summary>
        /// Returns the maximum allowed binning for the X camera axis
        /// </summary>
        /// <returns></returns>
        int GetMaxBinX();

        /// <summary>
        /// Returns the maximum allowed binning for the Y camera axis
        /// </summary>
        /// <returns></returns>
        int GetMaxBinY();

        /// <summary>
        /// Returns the current subframe width, if binning is active, value is in binned pixels.
        /// </summary>
        /// <returns></returns>
        int GetNumX();

        /// <summary>
        /// Sets the current subframe width
        /// </summary>
        /// <param name="numX">Sets the subframe width, if binning is active, value is in binned pixels.</param>
        void SetNumX(int numX);

        /// <summary>
        /// Returns the current subframe height, if binning is active, value is in binned pixels.
        /// </summary>
        /// <returns></returns>
        int GetNumY();

        /// <summary>
        /// Sets the current subframe width
        /// </summary>
        /// <param name="numY">Sets the subframe height, if binning is active, value is in binned pixels.</param>
        void SetNumY(int numY);

        /// <summary>
        /// Returns the percentage of the current operation that is complete.
        /// If valid, returns an integer between 0 and 100, where 0 indicates 0%
        /// progress (function just started) and 100 indicates 100% progress (i.e. completion).
        /// </summary>
        /// <returns></returns>
        int GetPercentCompleted();

        /// <summary>
        /// Returns the width of the CCD chip pixels in microns.
        /// </summary>
        /// <returns></returns>
        double GetPixelSizeX();
        

        /// <summary>
        /// Returns the height of the CCD chip pixels in microns.
        /// </summary>
        /// <returns></returns>
        double GetPixelSizeY();

        /// <summary>
        /// Indicates the canera's readout mode as an index into the array ReadoutModes.
        /// ReadoutMode is an index into the array ReadoutModes and returns the
        /// desired readout mode for the camera. Defaults to 0 if not set.
        /// </summary>
        /// <returns></returns>
        int GetReadoutMode();

        /// <summary>
        /// Sets the ReadoutMode as an index into the array ReadoutModes.
        /// </summary>
        /// <returns></returns>
        void SetReadoutMode(int readoutMode);

        /// <summary>
        /// Get the list of available readout modes.
        /// This property provides an array of strings, each of which describes
        /// an available readout mode of the camera.
        /// At least one string must be present in the list.
        /// </summary>
        /// <returns></returns>
        IList<string> GetReadoutModes();

        /// <summary>
        /// Gets the name of the sensor used within the camera.
        /// </summary>
        /// <returns></returns>
        string GetSensorName();

        /// <summary>
        /// Returns a value indicating whether the sensor is monochrome, or what Bayer matrix it encodes.
        /// Please see the ASCOM Help file for more informaiton on the SensorType.
        /// </summary>
        /// <returns></returns>
        SensorType GetSensorType();

        /// <summary>
        /// Returns the current camera cooler setpoint in degrees Celsius.
        /// </summary>
        /// <returns></returns>
        double GetCCDTemperatureSetPoint();

        /// <summary>
        /// Set the camera's cooler setpoint (degrees Celsius).
        /// </summary>
        /// <param name="temperature">Temperature set point in degrees Celsius</param>
        void SetCCDTemperatureSetPoint(double temperature);

        /// <summary>
        /// Gets the subframe start position for the X axis (0 based).
        /// If binning is active, value is in binned pixels.
        /// </summary>
        /// <returns></returns>
        int GetStartX();

        /// <summary>
        /// Sets the current subframe X axis start position in binned pixels.
        /// </summary>
        /// <param name="startX">The subframe X axis start position in binned pixels.</param>
        void SetStartX(int startX);
        
        /// <summary>
        /// Gets the subframe start position for the Y axis (0 based).
        /// If binning is active, value is in binned pixels.
        /// </summary>
        /// <returns></returns>
        int GetStartY();

        /// <summary>
        /// Sets the current subframe Y axis start position in binned pixels.
        /// </summary>
        /// <param name="startY">The subframe Y axis start position in binned pixels.</param>
        void SetStartY(int startY);

        /// <summary>
        /// Aborts the current exposure, if any, and returns the camera to Idle state.
        /// </summary>
        void AbortExposure();

        /// <summary>
        /// Activates the Camera's mount control sytem to instruct the mount to move in a particular direction for a given period of time
        /// </summary>
        /// <param name="direction">Direction of movement</param>
        /// <param name="duration">Duration of movement in milli-seconds</param>
        void PulseGuide(GuideDirection direction, int duration);

        /// <summary>
        /// Starts an exposure. Use ImageReady to check when the exposure is complete.
        /// </summary>
        /// <param name="duration">Duration of exposure in seconds</param>
        /// <param name="isLight">True if light frame, false if dark frame.</param>
        void StartExposure(double duration, bool isLight);

        /// <summary>
        /// Stops the current exposure, if any. If an exposure is in progress, the readout process is initiated. Ignored if readout is already in process.
        /// </summary>
        void StopExposure();
    }
}