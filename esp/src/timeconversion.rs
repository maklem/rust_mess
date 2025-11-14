use esp_hal::time::Instant;

pub fn smoltcp_now() -> smoltcp::time::Instant {
    return smoltcp::time::Instant::from_micros(Instant::now().duration_since_epoch().as_micros() as i64);
}