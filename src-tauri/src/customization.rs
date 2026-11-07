use crate::store::State;
use base64::Engine;
pub fn validate_image(image: &str) -> Result<(), String> {
    if image.is_empty() {
        return Ok(());
    }
    if image.len() > 6 * 1024 * 1024 {
        return Err("Изображение должно быть меньше 4 МБ".into());
    }
    let (prefix, data) = image.split_once(',').ok_or("Нет данных картинки")?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|_| "Некорректная картинка")?;
    let valid = match prefix {
        "data:image/png;base64" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "data:image/jpeg;base64" => bytes.starts_with(b"\xff\xd8\xff"),
        "data:image/webp;base64" => bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP"),
        _ => false,
    };
    if !valid || bytes.len() > 4 * 1024 * 1024 {
        return Err("Выберите PNG, JPEG или WebP до 4 МБ".into());
    }
    Ok(())
}

pub fn set_pack_cover(state: &State, id: String, image: String) -> Result<(), String> {
    let busy = state.busy.lock().map_err(|e| e.to_string())?;
    if *busy {
        return Err("Дождитесь окончания операции".into());
    }
    validate_image(&image)?;
    {
        let mut data = state.data.lock().map_err(|e| e.to_string())?;
        data.modpacks
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or("Сборка не найдена")?
            .cover_image = image;
    }
    state.save()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_html_and_fake_images() {
        assert!(validate_image("data:text/html;base64,PHNjcmlwdD4=").is_err());
        assert!(validate_image("data:image/png;base64,YmFk").is_err());
        assert!(validate_image("").is_ok());
    }
}
