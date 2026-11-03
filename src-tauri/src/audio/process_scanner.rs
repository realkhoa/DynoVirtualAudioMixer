//! @file process_scanner.rs
//! @brief System process audio session discovery and process identification on Windows.
//! @details Enumerates WASAPI audio sessions, identifies running applications producing or
//! capable of producing audio (e.g., Spotify, Discord, web browsers, media players, games),
//! and retrieves their Process IDs, executable names, and window titles.

use crate::audio::capture::{linear_to_db, METER_SILENCE_DB};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[cfg(target_os = "windows")]
use windows::{
    core::Interface,
    Win32::{
        Foundation::{BOOL, HWND, LPARAM},
        Media::Audio::{
            eRender, Endpoints::IAudioMeterInformation, IAudioSessionControl2, IAudioSessionManager2,
            IMMDeviceEnumerator, MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
        },
        System::{
            Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED},
            Threading::{
                OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
                PROCESS_QUERY_LIMITED_INFORMATION,
            },
        },
        UI::WindowsAndMessaging::{
            EnumWindows, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
            IsWindowVisible,
        },
    },
};

/// @brief Metadata descriptor representing an audio-producing or audio-capable process.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AudioProcessDescriptor {
    /// @brief Operating system Process ID (PID).
    pub pid: u32,
    /// @brief Executable binary name (e.g., "spotify.exe", "discord.exe").
    pub name: String,
    /// @brief Main window or application title (e.g., "Spotify Free", "Discord").
    pub title: String,
    /// @brief Flag indicating if the process currently holds an active WASAPI audio session.
    pub has_audio_session: bool,
}

impl AudioProcessDescriptor {
    /// @brief Constructs a new AudioProcessDescriptor.
    /// @param pid Operating system Process ID.
    /// @param name Executable name.
    /// @param title Window or friendly title.
    /// @param has_audio_session Whether process has an active audio session.
    /// @return Initialized AudioProcessDescriptor.
    pub fn new(
        pid: u32,
        name: impl Into<String>,
        title: impl Into<String>,
        has_audio_session: bool,
    ) -> Self {
        Self {
            pid,
            name: name.into(),
            title: title.into(),
            has_audio_session,
        }
    }
}

/// @brief Real-time audio peak telemetry snapshot for active system applications.
#[derive(Debug, Default, Clone)]
pub struct ProcessAudioMeterSnapshot {
    /// @brief Peak levels indexed by process ID: `pid -> (peak_left_db, peak_right_db)`.
    pub peaks_by_pid: HashMap<u32, (f32, f32)>,
    /// @brief Peak levels indexed by normalized process name (e.g., "spotify", "chrome").
    pub peaks_by_name: HashMap<String, (f32, f32)>,
}

impl ProcessAudioMeterSnapshot {
    /// @brief Queries peak level for a specific process by PID or name.
    /// @param pid Optional target process ID.
    /// @param name Display or executable name of the process.
    /// @return Tuple containing (peak_left_db, peak_right_db).
    pub fn get_peak(&self, pid: Option<u32>, name: &str) -> (f32, f32) {
        if let Some(p) = pid {
            if let Some(&levels) = self.peaks_by_pid.get(&p) {
                return levels;
            }
        }

        let lower = name.to_lowercase();
        for (proc_name, &levels) in &self.peaks_by_name {
            if lower.contains(proc_name) || proc_name.contains(&lower) {
                return levels;
            }
        }

        (METER_SILENCE_DB, METER_SILENCE_DB)
    }
}

/// @brief Collects real-time audio peak levels across all active Windows WASAPI application sessions.
/// @return Populated ProcessAudioMeterSnapshot with peak decibels per PID and process name.
pub fn get_active_process_meter_snapshot() -> ProcessAudioMeterSnapshot {
    let mut snapshot = ProcessAudioMeterSnapshot::default();
    if std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok() {
        return snapshot;
    }

    #[cfg(target_os = "windows")]
    {
        let _com = ComGuard::new();

        unsafe {
            let enumerator: Result<IMMDeviceEnumerator, _> =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL);
            if let Ok(enumerator) = enumerator {
                if let Ok(collection) = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE) {
                    if let Ok(count) = collection.GetCount() {
                        for i in 0..count {
                            if let Ok(device) = collection.Item(i) {
                                if let Ok(manager) =
                                    device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)
                                {
                                    if let Ok(session_enum) = manager.GetSessionEnumerator() {
                                        if let Ok(session_count) = session_enum.GetCount() {
                                            for s in 0..session_count {
                                                if let Ok(session) = session_enum.GetSession(s) {
                                                    if let Ok(control2) =
                                                        session.cast::<IAudioSessionControl2>()
                                                    {
                                                        if let Ok(pid) = control2.GetProcessId() {
                                                            if pid > 0 {
                                                                let mut db_l = METER_SILENCE_DB;
                                                                let mut db_r = METER_SILENCE_DB;

                                                                if let Ok(meter) = session.cast::<IAudioMeterInformation>() {
                                                                    if let Ok(channel_count) = meter.GetMeteringChannelCount() {
                                                                        if channel_count > 0 {
                                                                            let mut peaks = vec![0.0f32; channel_count as usize];
                                                                            if meter.GetChannelsPeakValues(&mut peaks).is_ok() {
                                                                                let p_l = peaks[0];
                                                                                let p_r = if peaks.len() > 1 { peaks[1] } else { p_l };
                                                                                db_l = linear_to_db(p_l);
                                                                                db_r = linear_to_db(p_r);
                                                                            }
                                                                        }
                                                                    } else if let Ok(peak) = meter.GetPeakValue() {
                                                                        let db = linear_to_db(peak);
                                                                        db_l = db;
                                                                        db_r = db;
                                                                    }
                                                                }

                                                                snapshot.peaks_by_pid.entry(pid)
                                                                    .and_modify(|(cur_l, cur_r)| {
                                                                        if db_l > *cur_l { *cur_l = db_l; }
                                                                        if db_r > *cur_r { *cur_r = db_r; }
                                                                    })
                                                                    .or_insert((db_l, db_r));

                                                                let exe_name = get_process_name_by_pid(pid);
                                                                if !exe_name.is_empty() {
                                                                    let clean_name = exe_name
                                                                        .to_lowercase()
                                                                        .trim_end_matches(".exe")
                                                                        .to_string();
                                                                    snapshot.peaks_by_name.entry(clean_name)
                                                                        .and_modify(|(cur_l, cur_r)| {
                                                                            if db_l > *cur_l { *cur_l = db_l; }
                                                                            if db_r > *cur_r { *cur_r = db_r; }
                                                                        })
                                                                        .or_insert((db_l, db_r));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    snapshot
}

/// @brief RAII guard ensuring proper COM library initialization and cleanup.
#[cfg(target_os = "windows")]
struct ComGuard {
    /// @brief Indicates whether CoInitializeEx succeeded and requires CoUninitialize.
    initialized: bool,
}

#[cfg(target_os = "windows")]
impl ComGuard {
    /// @brief Initializes COM multithreaded runtime on the current thread.
    /// @return Newly constructed ComGuard instance.
    fn new() -> Self {
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        Self {
            initialized: hr.is_ok(),
        }
    }
}

#[cfg(target_os = "windows")]
impl Drop for ComGuard {
    /// @brief Cleans up COM runtime if successfully initialized by this guard.
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                CoUninitialize();
            }
        }
    }
}

/// @brief Checks whether an executable name corresponds to a known audio candidate application.
/// @param exe_name Binary name in lowercase.
/// @return `true` if recognized as a media/audio candidate, `false` otherwise.
pub fn is_audio_candidate_application(exe_name: &str) -> bool {
    let lower = exe_name.to_lowercase();
    lower.contains("spotify")
        || lower.contains("discord")
        || lower.contains("chrome")
        || lower.contains("firefox")
        || lower.contains("msedge")
        || lower.contains("opera")
        || lower.contains("brave")
        || lower.contains("vlc")
        || lower.contains("foobar2000")
        || lower.contains("steam")
        || lower.contains("obs64")
        || lower.contains("obs32")
        || lower.contains("itunes")
        || lower.contains("music")
        || lower.contains("audacity")
        || lower.contains("game")
}

#[cfg(target_os = "windows")]
/// @brief Resolves executable file name from process ID using Windows API.
/// @param pid Target process identifier.
/// @return Executable file name string or empty string on failure.
fn get_process_name_by_pid(pid: u32) -> String {
    if pid == 0 {
        return "System".to_string();
    }

    unsafe {
        let handle_res = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
        if let Ok(handle) = handle_res {
            if !handle.is_invalid() {
                let mut buffer = [0u16; 1024];
                let mut size = buffer.len() as u32;
                if QueryFullProcessImageNameW(
                    handle,
                    PROCESS_NAME_FORMAT(0),
                    windows::core::PWSTR(buffer.as_mut_ptr()),
                    &mut size,
                )
                .is_ok()
                {
                    let path = String::from_utf16_lossy(&buffer[..size as usize]);
                    if let Some(filename) = path.rsplit('\\').next() {
                        return filename.to_string();
                    }
                }
            }
        }
    }
    String::new()
}

#[cfg(target_os = "windows")]
/// @brief Callback data container for EnumWindows API.
struct WindowEnumContext {
    /// @brief Map of PID to highest priority non-empty window title.
    titles: HashMap<u32, String>,
}

#[cfg(target_os = "windows")]
/// @brief Native callback function invoked by EnumWindows.
/// @param hwnd Window handle.
/// @param lparam User data pointer cast to WindowEnumContext.
/// @return TRUE to continue enumeration.
extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        if IsWindowVisible(hwnd).as_bool() {
            let length = GetWindowTextLengthW(hwnd);
            if length > 0 {
                let mut title_buf = vec![0u16; (length + 1) as usize];
                let copied = GetWindowTextW(hwnd, &mut title_buf);
                if copied > 0 {
                    let title = String::from_utf16_lossy(&title_buf[..copied as usize]);
                    let trimmed = title.trim();
                    if !trimmed.is_empty() {
                        let mut pid = 0u32;
                        GetWindowThreadProcessId(hwnd, Some(&mut pid));
                        if pid > 0 {
                            let context = &mut *(lparam.0 as *mut WindowEnumContext);
                            context.titles.entry(pid).or_insert_with(|| trimmed.to_string());
                        }
                    }
                }
            }
        }
    }
    BOOL(1)
}

#[cfg(target_os = "windows")]
/// @brief Collects all active WASAPI audio session PIDs across active render endpoints.
/// @param session_pids Target set to populate with PIDs holding active audio sessions.
fn collect_wasapi_audio_session_pids(session_pids: &mut HashSet<u32>) {
    if std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok() {
        return;
    }

    let _com = ComGuard::new();

    unsafe {
        let enumerator: Result<IMMDeviceEnumerator, _> =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL);
        if let Ok(enumerator) = enumerator {
            if let Ok(collection) = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE) {
                if let Ok(count) = collection.GetCount() {
                    for i in 0..count {
                        if let Ok(device) = collection.Item(i) {
                            if let Ok(manager) =
                                device.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None)
                            {
                                if let Ok(session_enum) = manager.GetSessionEnumerator() {
                                    if let Ok(session_count) = session_enum.GetCount() {
                                        for s in 0..session_count {
                                            if let Ok(session) = session_enum.GetSession(s) {
                                                if let Ok(control2) =
                                                    session.cast::<IAudioSessionControl2>()
                                                {
                                                    if let Ok(pid) = control2.GetProcessId() {
                                                        if pid > 0 {
                                                            session_pids.insert(pid);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// @brief Scans active audio processes and applications running on the system.
/// @details On Windows, interrogates WASAPI endpoint managers to detect active audio sessions,
/// cross-references them against running processes and application windows, and includes
/// active multimedia/communication applications even when temporarily silent.
/// On non-Windows platforms or headless environments, returns a clean list or mock entries.
/// @return `Ok(Vec<AudioProcessDescriptor>)` listing detected processes, or `Err(String)` on critical failure.
pub fn list_audio_processes() -> Result<Vec<AudioProcessDescriptor>, String> {
    if std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok() {
        return Ok(vec![
            AudioProcessDescriptor::new(1001, "spotify.exe", "Spotify Free", true),
            AudioProcessDescriptor::new(1002, "discord.exe", "Discord", false),
        ]);
    }

    #[cfg(target_os = "windows")]
    {
        let mut session_pids = HashSet::new();
        collect_wasapi_audio_session_pids(&mut session_pids);

        // Enumerate visible window titles mapped by PID
        let mut window_context = WindowEnumContext {
            titles: HashMap::new(),
        };
        unsafe {
            let context_ptr = &mut window_context as *mut WindowEnumContext;
            let _ = EnumWindows(Some(enum_windows_callback), LPARAM(context_ptr as isize));
        }

        let mut processes = HashMap::new();

        // Add all processes detected with active WASAPI audio sessions
        for &pid in &session_pids {
            let name = get_process_name_by_pid(pid);
            let display_name = if name.is_empty() {
                format!("Process-{}", pid)
            } else {
                name
            };
            let title = window_context
                .titles
                .get(&pid)
                .cloned()
                .unwrap_or_else(|| display_name.clone());

            processes.insert(
                pid,
                AudioProcessDescriptor {
                    pid,
                    name: display_name,
                    title,
                    has_audio_session: true,
                },
            );
        }

        // Add running GUI application processes with window titles or audio candidates
        for (&pid, title) in &window_context.titles {
            let name = get_process_name_by_pid(pid);
            if !name.is_empty() {
                let has_audio = session_pids.contains(&pid);
                let is_candidate = is_audio_candidate_application(&name);

                // Keep processes that have audio sessions, are candidates, or have substantial titles
                if has_audio || is_candidate || !title.is_empty() {
                    processes.entry(pid).or_insert_with(|| AudioProcessDescriptor {
                        pid,
                        name: name.clone(),
                        title: title.clone(),
                        has_audio_session: has_audio,
                    });
                }
            }
        }

        let mut result: Vec<AudioProcessDescriptor> = processes.into_values().collect();
        result.sort_by(|a, b| {
            b.has_audio_session
                .cmp(&a.has_audio_session)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });

        Ok(result)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(Vec::new())
    }
}
