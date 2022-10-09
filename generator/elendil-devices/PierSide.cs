namespace ES.Ascom.Alpaca.Devices
{
    /// <summary>
    /// The pointing state of the mount.
    /// ASCOM has adopted the convention that the Normal pointing state will be the state where the mount is on the East side of the pier,
    /// looking West with the counterweights below the optical assembly.
    /// </summary>
    public enum PierSide
    {
        /// <summary>
        /// Unknown or indeterminate.
        /// </summary>
        Unknown = -1,
        
        /// <summary>
        /// Normal pointing state.
        /// Mount on the East side of pier (looking West)
        /// </summary>
        East = 0,
        

        /// <summary>
        /// Through the pole pointing state
        /// Mount on the West side of pier (looking East)
        /// </summary>
        West = 1
    }
}