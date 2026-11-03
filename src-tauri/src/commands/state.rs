//! @file state.rs
//! @brief Thread-safe application state container and real-time peak meter telemetry extractor.
//! @details Holds reference-counted read-write locks for the active audio routing matrix and virtual
//! device manager, physical microphone capture worker, atomic flags controlling audio processing,
//! and 60 FPS peak meter broadcast telemetry.

use crate::audio::capture::{spawn_microphone_capture_thread, LiveMeterTracker};
use crate::audio::matrix::RoutingMatrix;
use crate::audio::types::MIN_PEAK_DB;
use crate::audio::virtual_device::VirtualDeviceManager;
use crate::commands::dto::{MeterUpdateDto, NodeMeterDto};
use crate::presets::ProfileManager;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::{SystemTime, UNIX_EPOCH};

/// @brief Thread-safe application state shared across Tauri command invocations and background tasks.
#[derive(Clone)]
pub struct AppState {
    /// @brief Thread-safe handle to the active audio routing matrix.
    pub matrix: Arc<RwLock<RoutingMatrix>>,
    /// @brief Thread-safe handle to the virtual audio cable device manager.
    pub virtual_devices: Arc<RwLock<VirtualDeviceManager>>,
    /// @brief Atomic flag indicating whether the audio processing and telemetry engine is running.
    pub is_running: Arc<AtomicBool>,
    /// @brief Thread-safe handle to the preset profiles manager.
    pub profiles: Arc<RwLock<ProfileManager>>,
    /// @brief Lock-free real-time peak meter tracker updated from the CPAL audio callback thread.
    pub mic_tracker: Arc<LiveMeterTracker>,
}

impl Default for AppState {
    /// @brief Constructs a default AppState with empty routing matrix and initialized device manager.
    /// @return Default AppState instance.
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    /// @brief Creates a new AppState instance with initialized routing matrix, virtual device manager, and live mic capture.
    /// @return Initialized AppState instance.
    pub fn new() -> Self {
        let mic_tracker = Arc::new(LiveMeterTracker::new());
        let is_running = Arc::new(AtomicBool::new(true));

        // Start live WASAPI microphone capture worker thread
        spawn_microphone_capture_thread(mic_tracker.clone(), is_running.clone());

        Self {
            matrix: Arc::new(RwLock::new(RoutingMatrix::new())),
            virtual_devices: Arc::new(RwLock::new(VirtualDeviceManager::new())),
            is_running,
            profiles: Arc::new(RwLock::new(ProfileManager::default())),
            mic_tracker,
        }
    }

    /// @brief Constructs an AppState instance with specific matrix and virtual device manager components.
    /// @param matrix Initialized RoutingMatrix.
    /// @param virtual_devices Initialized VirtualDeviceManager.
    /// @return Initialized AppState instance.
    pub fn with_matrix_and_devices(
        matrix: RoutingMatrix,
        virtual_devices: VirtualDeviceManager,
    ) -> Self {
        let mic_tracker = Arc::new(LiveMeterTracker::new());
        let is_running = Arc::new(AtomicBool::new(true));

        Self {
            matrix: Arc::new(RwLock::new(matrix)),
            virtual_devices: Arc::new(RwLock::new(virtual_devices)),
            is_running,
            profiles: Arc::new(RwLock::new(ProfileManager::default())),
            mic_tracker,
        }
    }

    /// @brief Sets a custom ProfileManager for this AppState instance.
    /// @param manager Initialized ProfileManager with custom base directory.
    /// @return Modified AppState instance.
    pub fn with_profile_manager(self, manager: ProfileManager) -> Self {
        Self {
            matrix: self.matrix,
            virtual_devices: self.virtual_devices,
            is_running: self.is_running,
            profiles: Arc::new(RwLock::new(manager)),
            mic_tracker: self.mic_tracker,
        }
    }

    /// @brief Acquires an exclusive write lock on the underlying profile manager.
    /// @return `Ok(RwLockWriteGuard<ProfileManager>)` on success, or `Err(String)` on lock failure.
    pub fn write_profiles(&self) -> Result<RwLockWriteGuard<'_, ProfileManager>, String> {
        self.profiles
            .write()
            .map_err(|e| format!("Failed to acquire write lock on profile manager: {e}"))
    }

    /// @brief Acquires a shared read lock on the underlying profile manager.
    /// @return `Ok(RwLockReadGuard<ProfileManager>)` on success, or `Err(String)` on lock failure.
    pub fn read_profiles(&self) -> Result<RwLockReadGuard<'_, ProfileManager>, String> {
        self.profiles
            .read()
            .map_err(|e| format!("Failed to acquire read lock on profile manager: {e}"))
    }

    /// @brief Acquires an exclusive write lock on the underlying routing matrix.
    /// @return `Ok(RwLockWriteGuard<RoutingMatrix>)` on success, or `Err(String)` on lock failure.
    pub fn write_matrix(&self) -> Result<RwLockWriteGuard<'_, RoutingMatrix>, String> {
        self.matrix
            .write()
            .map_err(|e| format!("Failed to acquire write lock on routing matrix: {e}"))
    }

    /// @brief Acquires a shared read lock on the underlying routing matrix.
    /// @return `Ok(RwLockReadGuard<RoutingMatrix>)` on success, or `Err(String)` on lock failure.
    pub fn read_matrix(&self) -> Result<RwLockReadGuard<'_, RoutingMatrix>, String> {
        self.matrix
            .read()
            .map_err(|e| format!("Failed to acquire read lock on routing matrix: {e}"))
    }

    /// @brief Acquires an exclusive write lock on the virtual device manager.
    /// @return `Ok(RwLockWriteGuard<VirtualDeviceManager>)` on success, or `Err(String)` on lock failure.
    pub fn write_virtual_devices(&self) -> Result<RwLockWriteGuard<'_, VirtualDeviceManager>, String> {
        self.virtual_devices
            .write()
            .map_err(|e| format!("Failed to acquire write lock on virtual devices: {e}"))
    }

    /// @brief Acquires a shared read lock on the virtual device manager.
    /// @return `Ok(RwLockReadGuard<VirtualDeviceManager>)` on success, or `Err(String)` on lock failure.
    pub fn read_virtual_devices(&self) -> Result<RwLockReadGuard<'_, VirtualDeviceManager>, String> {
        self.virtual_devices
            .read()
            .map_err(|e| format!("Failed to acquire read lock on virtual devices: {e}"))
    }

    /// @brief Generates a real-time peak meter snapshot for all registered nodes in the matrix.
    /// @details Queries current live microphone hardware readings, evaluates wire connections
    /// across the routing matrix, scales by volume faders and mute flags, and applies smooth decay ballistics.
    /// @return MeterUpdateDto with current timestamp and node peak readings.
    pub fn generate_meter_snapshot(&self) -> MeterUpdateDto {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let matrix_guard = match self.matrix.read() {
            Ok(g) => g,
            Err(_) => {
                return MeterUpdateDto {
                    timestamp_ms,
                    meters: vec![],
                }
            }
        };

        // Extract and decay the live hardware microphone peak decibels (decay 1.2 dB per 60 FPS tick)
        let (mic_db_l, mic_db_r) = self.mic_tracker.get_and_decay(1.2);

        // Retrieve real-time active Windows application audio peak snapshot
        let process_meters = crate::audio::process_scanner::get_active_process_meter_snapshot();

        // Precalculate source node outputs (peak dB)
        let mut source_peaks: HashMap<String, (f32, f32)> = HashMap::new();

        for (node_id, node) in matrix_guard.nodes() {
            if node.input_channels == 0 {
                let (raw_l, raw_r) = if node.muted || node.is_offline || node.volume <= 0.001 {
                    (MIN_PEAK_DB, MIN_PEAK_DB)
                } else {
                    let vol_gain_db = 20.0 * node.volume.log10();

                    // Determine independent audio level depending on node classification:
                    let (src_l, src_r) = if node.is_microphone() {
                        // Physical hardware microphone input
                        (mic_db_l, mic_db_r)
                    } else if node.is_application() {
                        // Application audio session (Spotify, Discord, Chrome, games)
                        process_meters.get_peak(node.pid, &node.name)
                    } else {
                        // Silent or unidentified source without active stream
                        (MIN_PEAK_DB, MIN_PEAK_DB)
                    };

                    (
                        (src_l + vol_gain_db).clamp(MIN_PEAK_DB, 0.0),
                        (src_r + vol_gain_db).clamp(MIN_PEAK_DB, 0.0),
                    )
                };
                source_peaks.insert(node_id.clone(), (raw_l, raw_r));
            }
        }

        let mut meters = Vec::with_capacity(matrix_guard.nodes().len());

        for (node_id, node) in matrix_guard.nodes() {
            let (peak_left, peak_right) = if node.muted || node.is_offline || node.volume <= 0.001 {
                (MIN_PEAK_DB, MIN_PEAK_DB)
            } else if let Some(&(src_l, src_r)) = source_peaks.get(node_id) {
                // Source node uses its calculated peak directly
                (src_l, src_r)
            } else {
                // Destination / Virtual / Output node: calculate peak sum of incoming connected wires
                let mut max_in_l = MIN_PEAK_DB;
                let mut max_in_r = MIN_PEAK_DB;

                for wire in matrix_guard.wires().values() {
                    if wire.destination.node_id == *node_id && !wire.muted {
                        if let Some(&(src_l, src_r)) = source_peaks.get(&wire.source.node_id) {
                            let wire_gain_db = if wire.gain <= 0.001 {
                                MIN_PEAK_DB
                            } else {
                                20.0 * wire.gain.log10()
                            };
                            let in_signal = if wire.source.channel_index == 0 {
                                src_l + wire_gain_db
                            } else {
                                src_r + wire_gain_db
                            };

                            if wire.destination.channel_index == 0 {
                                if in_signal > max_in_l {
                                    max_in_l = in_signal;
                                }
                            } else if in_signal > max_in_r {
                                max_in_r = in_signal;
                            }
                        }
                    }
                }

                // If mono destination, mirror Left to Right
                if node.input_channels == 1 && max_in_r == MIN_PEAK_DB {
                    max_in_r = max_in_l;
                }

                let vol_gain_db = 20.0 * node.volume.log10();
                (
                    (max_in_l + vol_gain_db).clamp(MIN_PEAK_DB, 0.0),
                    (max_in_r + vol_gain_db).clamp(MIN_PEAK_DB, 0.0),
                )
            };

            meters.push(NodeMeterDto {
                node_id: node_id.clone(),
                peak_left,
                peak_right,
            });
        }

        meters.sort_by(|a, b| a.node_id.cmp(&b.node_id));
        MeterUpdateDto {
            timestamp_ms,
            meters,
        }
    }
}
