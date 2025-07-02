use std::{collections::HashSet, ptr, sync::Arc};

use ash::{khr, vk};

use crate::{
    Device, DeviceInner, Extent2d, Format, ImageUsages, Requires, Sharing, Surface, Validated,
    ValidationError, VulkanError, is_validation_enabled,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorSpace {
    SrgbNonlinear = 0,
    DisplayP3Nonlinear = 1000104001,
    ExtendedSrgbLinear = 1000104002,
    DisplayP3Linear = 1000104003,
    DciP3Nonlinear = 1000104004,
    Bt709Linear = 1000104005,
    Bt709Nonlinear = 1000104006,
    Bt2020Linear = 1000104007,
    Hdr10St2084 = 1000104008,
    Dolbyvision = 1000104009,
    Hdr10Hlg = 1000104010,
    AdobergbLinear = 1000104011,
    AdobergbNonlinear = 1000104012,
    PassThrough = 1000104013,
    ExtendedSrgbNonlinear = 1000104014,
    DisplayNative = 1000213000,
}

impl ColorSpace {
    pub fn from_raw(raw: i32) -> Option<Self> {
        Some(match raw {
            0 => ColorSpace::SrgbNonlinear,
            1000104001 => ColorSpace::DisplayP3Nonlinear,
            1000104002 => ColorSpace::ExtendedSrgbLinear,
            1000104003 => ColorSpace::DisplayP3Linear,
            1000104004 => ColorSpace::DciP3Nonlinear,
            1000104005 => ColorSpace::Bt709Linear,
            1000104006 => ColorSpace::Bt709Nonlinear,
            1000104007 => ColorSpace::Bt2020Linear,
            1000104008 => ColorSpace::Hdr10St2084,
            1000104009 => ColorSpace::Dolbyvision,
            1000104010 => ColorSpace::Hdr10Hlg,
            1000104011 => ColorSpace::AdobergbLinear,
            1000104012 => ColorSpace::AdobergbNonlinear,
            1000104013 => ColorSpace::PassThrough,
            1000104014 => ColorSpace::ExtendedSrgbNonlinear,
            1000213000 => ColorSpace::DisplayNative,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PresentMode {
    Immediate = 0,
    Mailbox = 1,
    Fifo = 2,
    FifoRelaxed = 3,
    SharedDemandRefresh = 1000111000,
    SharedContinuousRefresh = 1000111001,
    FifoLatestReady = 1000361000,
}

impl PresentMode {
    pub fn from_raw(raw: i32) -> Option<Self> {
        Some(match raw {
            0 => PresentMode::Immediate,
            1 => PresentMode::Mailbox,
            2 => PresentMode::Fifo,
            3 => PresentMode::FifoRelaxed,
            1000111000 => PresentMode::SharedDemandRefresh,
            1000111001 => PresentMode::SharedContinuousRefresh,
            1000361000 => PresentMode::FifoLatestReady,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug)]
pub struct SwapchainDescriptor<'a> {
    pub min_image_count: u32,
    pub image_format: Format,
    pub image_color_space: ColorSpace,
    pub image_extent: Extent2d,
    pub image_array_layers: u32,
    pub image_usage: ImageUsages,
    pub image_sharing: Sharing<&'a [u32]>,
    pub present_mode: PresentMode,
    pub clipped: bool,
}

impl Default for SwapchainDescriptor<'_> {
    fn default() -> Self {
        SwapchainDescriptor {
            min_image_count: 2,
            image_format: Format::B8g8r8a8Srgb,
            image_color_space: ColorSpace::SrgbNonlinear,
            image_extent: Extent2d::ZERO,
            image_array_layers: 1,
            image_usage: ImageUsages::empty(),
            image_sharing: Sharing::Exclusive,
            present_mode: PresentMode::Fifo,
            clipped: true,
        }
    }
}

pub struct Swapchain {
    pub(crate) handle: vk::SwapchainKHR,
    pub(crate) surface: Surface,

    pub(crate) device: Arc<DeviceInner>,
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        let khr = khr::swapchain::Device::new(
            &self.surface.instance.handle,
            &self.device.handle, //
        );

        unsafe {
            khr.destroy_swapchain(self.handle, None);
        }
    }
}

impl Device {
    /// Create a swapchain for the given `surface`.
    #[track_caller]
    pub fn create_swapchain(&self, surface: Surface, desc: &SwapchainDescriptor<'_>) -> Swapchain {
        self.try_create_swapchain(surface, desc)
            .expect("Failed to create swapchain")
    }

    /// Create a swapchain for the given `surface`.
    pub fn try_create_swapchain(
        &self,
        surface: Surface,
        desc: &SwapchainDescriptor<'_>,
    ) -> Result<Swapchain, Validated<VulkanError>> {
        if is_validation_enabled() {
            self.validate_create_swapchain(&surface, desc)?;
        }

        // SAFETY: validated
        unsafe {
            self.try_create_swapchain_unchecked(surface, desc)
                .map_err(From::from)
        }
    }

    /// # Safety
    /// - `surface` must be supported by the device.
    /// - `min_image_count` must be within the bounds of `surface` capabilities.
    /// - `min_image_count` must be `1` if `present_mode` is `shared_demand_refresh` or
    ///   `shared_continuous_refresh`.
    /// - `image_format` must be supported by `surface`.
    /// - `image_color_space` must be supported by `surface`.
    /// - `image_extent` `width` and `height` must be greater than `0`.
    /// - `image_extent` must be within the bounds of `surface` capabilities.
    /// - `image_array_layers` must be `1`.
    /// - `image_array_layers` must be within the bounds of `surface` capabilities.
    /// - `image_usage` must be be contained within the bounds of `surface` capabilities.
    /// - if `image_sharing` is `Sharing::Concurrent`, there must more that `1` queue family
    ///   index and the indices must be unique and within the bounds of the device.
    /// - `present_mode` must be supported by `surface`.
    /// - `surface` must be created by the same instance as the device.
    pub unsafe fn try_create_swapchain_unchecked(
        &self,
        surface: Surface,
        desc: &SwapchainDescriptor<'_>,
    ) -> Result<Swapchain, VulkanError> {
        let create_info = vk::SwapchainCreateInfoKHR {
            flags: vk::SwapchainCreateFlagsKHR::empty(),
            surface: surface.handle,
            min_image_count: desc.min_image_count,
            image_format: vk::Format::from_raw(desc.image_format as i32),
            image_color_space: vk::ColorSpaceKHR::from_raw(desc.image_color_space as i32),
            image_extent: vk::Extent2D {
                width: desc.image_extent.width,
                height: desc.image_extent.height,
            },
            image_array_layers: desc.image_array_layers,
            image_usage: vk::ImageUsageFlags::from_raw(desc.image_usage.bits()),
            image_sharing_mode: match desc.image_sharing {
                Sharing::Exclusive => vk::SharingMode::EXCLUSIVE,
                Sharing::Concurrent(_) => vk::SharingMode::CONCURRENT,
            },
            queue_family_index_count: match desc.image_sharing {
                Sharing::Exclusive => 0,
                Sharing::Concurrent(indices) => indices.len() as u32,
            },
            p_queue_family_indices: match desc.image_sharing {
                Sharing::Exclusive => ptr::null(),
                Sharing::Concurrent(indices) => indices.as_ptr(),
            },
            pre_transform: vk::SurfaceTransformFlagsKHR::IDENTITY,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode: vk::PresentModeKHR::from_raw(desc.present_mode as i32),
            clipped: vk::Bool32::from(desc.clipped),
            old_swapchain: vk::SwapchainKHR::null(),
            ..Default::default()
        };

        let khr = khr::swapchain::Device::new(
            &self.inner.instance().handle,
            &self.inner.handle, //
        );

        let swapchain = unsafe { khr.create_swapchain(&create_info, None)? };

        Ok(Swapchain {
            handle: swapchain,
            surface,

            device: self.inner.clone(),
        })
    }

    fn validate_create_swapchain(
        &self,
        surface: &Surface,
        desc: &SwapchainDescriptor<'_>,
    ) -> Result<(), Validated<VulkanError>> {
        if !self.enabled_extensions().khr_swapchain {
            return Err(From::from(ValidationError {
                context: "device".into(),
                problem: "does not have the `khr_swapchain` extension enabled".into(),
                requires_one_of: const { &[Requires::device_extensions(&["khr_swapchain"])] },
                ..Default::default()
            }));
        }

        if !Arc::ptr_eq(self.inner.instance(), &surface.instance) {
            return Err(From::from(ValidationError {
                context: "surface".into(),
                problem: "is not created from the same instance as the device".into(),
                vuids: &["VUID-vkSwapchainCreateInfoKHR-commonparent"],
                ..Default::default()
            }));
        }

        if !self.physical().is_surface_supported_by_any(surface) {
            return Err(From::from(ValidationError {
                context: "surface".into(),
                problem: "is not supported by the device".into(),
                vuids: &["VUID-VkSwapchainCreateInfoKHR-surface-01270"],
                ..Default::default()
            }));
        }

        let formats = unsafe { self.physical().try_get_surface_formats_unchecked(surface)? };

        let capabilities = unsafe {
            self.physical()
                .try_get_surface_capabilities_unchecked(surface)?
        };

        let present_modes = unsafe {
            self.physical()
                .try_get_surface_present_modes_unchecked(surface)?
        };

        if desc.min_image_count > capabilities.max_image_count.unwrap_or(u32::MAX) {
            return Err(From::from(ValidationError {
                context: "desc.min_image_count".into(),
                problem: "is greater than the maximum image count supported by the surface".into(),
                vuids: &["VUID-VkSwapchainCreateInfoKHR-minImageCount-01272"],
                ..Default::default()
            }));
        }

        if desc.min_image_count < capabilities.min_image_count {
            return Err(From::from(ValidationError {
                context: "desc.min_image_count".into(),
                problem: "is less than the minimum image count supported by the surface".into(),
                vuids: &["VUID-VkSwapchainCreateInfoKHR-minImageCount-01273"],
                ..Default::default()
            }));
        }

        if !(formats.iter())
            .any(|f| f.format == desc.image_format && f.color_space == desc.image_color_space)
        {
            return Err(From::from(ValidationError {
                context: "desc.image_format and desc.image_color_space".into(),
                problem: "are not supported by the surface".into(),
                vuids: &["VUID-VkSwapchainCreateInfoKHR-imageFormat-01273"],
                ..Default::default()
            }));
        }

        if matches!(
            desc.present_mode,
            PresentMode::SharedDemandRefresh | PresentMode::SharedContinuousRefresh
        ) && desc.min_image_count != 1
        {
            return Err(From::from(ValidationError {
                context: "desc.min_image_count".into(),
                problem: "is not 1 when present mode is shared demand or continuous refresh".into(),
                vuids: &["VUID-VkSwapchainCreateInfoKHR-minImageCount-01383"],
                ..Default::default()
            }));
        }

        if desc.image_extent.width == 0 || desc.image_extent.height == 0 {
            return Err(From::from(ValidationError {
                context: "desc.image_extent".into(),
                problem: "width or height is zero".into(),
                vuids: &["VUID-VkSwapchainCreateInfoKHR-imageExtent-01689"],
                ..Default::default()
            }));
        }

        if desc.image_array_layers == 0
            || desc.image_array_layers > capabilities.max_image_array_layers
        {
            return Err(From::from(ValidationError {
                context: "desc.image_array_layers".into(),
                problem: "is zero or greater than allowed by the surface".into(),
                vuids: &["VUID-VkSwapchainCreateInfoKHR-imageArrayLayers-01275"],
                ..Default::default()
            }));
        }

        if !capabilities.image_usages.contains(desc.image_usage) {
            return Err(From::from(ValidationError {
                context: "desc.image_usage".into(),
                problem: "is not supported by the surface".into(),
                vuids: &["VUID-VkSwapchainCreateInfoKHR-presentMode-01427"],
                ..Default::default()
            }));
        }

        if desc.image_usage.is_empty() {
            return Err(From::from(ValidationError {
                context: "desc.image_usage".into(),
                problem: "is empty".into(),
                vuids: &["VUID-VkSwapchainCreateInfoKHR-imageUsage-requiredbitmask"],
                ..Default::default()
            }));
        }

        if let Sharing::Concurrent(indices) = desc.image_sharing {
            if indices.len() < 2 {
                return Err(From::from(ValidationError {
                    context: "desc.image_sharing".into(),
                    problem: "is concurrent with less than 2 queue family indices".into(),
                    vuids: &["VUID-VkSwapchainCreateInfoKHR-imageSharingMode-01276"],
                    ..Default::default()
                }));
            }

            if indices.len() != indices.iter().collect::<HashSet<_>>().len() {
                return Err(From::from(ValidationError {
                    context: "desc.image_sharing".into(),
                    problem: "contains duplicate queue family indices".into(),
                    vuids: &["VUID-VkSwapchainCreateInfoKHR-imageSharingMode-01428"],
                    ..Default::default()
                }));
            }

            let queue_family_count = self.physical().queue_families().len() as u32;
            for &index in indices {
                if unsafe {
                    !self
                        .physical()
                        .try_is_surface_supported_unchecked(surface, index)
                        .unwrap_or(false)
                } {
                    return Err(From::from(ValidationError {
                        context: "desc.image_sharing".into(),
                        problem: "queue family index does not support the surface".into(),
                        vuids: &["VUID-VkSwapchainCreateInfoKHR-surface-01270"],
                        ..Default::default()
                    }));
                }

                if index >= queue_family_count {
                    return Err(From::from(ValidationError {
                        context: "desc.image_sharing".into(),
                        problem: "contains queue family indices out of bounds".into(),
                        vuids: &["self.physical().queue_families().len() as u32"],
                        ..Default::default()
                    }));
                }
            }
        }

        if !present_modes.contains(&desc.present_mode) {
            return Err(From::from(ValidationError {
                context: "desc.present_mode".into(),
                problem: "is not supported by the surface".into(),
                vuids: &["VUID-VkSwapchainCreateInfoKHR-presentMode-01281"],
                ..Default::default()
            }));
        }

        Ok(())
    }
}
