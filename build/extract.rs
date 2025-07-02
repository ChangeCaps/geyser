use std::io::Cursor;

use xml::{EventReader, attribute::OwnedAttribute, reader::XmlEvent};

pub struct Extracted {
    pub extensions: Vec<Extension>,
    pub formats: Vec<Format>,
}

impl Extracted {
    pub fn get_format_or_insert(&mut self, name: &str) -> &mut Format {
        if let Some(index) = self.formats.iter_mut().position(|f| f.name == name) {
            return &mut self.formats[index];
        }

        let format = Format {
            name: name.to_string(),
            enum_value: None,
            block_size: None,
            texels_per_block: None,
            packed: None,
            r: None,
            g: None,
            b: None,
            a: None,
        };

        self.formats.push(format);
        self.formats.last_mut().unwrap()
    }
}

#[derive(Debug)]
pub struct Extension {
    pub name: String,
    pub any_of: AnyOf,
    pub is_instance: bool,
}

impl Extension {
    pub fn rust_name(&self) -> String {
        (self.name).trim_start_matches("VK_").to_lowercase()
    }
}

#[derive(Debug)]
pub struct AnyOf(pub Vec<AllOf>);

#[derive(Debug)]
pub struct AllOf {
    pub api_version: Option<u32>,
    pub extensions: Vec<String>,
}

impl AllOf {
    pub fn from_api_version(version: u32) -> Self {
        AllOf {
            api_version: Some(version),
            extensions: Vec::new(),
        }
    }

    pub fn from_extension(name: String) -> Self {
        AllOf {
            api_version: None,
            extensions: vec![name],
        }
    }
}

#[derive(Debug, Default)]
pub struct Format {
    pub name: String,
    pub enum_value: Option<String>,
    pub block_size: Option<u32>,
    pub texels_per_block: Option<u32>,
    pub packed: Option<u32>,
    pub r: Option<Component>,
    pub g: Option<Component>,
    pub b: Option<Component>,
    pub a: Option<Component>,
}

impl Format {
    pub fn rust_name(&self) -> String {
        all_caps_to_pascal_case(self.name.trim_start_matches("VK_FORMAT_"))
    }
}

pub fn all_caps_to_pascal_case(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in name.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c.to_ascii_lowercase());
        }
    }

    result
}

#[derive(Debug)]
pub struct Component {
    pub bits: Option<u32>,
    pub numeric_format: String,
}

enum State {
    None,
    Extensions,
    Extension(Extension),
    Formats,
    Format(String),
}

pub fn extract() -> Extracted {
    let vk_xml = Cursor::new(include_bytes!("vk.xml"));

    let mut extracted = Extracted {
        extensions: Vec::new(),
        formats: Vec::new(),
    };

    let mut state = State::None;

    let parser = EventReader::new(vk_xml);
    for event in parser {
        match event {
            Ok(XmlEvent::StartElement {
                name, attributes, ..
            }) => {
                // check if the extensions section is starting
                match (name.local_name.as_str(), &mut state) {
                    ("extensions", _) => state = State::Extensions,
                    ("formats", _) => state = State::Formats,

                    ("extension", State::Extensions) => {
                        let Some(name) = find_attribute(&attributes, "name") else {
                            continue;
                        };

                        let supported = find_attribute(&attributes, "supported")
                            .is_some_and(|attr| attr.contains("vulkan"));

                        if !supported {
                            continue;
                        }

                        let is_instance = find_attribute(&attributes, "type")
                            .is_some_and(|attr| attr == "instance");

                        let dependencies = match find_attribute(&attributes, "depends") {
                            Some(depends) => parse_dependencies(depends).0,
                            None => AnyOf(Vec::new()),
                        };

                        state = State::Extension(Extension {
                            name: name.to_string(),
                            is_instance,
                            any_of: dependencies,
                        });
                    }

                    ("enum", _) => {
                        let Some(name) = find_attribute(&attributes, "name") else {
                            continue;
                        };

                        if name.starts_with("VK_FORMAT_") {
                            let Some(value) = find_attribute(&attributes, "value") else {
                                continue;
                            };

                            let format = extracted.get_format_or_insert(name);
                            format.enum_value = Some(value.to_string());
                        }
                    }

                    ("format", State::Formats) => {
                        let Some(name) = find_attribute(&attributes, "name") else {
                            continue;
                        };

                        let format = extracted.get_format_or_insert(name);

                        format.block_size = find_attribute(&attributes, "blockSize")
                            .and_then(|s| s.parse::<u32>().ok());

                        format.texels_per_block = find_attribute(&attributes, "texelsPerBlock")
                            .and_then(|s| s.parse::<u32>().ok());

                        format.packed = find_attribute(&attributes, "packed")
                            .and_then(|s| s.parse::<u32>().ok());

                        state = State::Format(name.to_string());
                    }

                    ("component", State::Format(name)) => {
                        let format = extracted.get_format_or_insert(name);

                        let Some(name) = find_attribute(&attributes, "name") else {
                            continue;
                        };

                        let bits = find_attribute(&attributes, "bits") //
                            .and_then(|s| s.parse::<u32>().ok());

                        let numeric_format = find_attribute(&attributes, "numericFormat")
                            .expect("numericFormat attribute is required");

                        let component = Component {
                            bits,
                            numeric_format: numeric_format.to_string(),
                        };

                        match name {
                            "R" => format.r = Some(component),
                            "G" => format.g = Some(component),
                            "B" => format.b = Some(component),
                            "A" => format.a = Some(component),
                            _ => {}
                        }
                    }

                    _ => {}
                }
            }

            Ok(XmlEvent::EndElement { name }) => {
                // check if the extensions section is ending
                match (name.local_name.as_str(), state) {
                    ("extensions", _) => state = State::None,
                    ("formats", _) => state = State::None,
                    ("format", State::Format(_)) => state = State::Formats,

                    ("extension", State::Extension(extension)) => {
                        extracted.extensions.push(extension);
                        state = State::Extensions;
                    }

                    (_, s) => state = s,
                }
            }

            Err(e) => panic!("Error parsing XML: {}", e),

            _ => {}
        }
    }

    extracted.extensions.sort_by_key(|ext| ext.name.clone());
    extracted
}

fn find_attribute<'a>(attributes: &'a [OwnedAttribute], name: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|attr| attr.name.local_name == name)
        .map(|attr| attr.value.as_str())
}

fn parse_dependencies(input: &str) -> (AnyOf, &str) {
    let (mut lhs, rest) = match input.strip_prefix('(') {
        Some(input) => {
            let (lhs, rest) = parse_dependencies(input);
            let rest = rest.strip_prefix(')').unwrap_or_else(|| {
                panic!("Expected closing parenthesis in dependencies: {}", input)
            });

            (lhs, rest)
        }
        None => match input.find([')', ',', '+']) {
            Some(i) => {
                let (lhs, rest) = input.split_at(i);
                let (lhs, _) = parse_dependencies(lhs);
                (lhs, rest)
            }
            None => {
                let all_of = match input {
                    "VK_VERSION_1_0" => AllOf::from_api_version(0),
                    "VK_VERSION_1_1" => AllOf::from_api_version(1),
                    "VK_VERSION_1_2" => AllOf::from_api_version(2),
                    "VK_VERSION_1_3" => AllOf::from_api_version(3),
                    "VK_VERSION_1_4" => AllOf::from_api_version(4),
                    "VK_VERSION_1_5" => AllOf::from_api_version(5),
                    "VK_VERSION_1_6" => AllOf::from_api_version(6),
                    _ => AllOf::from_extension(input.to_string()),
                };

                (AnyOf(vec![all_of]), "")
            }
        },
    };

    match rest.chars().next() {
        Some(',') => {
            let (mut rhs, rest) = parse_dependencies(&rest[1..]);
            lhs.0.append(&mut rhs.0);
            (lhs, rest)
        }

        Some('+') => {
            let (rhs, rest) = parse_dependencies(&rest[1..]);

            let mut options = Vec::new();

            for lhs in &lhs.0 {
                for rhs in &rhs.0 {
                    let api_version = match (lhs.api_version, rhs.api_version) {
                        (Some(lhs), Some(rhs)) => Some(lhs.max(rhs)),
                        (Some(version), None) | (None, Some(version)) => Some(version),
                        (None, None) => None,
                    };

                    let mut extensions = lhs.extensions.clone();
                    extensions.extend(rhs.extensions.iter().cloned());

                    options.push(AllOf {
                        api_version,
                        extensions,
                    });
                }
            }

            (AnyOf(options), rest)
        }

        _ => (lhs, rest),
    }
}
