//! @file virtual_device.rs
//! @brief Virtual audio device adapter and software cable manager for Dyno Virtual Audio Mixer.
//! @details Facilitates discovery, registration, and management of virtual audio endpoints,
//! bridging third-party virtual cable drivers (VB-Audio CABLE, Virtual Audio Cable) as well as
//! fallback internal software loopback endpoints for zero-hardware WASAPI routing.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

use crate::audio::devices::list_audio_devices;

/// @brief Supported driver implementations and virtualization modes for audio cables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VirtualCableType {
    /// @brief VB-Audio Virtual Cable (CABLE Input / CABLE Output).
    VBCable,
    /// @brief Eugene Muzychenko's Virtual Audio Cable (VAC).
    VirtualAudioCable,
    /// @brief Internal software loopback audio endpoint managed directly by Dyno WASAPI mixer.
    SoftwareLoopbackEndpoint,
}

impl VirtualCableType {
    /// @brief Retrieves the human-readable display label for the cable type.
    /// @return Static string slice with the driver name.
    #[inline]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::VBCable => "VB-Audio Cable",
            Self::VirtualAudioCable => "Virtual Audio Cable",
            Self::SoftwareLoopbackEndpoint => "Dyno Software Loopback",
        }
    }

    /// @brief Determines if this cable type relies on an external kernel/WDM audio driver.
    /// @return `true` if backed by physical or installed driver, `false` if purely software.
    #[inline]
    pub fn is_physical_driver(&self) -> bool {
        matches!(self, Self::VBCable | Self::VirtualAudioCable)
    }

    /// @brief Determines if this is an internal software loopback endpoint.
    /// @return `true` if SoftwareLoopbackEndpoint, `false` otherwise.
    #[inline]
    pub fn is_software_loopback(&self) -> bool {
        matches!(self, Self::SoftwareLoopbackEndpoint)
    }
}

/// @brief Metadata descriptor representing an active virtual cable or loopback endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VirtualDeviceDescriptor {
    /// @brief Unique device identifier (e.g., "vdev-discord-aux-1").
    pub id: String,
    /// @brief Friendly display name of the virtual audio device.
    pub name: String,
    /// @brief Driver technology powering this virtual endpoint.
    pub driver_type: VirtualCableType,
    /// @brief System audio endpoint ID used for rendering/playback into this cable.
    pub render_endpoint_id: Option<String>,
    /// @brief System audio endpoint ID used for capturing/recording from this cable.
    pub capture_endpoint_id: Option<String>,
    /// @brief Number of audio channels supported (e.g. 2 for stereo).
    pub channels: u16,
    /// @brief Sample rate in Hertz (e.g., 48000).
    pub sample_rate: u32,
    /// @brief Whether this virtual device is active and accepting audio routing.
    pub is_active: bool,
}

impl VirtualDeviceDescriptor {
    /// @brief Constructs a new VirtualDeviceDescriptor instance.
    /// @param id Unique virtual device identifier.
    /// @param name Friendly display name.
    /// @param driver_type Driver classification.
    /// @param render_endpoint_id Associated WASAPI playback endpoint ID.
    /// @param capture_endpoint_id Associated WASAPI capture endpoint ID.
    /// @param channels Number of audio channels.
    /// @param sample_rate Native sample rate in Hertz.
    /// @param is_active Initial active status.
    /// @return Newly constructed VirtualDeviceDescriptor.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        driver_type: VirtualCableType,
        render_endpoint_id: Option<String>,
        capture_endpoint_id: Option<String>,
        channels: u16,
        sample_rate: u32,
        is_active: bool,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            driver_type,
            render_endpoint_id,
            capture_endpoint_id,
            channels,
            sample_rate,
            is_active,
        }
    }

    /// @brief Checks whether the virtual device is a software loopback endpoint.
    /// @return `true` if software loopback, `false` otherwise.
    #[inline]
    pub fn is_software_loopback(&self) -> bool {
        self.driver_type.is_software_loopback()
    }

    /// @brief Toggles the active status of the virtual device.
    /// @param active New active state flag.
    #[inline]
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }
}

/// @brief Helper to generate a sanitized slug from a string.
/// @param name Input string to slugify.
/// @return Lowercased alphanumeric string with hyphens.
fn slugify(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// @brief Manager for creating, enumerating, configuring, and removing virtual audio cables.
#[derive(Debug, Clone, Default)]
pub struct VirtualDeviceManager {
    /// @brief Registry of active virtual devices indexed by unique identifier.
    devices: BTreeMap<String, VirtualDeviceDescriptor>,
    /// @brief Cached list of detected available drivers on the current system.
    available_drivers: Vec<VirtualCableType>,
    /// @brief Monotonically increasing counter for collision-free ID generation.
    counter: usize,
}

impl VirtualDeviceManager {
    /// @brief Creates a new VirtualDeviceManager instance and detects host drivers.
    /// @details Automatically inspects host audio endpoints via CPAL/WASAPI to detect installed
    /// third-party drivers (such as VB-Cable or Virtual Audio Cable), always ensuring
    /// SoftwareLoopbackEndpoint is registered as a fallback.
    /// @return Newly initialized VirtualDeviceManager.
    pub fn new() -> Self {
        let mut manager = Self {
            devices: BTreeMap::new(),
            available_drivers: Vec::new(),
            counter: 0,
        };
        manager.refresh_available_drivers();
        manager
    }

    /// @brief Scans system audio devices and updates the list of available driver types.
    /// @details Queries all host devices. Matches driver signatures and guarantees that
    /// SoftwareLoopbackEndpoint is always included.
    /// @return Cloned slice of available driver types.
    pub fn refresh_available_drivers(&mut self) -> Vec<VirtualCableType> {
        let mut drivers = Vec::new();
        if let Ok(devices) = list_audio_devices() {
            let mut has_vb = false;
            let mut has_vac = false;

            for dev in devices {
                let lower = dev.name.to_lowercase();
                if lower.contains("vb-audio")
                    || lower.contains("cable input")
                    || lower.contains("cable output")
                    || lower.contains("cable-a")
                    || lower.contains("cable-b")
                {
                    has_vb = true;
                }
                if lower.contains("virtual audio cable") || lower.contains("vac ") {
                    has_vac = true;
                }
            }

            if has_vb {
                drivers.push(VirtualCableType::VBCable);
            }
            if has_vac {
                drivers.push(VirtualCableType::VirtualAudioCable);
            }
        }

        if !drivers.contains(&VirtualCableType::SoftwareLoopbackEndpoint) {
            drivers.push(VirtualCableType::SoftwareLoopbackEndpoint);
        }

        self.available_drivers = drivers.clone();
        drivers
    }

    /// @brief Returns the list of currently detected available virtual cable drivers.
    /// @return Vector of detected `VirtualCableType` variants.
    pub fn detect_available_drivers(&self) -> Vec<VirtualCableType> {
        self.available_drivers.clone()
    }

    /// @brief Creates a new virtual device descriptor and registers it in the manager.
    /// @details Validates the device name, resolves duplicate names automatically by appending
    /// a numerical index, binds discovered hardware endpoints or allocates synthetic loopback
    /// endpoints, and activates the virtual device.
    /// @param name User-assigned friendly name (must not be empty or whitespace-only).
    /// @param cable_type Virtual cable driver type to instantiate.
    /// @return `Ok(VirtualDeviceDescriptor)` on success, or `Err(String)` on validation failure.
    pub fn create_virtual_device(
        &mut self,
        name: &str,
        cable_type: VirtualCableType,
    ) -> Result<VirtualDeviceDescriptor, String> {
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return Err("Virtual device name cannot be empty or whitespace-only".to_string());
        }

        self.counter += 1;
        let final_name = self.disambiguate_name(trimmed_name);
        let slug = slugify(&final_name);
        let id = format!(
            "vdev-{}-{}",
            if slug.is_empty() { "cable" } else { &slug },
            self.counter
        );

        let (render_endpoint_id, capture_endpoint_id) = self.resolve_endpoints(&id, cable_type);

        let descriptor = VirtualDeviceDescriptor {
            id: id.clone(),
            name: final_name,
            driver_type: cable_type,
            render_endpoint_id,
            capture_endpoint_id,
            channels: 2,
            sample_rate: 48000,
            is_active: true,
        };

        self.devices.insert(id, descriptor.clone());
        Ok(descriptor)
    }

    /// @brief Removes a registered virtual audio device by its unique identifier.
    /// @param device_id Unique ID of the virtual device to remove.
    /// @return `Ok(true)` if removed, `Ok(false)` if not found.
    pub fn remove_virtual_device(&mut self, device_id: &str) -> Result<bool, String> {
        Ok(self.devices.remove(device_id).is_some())
    }

    /// @brief Retrieves a list of all currently registered virtual audio devices.
    /// @return Vector of `VirtualDeviceDescriptor` items.
    pub fn list_devices(&self) -> Vec<VirtualDeviceDescriptor> {
        self.devices.values().cloned().collect()
    }

    /// @brief Retrieves an immutable reference to a virtual device descriptor by ID.
    /// @param device_id Unique device identifier to look up.
    /// @return `Some(&VirtualDeviceDescriptor)` if present, `None` otherwise.
    pub fn get_device(&self, device_id: &str) -> Option<&VirtualDeviceDescriptor> {
        self.devices.get(device_id)
    }

    /// @brief Retrieves a mutable reference to a virtual device descriptor by ID.
    /// @param device_id Unique device identifier to look up.
    /// @return `Some(&mut VirtualDeviceDescriptor)` if present, `None` otherwise.
    pub fn get_device_mut(&mut self, device_id: &str) -> Option<&mut VirtualDeviceDescriptor> {
        self.devices.get_mut(device_id)
    }

    /// @brief Toggles the active status of a registered virtual device.
    /// @param device_id Unique identifier of the target virtual device.
    /// @param active New active state flag.
    /// @return `Ok(true)` if updated, `Ok(false)` if device was not found.
    pub fn set_device_active(&mut self, device_id: &str, active: bool) -> Result<bool, String> {
        if let Some(device) = self.devices.get_mut(device_id) {
            device.set_active(active);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// @brief Disambiguates duplicate device names by appending incremental counters.
    /// @param base_name Base candidate name requested by the caller.
    /// @return Unique string name not currently in use by any registered device.
    fn disambiguate_name(&self, base_name: &str) -> String {
        let existing_names: Vec<&str> = self.devices.values().map(|d| d.name.as_str()).collect();
        if !existing_names.contains(&base_name) {
            return base_name.to_string();
        }

        let mut index = 2;
        loop {
            let candidate = format!("{base_name} ({index})");
            if !existing_names.contains(&candidate.as_str()) {
                return candidate;
            }
            index += 1;
        }
    }

    /// @brief Resolves render and capture endpoints for a newly created virtual device.
    /// @param id Unique virtual device ID used as fallback synthetic identifier.
    /// @param cable_type Target cable driver type.
    /// @return Tuple containing optional render and capture endpoint identifiers.
    fn resolve_endpoints(
        &self,
        id: &str,
        cable_type: VirtualCableType,
    ) -> (Option<String>, Option<String>) {
        match cable_type {
            VirtualCableType::SoftwareLoopbackEndpoint => (
                Some(format!("{id}-sink")),
                Some(format!("{id}-source")),
            ),
            VirtualCableType::VBCable => {
                let mut render_id = None;
                let mut capture_id = None;
                if let Ok(devices) = list_audio_devices() {
                    for dev in &devices {
                        let lower = dev.name.to_lowercase();
                        if dev.device_type.is_output()
                            && (lower.contains("cable input") || lower.contains("vb-audio"))
                            && render_id.is_none()
                        {
                            render_id = Some(dev.id.clone());
                        } else if dev.device_type.is_input()
                            && (lower.contains("cable output") || lower.contains("vb-audio"))
                            && capture_id.is_none()
                        {
                            capture_id = Some(dev.id.clone());
                        }
                    }
                }
                (
                    render_id.or_else(|| Some(format!("{id}-vbcable-render"))),
                    capture_id.or_else(|| Some(format!("{id}-vbcable-capture"))),
                )
            }
            VirtualCableType::VirtualAudioCable => {
                let mut render_id = None;
                let mut capture_id = None;
                if let Ok(devices) = list_audio_devices() {
                    for dev in &devices {
                        let lower = dev.name.to_lowercase();
                        if dev.device_type.is_output()
                            && (lower.contains("line ") || lower.contains("virtual audio cable"))
                            && render_id.is_none()
                        {
                            render_id = Some(dev.id.clone());
                        } else if dev.device_type.is_input()
                            && (lower.contains("line ") || lower.contains("virtual audio cable"))
                            && capture_id.is_none()
                        {
                            capture_id = Some(dev.id.clone());
                        }
                    }
                }
                (
                    render_id.or_else(|| Some(format!("{id}-vac-render"))),
                    capture_id.or_else(|| Some(format!("{id}-vac-capture"))),
                )
            }
        }
    }
}
