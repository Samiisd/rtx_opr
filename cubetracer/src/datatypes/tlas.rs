use ash::vk;
use ash::version::DeviceV1_0;

use crate::context::Context;
use crate::datatypes::*;

use std::sync::Arc;
use std::collections::HashMap;

pub struct TlasVariable {
    modified: bool,

    blas_map: HashMap<BlasName, BlasVariable>,
    instance_buffer: Option<BufferVariable>,

    acceleration_structure: Option<AccelerationStructure>,
    structures: Vec<vk::AccelerationStructureNV>,
    info: Option<vk::WriteDescriptorSetAccelerationStructureNV>,
}

impl TlasVariable {
    pub fn new() -> TlasVariable {
        TlasVariable {
            blas_map: HashMap::new(),
            structures: vec![],
            acceleration_structure: None,
            instance_buffer: None,
            info: None,
            modified: true,
        }
    }

    pub fn register(&mut self, name: BlasName, blas: BlasVariable) {
        self.blas_map.insert(name, blas);
        self.modified = true;
    }

    pub fn unregister(&mut self, name: BlasName) {
        if self.blas_map.contains_key(&name) {
            self.blas_map.remove(&name);
            self.modified = true;
        }
    }

    /// build or rebuild the acceleration structure
    /// returns if something has been done
    pub fn build(&mut self, context: &Arc<Context>) -> bool {
        if !self.modified {
            return false;
        }

        self.modified = false;

        let data = self.blas_map
                        .iter()
                        .map(|(_, v)| v.instance_data())
                        .collect::<Vec<_>>();

        let acceleration_structure_info = vk::AccelerationStructureInfoNV::builder()
            .ty(vk::AccelerationStructureTypeNV::TOP_LEVEL)
            .instance_count(data.len() as u32)
            .build();

        self.acceleration_structure = Some(
            AccelerationStructure::new(Arc::clone(context), acceleration_structure_info)
        );

        self.structures = vec![
            self.acceleration_structure.as_ref().unwrap().acceleration_structure
        ];

        let instance_buffer = BufferVariable::device_buffer(
            context,
            vk::BufferUsageFlags::RAY_TRACING_NV,
            &data,
        ).0;

        // Build acceleration structure
        let scratch_buffer_size = self.blas_map
            .iter()
            .filter_map(|(_, v)| v.build_memory_requirements())
            .max()
            .unwrap_or(0)
            .max(self.acceleration_structure.as_ref().unwrap().get_memory_requirements(
                vk::AccelerationStructureMemoryRequirementsTypeNV::BUILD_SCRATCH,
            ).memory_requirements.size);

        let scratch_buffer = BufferVariable::create(
            context,
            scratch_buffer_size,
            scratch_buffer_size as _,
            vk::BufferUsageFlags::RAY_TRACING_NV,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        context.execute_one_time_commands(|command_buffer| {
            let memory_barrier = [
                vk::MemoryBarrier::builder()
                .src_access_mask(
                    vk::AccessFlags::ACCELERATION_STRUCTURE_READ_NV
                        | vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_NV,
                )
                .dst_access_mask(
                    vk::AccessFlags::ACCELERATION_STRUCTURE_READ_NV
                        | vk::AccessFlags::ACCELERATION_STRUCTURE_WRITE_NV,
                )
                .build()
            ];

            // Build bottom AS
            self.blas_map.iter_mut().for_each(|(_, blas)| {
                if blas.build(command_buffer, &scratch_buffer) {
                    // memory barrier if we build the BLAS
                    unsafe {
                        context.device().cmd_pipeline_barrier(
                            command_buffer,
                            vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_NV,
                            vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_NV,
                            vk::DependencyFlags::empty(),
                            &memory_barrier,
                            &[],
                            &[],
                        )
                    };
                }
            });

            // Build top AS
            self.acceleration_structure.as_ref().unwrap().cmd_build(
                command_buffer, &scratch_buffer, Some(&instance_buffer)
            );

            unsafe {
                context.device().cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_NV,
                    vk::PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_NV,
                    vk::DependencyFlags::empty(),
                    &memory_barrier,
                    &[],
                    &[],
                )
            };
        });

        self.instance_buffer = Some(instance_buffer);
        true
    }
}

impl DataType for TlasVariable {
    fn write_descriptor_builder(&mut self) -> vk::WriteDescriptorSetBuilder {

        self.info = Some(
            vk::WriteDescriptorSetAccelerationStructureNV::builder()
                .acceleration_structures(&self.structures)
                .build(),
        );

        match self.info {
            Some(ref mut e) => vk::WriteDescriptorSet::builder().push_next(e),
            None => panic!("should not happen"),
        }
    }
}
