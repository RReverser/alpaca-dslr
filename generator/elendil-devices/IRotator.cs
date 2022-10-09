using System.Threading.Tasks;

namespace ES.Ascom.Alpaca.Devices
{
    /// <summary>
    /// Defines the capabilities supported by an ASCOM Alpaca Rotator device
    /// </summary>
    /// <seealso cref="IRotatorAsync"/>
    public interface IRotator : IDevice
    {
        /// <summary>
        /// Indicates whether the rotator supports the Reverse method.
        /// </summary>
        /// <returns>True if the rotator supports the Reverse method.</returns>
        bool CanReverse();

        /// <summary>
        /// Indicates whether the rotator is currently moving.
        /// </summary>
        /// <returns></returns>
        bool IsMoving();

        /// <summary>
        /// Returns the rotator's current position.
        /// </summary>
        /// <returns>Current instantaneous Rotator position, in degrees.</returns>
        double GetPosition();

        /// <summary>
        /// Returns the rotator’s Reverse state.
        /// </summary>
        /// <returns></returns>
        bool IsReversed();

        /// <summary>
        /// Sets the rotator’s Reverse state.
        /// </summary>
        /// <param name="reversed">True if the rotation and angular direction must be reversed to match the optical characteristcs</param>
        void SetReversed(bool reversed);

        /// <summary>
        /// Returns the minimum StepSize
        /// </summary>
        /// <returns>The minimum StepSize, in degrees.</returns>
        double GetStepSize();

        /// <summary>
        /// Returns the minimum StepSize
        /// </summary>
        /// <returns>The minimum StepSize, in degrees.</returns>
        Task<double> GetStepSizeAsync();

        /// <summary>
        /// Returns the destination position angle for Move() and MoveAbsolute().
        /// </summary>
        /// <returns></returns>
        double GetTargetPosition();

        /// <summary>
        /// Immediately stop any Rotator motion due to a previous Move or MoveAbsolute method call.
        /// </summary>
        void Halt();

        /// <summary>
        /// Causes the rotator to move Position degrees relative to the current Position value.
        /// </summary>
        /// <param name="position">Relative position to move in degrees from current Position.</param>
        void Move(double position);

        /// <summary>
        /// Causes the rotator to move the absolute position of Position degrees.
        /// </summary>
        /// <param name="position">Absolute position in degrees.</param>
        void MoveAbsolute(double position);
    }
}