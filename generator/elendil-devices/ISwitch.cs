namespace ES.Ascom.Alpaca.Devices
{
    /// <summary>
    /// Defines the capabilities supported by an ASCOM Alpaca Switch device
    /// </summary>
    /// <seealso cref="ISwitchAsync"/>
    public interface ISwitch : IDevice
    {
        /// <summary>
        /// Returns the number of switch devices managed by this driver.
        /// Devices are numbered from 0 to MaxSwitch - 1
        /// </summary>
        /// <returns></returns>
        int GetMaxSwitch();

        /// <summary>
        /// Reports if the specified switch device can be written to, default true.
        /// This is false if the device cannot be written to, for example a limit
        /// switch or a sensor. Devices are numbered from 0 to MaxSwitch - 1
        /// </summary>
        /// <param name="id">The device number (0 to MaxSwitch - 1)</param>
        /// <returns></returns>
        bool CanWrite(int id);

        /// <summary>
        /// Return the state of switch device id as a boolean.
        /// Devices are numbered from 0 to MaxSwitch - 1
        /// </summary>
        /// <param name="id">The device number (0 to MaxSwitch - 1)</param>
        /// <returns></returns>
        bool GetSwitch(int id);

        /// <summary>
        /// Sets a switch controller device to the specified state, true or false.
        /// </summary>
        /// <param name="id">The device number (0 to MaxSwitch - 1)</param>
        /// <param name="state">The required control state (True or False)</param>
        void SetSwitch(int id, bool state);

        /// <summary>
        /// Gets the description of the specified switch device.
        /// This is to allow a fuller description of the device to be returned,
        /// for example for a tool tip.
        /// Devices are numbered from 0 to MaxSwitch - 1
        /// </summary>
        /// <param name="id">The device number (0 to MaxSwitch - 1)</param>
        /// <returns></returns>
        string GetSwitchDescription(int id);

        /// <summary>
        /// Gets the name of the specified switch device.
        /// Devices are numbered from 0 to MaxSwitch - 1
        /// </summary>
        /// <param name="id">The device number (0 to MaxSwitch - 1)</param>
        /// <returns></returns>
        string GetSwitchName(int id);

        /// <summary>
        /// Sets a switch device name to the specified value.
        /// </summary>
        /// <param name="id">The device number (0 to MaxSwitch - 1)</param>
        /// <param name="name">The name of the device</param>
        void SetSwitchName(int id, string name);
        
        /// <summary>
        /// Gets the value of the specified switch device as a double.
        /// Devices are numbered from 0 to MaxSwitch - 1, The value of this
        /// switch is expected to be between MinSwitchValue and MaxSwitchValue.
        /// </summary>
        /// <param name="id">The device number (0 to MaxSwitch - 1)</param>
        /// <returns></returns>
        double GetSwitchValue(int id);

        /// <summary>
        /// Sets a switch device value to the specified value.
        /// </summary>
        /// <param name="id">The device number (0 to MaxSwitch - 1)</param>
        /// <param name="value">The value to be set, between MinSwitchValue and MaxSwitchValue</param>
        void SetSwitchValue(int id, double value);
        
        /// <summary>
        /// Gets the minimum value of the specified switch device as a double.
        /// Devices are numbered from 0 to MaxSwitch - 1.
        /// </summary>
        /// <param name="id">The device number (0 to MaxSwitch - 1)</param>
        /// <returns></returns>
        double GetMinSwitchValue(int id);
        
        /// <summary>
        /// Gets the maximum value of the specified switch device as a double.
        /// Devices are numbered from 0 to MaxSwitch - 1.
        /// </summary>
        /// <param name="id">The device number (0 to MaxSwitch - 1)</param>
        /// <returns></returns>
        double GetMaxSwitchValue(int id);

        /// <summary>
        /// Returns the step size that this device supports (the difference
        /// between successive values of the device). Devices are numbered from
        /// 0 to MaxSwitch - 1.
        /// </summary>
        /// <param name="id">The device number (0 to MaxSwitch - 1)</param>
        /// <returns></returns>
        double GetSwitchStep(int id);
    }
}