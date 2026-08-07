use std::path::PathBuf;

pub const APP_NAME: &str = "afterbudget";

pub fn app_data_dir() -> PathBuf {
    let dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    let app_dir = dir.join(APP_NAME);
    std::fs::create_dir_all(&app_dir).ok();
    app_dir
}

pub fn database_path() -> PathBuf {
    app_data_dir().join("afterbudget.sqlite")
}

pub fn backup_dir() -> PathBuf {
    app_data_dir().join("backups")
}
