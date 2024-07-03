use crate::svd::{RegisterCluster, Register, Cluster};
use crate::config::{Config};
use anyhow::{Result};
use crate::generate_c::{field};

fn render_register(r: &Register, config: &Config, base_address: u64) -> Result<String> {
    let out = match &r {
        Register::Single(ri) => {
            let mut vec = Vec::new();
            let name = &ri.name;
            let absolute_address = base_address + ri.address_offset as u64;
            let size = if let Some(size) = ri.properties.size {
                size
            } else {
                32
            };

            vec.push(format!("#define {} {} SFR_MMIO32({:#02x})", name, size, absolute_address));

            if let Some(fields) = &ri.fields {
                for f in fields {
                    vec.push(field::render(&f, config)?);
                }
            }

            vec.join("\n")
            
        },
        Register::Array(ri,_) => {
            format!(" ")
        }
    };

    Ok(out)
}

pub fn render(r: &RegisterCluster, config: &Config, base_address: u64) -> Result<String> {
    let out = match &r {
        RegisterCluster::Register(r) => { 
            render_register(&r, config, base_address)?
        },

        RegisterCluster::Cluster(c) => {
            format!(" ")
        }
    };

    Ok(out)
}