use std::{fmt, num::NonZero, sync::Arc};

use ash::{khr, vk};

use crate::{
    ColorSpace, Extent2d, Format, ImageUsages, Instance, PhysicalDevice, PresentMode, RawInstance,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceFormat {
    pub format: Format,
    pub color_space: ColorSpace,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceCapabilities {
    pub min_image_count: u32,
    pub max_image_count: Option<NonZero<u32>>,
    pub current_extent: Option<Extent2d>,
    pub min_image_extent: Extent2d,
    pub max_image_extent: Extent2d,
    pub max_image_array_layers: u32,
    pub supported_usages: ImageUsages,
}

pub struct Surface {
    pub(crate) raw: Arc<RawSurface>,
}

impl Surface {
    pub fn raw_surface(&self) -> vk::SurfaceKHR {
        self.raw.surface
    }
}

impl PhysicalDevice {
    #[track_caller]
    pub fn is_surface_supported(&self, surface: &Surface, queue_family_index: u32) -> bool {
        self.try_is_surface_supported(surface, queue_family_index)
            .expect("Failed to get surface support")
    }

    pub fn try_is_surface_supported(
        &self,
        surface: &Surface,
        queue_family_index: u32,
    ) -> Result<bool, vk::Result> {
        let supported = unsafe {
            (surface.raw.instance()).get_physical_device_surface_support(
                self.raw_physical_device(),
                queue_family_index,
                surface.raw.surface,
            )?
        };

        Ok(supported)
    }

    #[track_caller]
    pub fn get_surface_formats(&self, surface: &Surface) -> Vec<SurfaceFormat> {
        self.try_get_surface_formats(surface)
            .expect("Failed to get surface formats")
    }

    pub fn try_get_surface_formats(
        &self,
        surface: &Surface,
    ) -> Result<Vec<SurfaceFormat>, vk::Result> {
        let formats = unsafe {
            (surface.raw.instance()).get_physical_device_surface_formats(
                self.raw_physical_device(), // PhysicalDevice
                surface.raw.surface,
            )?
        };

        Ok(formats
            .into_iter()
            .map(|f| SurfaceFormat {
                format: Format::from_raw(f.format.as_raw()).unwrap(),
                color_space: ColorSpace::from_raw(f.color_space.as_raw()).unwrap(),
            })
            .collect())
    }

    #[track_caller]
    pub fn get_surface_present_modes(&self, surface: &Surface) -> Vec<PresentMode> {
        self.try_get_surface_present_modes(surface)
            .expect("Failed to get surface present modes")
    }

    pub fn try_get_surface_present_modes(
        &self,
        surface: &Surface,
    ) -> Result<Vec<PresentMode>, vk::Result> {
        let modes = unsafe {
            (surface.raw.instance()).get_physical_device_surface_present_modes(
                self.raw_physical_device(), // PhysicalDevice
                surface.raw.surface,
            )?
        };

        let modes = modes
            .into_iter()
            .map(|m| PresentMode::from_raw(m.as_raw()).unwrap())
            .collect();

        Ok(modes)
    }

    #[track_caller]
    pub fn get_surface_capabilities(&self, surface: &Surface) -> SurfaceCapabilities {
        self.try_get_surface_capabilities(surface)
            .expect("Failed to get surface capabilities")
    }

    pub fn try_get_surface_capabilities(
        &self,
        surface: &Surface,
    ) -> Result<SurfaceCapabilities, vk::Result> {
        let capabilities = unsafe {
            (surface.raw.instance()).get_physical_device_surface_capabilities(
                self.raw_physical_device(), // PhysicalDevice
                surface.raw.surface,
            )?
        };

        let has_current_extent = capabilities.current_extent.width != u32::MAX
            && capabilities.current_extent.height != u32::MAX;

        let current_extent = match has_current_extent {
            true => Some(Extent2d {
                width: capabilities.current_extent.width,
                height: capabilities.current_extent.height,
            }),
            false => None,
        };

        Ok(SurfaceCapabilities {
            min_image_count: capabilities.min_image_count,
            max_image_count: NonZero::new(capabilities.max_image_count),
            current_extent,
            min_image_extent: Extent2d {
                width: capabilities.min_image_extent.width,
                height: capabilities.min_image_extent.height,
            },
            max_image_extent: Extent2d {
                width: capabilities.max_image_extent.width,
                height: capabilities.max_image_extent.height,
            },
            max_image_array_layers: capabilities.max_image_array_layers,
            supported_usages: ImageUsages::from_bits_truncate(
                capabilities.supported_usage_flags.as_raw(),
            ),
        })
    }
}

impl fmt::Debug for Surface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Surface").finish()
    }
}

pub(crate) struct RawSurface {
    #[allow(dead_code)]
    pub(crate) instance: Arc<RawInstance>,
    pub(crate) surface: vk::SurfaceKHR,
}

impl RawSurface {
    fn instance(&self) -> khr::surface::Instance {
        khr::surface::Instance::new(Instance::entry(), &self.instance.instance)
    }
}

impl Drop for RawSurface {
    fn drop(&mut self) {
        unsafe {
            tracing::trace!(handle = ?self.surface, "Destroying surface");
            self.instance().destroy_surface(self.surface, None);
        }
    }
}
