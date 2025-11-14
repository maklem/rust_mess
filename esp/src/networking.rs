use alloc::string::String;

use esp_hal::time::{Duration, Instant};
use rtt_target::rprintln;
use esp_wifi::wifi::{Interfaces, WifiController};
use smoltcp::{iface::Interface, wire::{EthernetAddress, IpCidr}};

use crate::timeconversion;


pub fn wifi_connect_blocking(wifi_controller: &mut WifiController , wifi_ssid: String, wifi_pass: String) -> () {

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
            break;
        }else{
            let connected = status.ok().unwrap();
            if connected { break; }
            rprintln!(".");
            let delay_start = Instant::now();
            while delay_start.elapsed() < Duration::from_millis(500) {}
        }
    }
}

pub fn wifi_configure_static_network(wifi_interfaces: &mut Interfaces, ipv4: [u8; 4]) -> Interface {
   /* configure IPv4 for Wifi */
    let iface_config = smoltcp::iface::Config::new(
        smoltcp::wire::HardwareAddress::Ethernet(
            EthernetAddress(wifi_interfaces.sta.mac_address()).into()));
    let mut iface = smoltcp::iface::Interface::new(
        iface_config,
        &mut wifi_interfaces.sta,
        timeconversion::smoltcp_now());
    iface.update_ip_addrs(|ipaddrs| {
        ipaddrs.push(IpCidr::new(smoltcp::wire::IpAddress::v4(ipv4[0],ipv4[1],ipv4[2],ipv4[3]), 24)).unwrap()
    });

    iface
        .routes_mut()
        .add_default_ipv4_route(smoltcp::wire::Ipv4Address::new(ipv4[0], ipv4[1], ipv4[2], 1))
        .unwrap();
    return iface;
}