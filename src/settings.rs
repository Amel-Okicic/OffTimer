use crate::actions::PowerAction;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppSettings {
    pub use_24_hour_clock: bool,
    pub close_to_tray: bool,
    pub selected_action: PowerAction,
    pub theme: String,
    pub custom_duration: Option<(u64, u64)>,
    pub quick_preset_custom_durations: [Option<(u64, u64)>; 4],
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            use_24_hour_clock: true,
            close_to_tray: false,
            selected_action: PowerAction::Shutdown,
            theme: "default".to_owned(),
            custom_duration: None,
            quick_preset_custom_durations: [None; 4],
        }
    }
}

impl AppSettings {
    fn to_json(&self) -> String {
        let quick_presets = self
            .quick_preset_custom_durations
            .iter()
            .map(|duration| duration_to_json(*duration))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            concat!(
                "{{\n",
                "  \"version\": 1,\n",
                "  \"use_24_hour_clock\": {},\n",
                "  \"close_to_tray\": {},\n",
                "  \"selected_action\": \"{}\",\n",
                "  \"theme\": \"{}\",\n",
                "  \"custom_duration\": {},\n",
                "  \"quick_preset_custom_durations\": [{}]\n",
                "}}\n"
            ),
            self.use_24_hour_clock,
            self.close_to_tray,
            power_action_settings_key(self.selected_action),
            self.theme,
            duration_to_json(self.custom_duration),
            quick_presets,
        )
    }

    fn from_json(input: &str) -> Result<Self, String> {
        let input = input.trim();
        if !input.starts_with('{') || !input.ends_with('}') {
            return Err("The settings file is not a JSON object.".to_owned());
        }

        let mut settings = Self::default();

        if let Some(value) = json_value_for_key(input, "version") {
            let version = value
                .parse::<u32>()
                .map_err(|_| "The settings file has an invalid version.".to_owned())?;
            if version != 1 {
                return Err(format!("Unsupported settings file version: {version}."));
            }
        }

        if let Some(value) = json_value_for_key(input, "use_24_hour_clock") {
            settings.use_24_hour_clock = parse_json_bool(value, "use_24_hour_clock")?;
        }

        if let Some(value) = json_value_for_key(input, "close_to_tray") {
            settings.close_to_tray = parse_json_bool(value, "close_to_tray")?;
        }

        if let Some(value) = json_value_for_key(input, "selected_action") {
            let action = parse_json_string(value)
                .ok_or_else(|| "The selected_action setting must be a JSON string.".to_owned())?;
            settings.selected_action = parse_power_action_settings_key(action)
                .ok_or_else(|| "The selected_action setting is not recognized.".to_owned())?;
        }

        if let Some(value) = json_value_for_key(input, "theme") {
            let theme = parse_json_string(value)
                .ok_or_else(|| "The theme setting must be a JSON string.".to_owned())?;
            if !matches!(
                theme,
                "default" | "midnight" | "light" | "red_moon" | "windows_xp" | "windows_classic"
            ) {
                return Err("The theme setting is not recognized.".to_owned());
            }
            settings.theme = if theme == "windows_xp" {
                "red_moon"
            } else {
                theme
            }
            .to_owned();
        }

        if let Some(value) = json_value_for_key(input, "custom_duration") {
            settings.custom_duration = parse_duration_setting(value)?;
        }

        if let Some(value) = json_value_for_key(input, "quick_preset_custom_durations") {
            let entries = json_array_items(value)
                .ok_or_else(|| "The quick preset settings must be a JSON array.".to_owned())?;
            if entries.len() != 4 {
                return Err(
                    "The quick preset settings must contain exactly four entries.".to_owned(),
                );
            }

            for (index, entry) in entries.iter().enumerate() {
                settings.quick_preset_custom_durations[index] = parse_duration_setting(entry)?;
            }
        }

        Ok(settings)
    }
}

pub fn load() -> Result<Option<AppSettings>, String> {
    let path = settings_file_path()?;

    match fs::read_to_string(&path) {
        Ok(contents) => {
            let settings = AppSettings::from_json(&contents)
                .map_err(|error| format!("{}: {error}", path.display()))?;
            remove_legacy_settings_file(&path);
            Ok(Some(settings))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let Some(legacy_path) = legacy_settings_file_path() else {
                return Ok(None);
            };
            if legacy_path == path {
                return Ok(None);
            }

            match fs::read_to_string(&legacy_path) {
                Ok(contents) => {
                    // A settings.json beside the executable may belong to another app.
                    if !is_offtimer_settings(&contents) {
                        return Ok(None);
                    }

                    let settings = AppSettings::from_json(&contents)
                        .map_err(|error| format!("{}: {error}", legacy_path.display()))?;
                    save(&settings)?;
                    let _ = fs::remove_file(&legacy_path);
                    Ok(Some(settings))
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
                Err(error) => Err(format!("Could not read {}: {error}", legacy_path.display())),
            }
        }
        Err(error) => Err(format!("Could not read {}: {error}", path.display())),
    }
}

pub fn save(settings: &AppSettings) -> Result<(), String> {
    let path = settings_file_path()?;
    let directory = path
        .parent()
        .ok_or_else(|| "Could not locate the OffTimer settings folder.".to_owned())?;
    fs::create_dir_all(directory)
        .map_err(|error| format!("Could not create {}: {error}", directory.display()))?;

    let temporary_path = path.with_file_name("settings.json.tmp");
    fs::write(&temporary_path, settings.to_json())
        .map_err(|error| format!("Could not write {}: {error}", temporary_path.display()))?;
    fs::rename(&temporary_path, &path).map_err(|error| {
        let _ = fs::remove_file(&temporary_path);
        format!("Could not update {}: {error}", path.display())
    })
}

fn settings_file_path() -> Result<PathBuf, String> {
    Ok(settings_directory()?.join("settings.json"))
}

fn settings_directory() -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        let app_data = env::var_os("APPDATA").ok_or_else(|| {
            "Could not locate the Windows roaming AppData folder (APPDATA).".to_owned()
        })?;
        Ok(PathBuf::from(app_data).join("OffTimer"))
    }

    #[cfg(not(windows))]
    {
        let base = env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .ok_or_else(|| "Could not locate the user configuration folder.".to_owned())?;
        Ok(base.join("OffTimer"))
    }
}

fn legacy_settings_file_path() -> Option<PathBuf> {
    let executable = env::current_exe().ok()?;
    let directory = executable.parent()?;
    Some(directory.join("settings.json"))
}

fn remove_legacy_settings_file(current_path: &Path) {
    if let Some(legacy_path) = legacy_settings_file_path() {
        remove_legacy_settings_file_if_valid(&legacy_path, current_path);
    }
}

fn remove_legacy_settings_file_if_valid(legacy_path: &Path, current_path: &Path) {
    if legacy_path == current_path {
        return;
    }

    let Ok(contents) = fs::read_to_string(legacy_path) else {
        return;
    };

    // The executable folder may be shared. Require OffTimer's own preference keys.
    if is_offtimer_settings(&contents) {
        let _ = fs::remove_file(legacy_path);
    }
}

fn is_offtimer_settings(contents: &str) -> bool {
    let has_offtimer_keys = ["use_24_hour_clock", "close_to_tray", "selected_action"]
        .iter()
        .all(|key| json_value_for_key(contents, key).is_some());

    has_offtimer_keys && AppSettings::from_json(contents).is_ok()
}

fn power_action_settings_key(action: PowerAction) -> &'static str {
    match action {
        PowerAction::Shutdown => "shutdown",
        PowerAction::Sleep => "sleep",
        PowerAction::SignOut => "sign_out",
        PowerAction::Restart => "restart",
    }
}

fn parse_power_action_settings_key(value: &str) -> Option<PowerAction> {
    match value {
        "shutdown" => Some(PowerAction::Shutdown),
        "sleep" => Some(PowerAction::Sleep),
        "sign_out" => Some(PowerAction::SignOut),
        "restart" => Some(PowerAction::Restart),
        _ => None,
    }
}

fn duration_to_json(duration: Option<(u64, u64)>) -> String {
    match duration {
        Some((hours, minutes)) => format!("[{hours}, {minutes}]"),
        None => "null".to_owned(),
    }
}

fn parse_duration_setting(value: &str) -> Result<Option<(u64, u64)>, String> {
    let value = value.trim();
    if value == "null" {
        return Ok(None);
    }

    let parts = json_array_items(value)
        .ok_or_else(|| "A saved preset duration must be null or a two-number array.".to_owned())?;
    if parts.len() != 2 {
        return Err("A saved preset duration must contain hours and minutes.".to_owned());
    }

    let hours = parts[0]
        .parse::<u64>()
        .map_err(|_| "A saved preset has invalid hours.".to_owned())?;
    let minutes = parts[1]
        .parse::<u64>()
        .map_err(|_| "A saved preset has invalid minutes.".to_owned())?;

    if hours > 999 || minutes > 59 || (hours == 0 && minutes == 0) {
        return Err("A saved preset duration is outside the supported range.".to_owned());
    }

    Ok(Some((hours, minutes)))
}

fn parse_json_bool(value: &str, setting: &str) -> Result<bool, String> {
    match value.trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("The {setting} setting must be true or false.")),
    }
}

fn parse_json_string(value: &str) -> Option<&str> {
    let value = value.trim();
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        Some(&value[1..value.len() - 1])
    } else {
        None
    }
}

fn json_value_for_key<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\"");
    let key_start = json.find(&needle)?;
    let after_key = &json[key_start + needle.len()..];
    let after_colon = after_key[after_key.find(':')? + 1..].trim_start();
    let bytes = after_colon.as_bytes();
    let first = *bytes.first()?;

    if first == b'[' || first == b'{' {
        let closing = if first == b'[' { b']' } else { b'}' };
        let mut depth = 0usize;
        let mut in_string = false;
        let mut escaped = false;

        for (index, byte) in bytes.iter().copied().enumerate() {
            if in_string {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    in_string = false;
                }
                continue;
            }

            if byte == b'"' {
                in_string = true;
            } else if byte == first {
                depth += 1;
            } else if byte == closing {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(&after_colon[..=index]);
                }
            }
        }
        None
    } else if first == b'"' {
        let mut escaped = false;
        for (index, byte) in bytes.iter().copied().enumerate().skip(1) {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                return Some(&after_colon[..=index]);
            }
        }
        None
    } else {
        let end = after_colon
            .find(|character: char| {
                character == ',' || character == '}' || character.is_whitespace()
            })
            .unwrap_or(after_colon.len());
        Some(&after_colon[..end])
    }
}

fn json_array_items(value: &str) -> Option<Vec<&str>> {
    let value = value.trim();
    let inner = value.strip_prefix('[')?.strip_suffix(']')?;
    if inner.trim().is_empty() {
        return Some(Vec::new());
    }

    let mut items = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, byte) in inner.bytes().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }

        match byte {
            b'"' => in_string = true,
            b'[' | b'{' => depth += 1,
            b']' | b'}' => depth = depth.checked_sub(1)?,
            b',' if depth == 0 => {
                items.push(inner[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }

    items.push(inner[start..].trim());
    Some(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_all_ui_preferences() {
        let settings = AppSettings {
            use_24_hour_clock: false,
            close_to_tray: true,
            selected_action: PowerAction::Restart,
            theme: "red_moon".to_owned(),
            custom_duration: Some((3, 25)),
            quick_preset_custom_durations: [Some((0, 15)), None, Some((5, 0)), Some((1, 45))],
        };

        assert_eq!(
            AppSettings::from_json(&settings.to_json()).unwrap(),
            settings
        );
    }

    #[test]
    fn missing_fields_use_defaults() {
        let settings = AppSettings::from_json("{\"version\": 1}").unwrap();
        assert_eq!(settings, AppSettings::default());
    }

    #[test]
    fn invalid_preset_duration_is_rejected() {
        assert!(AppSettings::from_json("{\"custom_duration\": [0, 60]}").is_err());
    }

    #[test]
    fn invalid_action_is_rejected() {
        assert!(AppSettings::from_json("{\"selected_action\": \"hibernate\"}").is_err());
    }
    #[test]
    fn legacy_windows_xp_theme_is_migrated_to_red_moon() {
        let settings = AppSettings::from_json("{\"theme\": \"windows_xp\"}").unwrap();
        assert_eq!(settings.theme, "red_moon");
    }

    #[test]
    fn invalid_theme_is_rejected() {
        assert!(AppSettings::from_json("{\"theme\": \"solarized\"}").is_err());
    }

    #[test]
    fn unrelated_legacy_json_is_not_deleted() {
        let legacy_path = std::env::temp_dir().join(format!(
            "offtimer-unrelated-settings-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let current_path = legacy_path.with_file_name("offtimer-current-settings.json");
        fs::write(&legacy_path, "{\"some_other_app\": true}").unwrap();

        remove_legacy_settings_file_if_valid(&legacy_path, &current_path);

        assert!(legacy_path.exists());
        let _ = fs::remove_file(legacy_path);
    }

    #[test]
    fn generic_json_objects_are_not_recognized_as_offtimer_settings() {
        assert!(!is_offtimer_settings("{\"some_other_app\": true}"));
        assert!(!is_offtimer_settings("{}"));
    }

    #[test]
    fn valid_legacy_offtimer_settings_are_removed_after_migration() {
        let legacy_path = std::env::temp_dir().join(format!(
            "offtimer-legacy-settings-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let current_path = legacy_path.with_file_name("offtimer-current-settings.json");
        fs::write(&legacy_path, AppSettings::default().to_json()).unwrap();

        remove_legacy_settings_file_if_valid(&legacy_path, &current_path);

        assert!(!legacy_path.exists());
    }
}
