use super::settings::Settings;

pub struct SettingsLoader;

impl SettingsLoader {
    #[must_use]
    pub fn load_default() -> Settings {
        Settings::default()
    }
}
