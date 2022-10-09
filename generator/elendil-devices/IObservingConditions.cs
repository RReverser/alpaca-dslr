using System;

namespace ES.Ascom.Alpaca.Devices
{
    /// <summary>
    /// Defines the capabilities supported by an ASCOM Alpaca Observing conditions device
    /// </summary>
    /// <seealso cref="IObservingConditionsAsync"/>
    public interface IObservingConditions : IDevice
    {
        /// <summary>
        /// Gets the time period over which observations will be averaged
        /// </summary>
        /// <returns></returns>
        double GetAveragePeriod();
        
        /// <summary>
        /// Sets the time period over which observations will be averaged
        /// </summary>
        /// <param name="period">Time period (hours) over which to average sensor readings</param>
        void SetAveragePeriod(double period);

        /// <summary>
        /// Gets the percentage of the sky obscured by cloud
        /// </summary>
        /// <returns></returns>
        double GetCloudCover();

        /// <summary>
        /// Gets the atmospheric dew point at the observatory reported in °C.
        /// </summary>
        /// <returns></returns>
        double GetDewPoint();

        /// <summary>
        /// Gets the atmospheric  humidity (%) at the observatory
        /// </summary>
        /// <returns></returns>
        double GetHumidity();

        /// <summary>
        /// Gets the atmospheric pressure in hectoPascals at the observatory's altitude - NOT reduced to sea level.
        /// </summary>
        /// <returns></returns>
        double GetPressure();

        /// <summary>
        /// Gets the rain rate (mm/hour) at the observatory.
        /// </summary>
        /// <returns></returns>
        double GetRainRate();

        /// <summary>
        /// Gets the sky brightness at the observatory (Lux)
        /// </summary>
        /// <returns></returns>
        double GetSkyBrightness();

        /// <summary>
        /// Gets the sky quality at the observatory (magnitudes per square arc second)
        /// </summary>
        /// <returns></returns>
        double GetSkyQuality();

        /// <summary>
        /// Gets the sky temperature(°C) at the observatory.
        /// </summary>
        /// <returns></returns>
        double GetSkyTemperature();

        /// <summary>
        /// Gets the seeing at the observatory measured as star full width half maximum (FWHM) in arc secs.
        /// </summary>
        /// <returns></returns>
        double GetStarFwhm();

        /// <summary>
        /// Gets the temperature(°C) at the observatory.
        /// </summary>
        /// <returns></returns>
        double GetTemperature();

        /// <summary>
        /// Gets the wind direction. The returned value must be between 0.0 and 360.0,
        /// interpreted according to the meteorological standard, where a special value
        /// of 0.0 is returned when the wind speed is 0.0. Wind direction is measured
        /// clockwise from north, through east, where East=90.0, South=180.0,
        /// West=270.0 and North=360.0.
        /// </summary>
        /// <returns></returns>
        double GetWindDirection();

        /// <summary>
        /// Gets the peak 3 second wind gust(m/s) at the observatory over the last 2 minutes.
        /// </summary>
        /// <returns></returns>
        double GetWindGust();

        /// <summary>
        /// Gets the wind speed(m/s) at the observatory.
        /// </summary>
        /// <returns></returns>
        double GetWindSpeed();

        /// <summary>
        /// Forces the driver to immediately query its attached hardware to refresh sensor values.
        /// </summary>
        void Refresh();

        /// <summary>
        /// Gets a description of the sensor with the name specified in the PropertyName parameter
        /// </summary>
        /// <returns></returns>
        string GetSensorDescription(ObservingConditionSensorName sensorName);

        /// <summary>
        /// Gets the time since the sensor was last updated
        /// </summary>
        /// <returns></returns>
        TimeSpan GetTimeSinceLastUpdate(ObservingConditionSensorName sensorName);
    }
}