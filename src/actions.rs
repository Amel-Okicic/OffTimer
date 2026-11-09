#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction {
    Shutdown,
    Sleep,
    SignOut,
    Restart,
}

impl PowerAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Shutdown => "SHUT DOWN",
            Self::Sleep => "SLEEP",
            Self::SignOut => "SIGN OUT",
            Self::Restart => "RESTART",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerClockSnapshot {
    uptime_ms: u64,
    awake_time_100ns: u64,
}

impl PowerClockSnapshot {
    /// Returns the time spent in sleep/hibernation between two Windows clock snapshots.
    pub fn sleep_duration_since(self, previous: Self) -> std::time::Duration {
        let elapsed_including_sleep_ms = self.uptime_ms.saturating_sub(previous.uptime_ms);
        let elapsed_awake_ms = self
            .awake_time_100ns
            .saturating_sub(previous.awake_time_100ns)
            / 10_000;

        std::time::Duration::from_millis(
            elapsed_including_sleep_ms.saturating_sub(elapsed_awake_ms),
        )
    }
}

#[cfg(windows)]
pub struct SingleInstanceGuard {
    handle: windows_sys::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(self.handle);
        }
    }
}

#[cfg(windows)]
mod windows {
    use super::{PowerAction, PowerClockSnapshot, SingleInstanceGuard};
    use std::{mem, ptr, thread, time::Duration};

    use windows_sys::Win32::{
        Foundation::{
            CloseHandle, GetLastError, LUID, SetLastError, HANDLE, HWND, ERROR_ALREADY_EXISTS,
        },
        Security::{
            AdjustTokenPrivileges, LookupPrivilegeValueW, LUID_AND_ATTRIBUTES,
            SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
        },
        System::{
            LibraryLoader::GetModuleHandleW,
            Shutdown::{
                AbortSystemShutdownW, InitiateSystemShutdownExW,
                SHTDN_REASON_FLAG_PLANNED, SHTDN_REASON_MAJOR_APPLICATION,
                SHTDN_REASON_MINOR_MAINTENANCE,
            },
            SystemInformation::GetTickCount64,
            Threading::{
                CreateMutexW, GetCurrentProcess, GetCurrentProcessId, OpenProcess,
                OpenProcessToken, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
            },
            WindowsProgramming::QueryUnbiasedInterruptTime,
        },
        UI::{
            Shell::{
                Shell_NotifyIconW, NOTIFYICONDATAW, NIF_ICON, NIF_INFO, NIF_TIP,
                NIIF_INFO, NIM_ADD, NIM_DELETE, NIM_MODIFY,
            },
            WindowsAndMessaging::{
                FindWindowExW, GetWindowThreadProcessId, LoadIconW, MessageBoxW,
                SetForegroundWindow, ShowWindow, MB_ICONERROR, MB_OK, SW_HIDE, SW_RESTORE,
            },
        },
    };

    const TRAY_ICON_ID: u32 = 1;
    const ERROR_NOT_ALL_ASSIGNED: u32 = 1300;
    const ERROR_NO_SHUTDOWN_IN_PROGRESS: u32 = 1116;

    pub fn power_clock_snapshot() -> Option<PowerClockSnapshot> {
        unsafe {
            let mut awake_time_100ns = 0u64;
            if QueryUnbiasedInterruptTime(&mut awake_time_100ns) == 0 {
                return None;
            }

            Some(PowerClockSnapshot {
                uptime_ms: GetTickCount64(),
                awake_time_100ns,
            })
        }
    }

    pub fn acquire_single_instance() -> Result<Option<SingleInstanceGuard>, String> {
        let name = wide(r"Local\OffTimer.SingleInstance");

        unsafe {
            SetLastError(0);
            let handle = CreateMutexW(ptr::null(), 0, name.as_ptr());
            if handle.is_null() {
                return Err(last_error("Could not create the OffTimer single-instance lock"));
            }

            if GetLastError() == ERROR_ALREADY_EXISTS {
                CloseHandle(handle);
                return Ok(None);
            }

            Ok(Some(SingleInstanceGuard { handle }))
        }
    }

    fn is_offtimer_process(hwnd: HWND) -> bool {
        let mut process_id = 0u32;
        unsafe {
            GetWindowThreadProcessId(hwnd, &mut process_id);
        }
        if process_id == 0 {
            return false;
        }

        let Some(expected_executable) = std::env::current_exe()
            .ok()
            .and_then(|path| path.file_name().map(|name| name.to_string_lossy().into_owned()))
        else {
            return false;
        };

        let process = unsafe {
            OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id)
        };
        if process.is_null() {
            return false;
        }

        let mut image_path = vec![0u16; 32_768];
        let mut image_path_length = image_path.len() as u32;
        let succeeded = unsafe {
            QueryFullProcessImageNameW(
                process,
                0,
                image_path.as_mut_ptr(),
                &mut image_path_length,
            )
        };
        unsafe {
            CloseHandle(process);
        }
        if succeeded == 0 {
            return false;
        }

        let image_path = String::from_utf16_lossy(&image_path[..image_path_length as usize]);
        std::path::Path::new(&image_path)
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case(&expected_executable))
    }

    pub fn activate_existing_instance() {
        let title = wide("OffTimer");

        // The first process creates its mutex before the native window exists.
        // Give its startup a short window to create the window before looking it up.
        for _ in 0..30 {
            let mut previous_window: HWND = ptr::null_mut();
            loop {
                let hwnd = unsafe {
                    FindWindowExW(
                        ptr::null_mut(),
                        previous_window,
                        ptr::null(),
                        title.as_ptr(),
                    )
                };
                if hwnd.is_null() {
                    break;
                }
                previous_window = hwnd;

                if is_offtimer_process(hwnd) {
                    unsafe {
                        ShowWindow(hwnd, SW_RESTORE);
                        let _ = SetForegroundWindow(hwnd);
                    }
                    return;
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn show_single_instance_error(error: &str) {
        let title = wide("OffTimer");
        let message = wide(&format!(
            "OffTimer could not initialize its single-instance lock.\n\n{error}"
        ));

        unsafe {
            MessageBoxW(
                ptr::null_mut(),
                message.as_ptr(),
                title.as_ptr(),
                MB_OK | MB_ICONERROR,
            );
        }
    }

    struct TokenHandle(HANDLE);

    impl Drop for TokenHandle {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn last_error(context: &str) -> String {
        format!("{context}: {}", std::io::Error::last_os_error())
    }

    fn with_shutdown_privilege<T>(
        action: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        unsafe {
            let mut raw_token: HANDLE = ptr::null_mut();
            let access = TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY;
            if OpenProcessToken(GetCurrentProcess(), access, &mut raw_token) == 0 {
                return Err(last_error("Could not open the Windows process token"));
            }
            let token = TokenHandle(raw_token);

            let privilege_name = wide("SeShutdownPrivilege");
            let mut privileges = TOKEN_PRIVILEGES {
                PrivilegeCount: 1,
                Privileges: [LUID_AND_ATTRIBUTES {
                    Luid: LUID {
                        LowPart: 0,
                        HighPart: 0,
                    },
                    Attributes: SE_PRIVILEGE_ENABLED,
                }],
            };

            if LookupPrivilegeValueW(
                ptr::null(),
                privilege_name.as_ptr(),
                &mut privileges.Privileges[0].Luid,
            ) == 0
            {
                return Err(last_error("Could not locate the Windows shutdown privilege"));
            }

            let mut previous = TOKEN_PRIVILEGES::default();
            let mut previous_length = 0;
            SetLastError(0);
            if AdjustTokenPrivileges(
                token.0,
                0,
                &privileges,
                mem::size_of::<TOKEN_PRIVILEGES>() as u32,
                &mut previous,
                &mut previous_length,
            ) == 0
            {
                return Err(last_error("Could not enable the Windows shutdown privilege"));
            }

            let adjustment_error = GetLastError();
            if adjustment_error == ERROR_NOT_ALL_ASSIGNED {
                return Err(
                    "This Windows account is not permitted to shut down this computer."
                        .to_owned(),
                );
            }
            if adjustment_error != 0 {
                return Err(format!(
                    "Could not enable the Windows shutdown privilege (error {adjustment_error})."
                ));
            }

            let result = action();

            // Restore the token's original privilege state after the API call.
            let _ = AdjustTokenPrivileges(
                token.0,
                0,
                &previous,
                0,
                ptr::null_mut(),
                ptr::null_mut(),
            );

            result
        }
    }

    pub fn begin_system_countdown(action: PowerAction, timeout_seconds: u32) -> Result<(), String> {
        let is_restart = action == PowerAction::Restart;
        let message = match action {
            PowerAction::Shutdown => wide("OffTimer scheduled this computer to shut down."),
            PowerAction::Restart => wide("OffTimer scheduled this computer to restart."),
            _ => return Err("Windows countdown is only used for shutdown and restart.".to_owned()),
        };
        let timeout = timeout_seconds.max(1);
        let reason = SHTDN_REASON_FLAG_PLANNED
            | SHTDN_REASON_MAJOR_APPLICATION
            | SHTDN_REASON_MINOR_MAINTENANCE;

        with_shutdown_privilege(|| unsafe {
            let succeeded = InitiateSystemShutdownExW(
                ptr::null(),
                message.as_ptr(),
                timeout,
                0, // Do not forcibly close applications with unsaved work.
                if is_restart { 1 } else { 0 },
                reason,
            );

            if succeeded == 0 {
                Err(last_error("Windows could not start its shutdown countdown"))
            } else {
                Ok(())
            }
        })
    }

    pub fn abort_system_countdown() -> Result<(), String> {
        with_shutdown_privilege(|| unsafe {
            if AbortSystemShutdownW(ptr::null()) == 0 {
                let error = GetLastError();
                if error == ERROR_NO_SHUTDOWN_IN_PROGRESS {
                    // Windows confirms that there is no countdown left to cancel.
                    Ok(())
                } else {
                    Err(format!(
                        "Windows could not cancel the pending shutdown: {}",
                        std::io::Error::from_raw_os_error(error as i32)
                    ))
                }
            } else {
                Ok(())
            }
        })
    }

    fn find_offtimer_window() -> Result<isize, String> {
        let title = wide("OffTimer");
        let current_process_id = unsafe { GetCurrentProcessId() };
        let mut previous_window: HWND = ptr::null_mut();

        loop {
            let hwnd = unsafe {
                FindWindowExW(
                    ptr::null_mut(),
                    previous_window,
                    ptr::null(),
                    title.as_ptr(),
                )
            };
            if hwnd.is_null() {
                return Err(
                    "Could not find an OffTimer window owned by this process for its tray notification."
                        .to_owned(),
                );
            }
            previous_window = hwnd;

            let mut window_process_id = 0u32;
            unsafe {
                GetWindowThreadProcessId(hwnd, &mut window_process_id);
            }
            if window_process_id == current_process_id {
                return Ok(hwnd as isize);
            }
        }
    }

    pub fn hide_main_window() -> Result<(), String> {
        let hwnd = find_offtimer_window()? as HWND;
        unsafe {
            ShowWindow(hwnd, SW_HIDE);
        }
        Ok(())
    }

    fn copy_wide<const N: usize>(target: &mut [u16; N], text: &str) {
        target.fill(0);
        for (slot, unit) in target.iter_mut().take(N.saturating_sub(1)).zip(text.encode_utf16()) {
            *slot = unit;
        }
    }

    pub fn show_tray_notification(action: PowerAction, remaining_seconds: u64) -> Result<(), String> {
        let hwnd = find_offtimer_window()? as windows_sys::Win32::Foundation::HWND;
        let (action_name, message) = match action {
            PowerAction::Sleep => ("Sleep", "Sleep"),
            PowerAction::SignOut => ("Sign out", "Sign out"),
            _ => return Err("Tray notifications are only used for sleep and sign out.".to_owned()),
        };

        let timing = if remaining_seconds >= 55 {
            "in about one minute".to_owned()
        } else if remaining_seconds == 1 {
            "in 1 second".to_owned()
        } else {
            format!("in {remaining_seconds} seconds")
        };
        let body = format!("{action_name} is scheduled {timing}.");

        // build.rs embeds the exact OffTimer logo as the executable's icon resource (ID 1).
        let module = unsafe { GetModuleHandleW(ptr::null()) };
        if module.is_null() {
            return Err(last_error("Could not access the OffTimer executable icon"));
        }
        let icon = unsafe { LoadIconW(module, 1usize as *const u16) };
        if icon.is_null() {
            return Err(last_error("Could not load the OffTimer logo for its notification"));
        }

        let mut data = NOTIFYICONDATAW::default();
        data.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
        data.hWnd = hwnd;
        data.uID = TRAY_ICON_ID;
        data.uFlags = NIF_ICON | NIF_TIP;
        data.hIcon = icon;
        copy_wide(&mut data.szTip, "OffTimer");

        unsafe {
            // Remove a stale icon from a previous interrupted run, if one exists.
            let _ = Shell_NotifyIconW(NIM_DELETE, &mut data);

            if Shell_NotifyIconW(NIM_ADD, &mut data) == 0 {
                return Err(last_error("Windows could not add the OffTimer tray icon"));
            }

            data.uFlags = NIF_INFO;
            data.dwInfoFlags = NIIF_INFO;
            data.Anonymous.uTimeout = 10_000;
            copy_wide(&mut data.szInfoTitle, "OffTimer");
            copy_wide(&mut data.szInfo, &body);

            if Shell_NotifyIconW(NIM_MODIFY, &mut data) == 0 {
                let _ = Shell_NotifyIconW(NIM_DELETE, &mut data);
                return Err(last_error("Windows could not display the OffTimer tray notification"));
            }
        }

        Ok(())
    }

    pub fn remove_tray_notification() {
        let Ok(hwnd) = find_offtimer_window() else {
            return;
        };

        let mut data = NOTIFYICONDATAW::default();
        data.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
        data.hWnd = hwnd as windows_sys::Win32::Foundation::HWND;
        data.uID = TRAY_ICON_ID;

        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &mut data);
        }
    }
}

#[cfg(windows)]
pub fn power_clock_snapshot() -> Option<PowerClockSnapshot> {
    windows::power_clock_snapshot()
}

#[cfg(windows)]
pub fn acquire_single_instance() -> Result<Option<SingleInstanceGuard>, String> {
    windows::acquire_single_instance()
}

#[cfg(windows)]
pub fn activate_existing_instance() {
    windows::activate_existing_instance();
}

#[cfg(windows)]
pub fn show_single_instance_error(error: &str) {
    windows::show_single_instance_error(error);
}

#[cfg(windows)]
pub fn begin_system_countdown(action: PowerAction, timeout_seconds: u32) -> Result<(), String> {
    windows::begin_system_countdown(action, timeout_seconds)
}

#[cfg(not(windows))]
pub fn begin_system_countdown(_action: PowerAction, _timeout_seconds: u32) -> Result<(), String> {
    Err("Windows countdown dialogs are available on Windows only.".to_owned())
}

#[cfg(windows)]
pub fn abort_system_countdown() -> Result<(), String> {
    windows::abort_system_countdown()
}

#[cfg(not(windows))]
pub fn abort_system_countdown() -> Result<(), String> {
    Err("Windows countdown dialogs are available on Windows only.".to_owned())
}

#[cfg(windows)]
pub fn hide_main_window() -> Result<(), String> {
    windows::hide_main_window()
}

#[cfg(not(windows))]
pub fn hide_main_window() -> Result<(), String> {
    Err("Windows window hiding is available on Windows only.".to_owned())
}

#[cfg(windows)]
pub fn show_tray_notification(action: PowerAction, remaining_seconds: u64) -> Result<(), String> {
    windows::show_tray_notification(action, remaining_seconds)
}

#[cfg(not(windows))]
pub fn show_tray_notification(_action: PowerAction, _remaining_seconds: u64) -> Result<(), String> {
    Err("Windows tray notifications are available on Windows only.".to_owned())
}

#[cfg(windows)]
pub fn remove_tray_notification() {
    windows::remove_tray_notification();
}

#[cfg(not(windows))]
pub fn remove_tray_notification() {}

#[cfg(windows)]
pub fn execute(action: PowerAction) -> Result<(), String> {
    match action {
        PowerAction::Shutdown | PowerAction::Restart => {
            Err("Shutdown and restart use the Windows native countdown API.".to_owned())
        }
        PowerAction::SignOut => {
            use windows_sys::Win32::System::Shutdown::ExitWindowsEx;

            // EWX_LOGOFF is zero. No force flag is used, so applications can
            // display their normal unsaved-work prompts.
            let succeeded = unsafe { ExitWindowsEx(0, 0) };
            if succeeded == 0 {
                Err(std::io::Error::last_os_error().to_string())
            } else {
                Ok(())
            }
        }
        PowerAction::Sleep => {
            #[link(name = "PowrProf")]
            unsafe extern "system" {
                fn SetSuspendState(
                    hibernate: u8,
                    force_critical: u8,
                    disable_wake_event: u8,
                ) -> u8;
            }

            // Request sleep rather than hibernation; do not force the request.
            let succeeded = unsafe { SetSuspendState(0, 0, 0) };
            if succeeded == 0 {
                Err(std::io::Error::last_os_error().to_string())
            } else {
                Ok(())
            }
        }
    }
}

#[cfg(not(windows))]
pub fn execute(_action: PowerAction) -> Result<(), String> {
    Err("OffTimer power actions are available on Windows only.".to_owned())
}


#[cfg(test)]
mod power_clock_tests {
    use super::PowerClockSnapshot;
    use std::time::Duration;

    #[test]
    fn measures_time_spent_in_sleep() {
        let before = PowerClockSnapshot {
            uptime_ms: 50_000,
            awake_time_100ns: 500_000_000,
        };
        let after = PowerClockSnapshot {
            uptime_ms: 80_000,
            awake_time_100ns: 510_000_000,
        };

        assert_eq!(
            after.sleep_duration_since(before),
            Duration::from_secs(29)
        );
    }

    #[test]
    fn normal_awake_time_is_not_mistaken_for_sleep() {
        let before = PowerClockSnapshot {
            uptime_ms: 50_000,
            awake_time_100ns: 500_000_000,
        };
        let after = PowerClockSnapshot {
            uptime_ms: 52_000,
            awake_time_100ns: 520_000_000,
        };

        assert_eq!(
            after.sleep_duration_since(before),
            Duration::ZERO
        );
    }

    #[test]
    fn clock_rounding_never_produces_negative_sleep_time() {
        let before = PowerClockSnapshot {
            uptime_ms: 50_000,
            awake_time_100ns: 500_000_000,
        };
        let after = PowerClockSnapshot {
            uptime_ms: 50_010,
            awake_time_100ns: 500_200_000,
        };

        assert_eq!(
            after.sleep_duration_since(before),
            Duration::ZERO
        );
    }
}
