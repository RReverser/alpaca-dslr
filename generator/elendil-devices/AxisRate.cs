namespace ES.Ascom.Alpaca.Devices
{
    /// <summary>
    /// Describe a rate at which the telescope may be moved about the specified axis by the
    /// <see cref="!:https://ascom-standards.org/api/#/Telescope%20Specific%20Methods/put_telescope__device_number__moveaxis">MoveAxis</see> command.
    /// </summary>
    public class AxisRate
    {
        /// <summary>
        /// The minimum rate (degrees per second) This must always be a positive number. It indicates the maximum rate in either direction about the axis.
        /// </summary>
        public double Minimum { get; set; }
        
        /// <summary>
        /// The maximum rate (degrees per second) This must always be a positive number. It indicates the maximum rate in either direction about the axis.
        /// </summary>
        public double Maximum { get; set; }
    }
}