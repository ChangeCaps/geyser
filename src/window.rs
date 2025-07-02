use std::ffi;

use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::{
    Instance, InstanceExtensions, Surface, Validated, ValidationError, VulkanError,
    is_validation_enabled,
};

impl Instance {
    #[track_caller]
    pub fn required_surface_extensions(display_handle: RawDisplayHandle) -> InstanceExtensions {
        Self::try_required_surface_extensions(display_handle)
            .expect("Failed to enumerate required surface extensions")
    }

    pub fn try_required_surface_extensions(
        display_handle: RawDisplayHandle,
    ) -> Result<InstanceExtensions, VulkanError> {
        let extensions = ash_window::enumerate_required_extensions(display_handle)?;

        let names = extensions.iter().map(|ext| unsafe {
            ffi::CStr::from_ptr(*ext)
                .to_str()
                .expect("Invalid UTF-8 in extension name")
        });

        Ok(InstanceExtensions::from_names(names))
    }

    /// Create a new [`Surface`] for the given display and window handles.
    ///
    /// # Safety
    /// - `display_handle` must be a valid display handle.
    /// - `window_handle` must be a valid window handle for the display associated with
    ///   `display_handle`.
    /// - The display and window associated with `display_handle` and `window_handle`
    ///   must outlive the created `Surface` instance.
    #[track_caller]
    pub unsafe fn create_surface(
        &self,
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
    ) -> Surface {
        unsafe {
            self.try_create_surface(display_handle, window_handle)
                .expect("Failed to create surface")
        }
    }

    /// Create a new [`Surface`] for the given display and window handles.
    ///
    /// # Safety
    /// - `display_handle` must be a valid display handle.
    /// - `window_handle` must be a valid window handle for the display associated with
    ///   `display_handle`.
    /// - The display and window associated with `display_handle` and `window_handle`
    ///   must outlive the created `Surface` instance.
    pub unsafe fn try_create_surface(
        &self,
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
    ) -> Result<Surface, Validated<VulkanError>> {
        if is_validation_enabled() {
            self.validate_create_surface(display_handle)?;
        }

        unsafe {
            self.try_create_surface_unchecked(display_handle, window_handle)
                .map_err(From::from)
        }
    }

    /// # Safety
    /// - `display_handle` must be a valid display handle.
    /// - `window_handle` must be a valid window handle for the display associated with
    ///   `display_handle`.
    /// - The display and window associated with `display_handle` and `window_handle`
    ///   must outlive the created `Surface` instance.
    /// - `instance` must have been created with the extensions from
    ///   [`Entry::required_surface_extensions`] for `display_handle` enabled.
    pub unsafe fn try_create_surface_unchecked(
        &self,
        display_handle: RawDisplayHandle,
        window_hanlde: RawWindowHandle,
    ) -> Result<Surface, VulkanError> {
        let surface = unsafe {
            ash_window::create_surface(
                &self.inner.entry,
                &self.inner.handle,
                display_handle,
                window_hanlde,
                None,
            )?
        };

        Ok(Surface {
            handle: surface,

            instance: self.inner.clone(),
        })
    }

    fn validate_create_surface(
        &self,
        display_handle: RawDisplayHandle,
    ) -> Result<(), Validated<VulkanError>> {
        let required_extensions = Self::try_required_surface_extensions(display_handle)?;

        let difference = required_extensions.difference(self.enabled_extensions());
        if !difference.is_empty() {
            return Err(From::from(ValidationError {
                context: "instance.enabled_extensions()".into(),
                problem: "Missing required surface extensions".into(),
                ..Default::default()
            }));
        }

        Ok(())
    }
}
