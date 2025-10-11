use anyhow::Result;
use std::collections::HashMap;
use svd_rs::Device;

// Generate interrupts section

pub fn render(d: &Device) -> Result<String> {
    // Obtain sorted vector of interrupts
    let mut result = String::new();
    let interrupts = d
        .peripherals
        .iter()
        .flat_map(|p| p.interrupt.iter())
        .map(|i| (i.value, i))
        .collect::<HashMap<_, _>>();

    let mut interrupts = interrupts.iter().map(|i| i.1).collect::<Vec<_>>();
    interrupts.sort_by_key(|i| i.value);

    // Rendering interrupt section
    result.push_str("typedef enum {\n");

    interrupts.iter().for_each(|i| {
        let (name, number, description) = (
            format!("{}_IRQn", &i.name),
            i.value,
            match &i.description {
                Some(d) => d.to_owned(),
                None => String::new(),
            },
        );
        let line = format!(
            "  {name:<25} =  {number}, {: <12} /*!< {description:<73} */\n",
            " "
        );
        result.push_str(&line);
    });

    result.push_str("} IRQn_Type;");
    Ok(result)
}
