use crate::app_context::AppContext;
use serde_json::Value;
use std::sync::Arc;
use tauri::{Emitter, Manager};

#[tauri::command]
async fn snapshot(context: tauri::State<'_, AppContext>) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let mut result = crate::snapshot(state)?;
    result["launch"] = serde_json::json!({"can_cancel":app.launch.can_cancel(),"cancelling":app.launch.is_cancelling(),"running":app.launch.is_running()});
    result["discord_status"] = serde_json::json!(app.presence.status());
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn offline_account(
    context: tauri::State<'_, AppContext>,
    action: String,
    value: String,
) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::offline_account(state, action, value)?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_settings(
    context: tauri::State<'_, AppContext>,
    settings: crate::store::Settings,
) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::save_settings(state, settings)?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn notices(
    context: tauri::State<'_, AppContext>,
    action: String,
    id: Option<String>,
) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::notices(state, action, id)?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn versions(_context: tauri::State<'_, AppContext>) -> Result<Value, String> {
    let result = crate::versions().await?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn version_library(context: tauri::State<'_, AppContext>) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::version_library(state).await?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_modrinth(
    _context: tauri::State<'_, AppContext>,
    query: String,
    kind: String,
    version: String,
    loader: String,
    offset: u32,
) -> Result<Value, String> {
    let result = crate::search_modrinth(query, kind, version, loader, offset).await?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_mod(
    context: tauri::State<'_, AppContext>,
    project: String,
    name: String,
    kind: String,
) -> Result<Value, String> {
    let app = context.inner().clone();
    let result = crate::install_mod(app, project, name, kind).await?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn launch_game(context: tauri::State<'_, AppContext>) -> Result<Value, String> {
    let app = context.inner().clone();
    let result = crate::launch_game(app).await?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn verify_files(context: tauri::State<'_, AppContext>) -> Result<Value, String> {
    let app = context.inner().clone();
    let result = crate::verify_files(app).await?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_folder(
    context: tauri::State<'_, AppContext>,
    target: String,
) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::open_folder(state, target)?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn login_begin(context: tauri::State<'_, AppContext>) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::login_begin(state).await?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn login_poll(context: tauri::State<'_, AppContext>, code: String) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::login_poll(state, code).await?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn logout(_context: tauri::State<'_, AppContext>) -> Result<Value, String> {
    let result = crate::logout()?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_microsoft_login(
    _context: tauri::State<'_, AppContext>,
    url: String,
) -> Result<Value, String> {
    let result = crate::open_microsoft_login(url)?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn select_modpack(
    context: tauri::State<'_, AppContext>,
    id: String,
) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::modpacks::select_modpack(state, id)?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn scan_mods(context: tauri::State<'_, AppContext>, refresh: bool) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let mut result = crate::managed_mods::scan_mods(state, refresh).await?;
    result["fps_enabled"] = serde_json::json!(crate::fps_boost::enabled_count(&result));
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn toggle_mod(
    context: tauri::State<'_, AppContext>,
    filename: String,
    enabled: bool,
) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::managed_mods::toggle_mod(state, filename, enabled)?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_mod(
    context: tauri::State<'_, AppContext>,
    filename: String,
) -> Result<Value, String> {
    let app = context.inner().clone();
    let result = crate::managed_mods::update_mod(app, filename).await?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_java(context: tauri::State<'_, AppContext>) -> Result<Value, String> {
    let app = context.inner().clone();
    let result = crate::runtime::install_java(app).await?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_launcher_update(_context: tauri::State<'_, AppContext>) -> Result<Value, String> {
    let result = crate::updates::check_launcher_update().await?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn install_launcher_update(context: tauri::State<'_, AppContext>) -> Result<Value, String> {
    let app = context.inner().clone();
    let result = crate::updates::install_launcher_update(app).await?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_pack_cover(
    context: tauri::State<'_, AppContext>,
    id: String,
    image: String,
) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::customization::set_pack_cover(state, id, image)?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_compatibility(context: tauri::State<'_, AppContext>) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::compatibility::check_compatibility(state)?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn diagnose_launch(context: tauri::State<'_, AppContext>) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::diagnostics::diagnose_launch(state)?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_screenshots(context: tauri::State<'_, AppContext>) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::playtools::list_screenshots(state)?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn screenshot_image(
    context: tauri::State<'_, AppContext>,
    name: String,
) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::playtools::screenshot_image(state, name)?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn apply_graphics_preset(
    context: tauri::State<'_, AppContext>,
    id: String,
) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::playtools::apply_graphics_preset(state, id)?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn restore_graphics_preset(context: tauri::State<'_, AppContext>) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::playtools::restore_graphics_preset(state)?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_server(
    context: tauri::State<'_, AppContext>,
    name: String,
    address: String,
) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::servers::save_server(state, name, address)?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_server(context: tauri::State<'_, AppContext>, id: String) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::servers::remove_server(state, id)?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn server_status(context: tauri::State<'_, AppContext>, id: String) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::servers::server_status(state, id).await?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn launch_server(context: tauri::State<'_, AppContext>, id: String) -> Result<Value, String> {
    let app = context.inner().clone();
    let result = crate::servers::launch_server(app, id).await?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn fps_plan(context: tauri::State<'_, AppContext>) -> Result<Value, String> {
    let app = context.inner().clone();
    let state = app.state();

    let result = crate::fps_boost::plan(state).await?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn fps_install(context: tauri::State<'_, AppContext>, plan: Value) -> Result<Value, String> {
    let app = context.inner().clone();
    let result = crate::fps_boost::install(app, plan).await?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[tauri::command]
async fn cancel_launch(context: tauri::State<'_, AppContext>) -> Result<Value, String> {
    let app = context.inner().clone();
    let result = Ok::<_, String>(app.launch.cancel())?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

pub fn run() -> tauri::Result<()> {
    let app = tauri::Builder::default()
        .setup(|app| {
            app.manage(AppContext::new(
                Arc::new(crate::store::State::new()),
                app.handle().clone(),
            ));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            snapshot,
            offline_account,
            save_settings,
            notices,
            versions,
            version_library,
            search_modrinth,
            install_mod,
            launch_game,
            verify_files,
            open_folder,
            login_begin,
            login_poll,
            logout,
            open_microsoft_login,
            select_modpack,
            scan_mods,
            toggle_mod,
            update_mod,
            install_java,
            check_launcher_update,
            install_launcher_update,
            set_pack_cover,
            check_compatibility,
            diagnose_launch,
            list_screenshots,
            screenshot_image,
            apply_graphics_preset,
            restore_graphics_preset,
            save_server,
            remove_server,
            server_status,
            launch_server,
            fps_plan,
            fps_install,
            cancel_launch
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let context = window.state::<AppContext>();
                if *context
                    .state()
                    .busy
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                {
                    api.prevent_close();
                    let _ = window.emit("close-blocked", ());
                }
            }
        })
        .build(tauri::generate_context!())?;
    app.run(|handle, event| {
        if matches!(event, tauri::RunEvent::Exit) {
            handle.state::<AppContext>().presence.shutdown();
        }
    });
    Ok(())
}
