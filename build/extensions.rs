use std::{
    env,
    fs::File,
    io::{self, Write},
    path::Path,
};

use crate::extract::{Extension, Extracted};

pub fn generate(extracted: &Extracted) -> io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut f = File::create(Path::new(&out_dir).join("instance_extensions.rs")).unwrap();

    writeln!(f, "/// Extensions for an [`Instance`].")?;
    writeln!(f, "#[derive(Clone, Debug, Default, PartialEq, Eq)]")?;
    writeln!(f, "pub struct InstanceExtensions {{")?;

    write_fields(&mut f, &extracted.extensions, true)?;

    writeln!(f, "}}")?;
    writeln!(f)?;
    writeln!(f, "impl InstanceExtensions {{")?;

    write_from_names(&mut f, &extracted.extensions, true)?;
    write_extension_names(&mut f, &extracted.extensions, true)?;
    write_validate(&mut f, &extracted.extensions, true)?;
    write_is_empty(&mut f, &extracted.extensions, true)?;
    write_union(&mut f, &extracted.extensions, true)?;
    write_intersection(&mut f, &extracted.extensions, true)?;
    write_contains(&mut f, &extracted.extensions, true)?;
    write_difference(&mut f, &extracted.extensions, true)?;

    writeln!(f, "}}")?;

    let out_dir = env::var("OUT_DIR").unwrap();
    let mut f = File::create(Path::new(&out_dir).join("device_extensions.rs")).unwrap();

    writeln!(f, "/// Extensions for a [`Device`].")?;
    writeln!(f, "#[derive(Clone, Debug, Default, PartialEq, Eq)]")?;
    writeln!(f, "pub struct DeviceExtensions {{")?;

    write_fields(&mut f, &extracted.extensions, false)?;

    writeln!(f, "}}")?;
    writeln!(f)?;
    writeln!(f, "impl DeviceExtensions {{")?;

    write_from_names(&mut f, &extracted.extensions, false)?;
    write_extension_names(&mut f, &extracted.extensions, false)?;
    write_validate(&mut f, &extracted.extensions, false)?;
    write_is_empty(&mut f, &extracted.extensions, false)?;
    write_union(&mut f, &extracted.extensions, false)?;
    write_intersection(&mut f, &extracted.extensions, false)?;
    write_contains(&mut f, &extracted.extensions, false)?;
    write_difference(&mut f, &extracted.extensions, false)?;

    writeln!(f, "}}")?;

    Ok(())
}

fn write_fields(f: &mut File, extensions: &[Extension], is_instance: bool) -> io::Result<()> {
    for ext in extensions {
        if ext.is_instance != is_instance {
            continue;
        }

        writeln!(f, "    /// Enable the [`{}`]() extension.", ext.name)?;

        if !ext.any_of.0.is_empty() {
            writeln!(f, "    ///")?;
            writeln!(f, "    /// Requires any of the following:")?;

            for all_of in &ext.any_of.0 {
                match all_of.api_version {
                    Some(version) => {
                        write!(f, "    /// - [`API version {}`](crate::Version)", version,)?;

                        if !all_of.extensions.is_empty() {
                            writeln!(f, " and all of the following extensions:")?;
                        } else {
                            writeln!(f, ".")?;
                        }
                    }
                    None => {
                        writeln!(f, "    /// - All of the following extensions:")?;
                    }
                }

                for ext in &all_of.extensions {
                    let ext = extensions.iter().find(|e| e.name == *ext).unwrap();

                    let reference = match ext.is_instance {
                        true => format!("crate::InstanceExtensions::{}", ext.rust_name()),
                        false => format!("crate::DeviceExtensions::{}", ext.rust_name()),
                    };

                    writeln!(f, "    ///   - [`{}`]({})", ext.name, reference)?;
                }
            }
        }

        writeln!(f, "    pub {}: bool,", ext.rust_name())?;
    }

    Ok(())
}

fn write_from_names(f: &mut File, extensions: &[Extension], is_instance: bool) -> io::Result<()> {
    #[rustfmt::skip]
    {
        writeln!(f, "    pub(crate) fn from_names<'a>(names: impl Iterator<Item = &'a str>) -> Self {{")?;
        writeln!(f, "        let mut extensions = Self::default();")?;
        writeln!(f, )?;
        writeln!(f, "        for name in names {{")?;
        writeln!(f, "            match name {{")?;

        for ext in extensions {
            if ext.is_instance != is_instance {
                continue;
            }

            let rust_name = ext.rust_name();
            writeln!(f, "                \"{}\" => {{", ext.name)?;
            writeln!(f, "                    extensions.{rust_name} = true;")?;
            writeln!(f, "                }},")?;
        }

        writeln!(f, "                _ => {{}}")?;
        writeln!(f, "            }}")?;
        writeln!(f, "        }}")?;
        writeln!(f, )?;
        writeln!(f, "        extensions")?;
        writeln!(f, "    }}")?;
    };

    Ok(())
}

fn write_extension_names(
    f: &mut File,
    extensions: &[Extension],
    is_instance: bool,
) -> io::Result<()> {
    #[rustfmt::skip]
    {
        writeln!(f, "    fn extension_names(&self) -> Vec<*const ffi::c_char> {{")?;
        writeln!(f, "        let mut names = Vec::new();")?;
        writeln!(f)?;

        for ext in extensions {
            if ext.is_instance != is_instance {
                continue;
            }

            writeln!(f, "        if self.{} {{", ext.rust_name())?;
            writeln!(f, "            names.push(c\"{}\".as_ptr());", ext.name)?;
            writeln!(f, "        }}")?;
        }

        writeln!(f)?;
        writeln!(f, "        names")?;
        writeln!(f, "    }}")?;
    };

    Ok(())
}

fn write_is_empty(f: &mut File, extensions: &[Extension], is_instance: bool) -> io::Result<()> {
    writeln!(f, "    /// Check if no extensions are enabled.")?;
    writeln!(f, "    pub fn is_empty(&self) -> bool {{")?;
    writeln!(f, "        true")?;

    for ext in extensions {
        if ext.is_instance != is_instance {
            continue;
        }

        writeln!(f, " && !self.{}", ext.rust_name())?;
    }

    writeln!(f, "    }}")?;
    writeln!(f)?;

    Ok(())
}

fn write_union(f: &mut File, extensions: &[Extension], is_instance: bool) -> io::Result<()> {
    writeln!(f, "    /// Combine two sets of extensions into one.")?;
    writeln!(f, "    pub fn union(&self, other: &Self) -> Self {{")?;
    writeln!(f, "        Self {{")?;

    for ext in extensions {
        if ext.is_instance != is_instance {
            continue;
        }

        let name = ext.rust_name();
        writeln!(f, "            {name}: self.{name} || other.{name},")?;
    }

    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;

    Ok(())
}

fn write_intersection(f: &mut File, extensions: &[Extension], is_instance: bool) -> io::Result<()> {
    #[rustfmt::skip]
    writeln!(f, "    /// Compute the intersection of two sets of extensions.")?;
    writeln!(f, "    pub fn intersection(&self, other: &Self) -> Self {{")?;
    writeln!(f, "        Self {{")?;

    for ext in extensions {
        if ext.is_instance != is_instance {
            continue;
        }

        let name = ext.rust_name();
        writeln!(f, "            {name}: self.{name} && other.{name},")?;
    }

    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;

    Ok(())
}

fn write_difference(f: &mut File, extensions: &[Extension], is_instance: bool) -> io::Result<()> {
    writeln!(
        f,
        "    /// Compute the difference of two sets of extensions."
    )?;
    writeln!(f, "    pub fn difference(&self, other: &Self) -> Self {{")?;
    writeln!(f, "        Self {{")?;

    for ext in extensions {
        if ext.is_instance != is_instance {
            continue;
        }

        let name = ext.rust_name();
        writeln!(f, "            {name}: self.{name} && !other.{name},")?;
    }

    writeln!(f, "        }}")?;
    writeln!(f, "    }}")?;

    Ok(())
}

fn write_contains(f: &mut File, extensions: &[Extension], is_instance: bool) -> io::Result<()> {
    writeln!(f, "    /// Check if the extension is enabled.")?;
    writeln!(f, "    pub fn contains(&self, other: &Self) -> bool {{")?;
    writeln!(f, "        true")?;

    for ext in extensions {
        if ext.is_instance != is_instance {
            continue;
        }

        let name = ext.rust_name();
        writeln!(f, " && (!self.{name} || other.{name})")?;
    }

    writeln!(f, "    }}")?;

    Ok(())
}

fn write_validate(f: &mut File, extensions: &[Extension], is_instance: bool) -> io::Result<()> {
    if is_instance {
        writeln!(f, "    fn validate(")?;
        writeln!(f, "        &self,")?;
        writeln!(f, "        api_version: Version,")?;
        writeln!(f, "    ) -> Result<(), ValidationError> {{")?;
    } else {
        writeln!(f, "    fn validate(")?;
        writeln!(f, "        &self,")?;
        writeln!(f, "        instance_extensions: &InstanceExtensions,")?;
        writeln!(f, "        api_version: Version,")?;
        writeln!(f, "    ) -> Result<(), ValidationError> {{")?;
    }

    for ext in extensions {
        if ext.is_instance != is_instance {
            continue;
        }

        if ext.any_of.0.is_empty() {
            continue;
        }

        let mut any = format!("self.{}", ext.rust_name());

        for any_of in ext.any_of.0.iter() {
            let mut all = String::from("false");

            if let Some(version) = any_of.api_version {
                all = format!("api_version < Version::V1_{}", version);
            }

            for name in &any_of.extensions {
                let ext = extensions.iter().find(|e| e.name == *name).unwrap();

                match !is_instance && ext.is_instance {
                    true => all += &format!(" || !instance_extensions.{}", ext.rust_name()),
                    false => all += &format!(" || !self.{}", ext.rust_name()),
                }
            }

            any += &format!(" && ({})", all);
        }

        // write the if statement to check if the extension is enabled
        writeln!(f, "        if {any} {{")?;
        writeln!(f, "            return Err(ValidationError {{")?;
        #[rustfmt::skip]
        writeln!(f, "                context: \"desc.enabled_extensions.{}\".into(),", ext.rust_name())?;
        #[rustfmt::skip]
        writeln!(f, "                problem: \"Extension is enabled, but requirements aren't met.\".into(),")?;

        // write the vuids field
        #[rustfmt::skip]
        if is_instance {
            writeln!(f, "                vuids: &[\"VUID-vkCreateInstance-ppEnabledExtensionNames-01388\"],")?;
        } else {
            writeln!(f, "                vuids: &[\"VUID-vkCreateDevice-ppEnabledExtensionNames-01387\"],")?;
        };

        // write the requires_one_of field
        writeln!(f, "                requires_one_of: &[")?;

        for any_of in &ext.any_of.0 {
            // write the requires struct for each any_of
            writeln!(f, "                    Requires {{")?;

            // write the api_version field if it exists
            if let Some(version) = any_of.api_version {
                #[rustfmt::skip]
                writeln!(f, "                        api_version: Some(Version::V1_{version}),")?;
            } else {
                writeln!(f, "                        api_version: None,")?;
            }

            // write the instance_extensions field
            writeln!(f, "                        instance_extensions: &[")?;

            // write each extension name
            for name in &any_of.extensions {
                let ext = extensions.iter().find(|e| e.name == *name).unwrap();
                if ext.is_instance {
                    writeln!(f, "                            \"{}\",", name)?;
                }
            }

            writeln!(f, "                        ],")?;

            // write the device_extensions field
            writeln!(f, "                        device_extensions: &[")?;

            // write each extension name
            for name in &any_of.extensions {
                let ext = extensions.iter().find(|e| e.name == *name).unwrap();
                if !ext.is_instance {
                    writeln!(f, "                            \"{}\",", name)?;
                }
            }

            writeln!(f, "                        ],")?;
            writeln!(f, "                    }},")?;
        }

        writeln!(f, "                ],")?;
        writeln!(f, "            }});")?;
        writeln!(f, "        }}")?;
    }

    writeln!(f)?;
    writeln!(f, "        Ok(())")?;
    writeln!(f, "    }}")?;

    Ok(())
}
