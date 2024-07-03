use crate::svd::{Field, FieldInfo};
use crate::config::{Config};
use anyhow::{Result};

pub fn render(f: &Field, config: &Config) -> Result<String> {
    let out = match &f {
        Field::Single(fi) => {
            let name = &fi.name;
            format!("{}", name)

        },
        Field::Array(fi,_) => {
            format!(" ")
        }

    };

    Ok(out)
}