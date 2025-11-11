#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use alloc::{format, string::ToString};
use esp_hal::{
    clock::CpuClock, gpio::{Level, Output, OutputConfig}, main, system::software_reset, time::{Duration, Instant}, timer::timg::TimerGroup
};
use rtt_target::rprintln;

use smoltcp::wire::{EthernetAddress, IpCidr};
use tinymqtt::MqttClient;

use mess_lib::reset_on_failure_count::ResettingCounter;
mod analog;

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
    esp_alloc::heap_allocator!(size: 64 * 1024);
    let mut watchdog = ResettingCounter::new(reset_esp, 20);

    /* initialize HAL */
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    /* initialize pins */
    let mut led_pin = Output::new(peripherals.GPIO4, Level::High, OutputConfig::default());
    let mut analog_pin = analog::SingleAnalogInput::new(peripherals.GPIO35, peripherals.ADC1);

    /* initialize wifi hardware */
    let hw_timer_group_0 = TimerGroup::new(peripherals.TIMG0);
    let hw_rng = esp_hal::rng::Rng::new(peripherals.RNG);
    let hw_wifi_controller = esp_wifi::init(hw_timer_group_0.timer0, hw_rng).unwrap();
    let (mut wifi_controller, mut wifi_interfaces) = esp_wifi::wifi::new(&hw_wifi_controller, peripherals.WIFI).unwrap();

    /* connect wifi */
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
   
   /* configure IPv4 for Wifi */
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

    /* Set up TCP socket */
    let mut rx_data =  [0u8; 1024];
    let rx_buffer = smoltcp::socket::tcp::SocketBuffer::new(&mut rx_data as &mut [u8]);
    let mut tx_data =  [0u8; 1024];
    let tx_buffer = smoltcp::socket::tcp::SocketBuffer::new(&mut tx_data as &mut [u8]);
    let mut socket = smoltcp::socket::tcp::Socket::new(rx_buffer,tx_buffer);
    socket.set_timeout(Option::Some(smoltcp::time::Duration::from_secs(600)));

    let mut socket_data =  [smoltcp::iface::SocketStorage::EMPTY; 10];
    let mut socket_set = smoltcp::iface::SocketSet::new(&mut socket_data as &mut [smoltcp::iface::SocketStorage]);
    let tcp_handle = socket_set.add(socket);

    /* Connect TCP */
    {
        let connection = socket_set.get_mut::<smoltcp::socket::tcp::Socket>(tcp_handle);
        rprintln!("trying to connect...");
        let con_stat = connection.connect(
            iface.context(),
            (smoltcp::wire::Ipv4Address::new(192,168,178,38), 1883),
            (smoltcp::wire::Ipv4Address::new(192,168,178,200), 50000)
        );
        if con_stat.is_err() {
            rprintln!("con_stat err = {:?}", con_stat.err().unwrap());
        }
    }

    /* Establish TCP Connection */
    loop {
        let timestamp = now();
        iface.poll(timestamp, &mut wifi_interfaces.sta, &mut socket_set);

        let connection = socket_set.get_mut::<smoltcp::socket::tcp::Socket>(tcp_handle);

        rprintln!("Establishing... -- Wifi: {:?} -- TCP: {:?}", wifi_controller.is_connected(), connection.state());

        match connection.state() {
            smoltcp::socket::tcp::State::Closed => {
                reset_esp();
            }
            smoltcp::socket::tcp::State::Established => {
                break;
            }
            _ => {
                watchdog.increment_failure();
            }
        }

        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}
    }    

    /* set up mqtt connection */
    let mut client: MqttClient<1024> = MqttClient::new();
    {
        let connection = socket_set.get_mut::<smoltcp::socket::tcp::Socket>(tcp_handle);

        let mqtt_auth =("rustesp", "helloworld");
        let client_id = "";

        let mqtt_connect = client.connect(client_id, Some(mqtt_auth)).unwrap();
        rprintln!("connection packet = {:?}", mqtt_connect);
        let send_stat = connection.send_slice(&mqtt_connect);
        if send_stat.is_err() {
            rprintln!("send_stat err = {:?}", send_stat.err().unwrap());
        }
    }

    /* main loop */
    /* if something breaks, `watchdog` will be notified. It will reset the chip after N failures. */
    loop {
        /* let wifi-hw communicate */
        let timestamp = now();
        iface.poll(timestamp, &mut wifi_interfaces.sta, &mut socket_set);

        /* assert wifi is connected */
        let wifi_status = wifi_controller.is_connected();
        if wifi_status.is_err() || ! wifi_status.ok().unwrap() {
            watchdog.increment_failure();
        }

        /* assert MQTT is connected */
        let connection = socket_set.get_mut::<smoltcp::socket::tcp::Socket>(tcp_handle);

        if connection.state() == smoltcp::socket::tcp::State::Closed {
            watchdog.increment_failure();
        }

        /* on MQTT - process messages*/
        if connection.may_recv() && connection.can_recv() {
            let mut data = [0u8; 256];
            let state = connection.recv_slice( &mut data as &mut [u8]);
            if state.is_ok(){
                let len = state.ok().unwrap();
                rprintln!("Received Data: {:?}",  &data[..len] );

                let status = client.receive_packet(&data[..len], |_client, topic, data| {
                    rprintln!("Received Packet: {:?} {:?}", topic, data);
                });
                if status.is_err() {
                    rprintln!("Error Receiving Packet: {:?}", status.err());
                }
                
            }
        }

        /* measure value */
        let pin_reading = analog_pin.read();
        let pin_value = if pin_reading.is_ok() {
            pin_reading.ok().unwrap()
        } else {
            rprintln!("Pin Read Error: {:?}", pin_reading.err());
            0.0
        };

        /* on MQTT - send measured value */
        if client.is_connected() {
            let payload_str = format!("{:.6e}", pin_value);
            let payload = payload_str.as_bytes();
            let publish_bytes = client.publish("sensordata/dev/rustesptest", payload);
            if publish_bytes.is_err() {
                rprintln!("Failed to send (assemble): {:?}", publish_bytes.err());
            } else {
                let send_status = connection.send_slice(publish_bytes.ok().unwrap());
                if send_status.is_ok() {
                    rprintln!("Sent {} = {}", payload_str, pin_value);
                } else {
                    rprintln!("Failed to send (connection): {:?}", send_status.err());
                }
            }
        }

        rprintln!("Measured: {:5.2e}      -- Wifi: {:?} -- TCP: {:?}", pin_value, wifi_controller.is_connected(), connection.state());
        
        /* blink LEDs ... and delay iterations */
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}
        led_pin.set_high();
        while delay_start.elapsed() < Duration::from_millis(1000) {}
        led_pin.set_low();
    }
}
