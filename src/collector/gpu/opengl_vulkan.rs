/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Reader for OpenGL and Vulkan driver information

use crate::collector::{gpu::GpuVendor, helpers::discovery::Discovery};
use std::{
    collections::HashMap,
    ffi::c_void,
    path::{Path, PathBuf},
};

/// Collects OpenGL and Vulkan driver information
pub struct Collector {
    vk_drivers: Discovery<VulkanInfo>,
    gl_drivers: Discovery<OpenglInfo>,
}

impl Default for Collector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector {
    pub fn new() -> Self {
        Self {
            vk_drivers: Discovery::default(),
            gl_drivers: Discovery::default(),
        }
    }

    /// Returns the OpenGL and Vulkan drivers for the GPU at the given path respectively.
    pub fn get_drivers(
        &mut self,
        path: &Path,
        vendor: &GpuVendor,
        device: u16,
    ) -> (String, String) {
        let gl_drivers = match self.gl_drivers.require(OpenglInfo::init) {
            Ok(gl_drivers) => gl_drivers,
            Err(e) => {
                tracing::warn!("failed to fetch opengl drivers: {e}");
                return ("".to_string(), "".to_string());
            }
        };

        let vk_drivers = match self.vk_drivers.require(VulkanInfo::init) {
            Ok(vk_drivers) => vk_drivers,
            Err(e) => {
                tracing::warn!("failed to fetch vulkan drivers: {e}");
                return ("".to_string(), "".to_string());
            }
        };

        let vendor = match vendor {
            GpuVendor::Intel => 0x8086,
            GpuVendor::Nvidia => 0x10DE,
            GpuVendor::Amd => 0x1002,
        };

        let Some(gl_driver) = gl_drivers.drivers.get(
            &path
                .file_name()
                .and_then(|f| f.to_str())
                .map(|f| f.to_string())
                .unwrap_or_default(),
        ) else {
            tracing::warn!(
                "Could not find opengl driver for gpu 0x{:x}/0x{:x}",
                vendor,
                device
            );
            return ("".to_string(), "".to_string());
        };

        let Some(vk_driver) = vk_drivers.drivers.get(&DeviceId { vendor, device }) else {
            tracing::warn!(
                "Could not find vulkan driver for gpu 0x{:x}/0x{:x}",
                vendor,
                device
            );
            return ("".to_string(), "".to_string());
        };

        (
            format!(
                "[{}] {} (OpenGL {})",
                gl_driver.name, gl_driver.driver_info, gl_driver.version
            ),
            format!(
                "[{}] {} (Vulkan {})",
                vk_driver.name, vk_driver.driver_info, vk_driver.version
            ),
        )
    }
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct DeviceId {
    vendor: u16,
    device: u16,
}

#[derive(Debug)]
struct GlDriverInfo {
    /// The name of the OpenGL driver (i.e. Nouveau, "AMD Open-source Driver")
    name: String,
    /// The driver info (i.e. Mesa 26.0...)
    driver_info: String,
    /// The OpenGL version (i.e. 4.6)
    version: String,
}

struct OpenglInfo {
    drivers: HashMap<String, GlDriverInfo>,
}

mod egl {
    pub use khronos_egl::*;
    pub use std::ffi::{c_char, c_void};

    pub type EGLLabelKHR = *const c_void;
    pub type EGLDeviceEXT = *const c_void;

    #[allow(non_snake_case)]
    pub type EGLDEBUGPROCKHR = unsafe extern "system" fn(
        error: Enum,
        command: *const c_char,
        messageType: Int,
        threadLabel: EGLLabelKHR,
        message: *const c_char,
    );

    pub const EGL_SUCCESS: Enum = 0x3000;
    pub const EGL_NOT_INITIALIZED: Enum = 0x3001;
    pub const EGL_BAD_ACCESS: Enum = 0x3002;
    pub const EGL_BAD_ALLOC: Enum = 0x3003;
    pub const EGL_BAD_ATTRIBUTE: Enum = 0x3004;
    pub const EGL_BAD_CONFIG: Enum = 0x3005;
    pub const EGL_BAD_CONTEXT: Enum = 0x3006;
    pub const EGL_BAD_CURRENT_SURFACE: Enum = 0x3007;
    pub const EGL_BAD_DISPLAY: Enum = 0x3008;
    pub const EGL_BAD_MATCH: Enum = 0x3009;
    pub const EGL_BAD_NATIVE_PIXMAP: Enum = 0x300A;
    pub const EGL_BAD_NATIVE_WINDOW: Enum = 0x300B;
    pub const EGL_BAD_PARAMETER: Enum = 0x300C;
    pub const EGL_BAD_SURFACE: Enum = 0x300D;
    pub const EGL_CONTEXT_LOST: Enum = 0x300E;

    pub const EGL_DEBUG_MSG_CRITICAL_KHR: Int = 0x33B9;
    pub const EGL_DEBUG_MSG_ERROR_KHR: Int = 0x33BA;
    pub const EGL_DEBUG_MSG_WARNING_KHR: Int = 0x33BB;
    pub const EGL_DEBUG_MSG_INFO_KHR: Int = 0x33BC;

    pub const EGL_PLATFORM_DEVICE_EXT: Enum = 0x313F;
    pub const EGL_DRM_RENDER_NODE_FILE_EXT: Int = 0x3377;

    pub type PFNEGLDEBUGMESSAGECONTROLKHRPROC =
        unsafe extern "system" fn(callback: EGLDEBUGPROCKHR, attrib_list: *const Attrib) -> Int;

    pub type PFNEGLQUERYDEVICESEXTPROC = unsafe extern "system" fn(
        max_devices: Int,
        devices: *mut EGLDeviceEXT,
        num_devices: *mut Int,
    ) -> Boolean;

    pub type PFNEGLQUERYDEVICESTRINGEXTPROC =
        unsafe extern "system" fn(device: EGLDeviceEXT, name: Int) -> *const c_char;
}

unsafe extern "system" fn egl_debug_callback(
    error: egl::Enum,
    command: *const egl::c_char,
    message_type: egl::Int,
    _: egl::EGLLabelKHR,
    message: *const egl::c_char,
) {
    let error_str = match error {
        egl::EGL_SUCCESS => "EGL_SUCCESS",
        egl::EGL_NOT_INITIALIZED => "EGL_NOT_INITIALIZED",
        egl::EGL_BAD_ACCESS => "EGL_BAD_ACCESS",
        egl::EGL_BAD_ALLOC => "EGL_BAD_ALLOC",
        egl::EGL_BAD_ATTRIBUTE => "EGL_BAD_ATTRIBUTE",
        egl::EGL_BAD_CONFIG => "EGL_BAD_CONFIG",
        egl::EGL_BAD_CONTEXT => "EGL_BAD_CONTEXT",
        egl::EGL_BAD_CURRENT_SURFACE => "EGL_BAD_CURRENT_SURFACE",
        egl::EGL_BAD_DISPLAY => "EGL_BAD_DISPLAY",
        egl::EGL_BAD_SURFACE => "EGL_BAD_SURFACE",
        egl::EGL_BAD_MATCH => "EGL_BAD_MATCH",
        egl::EGL_BAD_PARAMETER => "EGL_BAD_PARAMETER",
        egl::EGL_BAD_NATIVE_PIXMAP => "EGL_BAD_NATIVE_PIXMAP",
        egl::EGL_BAD_NATIVE_WINDOW => "EGL_BAD_NATIVE_WINDOW",
        egl::EGL_CONTEXT_LOST => "EGL_CONTEXT_LOST",
        _ => "UNKNOWN",
    };

    let error_message = if message != std::ptr::null() {
        unsafe { std::ffi::CStr::from_ptr(message) }.to_string_lossy()
    } else {
        "UNKNOWN".into()
    };
    let command_str = if command != std::ptr::null() {
        unsafe { std::ffi::CStr::from_ptr(command) }.to_string_lossy()
    } else {
        "UNKNOWN".into()
    };

    match message_type {
        egl::EGL_DEBUG_MSG_CRITICAL_KHR => {
            tracing::error!(
                "CRITICAL ({}): {} from {}",
                error_str,
                error_message,
                command_str
            )
        }
        egl::EGL_DEBUG_MSG_ERROR_KHR => {
            tracing::warn!("({}): {} from {}", error_str, error_message, command_str)
        }
        egl::EGL_DEBUG_MSG_WARNING_KHR => {
            tracing::info!("({}): {} from {}", error_str, error_message, command_str)
        }
        egl::EGL_DEBUG_MSG_INFO_KHR => {
            tracing::debug!("({}): {} from {}", error_str, error_message, command_str)
        }
        _ => {}
    }
}

impl OpenglInfo {
    /// Yeah this is ugly as hell but it's better than pulling in glutin or something which is bloated for this use case
    pub fn init() -> anyhow::Result<OpenglInfo> {
        let mut drivers = HashMap::new();
        let lib = unsafe { egl::DynamicInstance::<egl::EGL1_5>::load_required() }?;
        let egl_client_extensions = lib.query_string(None, egl::EXTENSIONS)?.to_string_lossy();

        // set EGL debug callback function for nice error messages
        if egl_client_extensions.contains("EGL_KHR_debug") {
            #[allow(non_snake_case)]
            let Some(eglDebugMessageControlKHR) = (unsafe {
                lib.get_proc_address("eglDebugMessageControlKHR")
                    .map(|proc| {
                        std::mem::transmute::<_, egl::PFNEGLDEBUGMESSAGECONTROLKHRPROC>(proc)
                    })
            }) else {
                return Ok(OpenglInfo { drivers });
            };
            let attrib = [
                egl::EGL_DEBUG_MSG_WARNING_KHR as egl::Attrib,
                egl::TRUE as egl::Attrib,
                egl::EGL_DEBUG_MSG_INFO_KHR as egl::Attrib,
                egl::TRUE as egl::Attrib,
                egl::NONE as egl::Attrib,
            ];
            unsafe { eglDebugMessageControlKHR(egl_debug_callback, attrib.as_ptr()) };
        }

        if egl_client_extensions.contains("EGL_EXT_device_base") {
            #[allow(non_snake_case)]
            let eglQueryDevicesEXT: egl::PFNEGLQUERYDEVICESEXTPROC =
                unsafe { std::mem::transmute(lib.get_proc_address("eglQueryDevicesEXT").unwrap()) };
            let mut device_count = 0i32;
            unsafe { eglQueryDevicesEXT(0, std::ptr::null_mut(), &mut device_count) };
            let mut devices: Vec<egl::EGLDeviceEXT> = Vec::new();
            devices.resize(device_count as usize, std::ptr::null());
            unsafe { eglQueryDevicesEXT(device_count, devices.as_mut_ptr(), &mut device_count) };

            #[allow(non_snake_case)]
            let eglQueryDeviceStringEXT: egl::PFNEGLQUERYDEVICESTRINGEXTPROC = unsafe {
                std::mem::transmute(lib.get_proc_address("eglQueryDeviceStringEXT").unwrap())
            };
            for &device in devices.iter() {
                // Read device extensions
                let device_extensions = unsafe {
                    std::ffi::CStr::from_ptr(eglQueryDeviceStringEXT(device, egl::EXTENSIONS))
                }
                .to_string_lossy();

                if device_extensions.contains("EGL_EXT_device_drm_render_node") {
                    let Some(render_node) = PathBuf::from(
                        unsafe {
                            let rn =
                                eglQueryDeviceStringEXT(device, egl::EGL_DRM_RENDER_NODE_FILE_EXT);
                            if rn == std::ptr::null() {
                                continue;
                            }
                            std::ffi::CStr::from_ptr(rn)
                        }
                        .to_string_lossy()
                        .to_string(),
                    )
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.to_string()) else {
                        continue;
                    };

                    // process each device
                    let display = unsafe {
                        lib.get_platform_display(
                            egl::EGL_PLATFORM_DEVICE_EXT,
                            device as *mut egl::c_void,
                            &[egl::ATTRIB_NONE],
                        )
                    }?;
                    if lib.initialize(display).is_ok() {
                        let display_extensions = lib
                            .query_string(Some(display), egl::EXTENSIONS)?
                            .to_string_lossy();

                        if lib.bind_api(egl::OPENGL_API).is_ok()
                            && display_extensions.contains("EGL_KHR_no_config_context")
                            && display_extensions.contains("EGL_KHR_surfaceless_context")
                        {
                            let Ok(context) = lib.create_context(
                                display,
                                unsafe { egl::Config::from_ptr(std::ptr::null_mut()) },
                                None,
                                &[egl::NONE],
                            ) else {
                                continue;
                            };
                            lib.make_current(display, None, None, Some(context))?;

                            gl::GetString::load_with(|name| {
                                lib.get_proc_address(name)
                                    .map(|p| p as *const c_void)
                                    .unwrap_or(std::ptr::null())
                            });

                            if gl::GetString::is_loaded() {
                                let version =
                                    unsafe {
                                        std::ffi::CStr::from_ptr(
                                            gl::GetString(gl::VERSION) as *const i8
                                        )
                                    }
                                    .to_string_lossy()
                                    .to_string();
                                let renderer = unsafe {
                                    std::ffi::CStr::from_ptr(
                                        gl::GetString(gl::RENDERER) as *const i8
                                    )
                                }
                                .to_string_lossy()
                                .to_string();

                                // get short names for the common opengl drivers
                                let name = if renderer.contains("zink") {
                                    "zink".to_string()
                                } else if renderer.contains("radeonsi") {
                                    "radeonsi".to_string()
                                } else if renderer.contains("nouveau") {
                                    "nouveau".to_string()
                                } else if renderer.contains("nvidia") {
                                    "nvidia".to_string()
                                } else {
                                    renderer
                                };

                                drivers.insert(
                                    render_node,
                                    GlDriverInfo {
                                        name,
                                        version: version
                                            .split_whitespace()
                                            .nth(0)
                                            .map(|v| v.to_string())
                                            .unwrap_or("unknown".to_string()),
                                        driver_info: version
                                            .split_whitespace()
                                            .skip(3)
                                            .map(|v| v.to_string())
                                            .collect::<Vec<_>>()
                                            .join(" "),
                                    },
                                );
                            }

                            lib.destroy_context(display, context)?;
                        }
                        lib.terminate(display)?;
                    }
                }
            }
        }

        Ok(OpenglInfo { drivers })
    }
}

#[derive(Debug)]
struct VkDriverInfo {
    /// The name of the vulkan driver (i.e. NVK, "AMD Open-source Driver")
    name: String,
    /// The driver info (i.e. Mesa 26.0...)
    driver_info: String,
    /// The Vulkan version (i.e. 1.4.318)
    version: String,
}

struct VulkanInfo {
    drivers: HashMap<DeviceId, VkDriverInfo>,
}

impl VulkanInfo {
    pub fn init() -> anyhow::Result<VulkanInfo> {
        use ash::vk;
        let entry = unsafe { ash::Entry::load()? };
        let api_version =
            unsafe { entry.try_enumerate_instance_version() }?.unwrap_or(vk::API_VERSION_1_0);
        let application_info = vk::ApplicationInfo::default().api_version(api_version);
        let instance_info = vk::InstanceCreateInfo::default().application_info(&application_info);
        let instance = unsafe { entry.create_instance(&instance_info, None)? };

        let physical_devices = unsafe { instance.enumerate_physical_devices() }?;

        let mut drivers = HashMap::new();
        for physical_device in physical_devices.iter() {
            let mut driver_props = vk::PhysicalDeviceDriverProperties::default();
            let mut props2 = vk::PhysicalDeviceProperties2::default().push_next(&mut driver_props);
            unsafe { instance.get_physical_device_properties2(*physical_device, &mut props2) };

            let driver_props = if props2.p_next.is_null() {
                vk::PhysicalDeviceDriverProperties::default()
            } else {
                unsafe { *(props2.p_next as *const vk::PhysicalDeviceDriverProperties) }
            };

            if driver_props.driver_id == vk::DriverId::MESA_LLVMPIPE {
                continue;
            }

            let device_id = DeviceId {
                vendor: props2.properties.vendor_id as u16,
                device: props2.properties.device_id as u16,
            };
            let driver_info = VkDriverInfo {
                version: format!(
                    "{}.{}.{}",
                    vk::api_version_major(props2.properties.api_version),
                    vk::api_version_minor(props2.properties.api_version),
                    vk::api_version_patch(props2.properties.api_version)
                ),
                name: driver_props
                    .driver_name_as_c_str()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                driver_info: driver_props
                    .driver_info_as_c_str()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            };
            drivers.insert(device_id, driver_info);
        }

        Ok(VulkanInfo { drivers })
    }
}
