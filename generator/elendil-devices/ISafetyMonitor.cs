namespace ES.Ascom.Alpaca.Devices
{
    /// <summary>
    /// Defines the capabilities supported by an ASCOM Alpaca Safety Monitor device
    /// </summary>
    /// <seealso cref="ISafetyMonitorAsync"/>
    public interface ISafetyMonitor : IDevice
    {
        /// <summary>
        /// Indicates whether the monitored state is safe for use.
        /// </summary>
        /// <returns>True if the state is safe, False if it is unsafe.</returns>
        bool IsSafe();
    }
}