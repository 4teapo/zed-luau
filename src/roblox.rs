use zed_extension_api::{self as zed, Result};

const API_DOCS_URL: &str = "https://raw.githubusercontent.com/MaximumADHD/Roblox-Client-Tracker/roblox/api-docs/en-us.json";
pub const SECURITY_LEVEL_NONE: &str = "None";
pub const SECURITY_LEVEL_LOCAL_USER: &str = "LocalUserSecurity";
pub const SECURITY_LEVEL_PLUGIN: &str = "PluginSecurity";
pub const SECURITY_LEVEL_ROBLOX_SCRIPT: &str = "RobloxScriptSecurity";
pub const API_DOCS_FILE_NAME: &str = "api-docs.json";

pub fn get_definitions_url_for_level(lvl: &str) -> String {
    format!("https://raw.githubusercontent.com/JohnnyMorganz/luau-lsp/main/scripts/globalTypes.{}.d.luau", lvl)
}

pub fn get_definitions_file_for_level(lvl: &str) -> String {
    format!("globalTypes.{}.d.luau", lvl)
}

pub fn download_api_docs() -> Result<()> {
    zed::download_file(
        API_DOCS_URL,
        API_DOCS_FILE_NAME,
        zed::DownloadedFileType::Uncompressed,
    )?;
    Ok(())
}

pub fn download_definitions(security_level: &str) -> Result<()> {
    let url = get_definitions_url_for_level(security_level);
    zed::download_file(
        &url,
        &format!("globalTypes.{}.d.luau", security_level),
        zed::DownloadedFileType::Uncompressed,
    )?;
    Ok(())
}
