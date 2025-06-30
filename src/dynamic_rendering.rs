use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use ash::vk;

use crate::{
    CommandEncoder, ImageLayout, ImageView, LoadOp, Rect2d, RenderingFlags, ResolveMode, StoreOp,
};

impl<'a> CommandEncoder<'a> {
    pub fn begin_rendering(&mut self, info: &RenderingInfo<'_>) -> RenderingEncoder<'_, 'a> {
        assert!(
            !info.area.extent.is_zero(),
            "Rendering area cannot have zero extent"
        );

        let color_attachments: Vec<_> = info
            .color_attachments
            .iter()
            .map(|attachment| {
                self.command_buffer.track_image_view(attachment.image_view);

                vk::RenderingAttachmentInfo {
                    image_view: attachment.image_view.raw_image_view(),
                    image_layout: vk::ImageLayout::from_raw(attachment.image_layout.as_raw()),
                    resolve_mode: vk::ResolveModeFlags::from_raw(
                        attachment.resolve_mode.as_raw() as u32
                    ),
                    resolve_image_view: attachment
                        .resolve_image_view
                        .map_or(vk::ImageView::null(), |view| view.raw_image_view()),
                    resolve_image_layout: vk::ImageLayout::from_raw(
                        attachment.resolve_image_layout.as_raw(),
                    ),
                    load_op: vk::AttachmentLoadOp::from_raw(attachment.load_op.as_raw()),
                    store_op: vk::AttachmentStoreOp::from_raw(attachment.store_op.as_raw()),
                    clear_value: vk::ClearValue {
                        color: vk::ClearColorValue {
                            float32: attachment.clear_value,
                        },
                    },
                    ..Default::default()
                }
            })
            .collect();

        let rendering_info = vk::RenderingInfo {
            flags: vk::RenderingFlags::from_raw(info.flags.bits()),
            render_area: vk::Rect2D {
                offset: vk::Offset2D {
                    x: info.area.offset.x,
                    y: info.area.offset.y,
                },
                extent: vk::Extent2D {
                    width: info.area.extent.width,
                    height: info.area.extent.height,
                },
            },
            layer_count: info.layers,
            view_mask: info.view_mask,
            color_attachment_count: color_attachments.len() as u32,
            p_color_attachments: color_attachments.as_ptr(),
            ..Default::default()
        };

        unsafe {
            self.device()
                .cmd_begin_rendering(self.command_buffer.raw.command_buffer, &rendering_info);
        }

        RenderingEncoder {
            command_encoder: self,
        }
    }
}

pub struct RenderingEncoder<'a, 'b> {
    pub(crate) command_encoder: &'a mut CommandEncoder<'b>,
}

impl RenderingEncoder<'_, '_> {
    pub(crate) fn device(&self) -> &ash::Device {
        self.command_encoder.device()
    }

    pub fn end(self) {}
}

impl<'a> Deref for RenderingEncoder<'_, 'a> {
    type Target = CommandEncoder<'a>;

    fn deref(&self) -> &Self::Target {
        self.command_encoder
    }
}

impl DerefMut for RenderingEncoder<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.command_encoder
    }
}

impl fmt::Debug for RenderingEncoder<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderingEncoder")
            .field("command_encoder", &self.command_encoder)
            .finish()
    }
}

impl Drop for RenderingEncoder<'_, '_> {
    fn drop(&mut self) {
        unsafe {
            self.device()
                .cmd_end_rendering(self.command_encoder.command_buffer.raw.command_buffer);
        }
    }
}

#[derive(Debug)]
pub struct RenderingInfo<'a> {
    pub flags: RenderingFlags,
    pub area: Rect2d,
    pub layers: u32,
    pub view_mask: u32,
    pub color_attachments: &'a [RenderingColorAttachment<'a>],
    pub depth_attachment: Option<RenderingDepthStencilAttachment<'a>>,
    pub stencil_attachment: Option<RenderingDepthStencilAttachment<'a>>,
}

impl Default for RenderingInfo<'_> {
    fn default() -> Self {
        Self {
            flags: RenderingFlags::empty(),
            area: Rect2d::default(),
            layers: 1,
            view_mask: 0,
            color_attachments: &[],
            depth_attachment: None,
            stencil_attachment: None,
        }
    }
}

#[derive(Debug)]
pub struct RenderingColorAttachment<'a> {
    pub image_view: &'a ImageView,
    pub image_layout: ImageLayout,
    pub resolve_mode: ResolveMode,
    pub resolve_image_view: Option<&'a ImageView>,
    pub resolve_image_layout: ImageLayout,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
    pub clear_value: [f32; 4],
}

impl<'a> RenderingColorAttachment<'a> {
    pub fn default(image_view: &'a ImageView) -> Self {
        Self {
            image_view,
            image_layout: ImageLayout::ColorAttachmentOptimal,
            resolve_mode: ResolveMode::None,
            resolve_image_view: None,
            resolve_image_layout: ImageLayout::Undefined,
            load_op: LoadOp::Clear,
            store_op: StoreOp::Store,
            clear_value: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

#[derive(Debug)]
pub struct RenderingDepthStencilAttachment<'a> {
    pub image_view: &'a ImageView,
    pub image_layout: ImageLayout,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
}
