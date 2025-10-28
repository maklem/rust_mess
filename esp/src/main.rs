#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use alloc::string::ToString;
use esp_hal::{
    main,
    clock::CpuClock,
    gpio::{Output, OutputConfig, Level},
    system::software_reset,
    time::{Duration, Instant},
    timer::timg::TimerGroup,
};
use rtt_target::rprintln;

use smoltcp::wire::{EthernetAddress, IpCidr};

use mess_lib::reset_on_failure_count::ResettingCounter;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

fn now() -> smoltcp::time::Instant {
    return smoltcp::time::Instant::from_micros(Instant::now().duration_since_epoch().as_micros() as i64);
}

fn reset_esp() -> () {
    software_reset();
}

#[main]
fn main() -> ! {
    rtt_target::rtt_init_print!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut watchdog = ResettingCounter::new(reset_esp, 10);

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

    let (mut wifi_controller, mut wifi_interfaces) = esp_wifi::wifi::new(&wifi_init, peripherals.WIFI).unwrap();

    let _ = wifi_controller.start();

    let _ = wifi_controller.set_configuration(&esp_wifi::wifi::Configuration::Client(wifi_config));
    let status = wifi_controller.connect();

    if status.is_err() {
        let err = status.err().unwrap();
        rprintln!("Wifi Error: {:?}", err);
    } else {
        rprintln!("Wifi connecting...");
    }

    loop {
        let status = wifi_controller.is_connected();
        if status.is_err() {
            rprintln!("Connecting failed: {:?}", status.err().unwrap());
        }else{
            let connected = status.ok().unwrap();
            if connected { break; }
            rprintln!(".");
            let delay_start = Instant::now();
            while delay_start.elapsed() < Duration::from_millis(500) {}
            watchdog.increment_failure();
        }
    }
    watchdog.reset();

    let mut rx_data =  [0u8; 1024];
    let rx_buffer = smoltcp::socket::tcp::SocketBuffer::new(&mut rx_data as &mut [u8]);
    let mut tx_data =  [0u8; 1024];
    let tx_buffer = smoltcp::socket::tcp::SocketBuffer::new(&mut tx_data as &mut [u8]);
    let mut socket = smoltcp::socket::tcp::Socket::new(rx_buffer,tx_buffer);
    socket.set_timeout(Option::Some(smoltcp::time::Duration::from_secs(5)));

    rprintln!("connection state {:?}", socket.state());

    let mut socket_data =  [smoltcp::iface::SocketStorage::EMPTY; 10];
    let mut socket_set = smoltcp::iface::SocketSet::new(&mut socket_data as &mut [smoltcp::iface::SocketStorage]);

    let iface_config = smoltcp::iface::Config::new(
        smoltcp::wire::HardwareAddress::Ethernet(
            EthernetAddress(wifi_interfaces.sta.mac_address()).into()));
    let mut iface = smoltcp::iface::Interface::new(
        iface_config,
        &mut wifi_interfaces.sta,
        now());
    iface.update_ip_addrs(|ipaddrs| {
        ipaddrs.push(IpCidr::new(smoltcp::wire::IpAddress::v4(192,168,178,200), 24)).unwrap()
    });

    iface
        .routes_mut()
        .add_default_ipv4_route(smoltcp::wire::Ipv4Address::new(192, 168, 178, 1))
        .unwrap();

    rprintln!("connection state {:?}", socket.state());
/*
    let mut client: MqttClient<1024> = MqttClient::new();
    let client_id = "";
    let mqtt_connect = client.connect(client_id, None).unwrap();
    let send_stat = socket.send_slice(&mqtt_connect);
    if send_stat.is_err() {
        rprintln!("send_stat err = {:?}", send_stat.err().unwrap());
    }
*/
    let tcp_handle = socket_set.add(socket);
    loop {
        let timestamp = now();
        iface.poll(timestamp, &mut wifi_interfaces.sta, &mut socket_set);

        let connection = socket_set.get_mut::<smoltcp::socket::tcp::Socket>(tcp_handle);

        if connection.state() == smoltcp::socket::tcp::State::Closed {
            rprintln!("trying to connect...");
            let con_stat = connection.connect(
                iface.context(),
                (smoltcp::wire::Ipv4Address::new(192,168,178,42), 1337),
                (smoltcp::wire::Ipv4Address::new(192,168,178,200), 50000)
            );
            if con_stat.is_err() {
                rprintln!("con_stat err = {:?}", con_stat.err().unwrap());
            }
        }


        rprintln!("Hello world!");
        rprintln!("Wifi connected: {:?}", wifi_controller.is_connected());
        rprintln!("connection state {:?}", connection.state());
        
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}
        led_pin.set_high();
        while delay_start.elapsed() < Duration::from_millis(1000) {}
        led_pin.set_low();
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.0/examples/src/bin
}
