use std::{collections::{HashMap, HashSet}, iter, result};

use anyhow::{Context, Result, anyhow};
use svd_parser::expand::{derive_peripheral, Index, BlockPath};
use svd_rs::{
    Access, DimElement, MaybeArray, Peripheral, RegisterInfo, RegisterProperties, ValidateLevel, DeriveFrom
};
use syn::token::Struct;

fn create_reserved(
    start_address: u32,
    size: u32,
    cnt: u32,
    fillers: &mut Vec<MaybeArray<RegisterInfo>>,
) -> u32 {
    let mut fillers_cnt = cnt;
    let mut offset = start_address;

    let whole_u32 = size / 4;
    match whole_u32 {
        0 => (),
        1 => {
            fillers.push(MaybeArray::Single(
                RegisterInfo::builder()
                    .name(format!("RESERVED{fillers_cnt}"))
                    .address_offset(offset)
                    .access(Some(Access::ReadOnly))
                    .properties(
                        RegisterProperties::default()
                            .size(Some(32))
                            .access(Some(Access::ReadOnly)),
                    )
                    .build(ValidateLevel::Disabled)
                    .unwrap(),
            ));
            fillers_cnt += 1;
            offset += 4;
        }
        n => {
            fillers.push(MaybeArray::Array(
                RegisterInfo::builder()
                    .name(format!("RESERVED{fillers_cnt}[%s]"))
                    .address_offset(offset)
                    .access(Some(Access::ReadOnly))
                    .properties(
                        RegisterProperties::default()
                            .size(Some(32))
                            .access(Some(Access::ReadOnly)),
                    )
                    .build(ValidateLevel::Disabled)
                    .unwrap(),
                DimElement::builder()
                    .dim(n)
                    .dim_increment(1)
                    .build(ValidateLevel::Disabled)
                    .unwrap(),
            ));
            fillers_cnt += 1;
            offset += n * 4;
        }
    };

    let remainder = size % 4;
    match remainder {
        1 => {
            fillers.push(MaybeArray::Single(
                RegisterInfo::builder()
                    .name(format!("RESERVED{fillers_cnt}"))
                    .address_offset(offset)
                    .access(Some(Access::ReadOnly))
                    .properties(
                        RegisterProperties::default()
                            .size(Some(8))
                            .access(Some(Access::ReadOnly)),
                    )
                    .build(ValidateLevel::Disabled)
                    .unwrap(),
            ));
            fillers_cnt += 1;
            offset += 1;
        }
        2 => {
            fillers.push(MaybeArray::Single(
                RegisterInfo::builder()
                    .name(format!("RESERVED{fillers_cnt}"))
                    .address_offset(offset)
                    .access(Some(Access::ReadOnly))
                    .properties(
                        RegisterProperties::default()
                            .size(Some(16))
                            .access(Some(Access::ReadOnly)),
                    )
                    .build(ValidateLevel::Disabled)
                    .unwrap(),
            ));
            fillers_cnt += 1;
            offset += 2;
        }
        3 => {
            let filler_u16 = MaybeArray::Single(
                RegisterInfo::builder()
                    .name(format!("RESERVED{fillers_cnt}"))
                    .address_offset(offset)
                    .access(Some(Access::ReadOnly))
                    .properties(
                        RegisterProperties::default()
                            .size(Some(16))
                            .access(Some(Access::ReadOnly)),
                    )
                    .build(ValidateLevel::Disabled)
                    .unwrap(),
            );
            fillers_cnt += 1;
            offset += 2;

            let filler_u8 = MaybeArray::Single(
                RegisterInfo::builder()
                    .name(format!("RESERVED{fillers_cnt}"))
                    .address_offset(offset)
                    .access(Some(Access::ReadOnly))
                    .properties(
                        RegisterProperties::default()
                            .size(Some(8))
                            .access(Some(Access::ReadOnly)),
                    )
                    .build(ValidateLevel::Disabled)
                    .unwrap(),
            );
            fillers_cnt += 1;
            // offset += 1;

            fillers.push(filler_u16);
            fillers.push(filler_u8);
        }
        _ => (),
    };

    fillers_cnt
}

fn get_register_dimmensions(r: &MaybeArray<RegisterInfo>) -> (u32, u32, u32) {
    let r_start = r.address_offset;
    let r_size = r.properties.size.unwrap() / 8;
    let r_end = r_start + r_size - 1;

    (r_start, r_end, r_size)
}

fn render_direct_member(r: &MaybeArray<RegisterInfo>) -> String {
    let acccess = match r.properties.access {
        Some(Access::ReadOnly) => "__IO",
        Some(Access::ReadWrite) | Some(Access::ReadWriteOnce) => "__O",
        Some(Access::WriteOnly) | Some(Access::WriteOnce) => "__I",
        _ => "__IO",
    };

    let size = match r.properties.size.unwrap() {
        8 => "uint8_t",
        16 => "uint16_t",
        32 => "uint32_t",
        _ => "",
    };

    let identifier = match r {
        MaybeArray::Single(r) => &r.name,
        MaybeArray::Array(r, d) => &r.name.replace("[%s]", &format!("[{}]", d.dim)),
    };

    format!("{acccess} {size} {identifier};\n")
}

fn render_union_member(r: &Vec<&MaybeArray<RegisterInfo>>) -> String {
    let members = r
        .iter()
        .map(|r| format!("  {}", render_direct_member(r)))
        .collect::<Vec<String>>()
        .join("");

    format!("union {{\n{members}}};\n")
}

fn render_base_define(p: &Peripheral) -> String {
    format!("#define {}_BASE 0x{:X}UL\n", p.name, p.base_address)
}

fn render_peripheral_define(p: &Peripheral) -> String {
    format!("#define {} (({}_Type*) {}_BASE)\n", p.name, p.name, p.name)
}

// Generate C struct defining single peripheral registers
fn render_struct(p_original: &Peripheral, i: &Index) -> Result<String> {
    // Derive if possible
    let mut p = p_original.clone();
    if let Some(dpath) = p.derived_from.clone() {
        let derpath = BlockPath::new(dpath);
        let d = i
            .peripherals
            .get(&derpath)
            .ok_or_else(|| anyhow!("peripheral not found"))?;
        dbg!(&d);
        p.derive_from(d);
        dbg!(p);
    }

    // Obtain sorted vector of registers
    let mut r_original = p.registers().collect::<Vec<_>>();
    r_original.sort_by_key(|r| r.address_offset);

    // Create new vector of reserved registers (fillers)
    let mut r_fillers = Vec::new();
    let mut previous_end: i32 = -1;
    let mut hole_counter = 0;
    r_original.iter().for_each(|r| {
        let r_size = r.properties.size.unwrap() / 8;
        let r_start = r.address_offset;
        let r_end = r_start + r_size - 1;

        let offsett_diff = ((r_start as i32 - previous_end).abs()) as u32;

        if offsett_diff > 0 {
            let (hole_start, hole_size) = ((previous_end + 1_i32) as u32, offsett_diff - 1);
            hole_counter = create_reserved(hole_start, hole_size, hole_counter, &mut r_fillers);
        }
        previous_end = r_end as i32;
    });

    // Merge original register list with fillers
    let mut r_final = Vec::new();
    let (mut oi, mut fi) = (r_original.iter(), r_fillers.iter());
    loop {
        match (oi.next(), fi.next()) {
            (Some(&o), Some(f)) => {
                if o.address_offset < f.address_offset {
                    r_final.push(o);
                    r_final.push(f);
                } else {
                    r_final.push(f);
                    r_final.push(o);
                }
            }
            (Some(&o), None) => {
                r_final.push(o);
            }
            (None, Some(f)) => {
                r_final.push(f);
            }
            (None, None) => break,
        }
    }

    let mut result = String::new();
    if !r_final.is_empty() {
        result.push_str("typedef struct {\n");

        let mut stack: Vec<&MaybeArray<RegisterInfo>> = Vec::new();
        for &r in r_final.iter() {
            if let Some(top) = stack.last() {
                if top.address_offset != r.address_offset {
                    if stack.len() == 1 {
                        result.push_str(&render_direct_member(top));
                    } else {
                        result.push_str(&render_union_member(&stack));
                    }
                    stack.clear();
                }
            }
            stack.push(r);
        }
    
        if !stack.is_empty() {
            if stack.len() == 1 {
                result.push_str(&render_direct_member(stack.last().unwrap()));
            } else {
                result.push_str(&render_union_member(&stack));
            }
        }

        result.push_str(&format!("}} {}_Type;\n", &p.name));
    }

    Ok(result)
}

fn foo(ps: &Vec<Peripheral>) -> (HashMap<&String, &String>, Vec<&Peripheral>) {
    let mut instances_map = HashMap::new();
    let mut structs = Vec::new();

    for p in ps.iter() {
        let name = &p.name;
        let is_derived = p.derived_from.as_ref().is_some_and(|s| !s.is_empty());
        let has_own_regs = p.registers.as_ref().is_some_and(|rs| !rs.is_empty());

        match (is_derived, has_own_regs) {
            (true, false) => {
                instances_map.entry(name).or_insert(p.derived_from.as_ref().unwrap());
            },
            (_, _) => {
                instances_map.entry(name).or_insert(name);
                structs.push(p);
            },
        };
    }
    
    (instances_map, structs)
}

pub fn render(ps: &Vec<Peripheral>, i: &Index) -> Result<String> {
    let mut bases = String::new();
    let mut instances = String::new();

    let (peripheral_map, peripherals) = foo(ps);

    let structs : String = 
        peripherals
        .iter()
        .flat_map(|p| render_struct(p, i))
        .collect();

    // for p in ps.iter() {
    //     structs.push_str(&render_struct(p, i)?);
    //     bases.push_str(&render_base_define(p));
    //     instances.push_str(&render_peripheral_define(p));
    // }

    Ok(format!("{structs}\n{bases}\n{instances}"))
}
