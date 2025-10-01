#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use alloc::string::ToString;
use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::{
    time::{Duration, Instant},
    gpio::{Output, OutputConfig, Level},
};
use esp_hal::timer::timg::TimerGroup;
use esp_wifi::wifi::new;
use rtt_target::rprintln;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    rtt_target::rtt_init_print!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut led_pin = Output::new(peripherals.GPIO4, Level::High, OutputConfig::default());

    esp_alloc::heap_allocator!(size: 64 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let wifi_init = esp_wifi::init(timg0.timer0, esp_hal::rng::Rng::new(peripherals.RNG)).unwrap();

    let wifi_ssid = env!("WIFI_SSID").to_string();
    let wifi_pass = env!("WIFI_PASS").to_string();
    rprintln!("Connecting to '{ssid}' using '{pass}' ", ssid=wifi_ssid, pass=wifi_pass);

    let wifi_config = esp_wifi::wifi::ClientConfiguration{
        ssid: wifi_ssid,
        password: wifi_pass,
        auth_method: esp_wifi::wifi::AuthMethod::WPA2Personal,
        channel: None,
        bssid: None,
    };

    let (mut wifi_controller, mut _wifi_interfaces) = esp_wifi::wifi::new(&wifi_init, peripherals.WIFI).unwrap();

    let _ = wifi_controller.start();

    let _ = wifi_controller.set_configuration(&esp_wifi::wifi::Configuration::Client(wifi_config));
    let status = wifi_controller.connect();

    if status.is_err() {
        let err = status.err().unwrap();
        rprintln!("Wifi Error: {:?}", err);
    } else {
        rprintln!("Wifi connected!");
    }

    let mut rx_data =  [0u8; 1024];
    let rx_buffer = smoltcp::socket::tcp::SocketBuffer::new(&mut rx_data as &mut [u8]);
    let mut tx_data =  [0u8; 1024];
    let tx_buffer = smoltcp::socket::tcp::SocketBuffer::new(&mut tx_data as &mut [u8]);
    let connection = smoltcp::socket::tcp::Socket::new(rx_buffer,tx_buffer);

    loop {
        rprintln!("Hello world!");
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}
        led_pin.set_high();
        while delay_start.elapsed() < Duration::from_millis(1000) {}
        led_pin.set_low();
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.0/examples/src/bin
}
