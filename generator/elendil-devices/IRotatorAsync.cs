using System.Threading.Tasks;

namespace ES.Ascom.Alpaca.Devices
{
    /// <summary>
    /// Defines the capabilities supported by an ASCOM Alpaca Rotator device
    /// </summary>
    /// <seealso cref="IRotator"/>
    public interface IRotatorAsync : IDeviceAsync
    {
        /// <summary>
        /// Indicates whether the rotator supports the Reverse method.
        /// </summary>
        /// <returns>True if the rotator supports the Reverse method.</returns>
        Task<bool> CanReverseAsync();

        /// <summary>
        /// Indicates whether the rotator is currently moving.
        /// </summary>
        /// <returns></returns>
        Task<bool> IsMovingAsync();

        /// <summary>
        /// Returns the rotator's current position.
        /// </summary>
        /// <returns>Current instantaneous Rotator position, in degrees.</returns>
        Task<double> GetPositionAsync();

        /// <summary>
        /// Returns the rotator’s Reverse state.
        /// </summary>
        /// <returns></returns>
        Task<bool> IsReversedAsync();

        /// <summary>
        /// Sets the rotator’s Reverse state.
        /// </summary>
        /// <param name="reversed">True if the rotation and angular direction must be reversed to match the optical characteristcs</param>
        Task SetReversedAsync(bool reversed);

        /// <summary>
        /// Returns the destination position angle for Move() and MoveAbsolute().
        /// </summary>
        /// <returns></returns>
        Task<double> GetTargetPositionAsync();

        /// <summary>
        /// Immediately stop any Rotator motion due to a previous Move or MoveAbsolute method call.
        /// </summary>
        Task HaltAsync();

        /// <summary>
        /// Causes the rotator to move Position degrees relative to the current Position value.
        /// </summary>
        /// <param name="position">Relative position to move in degrees from current Position.</param>
        Task MoveAsync(double position);

        /// <summary>
        /// Causes the rotator to move the absolute position of Position degrees.
        /// </summary>
        /// <param name="position">Absolute position in degrees.</param>
        Task MoveAbsoluteAsync(double position);
    }
}