/*
use crate::{is_file, LuauExtension};
use std::io;
use std::process::{Child, Command};
use zed::serde_json;
*/
use zed_extension_api::{self as zed, Result};

const API_DOCS_URL: &str = "https://raw.githubusercontent.com/MaximumADHD/Roblox-Client-Tracker/roblox/api-docs/en-us.json";
const SECURITY_LEVEL_NONE: &str = "None";
const SECURITY_LEVEL_LOCAL_USER: &str = "LocalUserSecurity";
const SECURITY_LEVEL_PLUGIN: &str = "PluginSecurity";
const SECURITY_LEVEL_ROBLOX_SCRIPT: &str = "RobloxScriptSecurity";
pub const DEFINITIONS_FILE_NAME_NONE: &str = "globalTypes.None.d.luau";
pub const DEFINITIONS_FILE_NAME_LOCAL_USER: &str = "globalTypes.LocalUser.d.luau";
pub const DEFINITIONS_FILE_NAME_PLUGIN: &str = "globalTypes.Plugin.d.luau";
pub const DEFINITIONS_FILE_NAME_ROBLOX_SCRIPT: &str = "globalTypes.RobloxScript.d.luau";
pub const API_DOCS_FILE_NAME: &str = "api-docs.json";

pub fn get_definitions_url_for_lvl(lvl: &str) -> String {
    format!("https://raw.githubusercontent.com/JohnnyMorganz/luau-lsp/main/scripts/globalTypes.{}.d.luau", lvl)
}

pub fn download_api_docs() -> Result<()> {
    zed::download_file(
        API_DOCS_URL,
        API_DOCS_FILE_NAME,
        zed::DownloadedFileType::Uncompressed,
    )?;
    Ok(())
}

pub fn download_definitions_with_url(url: String, file_name: &str) -> Result<()> {
    zed::download_file(&url, file_name, zed::DownloadedFileType::Uncompressed)?;
    Ok(())
}

pub fn download_definitions_none() -> Result<()> {
    let url = get_definitions_url_for_lvl(SECURITY_LEVEL_NONE);
    download_definitions_with_url(url, DEFINITIONS_FILE_NAME_NONE)
}

pub fn download_definitions_local_user() -> Result<()> {
    let url = get_definitions_url_for_lvl(SECURITY_LEVEL_LOCAL_USER);
    download_definitions_with_url(url, DEFINITIONS_FILE_NAME_LOCAL_USER)
}

pub fn download_definitions_plugin() -> Result<()> {
    let url = get_definitions_url_for_lvl(SECURITY_LEVEL_PLUGIN);
    download_definitions_with_url(url, DEFINITIONS_FILE_NAME_PLUGIN)
}

pub fn download_definitions_roblox_script() -> Result<()> {
    let url = get_definitions_url_for_lvl(SECURITY_LEVEL_ROBLOX_SCRIPT);
    download_definitions_with_url(url, DEFINITIONS_FILE_NAME_ROBLOX_SCRIPT)
}

/*

pub fn stop_sourcemap_generation_if_exists(ext: &mut LuauExtension) -> io::Result<()> {
    let Some(mut child) = ext.sourcemap_gen.take() else {
        return Ok(());
    };
    ext.sourcemap_gen = None;
    child.kill()?;
    Ok(())
}

pub fn create_sourcemap_generation_process(
    project_file: &str,
    include_non_scripts: bool,
) -> io::Result<Child> {
    let mut output = Command::new("rojo");
    output
        .arg("sourcemap")
        .arg(project_file)
        .arg("--output")
        .arg("sourcemap.json");
    if include_non_scripts {
        output.arg("--include-non-scripts");
    }
    output.spawn()
}

pub fn get_rojo_project_file(worktree: &zed::Worktree) -> Option<String> {
    let root_path = worktree.root_path();
    let project_file = format!("{}/default.project.json", root_path);
    match is_file(&project_file) {
        // TODO: Switch back.
        false => Some(project_file),
        true => None,
    }
}

pub fn start_sourcemap_generation(
    ext: &mut LuauExtension,
    project_file: &str,
    include_non_scripts: bool,
) -> std::io::Result<()> {
    stop_sourcemap_generation_if_exists(ext)?;
    ext.sourcemap_gen = Some(create_sourcemap_generation_process(
        project_file,
        include_non_scripts,
    )?);
    Ok(())
}

*/
