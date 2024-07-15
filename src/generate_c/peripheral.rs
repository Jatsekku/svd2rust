use crate::svd::{Peripheral, PeripheralInfo, RegisterInfo, RegisterCluster, Register};
use crate::config::{Config};
use anyhow::{Context, Result, bail};
use crate::generate_c::{register};

fn render_struct_member(ri : &RegisterInfo) -> Result<String> {
    let name = &ri.name;
    let size = ri.properties.size.context("No size")?;

    let size = match size {
        8 => {format!("uint8_t")},
        16 => {format!("uint16_t")},
        32 => {format!("uint32_t")},
        _=> {bail!("Not supported register size")}
    };

    Ok(format!("    {} {}", size, name))
}

fn render_struct(pi: &PeripheralInfo) -> Result<String> {
    let group_name = pi.group_name.as_deref().context("No group name")?;
    let registers = pi.registers.as_ref().context("No register node").and_then(|v| {
        if v.is_empty() {
            bail!("No registers")
        } else {
            Ok(v)
        }
    })?;

    let mut out = vec![format!("typedef struct {{")];
    for rc in registers {
        match rc {
            RegisterCluster::Register(rr) => {
                match rr {
                    Register::Single(ri) => {
                        out.push(render_struct_member(&ri)?);
                    }

                    Register::Array(r, _) => {
                        println!("Not supported!");
                    }
                }
            }

            RegisterCluster::Cluster(_) => {
                println!("Not supported yet")
            }

        }
    }

    out.push(format!("}} {}_Type;", group_name));
    Ok(out.join("\n"))
}

pub fn render(p: &Peripheral, config: &Config) -> Result<String> {
    let out = match &p {
        Peripheral::Array(pi,_) => { 
            format!("not supported yet")
        },

        Peripheral::Single(pi) => {
            println!("{}",render_struct(&pi)?);
            let mut out = String::new();
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