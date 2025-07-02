use std::{fmt, sync::Arc};

use ash::{khr, vk};

use crate::{
    ColorSpace, Extent2d, Format, ImageUsages, InstanceInner, PhysicalDevice, PresentMode,
    Validated, ValidationError, VulkanError, is_validation_enabled,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SurfaceFormat {
    pub format: Format,
    pub color_space: ColorSpace,
}

#[derive(Clone, Debug)]
pub struct SurfaceCapabilities {
    pub min_image_count: u32,
    pub max_image_count: Option<u32>,
    pub current_extent: Option<Extent2d>,
    pub min_image_extent: Extent2d,
    pub max_image_extent: Extent2d,
    pub max_image_array_layers: u32,
    pub image_usages: ImageUsages,
}

pub struct Surface {
    pub(crate) handle: vk::SurfaceKHR,

    pub(crate) instance: Arc<InstanceInner>,
}

impl fmt::Debug for Surface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Surface")
            .field("handle", &self.handle)
            .finish()
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            tracing::trace!(
                handle = ?self.handle,
                "Destroying Vulkan surface",
            );

            let khr = khr::surface::Instance::new(&self.instance.entry, &self.instance.handle);
            khr.destroy_surface(self.handle, None);
        }
    }
}

impl PhysicalDevice {
    /// Get the surface formats supported by `surface`.
    #[track_caller]
    pub fn get_surface_formats(&self, surface: &Surface) -> Vec<SurfaceFormat> {
        self.try_get_surface_formats(surface)
            .expect("Failed to get surface formats")
    }

    /// Get the surface formats supported by `surface`.
    pub fn try_get_surface_formats(
        &self,
        surface: &Surface,
    ) -> Result<Vec<SurfaceFormat>, Validated<VulkanError>> {
        if is_validation_enabled() {
            self.validate_get_surface_formats(surface)?;
        }

        // SAFETY: validated
        unsafe {
            self.try_get_surface_formats_unchecked(surface)
                .map_err(From::from)
        }
    }

    /// # Safety
    /// - `surface` must be supported by the physical device.
    /// - `surface` must be created from the same instance as `self`.
    pub unsafe fn try_get_surface_formats_unchecked(
        &self,
        surface: &Surface,
    ) -> Result<Vec<SurfaceFormat>, VulkanError> {
        let khr = khr::surface::Instance::new(
            &self.inner.instance.entry, //
            &self.inner.instance.handle,
        );

        let formats = unsafe {
            khr.get_physical_device_surface_formats(self.inner.handle, surface.handle)?
                .into_iter()
                .filter_map(|format| {
                    Some(SurfaceFormat {
                        format: Format::from_raw(format.format.as_raw())?,
                        color_space: ColorSpace::from_raw(format.color_space.as_raw())?,
                    })
                })
                .collect::<Vec<_>>()
        };

        Ok(formats)
    }

    fn validate_get_surface_formats(&self, surface: &Surface) -> Result<(), ValidationError> {
        if !Arc::ptr_eq(&self.inner.instance, &surface.instance) {
            return Err(ValidationError {
                context: "surface".into(),
                problem: "is not created from the same instance as the physical device".into(),
                vuids: &["VUID-vkGetPhysicalDeviceSurfaceFormatsKHR-commonparent"],
                ..Default::default()
            });
        }

        if !self.is_surface_supported_by_any(surface) {
            return Err(ValidationError {
                context: "surface".into(),
                problem: "not supported by the physical device".into(),
                vuids: &["VUID-vkGetPhysicalDeviceSurfaceFormatsKHR-surface-06525"],
                ..Default::default()
            });
        }

        Ok(())
    }

    /// Get the present modes supported by the `surface`.
    #[track_caller]
    pub fn get_surface_present_modes(&self, surface: &Surface) -> Vec<PresentMode> {
        self.try_get_surface_present_modes(surface)
            .expect("Failed to get surface present modes")
    }

    /// Get the present modes supported by the `surface`.
    pub fn try_get_surface_present_modes(
        &self,
        surface: &Surface,
    ) -> Result<Vec<PresentMode>, Validated<VulkanError>> {
        if is_validation_enabled() {
            self.validate_get_surface_present_modes(surface)?;
        }

        // SAFETY: validated
        unsafe {
            self.try_get_surface_present_modes_unchecked(surface)
                .map_err(From::from)
        }
    }

    /// # Safety
    /// - `surface` must be supported by the physical device.
    /// - `surface` must be created from the same instance as `self`.
    pub unsafe fn try_get_surface_present_modes_unchecked(
        &self,
        surface: &Surface,
    ) -> Result<Vec<PresentMode>, VulkanError> {
        let khr = khr::surface::Instance::new(
            &self.inner.instance.entry, //
            &self.inner.instance.handle,
        );

        let modes = unsafe {
            khr.get_physical_device_surface_present_modes(self.inner.handle, surface.handle)?
                .into_iter()
                .map(|mode| PresentMode::from_raw(mode.as_raw()).unwrap())
                .collect::<Vec<_>>()
        };

        Ok(modes)
    }

    fn validate_get_surface_present_modes(&self, surface: &Surface) -> Result<(), ValidationError> {
        if !Arc::ptr_eq(&self.inner.instance, &surface.instance) {
            return Err(ValidationError {
                context: "surface".into(),
                problem: "is not created from the same instance as the physical device".into(),
                vuids: &["VUID-vkGetPhysicalDeviceSurfacePresentModesKHR-commonparent"],
                ..Default::default()
            });
        }

        if !self.is_surface_supported_by_any(surface) {
            return Err(ValidationError {
                context: "surface".into(),
                problem: "not supported by the physical device".into(),
                vuids: &["VUID-vkGetPhysicalDeviceSurfacePresentModesKHR-surface-06525"],
                ..Default::default()
            });
        }

        Ok(())
    }

    /// Get the surface capabilities for the `surface`.
    #[track_caller]
    pub fn get_surface_capabilities(&self, surface: &Surface) -> SurfaceCapabilities {
        self.try_get_surface_capabilities(surface)
            .expect("Failed to get surface capabilities")
    }

    /// Get the capabilities of the `surface`.
    pub fn try_get_surface_capabilities(
        &self,
        surface: &Surface,
    ) -> Result<SurfaceCapabilities, Validated<VulkanError>> {
        if is_validation_enabled() {
            self.validate_get_surface_capabilities(surface)?;
        }

        // SAFETY: validated
        unsafe {
            self.try_get_surface_capabilities_unchecked(surface)
                .map_err(From::from)
        }
    }

    /// # Safety
    /// - `surface` must be supported by the physical device.
    /// - `surface` must be created from the same instance as `self`.
    pub unsafe fn try_get_surface_capabilities_unchecked(
        &self,
        surface: &Surface,
    ) -> Result<SurfaceCapabilities, VulkanError> {
        let capabilities = unsafe {
            let khr = khr::surface::Instance::new(
                &self.inner.instance.entry,
                &self.inner.instance.handle,
            );

            khr.get_physical_device_surface_capabilities(self.inner.handle, surface.handle)?
        };

        let has_max_image_count = capabilities.max_image_count > 0;

        let has_current_extent = capabilities.current_extent.width != u32::MAX
            && capabilities.current_extent.height != u32::MAX;

        let current_extent = Extent2d {
            width: capabilities.current_extent.width,
            height: capabilities.current_extent.height,
        };

        Ok(SurfaceCapabilities {
            min_image_count: capabilities.min_image_count,
            max_image_count: has_max_image_count.then_some(capabilities.max_image_count),
            current_extent: has_current_extent.then_some(current_extent),
            min_image_extent: Extent2d {
                width: capabilities.min_image_extent.width,
                height: capabilities.min_image_extent.height,
            },
            max_image_extent: Extent2d {
                width: capabilities.max_image_extent.width,
                height: capabilities.max_image_extent.height,
            },
            max_image_array_layers: capabilities.max_image_array_layers,
            image_usages: ImageUsages::from_bits_truncate(
                capabilities.supported_usage_flags.as_raw(),
            ),
        })
    }

    fn validate_get_surface_capabilities(&self, surface: &Surface) -> Result<(), ValidationError> {
        if !Arc::ptr_eq(&self.inner.instance, &surface.instance) {
            return Err(ValidationError {
                context: "surface".into(),
                problem: "is not created from the same instance as the physical device".into(),
                vuids: &["VUID-vkGetPhysicalDeviceSurfaceCapabilitiesKHR-commonparent"],
                ..Default::default()
            });
        }

        if !self.is_surface_supported_by_any(surface) {
            return Err(ValidationError {
                context: "surface".into(),
                problem: "not supported by the physical device".into(),
                vuids: &["VUID-vkGetPhysicalDeviceSurfaceCapabilitiesKHR-surface-06211"],
                ..Default::default()
            });
        }

        Ok(())
    }

    /// Check if the given queue family supports `surface`.
    #[track_caller]
    pub fn is_surface_supported(&self, surface: &Surface, queue_family_index: u32) -> bool {
        self.try_is_surface_supported(surface, queue_family_index)
            .expect("Failed to check surface support")
    }

    /// Check if the given queue family supports `surface`.
    pub fn try_is_surface_supported(
        &self,
        surface: &Surface,
        queue_family_index: u32,
    ) -> Result<bool, Validated<VulkanError>> {
        if is_validation_enabled() {
            self.validate_is_surface_supported(surface, queue_family_index)?;
        }

        // SAFETY: validated
        unsafe {
            self.try_is_surface_supported_unchecked(surface, queue_family_index)
                .map_err(From::from)
        }
    }

    /// # Safety
    /// - `queue_family_index < self.queue_families().len()` must be `true`.
    /// - `surface` must be created from the same instance as `self`.
    pub unsafe fn try_is_surface_supported_unchecked(
        &self,
        surface: &Surface,
        queue_family_index: u32,
    ) -> Result<bool, VulkanError> {
        unsafe {
            let khr = khr::surface::Instance::new(
                &self.inner.instance.entry,
                &self.inner.instance.handle,
            );

            khr.get_physical_device_surface_support(
                self.inner.handle,
                queue_family_index,
                surface.handle,
            )
            .map_err(From::from)
        }
    }

    pub(crate) fn is_surface_supported_by_any(&self, surface: &Surface) -> bool {
        if !Arc::ptr_eq(&self.inner.instance, &surface.instance) {
            return false;
        }

        (0..self.queue_families().len() as u32).any(|i| {
            unsafe { self.try_is_surface_supported_unchecked(surface, i) }.unwrap_or(false)
        })
    }

    fn validate_is_surface_supported(
        &self,
        surface: &Surface,
        queue_family_index: u32,
    ) -> Result<(), ValidationError> {
        if queue_family_index as usize >= self.queue_families().len() {
            return Err(ValidationError {
                context: "queue_family_index".into(),
                problem: "is out of bounds.".into(),
                vuids: &["VUID-vkGetPhysicalDeviceSurfaceSupportKHR-queueFamilyIndex-01269"],
                ..Default::default()
            });
        }

        if !Arc::ptr_eq(&self.inner.instance, &surface.instance) {
            return Err(ValidationError {
                context: "surface".into(),
                problem: "is not created from the same instance as the physical device".into(),
                vuids: &["VUID-vkGetPhysicalDeviceSurfaceSupportKHR-commonparent"],
                ..Default::default()
            });
        }

        Ok(())
    }
}
