use std::{
    ffi, fmt, ptr,
    sync::{Arc, OnceLock},
};

use ash::vk;

use crate::InstanceFlags;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceExtension([ffi::c_char; vk::MAX_EXTENSION_NAME_SIZE]);

impl InstanceExtension {
    pub const fn new(extension: &ffi::CStr) -> Self {
        unsafe { Self::from_raw(extension.as_ptr()) }
    }

    /// # Safety
    /// - `extension` must be a valid C string pointer.
    pub const unsafe fn from_raw(extension: *const ffi::c_char) -> Self {
        unsafe {
            let len = ffi::CStr::from_ptr(extension).count_bytes() + 1;
            assert!(len <= vk::MAX_EXTENSION_NAME_SIZE);

            let mut array = [0; vk::MAX_EXTENSION_NAME_SIZE];
            ptr::copy_nonoverlapping(extension, array.as_mut_ptr(), len);

            Self(array)
        }
    }

    pub fn as_c_str(&self) -> &ffi::CStr {
        unsafe { ffi::CStr::from_ptr(self.0.as_ptr()) }
    }

    pub fn as_ptr(&self) -> *const ffi::c_char {
        self.0.as_ptr()
    }
}

impl fmt::Debug for InstanceExtension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("InstanceExtension")
            .field(&self.as_c_str().to_string_lossy())
            .finish()
    }
}

impl fmt::Display for InstanceExtension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_c_str().to_string_lossy())
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceLayer([ffi::c_char; vk::MAX_EXTENSION_NAME_SIZE]);

impl InstanceLayer {
    pub const KHRONOS_VALIDATION: Self = Self::new(c"VK_LAYER_KHRONOS_validation");

    pub const fn new(extension: &ffi::CStr) -> Self {
        unsafe { Self::from_raw(extension.as_ptr()) }
    }

    /// # Safety
    /// - `layer` must be a valid C string pointer.
    pub const unsafe fn from_raw(extension: *const ffi::c_char) -> Self {
        unsafe {
            let len = ffi::CStr::from_ptr(extension).count_bytes() + 1;
            assert!(len <= vk::MAX_EXTENSION_NAME_SIZE);

            let mut array = [0; vk::MAX_EXTENSION_NAME_SIZE];
            ptr::copy_nonoverlapping(extension, array.as_mut_ptr(), len);

            Self(array)
        }
    }

    pub fn as_c_str(&self) -> &ffi::CStr {
        unsafe { ffi::CStr::from_ptr(self.0.as_ptr()) }
    }

    pub fn as_ptr(&self) -> *const ffi::c_char {
        self.0.as_ptr()
    }
}

impl fmt::Debug for InstanceLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("InstanceLayer")
            .field(&self.as_c_str().to_string_lossy())
            .finish()
    }
}

impl fmt::Display for InstanceLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_c_str().to_string_lossy())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InstanceExtensionProperties {
    pub name: InstanceExtension,
    pub version: Version,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Version(u32);

impl Version {
    pub const V1_0: Self = Self::new(0, 1, 0, 0);
    pub const V1_1: Self = Self::new(0, 1, 1, 0);
    pub const V1_2: Self = Self::new(0, 1, 2, 0);
    pub const V1_3: Self = Self::new(0, 1, 3, 0);

    pub const fn new(variant: u32, major: u32, minor: u32, patch: u32) -> Self {
        assert!(variant <= 0x7, "Variant must be in range [0, 7]");
        assert!(major <= 0x7F, "Major version must be in range [0, 127]");
        assert!(minor <= 0x7F, "Minor version must be in range [0, 127]");
        assert!(patch <= 0xFFF, "Patch version must be in range [0, 4095]");

        Self((variant << 29) | (major << 22) | (minor << 12) | patch)
    }

    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn as_raw(&self) -> u32 {
        self.0
    }

    pub fn variant(&self) -> u32 {
        (self.0 >> 29) & 0x7
    }

    pub fn major(&self) -> u32 {
        (self.0 >> 22) & 0x7F
    }

    pub fn minor(&self) -> u32 {
        (self.0 >> 12) & 0x7F
    }

    pub fn patch(&self) -> u32 {
        self.0 & 0xFFF
    }
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Version")
            .field("variant", &self.variant())
            .field("major", &self.major())
            .field("minor", &self.minor())
            .field("patch", &self.patch())
            .finish()
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.variant(),
            self.major(),
            self.minor(),
            self.patch()
        )
    }
}

#[derive(Clone, Debug)]
pub struct InstanceDescriptor<'a> {
    pub application_name: &'a str,
    pub application_version: Version,
    pub engine_name: &'a str,
    pub engine_version: Version,
    pub api_version: Version,
    pub flags: InstanceFlags,
    pub extensions: &'a [InstanceExtension],
    pub layers: &'a [InstanceLayer],
}

impl Default for InstanceDescriptor<'_> {
    fn default() -> Self {
        Self {
            application_name: "",
            application_version: Version::V1_0,
            engine_name: "",
            engine_version: Version::V1_0,
            api_version: Version::V1_0,
            flags: InstanceFlags::empty(),
            extensions: &[],
            layers: &[],
        }
    }
}

pub struct Instance {
    pub(crate) raw: Arc<RawInstance>,
}

impl Instance {
    pub fn entry() -> &'static ash::Entry {
        static ENTRY: OnceLock<ash::Entry> = OnceLock::new();
        ENTRY.get_or_init(ash::Entry::linked)
    }

    #[track_caller]
    pub fn new(desc: &InstanceDescriptor<'_>) -> Self {
        Self::try_new(desc).expect("Failed to create Vulkan instance")
    }

    pub fn try_new(desc: &InstanceDescriptor<'_>) -> Result<Self, vk::Result> {
        let application_name = ffi::CString::new(desc.application_name)
            .expect("Application name must be a valid C string");

        let engine_name = ffi::CString::new(desc.engine_name) // <- line
            .expect("Engine name must be a valid C string");

        let app_info = vk::ApplicationInfo {
            p_application_name: application_name.as_ptr(),
            application_version: desc.application_version.as_raw(),
            p_engine_name: engine_name.as_ptr(),
            engine_version: desc.engine_version.as_raw(),
            api_version: desc.api_version.as_raw(),
            ..Default::default()
        };

        let layers: Vec<_> = desc.layers.iter().map(|layer| layer.as_ptr()).collect();
        let extensions: Vec<_> = desc.extensions.iter().map(|ext| ext.as_ptr()).collect();

        let create_info = vk::InstanceCreateInfo {
            flags: vk::InstanceCreateFlags::from_raw(desc.flags.bits()),
            p_application_info: &app_info,
            enabled_layer_count: layers.len() as u32,
            pp_enabled_layer_names: layers.as_ptr(),
            enabled_extension_count: extensions.len() as u32,
            pp_enabled_extension_names: extensions.as_ptr(),
            ..Default::default()
        };

        let instance = unsafe { Self::entry().create_instance(&create_info, None)? };

        let raw_instance = RawInstance { instance };

        Ok(Self {
            raw: Arc::new(raw_instance),
        })
    }

    pub fn raw_instance(&self) -> &ash::Instance {
        &self.raw.instance
    }

    #[track_caller]
    pub fn extension_properties() -> Vec<InstanceExtensionProperties> {
        Self::try_extension_properties().expect("Failed to enumerate instance extension properties")
    }

    pub fn try_extension_properties() -> Result<Vec<InstanceExtensionProperties>, vk::Result> {
        unsafe {
            Self::entry()
                .enumerate_instance_extension_properties(None)?
                .into_iter()
                .map(|ext| {
                    let name = InstanceExtension::from_raw(ext.extension_name.as_ptr());
                    let version = Version::from_raw(ext.spec_version);
                    Ok(InstanceExtensionProperties { name, version })
                })
                .collect()
        }
    }

    pub fn is_extension_supported(extension: &InstanceExtension) -> bool {
        Self::extension_properties()
            .iter()
            .any(|ext| ext.name == *extension)
    }
}

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Instance").finish()
    }
}

pub(crate) struct RawInstance {
    pub(crate) instance: ash::Instance,
}

impl Drop for RawInstance {
    fn drop(&mut self) {
        unsafe {
            tracing::trace!(handle = ?self.instance.handle(), "Destroying instance");
            self.instance.destroy_instance(None);
        }
    }
}
