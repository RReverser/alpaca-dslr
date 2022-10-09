
namespace ES.Ascom.Alpaca.Devices
{
    /// <summary>
    /// Defines the capabilities supported by an ASCOM Alpaca Focuser device
    /// </summary>
    /// <seealso cref="IFocuserAsync"/>
    public interface IFocuser : IDevice
    {
        /// <summary>
        /// Indicates whether the focuser is capable of absolute position.
        /// </summary>
        /// <returns>True if the focuser is capable of absolute position</returns>
        bool IsAbsolute();

        /// <summary>
        /// Indicates whether the focuser is currently moving.
        /// </summary>
        /// <returns>True if the focuser is currently moving to a new position. False if the focuser is stationary.</returns>
        bool IsMoving();

        /// <summary>
        /// Maximum increment size allowed by the focuser; i.e. the maximum number
        /// of steps allowed in one move operation.
        /// </summary>
        /// <returns></returns>
        int GetMaxIncrement();

        /// <summary>
        /// Returns the focuser's maximum step size.
        /// </summary>
        /// <returns></returns>
        int GetMaxStep();

        /// <summary>
        /// Returns the focuser's current position.
        /// </summary>
        /// <returns></returns>
        int GetPosition();

        /// <summary>
        /// Returns the focuser's step size.
        /// </summary>
        /// <returns>Step size (microns) for the focuser.</returns>
        double GetStepSize();

        /// <summary>
        /// Gets the state of temperature compensation mode (if available), else always False.
        /// </summary>
        /// <returns></returns>
        bool IsTempComp();
        
        /// <summary>
        /// Sets the state of temperature compensation mode.
        /// </summary>
        /// <param name="tempComp">Set true to enable the focuser's temperature compensation mode, otherwise false for normal operation.</param>
        void SetTempComp(bool tempComp);

        /// <summary>
        /// Indicates whether the focuser has temperature compensation.
        /// </summary>
        /// <returns>True if focuser has temperature compensation available.</returns>
        bool IsTempCompAvailable();

        /// <summary>
        /// Returns the focuser's current temperature.
        /// </summary>
        /// <returns></returns>
        double GetTemperature();

        /// <summary>
        /// Immediately stops focuser motion.
        /// </summary>
        void Halt();

        /// <summary>
        /// Moves the focuser by the specified amount or to the specified position depending on the value of the Absolute property.
        /// </summary>
        /// <param name="position">Step distance or absolute position, depending on the value of the Absolute property</param>
        void Move(int position);
    }
}