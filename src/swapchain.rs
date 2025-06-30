use std::{fmt, ops::Deref, sync::Arc};

use ash::{khr, vk};

use crate::{
    ColorSpace, CompositeAlpha, Device, Extent2d, Extent3d, Fence, Format, Image, ImageBackend,
    ImageUsages, PresentMode, RawDevice, RawImage, RawSurface, Semaphore, SharingMode, Surface,
    SurfaceTransform, SwapchainFlags,
};

#[derive(Debug)]
pub struct SwapchainDescriptor<'a> {
    pub flags: SwapchainFlags,
    pub min_image_count: u32,
    pub image_format: Format,
    pub color_space: ColorSpace,
    pub image_extent: Extent2d,
    pub image_array_layers: u32,
    pub image_usage: ImageUsages,
    pub sharing_mode: SharingMode,
    pub queue_families: &'a [u32],
    pub pre_transform: SurfaceTransform,
    pub composite_alpha: CompositeAlpha,
    pub present_mode: PresentMode,
    pub clipped: bool,
}

impl Default for SwapchainDescriptor<'_> {
    fn default() -> Self {
        Self {
            flags: SwapchainFlags::empty(),
            min_image_count: 2,
            image_format: Format::B8G8R8A8Srgb,
            color_space: ColorSpace::SrgbNonlinear,
            image_extent: Extent2d::ZERO,
            image_array_layers: 1,
            image_usage: ImageUsages::empty(),
            sharing_mode: SharingMode::Exclusive,
            queue_families: &[],
            pre_transform: SurfaceTransform::Identity,
            composite_alpha: CompositeAlpha::Opaque,
            present_mode: PresentMode::Fifo,
            clipped: true,
        }
    }
}

impl Device {
    #[track_caller]
    pub fn create_swapchain(&self, surface: Surface, desc: &SwapchainDescriptor<'_>) -> Swapchain {
        self.try_create_swapchain(surface, desc)
            .expect("Failed to create swapchain")
    }

    pub fn try_create_swapchain(
        &self,
        surface: Surface,
        desc: &SwapchainDescriptor<'_>,
    ) -> Result<Swapchain, vk::Result> {
        assert!(!desc.image_extent.is_zero());
        assert!(!desc.image_usage.is_empty());

        let create_info = vk::SwapchainCreateInfoKHR {
            flags: vk::SwapchainCreateFlagsKHR::from_raw(desc.flags.bits()),
            surface: surface.raw_surface(),
            min_image_count: desc.min_image_count,
            image_format: vk::Format::from_raw(desc.image_format.as_raw()),
            image_color_space: vk::ColorSpaceKHR::from_raw(desc.color_space.as_raw()),
            image_extent: vk::Extent2D {
                width: desc.image_extent.width,
                height: desc.image_extent.height,
            },
            image_array_layers: desc.image_array_layers,
            image_usage: vk::ImageUsageFlags::from_raw(desc.image_usage.bits()),
            image_sharing_mode: vk::SharingMode::from_raw(desc.sharing_mode.as_raw()),
            queue_family_index_count: desc.queue_families.len() as u32,
            p_queue_family_indices: desc.queue_families.as_ptr(),
            pre_transform: vk::SurfaceTransformFlagsKHR::from_raw(
                desc.pre_transform.as_raw() as u32
            ),
            composite_alpha: vk::CompositeAlphaFlagsKHR::from_raw(
                desc.composite_alpha.as_raw() as u32
            ),
            present_mode: vk::PresentModeKHR::from_raw(desc.present_mode.as_raw()),
            clipped: desc.clipped.into(),
            ..Default::default()
        };

        let khr = khr::swapchain::Device::new(self.raw_instance(), self.raw_device());

        let swapchain = unsafe { khr.create_swapchain(&create_info, None)? };

        let raw = Arc::new(RawSwapchain {
            device: self.raw.clone(),
            surface: surface.raw.clone(),
            swapchain,
        });

        let images = unsafe {
            khr.get_swapchain_images(swapchain)?
                .into_iter()
                .map(|image| Image {
                    raw: Arc::new(RawImage {
                        backend: ImageBackend::Swapchain(raw.clone()),
                        image,
                        extent: Extent3d {
                            width: desc.image_extent.width,
                            height: desc.image_extent.height,
                            depth: 1,
                        },
                        format: desc.image_format,
                        levels: 1,
                        layers: desc.image_array_layers,
                        usages: desc.image_usage,
                    }),
                })
                .collect()
        };

        Ok(Swapchain {
            raw,
            surface,
            images,
        })
    }
}

#[derive(Debug)]
pub struct SwapchainImage<'a> {
    pub image: &'a Image,
    pub index: u32,
    pub is_suboptimal: bool,

    pub(crate) swapchain: vk::SwapchainKHR,
}

impl Deref for SwapchainImage<'_> {
    type Target = Image;

    fn deref(&self) -> &Self::Target {
        self.image
    }
}

pub struct Swapchain {
    raw: Arc<RawSwapchain>,
    surface: Surface,
    images: Vec<Image>,
}

impl Swapchain {
    pub fn raw_swapchain(&self) -> vk::SwapchainKHR {
        self.raw.swapchain
    }

    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    pub fn images(&self) -> &[Image] {
        &self.images
    }

    pub fn aquire_next_image(
        &self,
        timeout: Option<u64>,
        semaphore: Option<&Semaphore>,
        fence: Option<&Fence>,
    ) -> Result<SwapchainImage<'_>, vk::Result> {
        assert!(semaphore.is_some() || fence.is_some());

        let khr = self.raw.device();

        let (index, is_suboptimal) = unsafe {
            khr.acquire_next_image(
                self.raw.swapchain,
                timeout.unwrap_or(u64::MAX),
                semaphore.map_or(vk::Semaphore::null(), Semaphore::raw_semaphore),
                fence.map_or(vk::Fence::null(), Fence::raw_fence),
            )?
        };

        let image = &self.images[index as usize];

        Ok(SwapchainImage {
            image,
            index,
            is_suboptimal,
            swapchain: self.raw.swapchain,
        })
    }

    #[track_caller]
    pub fn recreate(&mut self, desc: &SwapchainDescriptor<'_>) {
        self.try_recreate(desc)
            .expect("Failed to recreate swapchain");
    }

    pub fn try_recreate(&mut self, desc: &SwapchainDescriptor<'_>) -> Result<(), vk::Result> {
        assert!(!desc.image_extent.is_zero());
        assert!(!desc.image_usage.is_empty());

        let create_info = vk::SwapchainCreateInfoKHR {
            flags: vk::SwapchainCreateFlagsKHR::from_raw(desc.flags.bits()),
            surface: self.surface.raw_surface(),
            min_image_count: desc.min_image_count,
            image_format: vk::Format::from_raw(desc.image_format.as_raw()),
            image_color_space: vk::ColorSpaceKHR::from_raw(desc.color_space.as_raw()),
            image_extent: vk::Extent2D {
                width: desc.image_extent.width,
                height: desc.image_extent.height,
            },
            image_array_layers: desc.image_array_layers,
            image_usage: vk::ImageUsageFlags::from_raw(desc.image_usage.bits()),
            image_sharing_mode: vk::SharingMode::from_raw(desc.sharing_mode.as_raw()),
            queue_family_index_count: desc.queue_families.len() as u32,
            p_queue_family_indices: desc.queue_families.as_ptr(),
            pre_transform: vk::SurfaceTransformFlagsKHR::from_raw(
                desc.pre_transform.as_raw() as u32
            ),
            composite_alpha: vk::CompositeAlphaFlagsKHR::from_raw(
                desc.composite_alpha.as_raw() as u32
            ),
            present_mode: vk::PresentModeKHR::from_raw(desc.present_mode.as_raw()),
            clipped: desc.clipped.into(),
            old_swapchain: self.raw.swapchain,
            ..Default::default()
        };

        let khr = self.raw.device();
        let swapchain = unsafe { khr.create_swapchain(&create_info, None)? };

        self.images.clear();
        self.raw = Arc::new(RawSwapchain {
            device: self.raw.device.clone(),
            surface: self.raw.surface.clone(),
            swapchain,
        });

        let images = unsafe { khr.get_swapchain_images(swapchain)? };

        for image in images {
            let image = Image {
                raw: Arc::new(RawImage {
                    backend: ImageBackend::Swapchain(self.raw.clone()),
                    image,
                    extent: Extent3d {
                        width: desc.image_extent.width,
                        height: desc.image_extent.height,
                        depth: 1,
                    },
                    format: desc.image_format,
                    levels: 1,
                    layers: desc.image_array_layers,
                    usages: desc.image_usage,
                }),
            };

            self.images.push(image);
        }

        Ok(())
    }
}

impl fmt::Debug for Swapchain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Swapchain").finish()
    }
}

pub(crate) struct RawSwapchain {
    pub(crate) device: Arc<RawDevice>,
    pub(crate) surface: Arc<RawSurface>,
    pub(crate) swapchain: vk::SwapchainKHR,
}

impl RawSwapchain {
    fn device(&self) -> khr::swapchain::Device {
        khr::swapchain::Device::new(&self.device.instance.instance, &self.device.device)
    }
}

impl Drop for RawSwapchain {
    fn drop(&mut self) {
        unsafe {
            tracing::trace!(handle = ?self.swapchain, "Destroying swapchain");
            self.device().destroy_swapchain(self.swapchain, None);
        }
    }
}
