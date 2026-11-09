#![cfg_attr(windows, windows_subsystem = "windows")]
use std::io::{self, BufRead, Write};
use serde_json::{json, Value};
use windows::Media::Control::{GlobalSystemMediaTransportControlsSessionManager as Manager,GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status};
fn poll(manager:&Manager, action:&str) -> windows::core::Result<Value> {
    let session=match manager.GetCurrentSession(){Ok(s)=>s,Err(_)=>return Ok(json!({"available":false}))};
    let playback=session.GetPlaybackInfo()?;
    let controls=playback.Controls()?;
    let accepted=match action {
        "toggle" if controls.IsPlayPauseToggleEnabled()? => session.TryTogglePlayPauseAsync()?.join()?,
        "next" if controls.IsNextEnabled()? => session.TrySkipNextAsync()?.join()?,
        "previous" if controls.IsPreviousEnabled()? => session.TrySkipPreviousAsync()?.join()?,
        "poll" => true,
        _ => false,
    };
    let props=session.TryGetMediaPropertiesAsync()?.join()?;
    let time=session.GetTimelineProperties()?;
    let start=time.StartTime()?.Duration;
    let end=time.EndTime()?.Duration;
    let position=time.Position()?.Duration;
    Ok(json!({"available":true,"title":props.Title()?.to_string(),"artist":props.Artist()?.to_string(),
        "source":session.SourceAppUserModelId()?.to_string(),"playing":session.GetPlaybackInfo()?.PlaybackStatus()?==Status::Playing,
        "position":(position-start).max(0)/10000,"duration":(end-start).max(0)/10000,
        "canToggle":controls.IsPlayPauseToggleEnabled()?,"canNext":controls.IsNextEnabled()?,"canPrevious":controls.IsPreviousEnabled()?,"accepted":accepted}))
}
fn main(){
    let manager=unsafe{windows::Win32::System::WinRT::RoInitialize(windows::Win32::System::WinRT::RO_INIT_MULTITHREADED)}
        .and_then(|_|Manager::RequestAsync()?.join());
    let stdin=io::stdin();let mut out=io::stdout().lock();
    for line in stdin.lock().lines(){
        let Ok(line)=line else{break;};
        let action=line.trim();
        let value=match &manager {Ok(manager)=>poll(manager,action).unwrap_or_else(|_|json!({"available":false,"error":"Медиасессия недоступна"})),Err(_)=>json!({"available":false,"error":"Медиапанель Windows недоступна"})};
        if writeln!(out,"{}",value).and_then(|_|out.flush()).is_err(){break;}
    }
}
