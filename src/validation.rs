use std::{borrow::Cow, error::Error, fmt};

use ash::vk;

use crate::{Version, VulkanError};

pub(crate) fn is_validation_enabled() -> bool {
    !cfg!(feature = "unsafe-disable-validation")
}

#[derive(Default)]
pub struct ValidationError {
    pub context: Cow<'static, str>,
    pub problem: Cow<'static, str>,
    pub vuids: &'static [&'static str],
    pub requires_one_of: &'static [Requires],
}

#[derive(Default)]
pub struct Requires {
    pub api_version: Option<Version>,
    pub instance_extensions: &'static [&'static str],
    pub device_extensions: &'static [&'static str],
}

impl Requires {
    pub const fn api_version(api_version: Version) -> Self {
        Self {
            api_version: Some(api_version),
            instance_extensions: &[],
            device_extensions: &[],
        }
    }

    pub const fn device_extensions(device_extensions: &'static [&'static str]) -> Self {
        Self {
            api_version: None,
            instance_extensions: &[],
            device_extensions,
        }
    }

    pub const fn instance_extensions(instance_extensions: &'static [&'static str]) -> Self {
        Self {
            api_version: None,
            instance_extensions,
            device_extensions: &[],
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} (VUIDs: {})",
            self.context,
            self.problem,
            self.vuids.join(", ")
        )
    }
}

impl fmt::Debug for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.context.is_empty() {
            write!(f, "{}", self.problem)?;
        } else {
            write!(f, "{}: {}", self.context, self.problem)?;
        }

        if !self.requires_one_of.is_empty() {
            if self.context.is_empty() && self.problem.is_empty() {
                writeln!(f, "requires one of:")?;
            } else {
                writeln!(f, "\n\nrequires one of:")?;
            }

            for requires in self.requires_one_of {
                writeln!(f, "  all of the following:")?;

                if let Some(api_version) = requires.api_version {
                    writeln!(f, "    - api version: {}", api_version)?;
                }

                for extensions in requires.instance_extensions {
                    writeln!(f, "    - instance extension: {}", extensions)?;
                }

                for extensions in requires.device_extensions {
                    writeln!(f, "    - device extension: {}", extensions)?;
                }
            }
        }

        if !self.vuids.is_empty() {
            writeln!(f, "\nVulkan VUIDs:")?;

            for vuid in self.vuids {
                writeln!(f, "  - {}", vuid)?;
            }
        }

        Ok(())
    }
}

impl Error for ValidationError {}

pub enum Validated<E> {
    Error(E),
    Validation(ValidationError),
}

impl<E: fmt::Debug> fmt::Debug for Validated<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Validated::Error(err) => write!(f, "a non-validation error occurred: {err:?}"),
            Validated::Validation(err) => {
                write!(f, "a validation error occurred\n\nCaused by:\n  {err:?}")
            }
        }
    }
}

impl<E: fmt::Display> fmt::Display for Validated<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Validated::Error(err) => write!(f, "{}", err),
            Validated::Validation(err) => write!(f, "{}", err),
        }
    }
}

impl<E: Error> Error for Validated<E> {}

impl From<vk::Result> for Validated<VulkanError> {
    fn from(result: vk::Result) -> Self {
        Validated::Error(VulkanError::from(result))
    }
}

impl From<VulkanError> for Validated<VulkanError> {
    fn from(err: VulkanError) -> Self {
        Validated::Error(err)
    }
}

impl<E> From<ValidationError> for Validated<E> {
    fn from(err: ValidationError) -> Self {
        Validated::Validation(err)
    }
}
