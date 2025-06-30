use std::sync::Arc;

use ash::vk;
use raw_window_handle::{DisplayHandle, WindowHandle};

use crate::{Instance, InstanceExtension, RawSurface, Surface};

impl Instance {
    /// Query the required instance extensions for a given display handle.
    pub fn required_window_extensions(
        display_handle: &DisplayHandle,
    ) -> Result<Vec<InstanceExtension>, vk::Result> {
        let extensions = ash_window::enumerate_required_extensions(display_handle.as_raw())?;
        let extensions = extensions
            .iter()
            .map(|ext| unsafe { InstanceExtension::from_raw(*ext) })
            .collect::<Vec<_>>();

        Ok(extensions)
    }

    pub fn is_display_supported(display_handle: &DisplayHandle) -> bool {
        let extensions = Self::required_window_extensions(display_handle);

        match extensions {
            Ok(extensions) => extensions.iter().all(Self::is_extension_supported),
            Err(_) => false,
        }
    }

    /// # Safety
    /// - The window represented by `window_handle` must be associated the display connection in
    ///   `display_handle`.
    /// - `window_handle` and `display_handle` must be associated with a valid window and display
    ///   connection, which must be valid for the lifetime of the returned [`Surface`] and any
    ///   [`Swapchain`](crate::Swapchain)s created by it.
    pub unsafe fn create_surface(
        &self,
        display_handle: &DisplayHandle,
        window_handle: &WindowHandle,
    ) -> Result<Surface, vk::Result> {
        // SAFETY: the caller ensures the safety requirements are met.
        let surface = unsafe {
            ash_window::create_surface(
                Self::entry(),
                self.raw_instance(),
                display_handle.as_raw(),
                window_handle.as_raw(),
                None,
            )?
        };

        Ok(Surface {
            raw: Arc::new(RawSurface {
                instance: self.raw.clone(),
                surface,
            }),
        })
    }
}
