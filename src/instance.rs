use std::{cmp::Ordering, ffi, fmt, ptr, sync::Arc};

use ash::vk;

use crate::{Requires, Validated, ValidationError, VulkanError, is_validation_enabled};

include!(concat!(env!("OUT_DIR"), "/instance_extensions.rs"));

pub struct Entry {
    pub(crate) handle: ash::Entry,
}

impl Entry {
    pub fn handle(&self) -> &ash::Entry {
        &self.handle
    }

    pub fn linked() -> Self {
        Self {
            handle: ash::Entry::linked(),
        }
    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entry").finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub const V1_0: Self = Self::new(1, 0, 0);
    pub const V1_1: Self = Self::new(1, 1, 0);
    pub const V1_2: Self = Self::new(1, 2, 0);
    pub const V1_3: Self = Self::new(1, 3, 0);
    pub const V1_4: Self = Self::new(1, 4, 0);
    pub const V1_5: Self = Self::new(1, 5, 0);
    pub const V1_6: Self = Self::new(1, 6, 0);

    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub const fn from_vk_version(version: u32) -> Self {
        let major = (version >> 22) & 0x3FF;
        let minor = (version >> 12) & 0x3FF;
        let patch = version & 0xFFF;

        Self {
            major,
            minor,
            patch,
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major.cmp(&other.major))
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Clone, Debug)]
pub struct InstanceDescriptor<'a> {
    pub app_name: Option<&'a str>,
    pub app_version: Version,
    pub engine_name: Option<&'a str>,
    pub engine_version: Version,
    pub max_api_version: Option<Version>,
    pub enabled_layers: &'a [&'a str],
    pub enabled_extensions: InstanceExtensions,
}

impl Default for InstanceDescriptor<'_> {
    fn default() -> Self {
        Self {
            app_name: None,
            app_version: Version::V1_0,
            engine_name: None,
            engine_version: Version::V1_0,
            max_api_version: Some(Version::V1_0),
            enabled_layers: &[],
            enabled_extensions: InstanceExtensions::default(),
        }
    }
}

pub struct Instance {
    pub(crate) inner: Arc<InstanceInner>,
}

impl Instance {
    pub fn handle(&self) -> ash::Instance {
        self.inner.handle()
    }

    pub fn api_version(&self) -> Version {
        self.inner.api_version
    }

    pub fn enabled_extensions(&self) -> &InstanceExtensions {
        &self.inner.enabled_extensions
    }
}

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Instance")
            .field("handle", &self.handle().handle())
            .finish()
    }
}

pub(crate) struct InstanceInner {
    pub(crate) handle: ash::Instance,

    pub(crate) entry: ash::Entry,

    pub(crate) api_version: Version,
    pub(crate) enabled_extensions: InstanceExtensions,
}

impl InstanceInner {
    pub fn handle(&self) -> ash::Instance {
        self.handle.clone()
    }
}

impl Drop for InstanceInner {
    fn drop(&mut self) {
        unsafe {
            tracing::trace!(
                handle = ?self.handle.handle(),
                "Destroying Vulkan instance",
            );

            self.handle.destroy_instance(None);
        }
    }
}

impl Entry {
    pub fn supported_instance_extensions(&self) -> Result<InstanceExtensions, VulkanError> {
        let properties = unsafe { self.handle.enumerate_instance_extension_properties(None)? };

        let names = properties.iter().map(|prop| unsafe {
            ffi::CStr::from_ptr(prop.extension_name.as_ptr())
                .to_str()
                .expect("Invalid UTF-8 in extension name")
        });

        Ok(InstanceExtensions::from_names(names))
    }

    #[track_caller]
    pub fn create_instance(&self, desc: &InstanceDescriptor<'_>) -> Instance {
        self.try_create_instance(desc)
            .expect("Failed to create Vulkan instance")
    }

    pub fn try_create_instance(
        &self,
        desc: &InstanceDescriptor<'_>,
    ) -> Result<Instance, Validated<VulkanError>> {
        if is_validation_enabled() {
            let api_version = self.select_api_version(desc.max_api_version)?;
            self.validate_create_instance(desc, api_version)?;
        }

        unsafe { self.try_create_instance_unchecked(desc).map_err(From::from) }
    }

    fn validate_create_instance(
        &self,
        desc: &InstanceDescriptor<'_>,
        api_version: Version,
    ) -> Result<(), ValidationError> {
        desc.enabled_extensions.validate(api_version)?;

        Ok(())
    }

    /// # Safety
    /// - All required extensions for each enabled extension must also be enabled.
    pub unsafe fn try_create_instance_unchecked(
        &self,
        desc: &InstanceDescriptor<'_>,
    ) -> Result<Instance, VulkanError> {
        let api_version = self.select_api_version(desc.max_api_version)?;

        tracing::debug!(
            api_version = %api_version,
            "Creating instance with API version",
        );

        let app_name = desc.app_name.map(ffi::CString::new).and_then(Result::ok);
        let engine_name = desc.engine_name.map(ffi::CString::new).and_then(Result::ok);

        let app_info = vk::ApplicationInfo {
            p_application_name: app_name.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
            p_engine_name: engine_name.as_ref().map_or(ptr::null(), |s| s.as_ptr()),
            application_version: vk::make_api_version(
                0,
                desc.app_version.major,
                desc.app_version.minor,
                desc.app_version.patch,
            ),
            engine_version: vk::make_api_version(
                0,
                desc.engine_version.major,
                desc.engine_version.minor,
                desc.engine_version.patch,
            ),
            api_version: vk::make_api_version(
                0,
                api_version.major,
                api_version.minor,
                api_version.patch,
            ),
            ..Default::default()
        };

        let enabled_layers: Vec<_> = desc
            .enabled_layers
            .iter()
            .copied()
            .map(ffi::CString::new)
            .filter_map(Result::ok)
            .collect();

        let enabled_layers_ptrs: Vec<_> = enabled_layers.iter().map(|s| s.as_ptr()).collect();

        let enabled_extensions = desc.enabled_extensions.extension_names();

        let create_info = vk::InstanceCreateInfo {
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            enabled_layer_count: enabled_layers_ptrs.len() as u32,
            pp_enabled_layer_names: enabled_layers_ptrs.as_ptr(),
            enabled_extension_count: enabled_extensions.len() as u32,
            pp_enabled_extension_names: enabled_extensions.as_ptr(),
            ..Default::default()
        };

        let handle = unsafe { self.handle.create_instance(&create_info, None)? };

        Ok(Instance {
            inner: Arc::new(InstanceInner {
                handle,

                entry: self.handle.clone(),

                api_version,
                enabled_extensions: desc.enabled_extensions.clone(),
            }),
        })
    }

    pub fn select_api_version(
        &self,
        max_api_version: Option<Version>,
    ) -> Result<Version, VulkanError> {
        let version = unsafe { self.handle.try_enumerate_instance_version()? };
        let version = version.unwrap_or(vk::API_VERSION_1_0);
        let version = Version::from_vk_version(version);

        let mut major = version.major;
        let mut minor = version.minor;
        let mut patch = version.patch;

        if let Some(max_api_version) = max_api_version {
            major = major.min(max_api_version.major);
            minor = minor.min(max_api_version.minor);
            patch = patch.min(max_api_version.patch);
        }

        Ok(Version {
            major,
            minor,
            patch,
        })
    }
}
