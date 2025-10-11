use anyhow::Result;
use svd_rs::Device;
use crate::generate_c::{interrupts, peripheral};

// pub fn render(d: &Device) -> Result<String> {
//     let mut result = String::new();

//     result.push_str(&interrupts::render(d).unwrap());
//     result.push_str(&peripheral::render(&d.peripherals));

//     Ok(result)
// }