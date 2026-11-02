//! @file state.rs
//! @brief Thread-safe application state container and real-time peak meter telemetry extractor.
//! @details Holds reference-counted read-write locks for the active audio routing matrix and virtual
//! device manager, plus atomic flags controlling audio processing and background meter broadcasting.

use crate::audio::matrix::RoutingMatrix;
use crate::audio::types::MIN_PEAK_DB;
use crate::audio::virtual_device::VirtualDeviceManager;
use crate::commands::dto::{MeterUpdateDto, NodeMeterDto};
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
}

impl Default for AppState {
    /// @brief Constructs a default AppState with empty routing matrix and initialized device manager.
    /// @return Default AppState instance.
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    /// @brief Creates a new AppState instance with initialized routing matrix and virtual device manager.
    /// @return Initialized AppState instance.
    pub fn new() -> Self {
        Self {
            matrix: Arc::new(RwLock::new(RoutingMatrix::new())),
            virtual_devices: Arc::new(RwLock::new(VirtualDeviceManager::new())),
            is_running: Arc::new(AtomicBool::new(true)),
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
        Self {
            matrix: Arc::new(RwLock::new(matrix)),
            virtual_devices: Arc::new(RwLock::new(virtual_devices)),
            is_running: Arc::new(AtomicBool::new(true)),
        }
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
    /// @details Queries current node volume, muting states, and channel layouts. Nodes that
    /// are muted or have volume zero return digital silence floor (-60.0 dB).
    /// @return MeterUpdateDto with current timestamp and node peak readings.
    pub fn generate_meter_snapshot(&self) -> MeterUpdateDto {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let matrix_guard = self.matrix.read().unwrap();
        let mut meters = Vec::with_capacity(matrix_guard.nodes().len());

        for (node_id, node) in matrix_guard.nodes() {
            let (peak_left, peak_right) = if node.muted || node.volume <= 0.0 {
                (MIN_PEAK_DB, MIN_PEAK_DB)
            } else {
                let db = if node.volume <= 0.001 {
                    MIN_PEAK_DB
                } else {
                    (20.0 * node.volume.min(1.0).log10()).clamp(MIN_PEAK_DB, 0.0)
                };
                (db, db)
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
