use geyser::*;

fn main() {
    tracing_subscriber::fmt::init();

    let entry = Entry::linked();
    let instance = entry.create_instance(&InstanceDescriptor {
        enabled_layers: &["VK_LAYER_KHRONOS_validation"],
        ..Default::default()
    });

    let physical_devices = instance.enumerate_physical_devices();

    for physical_device in &physical_devices {
        tracing::info!(
            "Found physical device: {} ({}, {})",
            physical_device.properties().device_name,
            physical_device.properties().api_version,
            physical_device.properties().driver_version,
        );

        for family in physical_device.queue_families() {
            tracing::info!(
                "  Queue family: {:?} ({} queues)",
                family.queue_flags,
                family.queue_count
            );
        }
    }
}
