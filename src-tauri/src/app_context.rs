//! Services shared by Tauri commands and background tasks.
use crate::store::State;
use serde::Serialize;
use std::sync::Arc;
use tauri::{Emitter, Manager};
#[derive(Debug, Clone, Copy)]
pub enum WindowAction {
    Hide,
    Minimize,
    Restore,
}
#[derive(Clone)]
pub struct AppContext {
    state: Arc<State>,
    handle: Option<tauri::AppHandle>,
    pub presence: Arc<crate::presence::Presence>,
    pub launch: Arc<crate::launch_control::LaunchControl>,
}
impl AppContext {
    pub fn new(state: Arc<State>, handle: tauri::AppHandle) -> Self {
        let presence = Arc::new(crate::presence::Presence::new(Arc::downgrade(&state)));
        Self {
            state,
            handle: Some(handle),
            presence,
            launch: Arc::default(),
        }
    }
    #[cfg(test)]
    pub fn headless(state: Arc<State>) -> Self {
        let presence = Arc::new(crate::presence::Presence::new(Arc::downgrade(&state)));
        Self {
            state,
            handle: None,
            presence,
            launch: Arc::default(),
        }
    }
    pub fn state(&self) -> &State {
        &self.state
    }
    pub fn emit(&self, name: &str, value: impl Serialize) -> Result<(), String> {
        let Some(handle) = &self.handle else {
            return Ok(());
        };
        handle
            .emit(
                name,
                serde_json::to_value(value).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())
    }
    pub fn window(&self, action: WindowAction) {
        if let Some(w) = self
            .handle
            .as_ref()
            .and_then(|handle| handle.get_webview_window("main"))
        {
            match action {
                WindowAction::Hide => {
                    let _ = w.hide();
                }
                WindowAction::Minimize => {
                    let _ = w.minimize();
                }
                WindowAction::Restore => {
                    let _ = w.show();
                    let _ = w.unminimize();
                    let _ = w.set_focus();
                }
            }
        }
    }
    pub fn exit(&self, code: i32) {
        if let Some(handle) = &self.handle {
            handle.exit(code);
        }
    }
}
