use std::{
    env,
    fs::File,
    io::{self, Write},
    path::Path,
};

use crate::extract::Extracted;

pub fn generate(extracted: &Extracted) -> io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut file = File::create(Path::new(&out_dir).join("format.rs"))?;

    writeln!(file, "#[repr(i32)]")?;
    writeln!(file, "#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]")?;
    writeln!(file, "pub enum Format {{")?;

    for format in &extracted.formats {
        let Some(ref enum_value) = format.enum_value else {
            continue;
        };

        writeln!(file, "    {} = {},", format.rust_name(), enum_value)?;
    }

    writeln!(file, "}}")?;

    writeln!(file, "impl Format {{")?;
    writeln!(file, "    pub fn from_raw(raw: i32) -> Option<Self> {{")?;
    writeln!(file, "        match raw {{")?;

    for format in &extracted.formats {
        let Some(ref enum_value) = format.enum_value else {
            continue;
        };

        writeln!(
            file,
            "            {} => Some(Self::{}),",
            enum_value,
            format.rust_name(),
        )?;
    }

    writeln!(file, "            _ => None,")?;
    writeln!(file, "        }}")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;

    Ok(())
}
