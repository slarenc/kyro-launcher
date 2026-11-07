//! Wake only this launcher's native window, including when hidden windows do not redraw.
use std::sync::atomic::{AtomicIsize, Ordering};

#[derive(Default)]
pub struct NativeWindow {
    handle: AtomicIsize,
}
impl NativeWindow {
    pub fn bind(&self, context: &eframe::CreationContext<'_>) {
        #[cfg(windows)]
        {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            if let Ok(handle) = context.window_handle() {
                if let RawWindowHandle::Win32(window) = handle.as_raw() {
                    self.handle.store(window.hwnd.get(), Ordering::SeqCst);
                }
            }
        }
        #[cfg(not(windows))]
        let _ = context;
    }
    pub fn unbind(&self) {
        self.handle.store(0, Ordering::SeqCst);
    }
    #[cfg(windows)]
    fn own_handle(&self) -> Option<windows_sys::Win32::Foundation::HWND> {
        use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
        let raw = self.handle.load(Ordering::SeqCst);
        if raw == 0 {
            return None;
        }
        let handle = raw as windows_sys::Win32::Foundation::HWND;
        let mut process = 0;
        // The handle comes directly from eframe, and is accepted only for this process.
        unsafe {
            GetWindowThreadProcessId(handle, &mut process);
        }
        (process == std::process::id()).then_some(handle)
    }
    pub fn restore(&self) {
        #[cfg(windows)]
        if let Some(handle) = self.own_handle() {
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                IsIconic, SetForegroundWindow, ShowWindowAsync, SW_RESTORE, SW_SHOW,
            };
            unsafe {
                ShowWindowAsync(
                    handle,
                    if IsIconic(handle) != 0 {
                        SW_RESTORE
                    } else {
                        SW_SHOW
                    },
                );
                SetForegroundWindow(handle);
            }
        }
    }
    pub fn is_visible(&self) -> bool {
        #[cfg(windows)]
        {
            use windows_sys::Win32::UI::WindowsAndMessaging::{IsIconic, IsWindowVisible};
            self.own_handle().is_some_and(|handle| unsafe {
                IsWindowVisible(handle) != 0 && IsIconic(handle) == 0
            })
        }
        #[cfg(not(windows))]
        {
            true
        }
    }
}
