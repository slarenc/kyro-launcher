use crate::net;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
#[derive(Clone, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub uuid: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires: i64,
}
fn entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new("dev.kyro.launcher", "microsoft").map_err(|e| e.to_string())
}
pub fn load() -> Option<Account> {
    entry()
        .ok()?
        .get_password()
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}
pub fn logout() -> Result<(), String> {
    match entry()?.delete_credential() {
        Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
async fn post(url: &str, body: Value) -> Result<Value, String> {
    net::client()?
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| format!("Ошибка входа: {e}"))?
        .json()
        .await
        .map_err(|e| e.to_string())
}
pub async fn begin(client_id: &str) -> Result<Value, String> {
    if client_id.trim().is_empty() {
        return Err("Укажите Microsoft OAuth Client ID в настройках учётной записи. Нужна собственная регистрация приложения Microsoft с public client flow и доступом Minecraft API.".into());
    }
    net::client()?
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode")
        .form(&[
            ("client_id", client_id),
            ("scope", "XboxLive.signin offline_access"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}
pub async fn poll(client_id: &str, code: &str) -> Result<Value, String> {
    let v: Value = net::client()?
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
        .form(&[
            ("client_id", client_id),
            ("device_code", code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    if let Some(err) = v["error"].as_str() {
        if err == "authorization_pending" || err == "slow_down" {
            return Ok(json!({"status":err}));
        }
        return Err(v["error_description"].as_str().unwrap_or(err).into());
    }
    let account = exchange(&v).await?;
    Ok(json!({"status":"complete", "name":account.name}))
}
async fn exchange(oauth: &Value) -> Result<Account, String> {
    let ms = oauth["access_token"]
        .as_str()
        .ok_or("Microsoft не вернул токен")?;
    let xbox = post("https://user.auth.xboxlive.com/user/authenticate", json!({"Properties":{"AuthMethod":"RPS","SiteName":"user.auth.xboxlive.com","RpsTicket":format!("d={ms}")},"RelyingParty":"http://auth.xboxlive.com","TokenType":"JWT"})).await?;
    let xbox_token = xbox["Token"].as_str().ok_or("Xbox не вернул токен")?;
    let xsts = post("https://xsts.auth.xboxlive.com/xsts/authorize", json!({"Properties":{"SandboxId":"RETAIL","UserTokens":[xbox_token]},"RelyingParty":"rp://api.minecraftservices.com/","TokenType":"JWT"})).await?;
    let hash = xsts["DisplayClaims"]["xui"][0]["uhs"]
        .as_str()
        .ok_or("Не найден Xbox профиль")?;
    let token = xsts["Token"].as_str().ok_or("XSTS не вернул токен")?;
    let mc = post(
        "https://api.minecraftservices.com/authentication/login_with_xbox",
        json!({"identityToken":format!("XBL3.0 x={hash};{token}")}),
    )
    .await?;
    let access = mc["access_token"]
        .as_str()
        .ok_or("Minecraft не вернул токен")?;
    let profile: Value = net::client()?
        .get("https://api.minecraftservices.com/minecraft/profile")
        .bearer_auth(access)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|_| "На этом аккаунте не найден Minecraft Java Edition".to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    let account = Account {
        name: profile["name"]
            .as_str()
            .ok_or("Нет имени Minecraft")?
            .into(),
        uuid: profile["id"].as_str().ok_or("Нет UUID")?.into(),
        access_token: access.into(),
        refresh_token: oauth["refresh_token"].as_str().unwrap_or_default().into(),
        expires: chrono::Utc::now().timestamp() + mc["expires_in"].as_i64().unwrap_or(3600) - 120,
    };
    entry()?
        .set_password(&serde_json::to_string(&account).map_err(|e| e.to_string())?)
        .map_err(|e| format!("Не удалось сохранить токен в Windows Credential Manager: {e}"))?;
    Ok(account)
}
pub async fn current(client_id: &str) -> Result<Account, String> {
    let account =
        load().ok_or("Добавьте офлайн-аккаунт или войдите в Microsoft через профиль внизу меню")?;
    if account.expires > chrono::Utc::now().timestamp() {
        return Ok(account);
    }
    let v: Value = net::client()?
        .post("https://login.microsoftonline.com/consumers/oauth2/v2.0/token")
        .form(&[
            ("client_id", client_id),
            ("refresh_token", account.refresh_token.as_str()),
            ("grant_type", "refresh_token"),
            ("scope", "XboxLive.signin offline_access"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|_| "Сессия истекла. Войдите в Microsoft снова".to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    exchange(&v).await
}
