//! Framework-independent services shared by the native UI and background tasks.
use crate::store::State;
use serde::Serialize;
use serde_json::Value;
use std::sync::{mpsc, Arc};

#[derive(Debug, Clone, Copy)]
pub enum WindowAction {
    Hide,
    Minimize,
    Restore,
    Exit,
}
pub enum Event {
    Message(String, Value),
    Window(WindowAction),
}

#[derive(Clone)]
pub struct AppContext {
    state: Arc<State>,
    events: mpsc::Sender<Event>,
    repaint: Arc<dyn Fn() + Send + Sync>,
}
impl AppContext {
    pub fn new(
        state: Arc<State>,
        events: mpsc::Sender<Event>,
        repaint: Arc<dyn Fn() + Send + Sync>,
    ) -> Self {
        Self {
            state,
            events,
            repaint,
        }
    }
    pub fn state(&self) -> &State {
        &self.state
    }
    pub fn emit(&self, name: &str, value: impl Serialize) -> Result<(), String> {
        self.events
            .send(Event::Message(
                name.into(),
                serde_json::to_value(value).map_err(|e| e.to_string())?,
            ))
            .map_err(|e| e.to_string())?;
        (self.repaint)();
        Ok(())
    }
    pub fn window(&self, action: WindowAction) {
        let _ = self.events.send(Event::Window(action));
        (self.repaint)();
    }
    pub fn exit(&self, _code: i32) {
        self.window(WindowAction::Exit);
    }
}
