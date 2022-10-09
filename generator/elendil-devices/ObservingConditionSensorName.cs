namespace ES.Ascom.Alpaca.Devices
{
    /// <summary>
    /// Available sensors in an Observing Conditions devices
    /// </summary>
    public enum ObservingConditionSensorName
    {
        /// <summary>
        /// Amount of sky obscured by cloud 
        /// </summary>
        CloudCover,
        
        /// <summary>
        /// Atmospheric dew point
        /// </summary>
        DewPoint,
        
        /// <summary>
        /// Atmospheric humidity 
        /// </summary>
        Humidity,
        
        /// <summary>
        /// Atmospheric pressure
        /// </summary>
        Pressure,
        
        /// <summary>
        /// Rain rate
        /// </summary>
        RainRate,
        
        /// <summary>
        /// Sky brightness 
        /// </summary>
        SkyBrightness,
        
        /// <summary>
        /// Sky quality
        /// </summary>
        SkyQuality,
        
        /// <summary>
        /// Sky temperature
        /// </summary>
        SkyTemperature,
        
        /// <summary>
        /// Seeing at the observatory 
        /// </summary>
        StarFWHM,
        
        /// <summary>
        /// Temperature 
        /// </summary>
        Temperature,
        
        /// <summary>
        /// Wind direction
        /// </summary>
        WindDirection,
        
        /// <summary>
        /// Peak 3 second wind gust 
        /// </summary>
        WindGust,
        
        /// <summary>
        /// Wind speed 
        /// </summary>
        WindSpeed,
    }
}