use crate::svd::{Device};
use crate::config::{Config};
use anyhow::{Result};
use crate::generate_c::{peripheral};

pub fn render(d: &Device, config: &Config, device_x: &mut String) -> Result<String> {
    let mut peripherals_out = Vec::new();

    for p in &d.peripherals {
        peripherals_out.push(peripheral::render(&p, config)?);
    }

    Ok(peripherals_out.join("\n"))
}