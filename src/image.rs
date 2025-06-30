use std::{fmt, sync::Arc};

use ash::vk;

use crate::{
    ComponentSwizzle, Extent3d, Format, ImageAspects, ImageUsages, ImageViewFlags, ImageViewKind,
    RawDevice, RawSwapchain,
};

pub struct Image {
    pub(crate) raw: Arc<RawImage>,
}

impl Image {
    pub fn raw_image(&self) -> vk::Image {
        self.raw.image
    }

    pub fn extent(&self) -> Extent3d {
        self.raw.extent
    }

    pub fn format(&self) -> Format {
        self.raw.format
    }

    pub fn levels(&self) -> u32 {
        self.raw.levels
    }

    pub fn layers(&self) -> u32 {
        self.raw.layers
    }

    pub fn usages(&self) -> ImageUsages {
        self.raw.usages
    }

    #[track_caller]
    pub fn create_view(&self, desc: &ImageViewDescriptor) -> ImageView {
        self.try_create_view(desc)
            .expect("Failed to create image view")
    }

    pub fn try_create_view(&self, desc: &ImageViewDescriptor) -> Result<ImageView, vk::Result> {
        let format = desc.format.unwrap_or(self.raw.format);

        let create_info = vk::ImageViewCreateInfo {
            flags: vk::ImageViewCreateFlags::from_raw(desc.flags.bits()),
            image: self.raw.image,
            view_type: vk::ImageViewType::from_raw(desc.kind.as_raw()),
            format: vk::Format::from_raw(format.as_raw()),
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::from_raw(desc.components.r.as_raw()),
                g: vk::ComponentSwizzle::from_raw(desc.components.g.as_raw()),
                b: vk::ComponentSwizzle::from_raw(desc.components.b.as_raw()),
                a: vk::ComponentSwizzle::from_raw(desc.components.a.as_raw()),
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::from_raw(desc.aspects.bits()),
                base_mip_level: desc.base_mip_level,
                level_count: desc.level_count,
                base_array_layer: desc.base_array_layer,
                layer_count: desc.layer_count,
            },
            ..Default::default()
        };

        let raw_view = unsafe {
            self.raw
                .backend
                .device()
                .create_image_view(&create_info, None)?
        };

        Ok(ImageView {
            raw: Arc::new(RawImageView {
                image: self.raw.clone(),
                device: self.raw.backend.device().clone(),
                image_view: raw_view,

                format,

                flags: desc.flags,
                kind: desc.kind,
                components: desc.components,
                aspects: desc.aspects,
                base_mip_level: desc.base_mip_level,
                level_count: desc.level_count,
                base_array_layer: desc.base_array_layer,
                layer_count: desc.layer_count,
            }),
        })
    }
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Image").finish()
    }
}

pub(crate) struct RawImage {
    pub(crate) backend: ImageBackend,
    pub(crate) image: vk::Image,
    pub(crate) extent: Extent3d,
    pub(crate) format: Format,
    pub(crate) levels: u32,
    pub(crate) layers: u32,
    pub(crate) usages: ImageUsages,
}

pub(crate) enum ImageBackend {
    Device(Arc<RawDevice>),

    #[allow(dead_code)]
    Swapchain(Arc<RawSwapchain>),
}

impl ImageBackend {
    fn device(&self) -> &Arc<RawDevice> {
        match self {
            ImageBackend::Device(device) => device,
            ImageBackend::Swapchain(swapchain) => &swapchain.device,
        }
    }
}

impl Drop for RawImage {
    fn drop(&mut self) {
        match self.backend {
            ImageBackend::Device(ref device) => unsafe {
                tracing::trace!(handle = ?self.image, "Destroying image");
                device.destroy_image(self.image, None);
            },

            ImageBackend::Swapchain(_) => {}
        }
    }
}

#[derive(Clone, Debug)]
pub struct ImageViewDescriptor {
    pub flags: ImageViewFlags,
    pub kind: ImageViewKind,
    pub format: Option<Format>,
    pub components: ComponentMapping,
    pub aspects: ImageAspects,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}

impl Default for ImageViewDescriptor {
    fn default() -> Self {
        Self {
            flags: ImageViewFlags::empty(),
            kind: ImageViewKind::D2,
            format: None,
            components: ComponentMapping::default(),
            aspects: ImageAspects::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComponentMapping {
    pub r: ComponentSwizzle,
    pub g: ComponentSwizzle,
    pub b: ComponentSwizzle,
    pub a: ComponentSwizzle,
}

impl Default for ComponentMapping {
    fn default() -> Self {
        Self {
            r: ComponentSwizzle::Identity,
            g: ComponentSwizzle::Identity,
            b: ComponentSwizzle::Identity,
            a: ComponentSwizzle::Identity,
        }
    }
}

pub struct ImageView {
    pub(crate) raw: Arc<RawImageView>,
}

impl ImageView {
    pub fn raw_image_view(&self) -> vk::ImageView {
        self.raw.image_view
    }

    pub fn flags(&self) -> ImageViewFlags {
        self.raw.flags
    }

    pub fn kind(&self) -> ImageViewKind {
        self.raw.kind
    }

    pub fn format(&self) -> Format {
        self.raw.format
    }

    pub fn components(&self) -> ComponentMapping {
        self.raw.components
    }

    pub fn aspects(&self) -> ImageAspects {
        self.raw.aspects
    }

    pub fn base_mip_level(&self) -> u32 {
        self.raw.base_mip_level
    }

    pub fn level_count(&self) -> u32 {
        self.raw.level_count
    }

    pub fn base_array_layer(&self) -> u32 {
        self.raw.base_array_layer
    }

    pub fn layer_count(&self) -> u32 {
        self.raw.layer_count
    }

    pub fn subresource_range(&self) -> ImageSubresourceRange {
        ImageSubresourceRange {
            aspects: self.raw.aspects,
            base_mip_level: self.raw.base_mip_level,
            level_count: self.raw.level_count,
            base_array_layer: self.raw.base_array_layer,
            layer_count: self.raw.layer_count,
        }
    }
}

impl fmt::Debug for ImageView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImageView").finish()
    }
}

pub(crate) struct RawImageView {
    #[allow(dead_code)]
    pub(crate) image: Arc<RawImage>,
    pub(crate) device: Arc<RawDevice>,
    pub(crate) image_view: vk::ImageView,

    pub(crate) flags: ImageViewFlags,
    pub(crate) kind: ImageViewKind,
    pub(crate) format: Format,
    pub(crate) components: ComponentMapping,
    pub(crate) aspects: ImageAspects,
    pub(crate) base_mip_level: u32,
    pub(crate) level_count: u32,
    pub(crate) base_array_layer: u32,
    pub(crate) layer_count: u32,
}

impl Drop for RawImageView {
    fn drop(&mut self) {
        unsafe {
            tracing::trace!(handle = ?self.image_view, "Destroying image view");
            self.device.destroy_image_view(self.image_view, None);
        }
    }
}

#[derive(Clone, Debug)]
pub struct ImageSubresourceRange {
    pub aspects: ImageAspects,
    pub base_mip_level: u32,
    pub level_count: u32,
    pub base_array_layer: u32,
    pub layer_count: u32,
}
