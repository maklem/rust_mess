use esp_hal::{analog::adc::{Adc, AdcConfig, AdcPin, AdcChannel}, gpio::AnalogPin};

pub struct SingleAnalogInput<'d, ADCI, PIN> 
where
    PIN: AdcChannel + AnalogPin
{
    adc_pin: AdcPin<PIN, ADCI>,
    adc: Adc<'d, ADCI, esp_hal::Blocking>,
}

impl<'d, ADCI, PIN> SingleAnalogInput<'d, ADCI, PIN>
where
    PIN: AdcChannel + AnalogPin,
    ADCI: esp_hal::analog::adc::RegisterAccess + 'd,
{
    pub fn new(pin: PIN, adci: ADCI) -> Self
    {     
        let mut adc_config = AdcConfig::new();
        let adc_pin = adc_config.enable_pin(pin, esp_hal::analog::adc::Attenuation::_11dB);
        let adc = Adc::new(adci, adc_config);

        SingleAnalogInput {
            adc_pin,
            adc,
        }
    }

    pub fn read(&mut self) -> nb::Result<f32, ()> {
        let adc_value = nb::block!(self.adc.read_oneshot(&mut self.adc_pin));
        if adc_value.is_err() {
            return Result::Err(nb::Error::Other(()));
        }
        let pin_value = adc_value.ok().unwrap();
        return Result::Ok(pin_value as f32 / 4095.0);
    }
}