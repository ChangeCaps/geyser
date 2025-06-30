use std::{fmt, sync::Arc};

use ash::{khr, vk};

use crate::{
    AccelBuildFlags, AccelBuildMode, Buffer, BufferUsages, CommandEncoder, Device, GeometryFlags,
    RawDevice,
};

impl Device {
    fn accel_khr(&self) -> khr::acceleration_structure::Device {
        khr::acceleration_structure::Device::new(&self.raw.instance.instance, &self.raw)
    }

    pub fn get_blas_build_sizes(&self, desc: &BlasDescriptor<'_>) -> AccelBuildSizes {
        let mut max_counts = Vec::new();
        let mut geometries = Vec::new();

        for geometry in desc.geometries {
            let max_count = match geometry.data {
                BlasGeometryData::Aabbs { max_count, .. } => max_count,
            };

            max_counts.push(max_count);

            let geometry = vk::AccelerationStructureGeometryKHR {
                geometry_type: match geometry.data {
                    BlasGeometryData::Aabbs { .. } => vk::GeometryTypeKHR::AABBS,
                },
                geometry: match geometry.data {
                    BlasGeometryData::Aabbs { stride, .. } => {
                        vk::AccelerationStructureGeometryDataKHR {
                            aabbs: vk::AccelerationStructureGeometryAabbsDataKHR {
                                data: vk::DeviceOrHostAddressConstKHR::default(),
                                stride,
                                ..Default::default()
                            },
                        }
                    }
                },
                flags: vk::GeometryFlagsKHR::from_raw(geometry.flags.bits()),
                ..Default::default()
            };

            geometries.push(geometry);
        }

        let info = vk::AccelerationStructureBuildGeometryInfoKHR {
            ty: vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL,
            flags: vk::BuildAccelerationStructureFlagsKHR::from_raw(desc.flags.bits()),
            mode: vk::BuildAccelerationStructureModeKHR::BUILD,
            src_acceleration_structure: vk::AccelerationStructureKHR::null(),
            dst_acceleration_structure: vk::AccelerationStructureKHR::null(),
            geometry_count: geometries.len() as u32,
            p_geometries: geometries.as_ptr(),
            scratch_data: vk::DeviceOrHostAddressKHR::default(),
            ..Default::default()
        };

        let mut sizes = vk::AccelerationStructureBuildSizesInfoKHR::default();

        unsafe {
            self.accel_khr().get_acceleration_structure_build_sizes(
                vk::AccelerationStructureBuildTypeKHR::DEVICE,
                &info,
                &max_counts,
                &mut sizes,
            );
        }

        AccelBuildSizes {
            accel_size: sizes.acceleration_structure_size,
            build_scratch_size: sizes.build_scratch_size,
            update_scratch_size: sizes.update_scratch_size,
        }
    }

    pub fn get_tlas_build_sizes(&self, desc: &TlasDescriptor) -> AccelBuildSizes {
        let geometry = vk::AccelerationStructureGeometryKHR {
            geometry_type: vk::GeometryTypeKHR::INSTANCES,
            geometry: vk::AccelerationStructureGeometryDataKHR {
                instances: vk::AccelerationStructureGeometryInstancesDataKHR {
                    array_of_pointers: vk::Bool32::from(false),
                    data: vk::DeviceOrHostAddressConstKHR::default(),
                    ..Default::default()
                },
            },
            flags: vk::GeometryFlagsKHR::empty(),
            ..Default::default()
        };

        let info = vk::AccelerationStructureBuildGeometryInfoKHR {
            ty: vk::AccelerationStructureTypeKHR::TOP_LEVEL,
            flags: vk::BuildAccelerationStructureFlagsKHR::from_raw(desc.flags.bits()),
            mode: vk::BuildAccelerationStructureModeKHR::BUILD,
            src_acceleration_structure: vk::AccelerationStructureKHR::null(),
            dst_acceleration_structure: vk::AccelerationStructureKHR::null(),
            geometry_count: 1,
            p_geometries: &geometry as *const _,
            scratch_data: vk::DeviceOrHostAddressKHR::default(),
            ..Default::default()
        };

        let mut sizes = vk::AccelerationStructureBuildSizesInfoKHR::default();

        unsafe {
            self.accel_khr().get_acceleration_structure_build_sizes(
                vk::AccelerationStructureBuildTypeKHR::DEVICE,
                &info,
                &[desc.max_instance_count],
                &mut sizes,
            );
        }

        AccelBuildSizes {
            accel_size: sizes.acceleration_structure_size,
            build_scratch_size: sizes.build_scratch_size,
            update_scratch_size: sizes.update_scratch_size,
        }
    }

    #[track_caller]
    pub fn create_blas(&self, buffer: &Buffer, offset: u64, size: u64) -> Blas {
        self.try_create_blas(buffer, offset, size)
            .expect("Failed to create BLAS")
    }

    pub fn try_create_blas(
        &self,
        buffer: &Buffer,
        offset: u64,
        size: u64,
    ) -> Result<Blas, vk::Result> {
        assert!(
            buffer.size() >= offset + size,
            "Buffer is too small for BLAS",
        );

        assert!(
            (buffer.usages()).contains(BufferUsages::ACCELERATION_STRUCTURE_STORAGE),
            "Buffer must have ACCELERATION_STRUCTURE_STORAGE usage",
        );

        let info = vk::AccelerationStructureCreateInfoKHR {
            ty: vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL,
            buffer: buffer.raw.handle,
            offset,
            size,
            ..Default::default()
        };

        let handle = unsafe {
            self.accel_khr()
                .create_acceleration_structure(&info, None)?
        };

        let raw = RawAccel {
            device: self.raw.clone(),
            handle,
            size,
        };

        Ok(Blas { raw: Arc::new(raw) })
    }

    #[track_caller]
    pub fn create_tlas(&self, buffer: &Buffer, offset: u64, size: u64) -> Tlas {
        self.try_create_tlas(buffer, offset, size)
            .expect("Failed to create TLAS")
    }

    pub fn try_create_tlas(
        &self,
        buffer: &Buffer,
        offset: u64,
        size: u64,
    ) -> Result<Tlas, vk::Result> {
        assert!(
            buffer.size() >= offset + size,
            "Buffer is too small for TLAS",
        );

        assert!(
            (buffer.usages()).contains(BufferUsages::ACCELERATION_STRUCTURE_STORAGE),
            "Buffer must have ACCELERATION_STRUCTURE_STORAGE usage",
        );

        let info = vk::AccelerationStructureCreateInfoKHR {
            ty: vk::AccelerationStructureTypeKHR::TOP_LEVEL,
            buffer: buffer.raw.handle,
            offset,
            size,
            ..Default::default()
        };

        let handle = unsafe {
            self.accel_khr()
                .create_acceleration_structure(&info, None)?
        };

        let raw = RawAccel {
            device: self.raw.clone(),
            handle,
            size,
        };

        Ok(Tlas { raw: Arc::new(raw) })
    }
}

impl CommandEncoder<'_> {
    pub fn build_acceleration_structures(
        &mut self,
        blas: &[BlasBuildDescriptor],
        tlas: &[TlasBuildDescriptor],
    ) {
        let device = &self.command_buffer.raw.command_pool.device;
        let accel_khr = khr::acceleration_structure::Device::new(
            &device.instance.instance,
            device, //
        );

        let mut geometries = Vec::new();
        let mut infos = Vec::new();
        let mut ranges = Vec::new();

        for blas in blas {
            self.command_buffer.track_blas(blas.blas);
            self.command_buffer.track_buffer(blas.scratch);

            let mut blas_geometries = Vec::new();
            let mut blas_ranges = Vec::new();
            let mut max_counts = Vec::new();

            for geometry in blas.geometries {
                let range = match geometry.data {
                    BlasBuildGeometryData::Aabbs { count, .. } => {
                        vk::AccelerationStructureBuildRangeInfoKHR {
                            primitive_count: count,
                            primitive_offset: 0,
                            first_vertex: 0,
                            transform_offset: 0,
                        }
                    }
                };

                let max_count = match geometry.data {
                    BlasBuildGeometryData::Aabbs { count, .. } => count,
                };

                let geometry = match geometry.data {
                    BlasBuildGeometryData::Aabbs {
                        buffer,
                        offset,
                        stride,
                        ..
                    } => vk::AccelerationStructureGeometryKHR {
                        geometry_type: vk::GeometryTypeKHR::AABBS,
                        geometry: vk::AccelerationStructureGeometryDataKHR {
                            aabbs: vk::AccelerationStructureGeometryAabbsDataKHR {
                                data: vk::DeviceOrHostAddressConstKHR {
                                    device_address: buffer.device_address() + offset,
                                },
                                stride,
                                ..Default::default()
                            },
                        },
                        flags: vk::GeometryFlagsKHR::from_raw(geometry.flags.bits()),
                        ..Default::default()
                    },
                };

                max_counts.push(max_count);
                blas_ranges.push(range);
                blas_geometries.push(geometry);
            }

            let info = vk::AccelerationStructureBuildGeometryInfoKHR {
                ty: vk::AccelerationStructureTypeKHR::BOTTOM_LEVEL,
                flags: vk::BuildAccelerationStructureFlagsKHR::from_raw(blas.flags.bits()),
                mode: vk::BuildAccelerationStructureModeKHR::from_raw(blas.mode.as_raw()),
                src_acceleration_structure: blas.blas.raw.handle,
                dst_acceleration_structure: blas.blas.raw.handle,
                geometry_count: blas_geometries.len() as u32,
                p_geometries: blas_geometries.as_ptr(),
                scratch_data: vk::DeviceOrHostAddressKHR {
                    device_address: blas.scratch.device_address(),
                },
                ..Default::default()
            };

            let mut sizes = vk::AccelerationStructureBuildSizesInfoKHR::default();
            unsafe {
                accel_khr.get_acceleration_structure_build_sizes(
                    vk::AccelerationStructureBuildTypeKHR::DEVICE,
                    &info,
                    &max_counts,
                    &mut sizes,
                );
            };

            assert!(
                sizes.acceleration_structure_size <= blas.blas.raw.size,
                "BLAS size exceeds allocated size"
            );

            assert_eq!(
                blas.mode,
                AccelBuildMode::Build,
                "Only build mode is supported for BLAS",
            );

            match blas.mode {
                AccelBuildMode::Update => {
                    assert!(
                        sizes.update_scratch_size <= blas.scratch.size(),
                        "BLAS update scratch size (0x{:x}) exceeds allocated size (0x{:x})",
                        sizes.update_scratch_size,
                        blas.scratch.size(),
                    );
                }
                AccelBuildMode::Build => {
                    assert!(
                        sizes.build_scratch_size <= blas.scratch.size(),
                        "BLAS build scratch size (0x{:x}) exceeds allocated size (0x{:x})",
                        sizes.build_scratch_size,
                        blas.scratch.size(),
                    );
                }
            }

            geometries.push(blas_geometries);
            ranges.push(blas_ranges);
            infos.push(info);
        }

        for tlas in tlas {
            self.command_buffer.track_tlas(tlas.tlas);

            let tlas_geometry = vec![vk::AccelerationStructureGeometryKHR {
                geometry_type: vk::GeometryTypeKHR::INSTANCES,
                geometry: vk::AccelerationStructureGeometryDataKHR {
                    instances: vk::AccelerationStructureGeometryInstancesDataKHR {
                        array_of_pointers: vk::Bool32::from(false),
                        data: vk::DeviceOrHostAddressConstKHR {
                            device_address: tlas.buffer.device_address() + tlas.offset,
                        },
                        ..Default::default()
                    },
                },
                flags: vk::GeometryFlagsKHR::empty(),
                ..Default::default()
            }];

            let info = vk::AccelerationStructureBuildGeometryInfoKHR {
                ty: vk::AccelerationStructureTypeKHR::TOP_LEVEL,
                flags: vk::BuildAccelerationStructureFlagsKHR::empty(),
                mode: vk::BuildAccelerationStructureModeKHR::BUILD,
                src_acceleration_structure: vk::AccelerationStructureKHR::null(),
                dst_acceleration_structure: tlas.tlas.raw.handle,
                geometry_count: 1,
                p_geometries: tlas_geometry.as_ptr(),
                scratch_data: vk::DeviceOrHostAddressKHR {
                    device_address: tlas.scratch.device_address(),
                },
                ..Default::default()
            };

            let range = vk::AccelerationStructureBuildRangeInfoKHR {
                primitive_count: tlas.count,
                primitive_offset: 0,
                first_vertex: 0,
                transform_offset: 0,
            };

            let mut sizes = vk::AccelerationStructureBuildSizesInfoKHR::default();

            unsafe {
                accel_khr.get_acceleration_structure_build_sizes(
                    vk::AccelerationStructureBuildTypeKHR::DEVICE,
                    &info,
                    &[tlas.count],
                    &mut sizes,
                );
            };

            assert!(
                sizes.acceleration_structure_size <= tlas.tlas.raw.size,
                "TLAS size exceeds allocated size"
            );

            assert_eq!(
                tlas.mode,
                AccelBuildMode::Build,
                "Only build mode is supported for TLAS",
            );

            match tlas.mode {
                AccelBuildMode::Update => {
                    assert!(
                        sizes.update_scratch_size <= tlas.scratch.size(),
                        "TLAS update scratch size (0x{:x}) exceeds allocated size (0x{:x})",
                        sizes.update_scratch_size,
                        tlas.scratch.size(),
                    );
                }
                AccelBuildMode::Build => {
                    assert!(
                        sizes.build_scratch_size <= tlas.scratch.size(),
                        "TLAS build scratch size (0x{:x}) exceeds allocated size (0x{:x})",
                        sizes.build_scratch_size,
                        tlas.scratch.size(),
                    );
                }
            }

            geometries.push(tlas_geometry);
            ranges.push(vec![range]);
            infos.push(info);
        }

        let ranges: Vec<_> = ranges.iter().map(Vec::as_slice).collect();

        unsafe {
            accel_khr.cmd_build_acceleration_structures(
                self.command_buffer.raw.command_buffer,
                &infos,
                &ranges,
            );
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccelBuildSizes {
    pub accel_size: u64,
    pub build_scratch_size: u64,
    pub update_scratch_size: u64,
}

#[derive(Debug)]
pub struct BlasDescriptor<'a> {
    pub flags: AccelBuildFlags,
    pub geometries: &'a [BlasGeometry],
}

#[derive(Debug)]
pub struct BlasBuildDescriptor<'a> {
    pub blas: &'a Blas,
    pub mode: AccelBuildMode,
    pub geometries: &'a [BlasBuildGeometry<'a>],
    pub scratch: &'a Buffer,
    pub flags: AccelBuildFlags,
}

#[derive(Debug)]
pub struct BlasBuildGeometry<'a> {
    pub data: BlasBuildGeometryData<'a>,
    pub flags: GeometryFlags,
}

#[derive(Debug)]
pub enum BlasBuildGeometryData<'a> {
    Aabbs {
        buffer: &'a Buffer,
        offset: u64,
        stride: u64,
        count: u32,
    },
}

#[derive(Debug)]
pub struct BlasGeometry {
    pub data: BlasGeometryData,
    pub flags: GeometryFlags,
}

#[derive(Debug)]
pub enum BlasGeometryData {
    Aabbs { stride: u64, max_count: u32 },
}

#[derive(Debug)]
pub struct TlasDescriptor {
    pub flags: AccelBuildFlags,
    pub max_instance_count: u32,
}

#[derive(Debug)]
pub struct TlasBuildDescriptor<'a> {
    pub tlas: &'a Tlas,
    pub mode: AccelBuildMode,
    pub buffer: &'a Buffer,
    pub offset: u64,
    pub count: u32,
    pub scratch: &'a Buffer,
}

pub struct Blas {
    pub(crate) raw: Arc<RawAccel>,
}

impl Blas {
    pub fn size(&self) -> u64 {
        self.raw.size
    }
}

impl fmt::Debug for Blas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Blas")
            .field("handle", &self.raw.handle)
            .finish()
    }
}

pub struct Tlas {
    pub(crate) raw: Arc<RawAccel>,
}

impl Tlas {
    pub fn size(&self) -> u64 {
        self.raw.size
    }
}

impl fmt::Debug for Tlas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tlas")
            .field("handle", &self.raw.handle)
            .finish()
    }
}

pub(crate) struct RawAccel {
    pub(crate) device: Arc<RawDevice>,
    pub(crate) handle: vk::AccelerationStructureKHR,

    pub(crate) size: u64,
}

impl Drop for RawAccel {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();

            let accel_khr = khr::acceleration_structure::Device::new(
                &self.device.instance.instance,
                &self.device,
            );

            tracing::trace!(
                handle = ?self.handle,
                "Destroying acceleration structure",
            );

            accel_khr.destroy_acceleration_structure(self.handle, None);
        }
    }
}
