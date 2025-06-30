use std::{env, error::Error, fs::File, path::Path};

mod generate;

macro_rules! enums {
    ($($name:ident, $prefix:literal, $file:literal),* $(,)?) => {
        let path = Path::new(&env::var("OUT_DIR").unwrap()).join("enums.rs");
        let mut enums = File::create(&path)?;

        $(
            generate::generate_enum(
                &mut enums,
                stringify!($name),
                $prefix,
                include_str!(concat!("headers/", $file)),
            )?;
        )*
    };
}

macro_rules! flags {
    ($($name:ident, $prefix:literal, $file:literal),* $(,)?) => {
        let path = Path::new(&env::var("OUT_DIR").unwrap()).join("flags.rs");
        let mut flags = File::create(&path)?;

        $(
            generate::generate_flags(
                &mut flags,
                stringify!($name),
                $prefix,
                include_str!(concat!("headers/", $file)),
            )?;
        )*
    };
}

fn main() -> Result<(), Box<dyn Error>> {
    enums! {
        AccelBuildKind, "VK_ACCELERATION_STRUCTURE_BUILD_TYPE_", "accel_build_kind.c",
        AccelBuildMode, "VK_BUILD_ACCELERATION_STRUCTURE_MODE_", "accel_build_mode.c",
        AccelKind, "VK_ACCELERATION_STRUCTURE_TYPE_", "accel_kind.c",
        ColorSpace, "VK_COLOR_SPACE_", "color_space.c",
        CommandBufferLevel, "VK_COMMAND_BUFFER_LEVEL_", "command_buffer_level.c",
        ComponentSwizzle, "VK_COMPONENT_SWIZZLE_", "component_swizzle.c",
        CompositeAlpha, "VK_COMPOSITE_ALPHA_", "composite_alpha.c",
        Format, "VK_FORMAT_", "format.c",
        ImageLayout, "VK_IMAGE_LAYOUT_", "image_layout.c",
        ImageViewKind, "VK_IMAGE_VIEW_TYPE_", "image_view_kind.c",
        LoadOp, "VK_ATTACHMENT_LOAD_OP_", "attachment_load_op.c",
        PresentMode, "VK_PRESENT_MODE_", "present_mode.c",
        ResolveMode, "VK_RESOLVE_MODE_", "resolve_mode.c",
        SharingMode, "VK_SHARING_MODE_", "sharing_mode.c",
        StoreOp, "VK_ATTACHMENT_STORE_OP_", "attachment_store_op.c",
        SurfaceTransform, "VK_SURFACE_TRANSFORM_", "surface_transform.c",
    }

    flags! {
        AccelBuildFlags, "VK_BUILD_ACCELERATION_STRUCTURE_", "accel_build_flags.c",
        AccelFlags, "VK_ACCELERATION_STRUCTURE_CREATE_", "accel_flags.c",
        Access, "VK_ACCESS_", "access.c",
        BufferFlags, "VK_BUFFER_CREATE_", "buffer_flags.c",
        BufferUsages, "VK_BUFFER_USAGE_", "buffer_usages.c",
        CommandBufferUsages, "VK_COMMAND_BUFFER_USAGE_", "command_buffer_usages.c",
        CommandPoolFlags, "VK_COMMAND_POOL_CREATE_", "command_pool_flags.c",
        Dependencies, "VK_DEPENDENCY_", "dependencies.c",
        GeometryFlags, "VK_GEOMETRY_", "geometry_flags.c",
        ImageAspects, "VK_IMAGE_ASPECT_", "image_aspects.c",
        ImageUsages, "VK_IMAGE_USAGE_", "image_usages.c",
        ImageViewFlags, "VK_IMAGE_VIEW_CREATE_", "image_view_flags.c",
        InstanceFlags, "VK_INSTANCE_CREATE_", "instance_flags.c",
        MemoryAllocateFlags, "VK_MEMORY_ALLOCATE_", "memory_allocate_flags.c",
        MemoryProperties, "VK_MEMORY_PROPERTY_", "memory_properties.c",
        PipelineStages, "VK_PIPELINE_STAGE_", "pipeline_stages.c",
        RenderingFlags, "VK_RENDERING_", "rendering_flags.c",
        SwapchainFlags, "VK_SWAPCHAIN_CREATE_", "swapchain_flags.c",
    }

    Ok(())
}
