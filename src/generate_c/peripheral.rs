use crate::svd::{Peripheral};
use crate::config::{Config};
use anyhow::{Result};
use crate::generate_c::{register};

pub fn render(p: &Peripheral, config: &Config) -> Result<String> {
    let out = match &p {
        Peripheral::Array(pi,_) => { 
            format!(" ")
        },

        Peripheral::Single(pi) => {
            let name = &pi.name;
            let base_address = pi.base_address;

            if let Some(registers) = &pi.registers {
                let mut registers_out = Vec::new();

                for r in registers {
                    registers_out.push(register::render(&r, config, base_address)?);
                }
                registers_out.join("\n")

            } else {
                format!(" ")
            }
        }
    };

    Ok(out)
}