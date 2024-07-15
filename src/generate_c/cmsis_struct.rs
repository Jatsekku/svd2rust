// use crate::svd::{Peripheral};
// use anyhow::{Result};

// /* struct {peripheral_type_prefix}_object {
//  * {__I | __O | __IO} {uint8_t | uint16_t | uint32_t | uint64_t} REGISTER_NAME
//  * } 
//  */
// pub fn generate_struct(pi: &PeripheralInfo) -> Result<String> {
//     let peripheral_type_prefix = if let Some(group_name) = &pi.group_name {
//         &pi.group_name;
//     } else {
//         &pi.name;
//     }
//     println!("{}", peripheral_type_prefix);
//     Ok(format!(" "));
// }