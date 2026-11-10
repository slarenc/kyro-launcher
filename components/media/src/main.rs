#![cfg_attr(windows, windows_subsystem = "windows")]
use std::io::{self, BufRead, Write};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use windows::{core::Interface, Media::Control::{GlobalSystemMediaTransportControlsSession as Session, GlobalSystemMediaTransportControlsSessionManager as Manager, GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status}, Storage::Streams::DataReader};
use windows::Win32::{Media::Audio::*, System::{Com::*, Threading::*}};

const WINDOWS_EPOCH_TICKS: i64 = 116_444_736_000_000_000;
fn utc_ticks() -> i64 {
    let elapsed=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    WINDOWS_EPOCH_TICKS.saturating_add((elapsed.as_nanos()/100).min(i64::MAX as u128) as i64)
}
fn valid_rate(rate:f64)->f64 {if rate.is_finite()&&rate>0.0&&rate<=16.0{rate}else{1.0}}
// Windows Position is a snapshot at LastUpdatedTime, not a continuously ticking clock.
fn timeline_position(position:i64,start:i64,end:i64,updated:i64,now:i64,playing:bool,rate:f64)->i64 {
    let elapsed=if playing&&updated>0&&updated<=now{now.saturating_sub(updated)}else{0};
    let delta=(elapsed as f64*valid_rate(rate)).round() as i64;
    let current=position.saturating_sub(start).max(0).saturating_add(delta);
    let duration=end.saturating_sub(start).max(0);
    (if duration>0{current.min(duration)}else{current})/10000
}

#[derive(Default)]
struct Artwork { key: String, encoded: String, revision: u64, retry: u32 }

fn artwork(session: &Session) -> windows::core::Result<String> {
    let stream = session.TryGetMediaPropertiesAsync()?.join()?.Thumbnail()?.OpenReadAsync()?.join()?;
    let size = stream.Size()?;
    if size == 0 || size > 4 * 1024 * 1024 { return Ok(String::new()); }
    let reader = DataReader::CreateDataReader(&stream.GetInputStreamAt(0)?)?;
    let loaded = reader.LoadAsync(size as u32)?.join()?;
    let mut bytes = vec![0; loaded as usize]; reader.ReadBytes(&mut bytes)?;
    Ok(STANDARD.encode(bytes))
}

// Only change the audio session belonging to the displayed media source.
fn audio_volume(source: &str, set: Option<f32>) -> Option<f32> {
    unsafe {
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia).ok()?;
        let manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None).ok()?;
        let sessions = manager.GetSessionEnumerator().ok()?;
        let source = source.to_lowercase(); let mut result = None;
        for i in 0..sessions.GetCount().ok()? {
            let Ok(control) = sessions.GetSession(i) else { continue };
            let Ok(control2) = control.cast::<IAudioSessionControl2>() else { continue };
            let Ok(pid) = control2.GetProcessId() else { continue };
            let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else { continue };
            let mut name = vec![0u16; 32768]; let mut len = name.len() as u32;
            let query = QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, windows::core::PWSTR(name.as_mut_ptr()), &mut len);
            let _ = windows::Win32::Foundation::CloseHandle(process);
            if query.is_err() { continue; }
            let full = String::from_utf16_lossy(&name[..len as usize]).to_lowercase();
            let stem = full.rsplit('\\').next().unwrap_or("").trim_end_matches(".exe");
            if stem.len() < 3 || !source.contains(stem) { continue; }
            let Ok(volume) = control.cast::<ISimpleAudioVolume>() else { continue };
            if let Some(value) = set { let _ = volume.SetMasterVolume(value.clamp(0.0, 1.0), std::ptr::null()); }
            if let Ok(value) = volume.GetMasterVolume() { result = Some(value); }
        }
        result
    }
}

fn select(manager: &Manager, source: Option<&str>) -> windows::core::Result<Session> {
    if let Some(source) = source {
        let sessions = manager.GetSessions()?;
        for i in 0..sessions.Size()? {
            let session = sessions.GetAt(i)?;
            if session.SourceAppUserModelId()?.to_string() == source { return Ok(session); }
        }
        return Err(windows::core::Error::from_hresult(windows::core::HRESULT(0x80070490u32 as i32)));
    }
    manager.GetCurrentSession()
}

fn poll(manager: &Manager, request: &Value, art: &mut Artwork) -> windows::core::Result<Value> {
    let session = match select(manager, request["session"].as_str()) { Ok(s) => s, Err(_) => { art.key.clear(); return Ok(json!({"available":false,"protocol":2})); } };
    let controls = session.GetPlaybackInfo()?.Controls()?;
    let time = session.GetTimelineProperties()?;
    let start = time.StartTime()?.Duration; let end = time.EndTime()?.Duration;
    let action = request["action"].as_str().unwrap_or("poll");
    let accepted = match action {
        "toggle" if controls.IsPlayPauseToggleEnabled()? => session.TryTogglePlayPauseAsync()?.join()?,
        "next" if controls.IsNextEnabled()? => session.TrySkipNextAsync()?.join()?,
        "previous" if controls.IsPreviousEnabled()? => session.TrySkipPreviousAsync()?.join()?,
        "seek" if controls.IsPlaybackPositionEnabled()? => {
            let ms = request["value"].as_i64().unwrap_or(0).clamp(0, (end-start).max(0)/10000);
            session.TryChangePlaybackPositionAsync(start.saturating_add(ms.saturating_mul(10000)))?.join()?
        }
        "volume" | "poll" => true,
        _ => false,
    };
    let props = session.TryGetMediaPropertiesAsync()?.join()?;
    let source = session.SourceAppUserModelId()?.to_string();
    let title = props.Title()?.to_string(); let artist = props.Artist()?.to_string();
    let key = format!("{source}\0{title}\0{artist}"); let changed = art.key != key;
    if changed { art.key = key; art.encoded.clear(); art.retry = 0; art.revision += 1; }
    if changed || (art.encoded.is_empty() && art.retry < 6) {
        art.retry += 1;
        if let Ok(encoded) = artwork(&session) { if encoded != art.encoded { art.encoded = encoded; art.revision += 1; } }
    }
    let volume = audio_volume(&source, if action == "volume" { request["value"].as_f64().map(|n| n as f32) } else { None });
    // Re-read after controls/metadata so a seek or track change doesn't mix old bounds
    // with a new position. Account for unchanged snapshots on every poll.
    let playback=session.GetPlaybackInfo()?;
    let playing=playback.PlaybackStatus()?==Status::Playing;
    let rate=valid_rate(playback.PlaybackRate().and_then(|value|value.Value()).unwrap_or(1.0));
    let timeline=session.GetTimelineProperties()?;
    let start=timeline.StartTime()?.Duration;let end=timeline.EndTime()?.Duration;
    let position=timeline_position(timeline.Position()?.Duration,start,end,timeline.LastUpdatedTime()?.UniversalTime,utc_ticks(),playing,rate);
    let mut result = json!({"protocol":2,"available":true,"title":title,"artist":artist,"source":source,
        "playing":playing,"playbackRate":rate,
        "position":position,"duration":end.saturating_sub(start).max(0)/10000,
        "canToggle":controls.IsPlayPauseToggleEnabled()?,"canNext":controls.IsNextEnabled()?,"canPrevious":controls.IsPreviousEnabled()?,
        "canSeek":controls.IsPlaybackPositionEnabled()?,"volume":volume.unwrap_or(-1.0),"artRevision":art.revision,"accepted":accepted});
    if request["artRevision"].as_u64() != Some(art.revision) { result["artwork"] = json!(art.encoded); }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    const SECOND:i64=10_000_000;
    #[test]
    fn stale_windows_snapshot_keeps_moving_across_polls(){
        let stamp=WINDOWS_EPOCH_TICKS+100*SECOND;
        assert_eq!(timeline_position(10*SECOND,0,240*SECOND,stamp,stamp+2*SECOND,true,1.0),12_000);
        assert_eq!(timeline_position(10*SECOND,0,240*SECOND,stamp,stamp+5*SECOND,true,1.0),15_000);
    }
    #[test]
    fn pause_speed_seek_bounds_and_unknown_timestamps(){
        let stamp=WINDOWS_EPOCH_TICKS+100*SECOND;
        assert_eq!(timeline_position(10*SECOND,0,240*SECOND,stamp,stamp+2*SECOND,false,1.0),10_000);
        assert_eq!(timeline_position(10*SECOND,0,240*SECOND,stamp,stamp+2*SECOND,true,2.0),14_000);
        assert_eq!(timeline_position(112*SECOND,100*SECOND,340*SECOND,stamp,stamp+3*SECOND,true,1.0),15_000);
        assert_eq!(timeline_position(239*SECOND,0,240*SECOND,stamp,stamp+3*SECOND,true,1.0),240_000);
        assert_eq!(timeline_position(10*SECOND,0,240*SECOND,0,stamp,true,1.0),10_000);
        assert_eq!(timeline_position(10*SECOND,0,240*SECOND,stamp+SECOND,stamp,true,1.0),10_000);
        assert_eq!(timeline_position(10*SECOND,0,240*SECOND,stamp,stamp+2*SECOND,true,f64::NAN),12_000);
    }
}

fn main() {
    let manager = unsafe { windows::Win32::System::WinRT::RoInitialize(windows::Win32::System::WinRT::RO_INIT_MULTITHREADED) }.and_then(|_| Manager::RequestAsync()?.join());
    let stdin = io::stdin(); let mut out = io::stdout().lock(); let mut art = Artwork::default();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let request = serde_json::from_str::<Value>(&line).unwrap_or_else(|_| json!({"action":line.trim()}));
        let value = match &manager {
            Ok(manager) => poll(manager, &request, &mut art).unwrap_or_else(|_| json!({"available":false,"error":"Медиасессия недоступна","protocol":2})),
            Err(_) => json!({"available":false,"error":"Медиапанель Windows недоступна","protocol":2})
        };
        if writeln!(out, "{}", value).and_then(|_| out.flush()).is_err() { break; }
    }
}
