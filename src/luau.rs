use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use zed::lsp::CompletionKind;
use zed::settings::LspSettings;
use zed::{CodeLabel, CodeLabelSpan, LanguageServerId, serde_json};
use zed_extension_api::{self as zed, Result};

mod roblox;

const FFLAG_URL: &str =
    "https://clientsettingscdn.roblox.com/v1/settings/application?applicationName=PCDesktopClient";
const FFLAG_PREFIXES: &[&str] = &["FFlag", "FInt", "DFFlag", "DFInt"];
const FFLAG_FILE_NAME: &str = "fflags.json";
const LUAU_LSP_BINARY_DIR_NAME: &str = "luau-lsp-binaries";
const PROXY_BINARY_DIR_NAME: &str = "proxy-binaries";

#[derive(Debug, Deserialize)]
#[serde(default)]
struct Settings {
    roblox: RobloxSettings,
    fflags: FFlagsSettings,
    binary: BinarySettings,
    plugin: PluginSettings,
    definitions: Vec<String>,
    documentation: Vec<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            roblox: Default::default(),
            fflags: Default::default(),
            binary: Default::default(),
            plugin: Default::default(),
            definitions: Default::default(),
            documentation: Default::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct RobloxSettings {
    enabled: bool,
    security_level: SecurityLevel,
    download_api_documentation: bool,
    download_definitions: bool,
}

impl Default for RobloxSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            security_level: SecurityLevel::Plugin,
            download_api_documentation: true,
            download_definitions: true,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SecurityLevel {
    RobloxScript,
    LocalUser,
    Plugin,
    None,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct FFlagsSettings {
    enable_by_default: bool,
    enable_new_solver: bool,
    sync: bool,
    #[serde(rename = "override")]
    overrides: HashMap<String, String>,
}

impl Default for FFlagsSettings {
    fn default() -> Self {
        Self {
            enable_by_default: false,
            enable_new_solver: false,
            sync: true,
            overrides: Default::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct BinarySettings {
    ignore_system_version: bool,
    path: Option<String>,
    args: Vec<String>,
}

impl Default for BinarySettings {
    fn default() -> Self {
        Self {
            ignore_system_version: false,
            path: None,
            args: Default::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct PluginSettings {
    enabled: bool,
    port: u16,
    proxy_path: Option<String>,
}

impl Default for PluginSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 3667,
            proxy_path: None,
        }
    }
}

struct LuauExtension {
    cached_binary_path: Option<String>,
    cached_proxy_path: Option<String>,
}

fn is_file(path: &str) -> bool {
    fs::metadata(path).map_or(false, |stat| stat.is_file())
}

fn is_dir(path: &str) -> bool {
    fs::metadata(path).map_or(false, |stat| stat.is_dir())
}

fn get_extension_settings(settings_val: Option<serde_json::Value>) -> Result<Settings> {
    let Some(mut settings_val) = settings_val else {
        return Ok(Settings::default());
    };

    let Some(settings) = settings_val.as_object_mut() else {
        return Err("invalid luau-lsp settings: `settings` must be an object, but isn't.".into());
    };

    let value = settings.remove("ext").unwrap_or(settings_val);

    serde_path_to_error::deserialize(value).map_err(|e| e.to_string())
}

fn download_fflags() -> Result<()> {
    zed::download_file(
        FFLAG_URL,
        FFLAG_FILE_NAME,
        zed::DownloadedFileType::Uncompressed,
    )
    .map_err(|e| format!("failed to download file: {e}"))?;
    Ok(())
}

struct BinaryPath {
    path: String,
    is_extension_owned: bool,
}

impl LuauExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
        settings: &Settings,
    ) -> Result<BinaryPath> {
        if let Some(path) = &settings.binary.path {
            return Ok(BinaryPath {
                path: path.clone(),
                is_extension_owned: false,
            });
        }

        if !settings.binary.ignore_system_version {
            if let Some(path) = worktree.which("luau-lsp") {
                return Ok(BinaryPath {
                    path,
                    is_extension_owned: false,
                });
            }
        }

        if let Some(path) = &self.cached_binary_path {
            if is_file(path) {
                return Ok(BinaryPath {
                    path: path.clone(),
                    is_extension_owned: false,
                });
            }
        }

        zed::set_language_server_installation_status(
            &language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = zed::latest_github_release(
            "JohnnyMorganz/luau-lsp",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();

        let asset_name = match platform {
            zed::Os::Mac => "luau-lsp-macos.zip",
            zed::Os::Windows => "luau-lsp-win64.zip",
            zed::Os::Linux => match arch {
                zed::Architecture::Aarch64 => "luau-lsp-linux-arm64.zip",
                _ => "luau-lsp-linux.zip",
            },
        };

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let dir_name = format!("luau-lsp-{}", release.version);
        let version_dir = format!("{LUAU_LSP_BINARY_DIR_NAME}/{dir_name}");
        let binary_path = format!("{version_dir}/luau-lsp");

        if !is_dir(LUAU_LSP_BINARY_DIR_NAME) {
            fs::create_dir(LUAU_LSP_BINARY_DIR_NAME)
                .map_err(|e| format!("failed to create directory for the luau-lsp binary: {e}"))?;
        }

        if !is_file(&binary_path) {
            zed::set_language_server_installation_status(
                &language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::Zip,
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            zed::make_file_executable(&binary_path)?;

            let entries = fs::read_dir(LUAU_LSP_BINARY_DIR_NAME)
                .map_err(|e| format!("failed to list luau-lsp binary directory {e}"))?;
            for entry in entries {
                let entry = entry
                    .map_err(|e| format!("failed to load luau-lsp binary directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&dir_name) {
                    fs::remove_dir_all(&entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());

        Ok(BinaryPath {
            path: binary_path,
            is_extension_owned: true,
        })
    }

    fn proxy_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        settings: &Settings,
    ) -> Result<String> {
        if let Some(path) = &settings.plugin.proxy_path {
            return Ok(path.clone());
        }

        zed::set_language_server_installation_status(
            &language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        // We version pin the proxy so that we don't need to worry about backwards compatibility for it.
        let release = zed::github_release_by_tag_name("4teapo/luau-lsp-proxy", "v0.1.0")?;

        let (platform, arch) = zed::current_platform();

        let asset_name = format!(
            "luau-lsp-proxy-{version}-{os}-{arch}.zip",
            version = {
                let mut chars = release.version.chars();
                chars.next();
                chars.as_str()
            },
            os = match platform {
                zed::Os::Mac => "macos",
                zed::Os::Windows => "windows",
                zed::Os::Linux => "linux",
            },
            arch = match arch {
                zed::Architecture::Aarch64 => "aarch64",
                _ => "x86_64",
            },
        );

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let dir_name = format!("luau-lsp-proxy-{}", release.version);
        let version_dir = format!("{PROXY_BINARY_DIR_NAME}/{dir_name}");
        let binary_path = format!("{version_dir}/luau-lsp-proxy");

        if !is_dir(PROXY_BINARY_DIR_NAME) {
            fs::create_dir(PROXY_BINARY_DIR_NAME)
                .map_err(|e| format!("failed to create directory for the proxy binary: {e}"))?;
        }

        if !is_file(&binary_path) {
            zed::set_language_server_installation_status(
                &language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::Zip,
            )
            .map_err(|e| format!("failed to download file: {e}"))?;

            zed::make_file_executable(&binary_path)?;

            let entries = fs::read_dir(PROXY_BINARY_DIR_NAME)
                .map_err(|e| format!("failed to list proxy binary directory {e}"))?;
            for entry in entries {
                let entry = entry
                    .map_err(|e| format!("failed to load proxy binary directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&dir_name) {
                    fs::remove_dir_all(&entry.path()).ok();
                }
            }
        }

        self.cached_proxy_path = Some(binary_path.clone());

        Ok(binary_path)
    }
}

impl zed::Extension for LuauExtension {
    fn new() -> Self {
        // Try deleting files for definitions, docs & fflags to make sure they are downloaded again
        // later, keeping them up to date.
        fs::remove_file(FFLAG_FILE_NAME).ok();
        fs::remove_file(roblox::API_DOCS_FILE_NAME).ok();
        fs::remove_file(roblox::get_definitions_file_for_level(
            roblox::SECURITY_LEVEL_ROBLOX_SCRIPT,
        ))
        .ok();
        fs::remove_file(roblox::get_definitions_file_for_level(
            roblox::SECURITY_LEVEL_LOCAL_USER,
        ))
        .ok();
        fs::remove_file(roblox::get_definitions_file_for_level(
            roblox::SECURITY_LEVEL_PLUGIN,
        ))
        .ok();
        fs::remove_file(roblox::get_definitions_file_for_level(
            roblox::SECURITY_LEVEL_NONE,
        ))
        .ok();
        Self {
            cached_binary_path: None,
            cached_proxy_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let lsp_settings = match LspSettings::for_worktree(language_server_id.as_ref(), worktree) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        let settings = get_extension_settings(lsp_settings.settings)?;

        let binary_path =
            self.language_server_binary_path(language_server_id, worktree, &settings)?;

        let current_dir = std::env::current_dir().unwrap();
        let current_dir_str = 'outer: {
            let (platform, _) = zed::current_platform();
            if platform == zed::Os::Windows {
                // Remove the '/' at the beginning of the path, as Windows paths don't have it.
                // (Since we're in WASM, paths always begin with a '/'.)
                if let Ok(path) = current_dir.strip_prefix("/") {
                    break 'outer path.display();
                }
            }
            current_dir.display()
        };

        fn is_path_absolute(path: &str) -> bool {
            let (platform, _) = zed::current_platform();
            match platform {
                // We need to handle Windows manually because of our UNIX-based WASM environment
                zed::Os::Windows => {
                    let mut chars = path.chars();
                    match (chars.next(), chars.next(), chars.next()) {
                        (Some(drive), Some(':'), Some(sep)) => {
                            drive.is_ascii_alphabetic() && (sep == '\\' || sep == '/')
                        }
                        // UNC path
                        _ => path.starts_with("//") || path.starts_with("\\\\"),
                    }
                }
                _ => Path::new(OsStr::new(path)).is_absolute(),
            }
        }

        let mut args: Vec<String> = Vec::new();
        if settings.plugin.enabled {
            args.push(settings.plugin.port.to_string());
            if binary_path.is_extension_owned {
                args.push(format!("{}/{}", current_dir_str, binary_path.path.clone()).into());
            } else {
                args.push(binary_path.path.clone().into());
            }
        }
        args.push("lsp".into());

        // Handle fflag settings.
        {
            if !settings.fflags.enable_by_default {
                args.push("--no-flags-enabled".into());
            }

            let mut fflags = HashMap::new();

            if settings.fflags.sync {
                if !is_file(FFLAG_FILE_NAME) {
                    download_fflags()?;
                }
                let as_str = fs::read_to_string(FFLAG_FILE_NAME)
                    .map_err(|e| format!("failed to read fflags.json: {e}"))?;
                let json: serde_json::Value = serde_json::from_str(&as_str)
                    .map_err(|e| format!("failed to parse fflags.json: {e}"))?;
                let Some(json_map) = json.as_object() else {
                    return Err("failed to sync fflags: error when parsing fetched fflags: fflags must be an object, but isn't.".into());
                };
                let Some(app_settings_val) = json_map.get("applicationSettings") else {
                    return Err("failed to sync fflags: error when reading parsed fflags: json.applicationSettings must exist, but doesn't.".into());
                };
                let Some(app_settings) = app_settings_val.as_object() else {
                    return Err("failed to sync fflags: error when reading parsed fflags: json.applicationSettings must be an object, but isn't.".into());
                };
                for (name, value) in app_settings.iter() {
                    let Some(value) = value.as_str() else {
                        return Err("failed to sync fflags: error when reading parsed fflags: all values in json.applicationSettings must be strings, but one or more aren't.".into());
                    };
                    for prefix in FFLAG_PREFIXES {
                        if name.starts_with(&format!("{prefix}Luau")) {
                            fflags.insert(name[prefix.len()..].into(), value.to_string());
                            break;
                        }
                    }
                }
            }

            for (name, value) in settings.fflags.overrides.iter() {
                if name.len() == 0 || value.len() == 0 {
                    return Err("failed to apply fflag overrides: all overrides must have a non-empty name and value.".into());
                }
                fflags.insert(name.clone(), value.to_string());
            }

            if settings.fflags.enable_new_solver {
                fflags.insert("LuauSolverV2".to_string(), "true".to_string());
                fflags.insert(
                    "LuauNewSolverPopulateTableLocations".to_string(),
                    "true".to_string(),
                );
                fflags.insert(
                    "LuauNewSolverPrePopulateClasses".to_string(),
                    "true".to_string(),
                );
            }

            for (name, value) in fflags.iter() {
                args.push(format!("--flag:{}={}", name, value).into());
            }
        }

        if settings.roblox.enabled {
            if settings.roblox.download_api_documentation {
                if !is_file(roblox::API_DOCS_FILE_NAME) {
                    roblox::download_api_docs()?;
                }
                args.push(
                    format!("--docs={}/{}", &current_dir_str, roblox::API_DOCS_FILE_NAME).into(),
                );
            }

            if settings.roblox.download_definitions {
                let security_level = match settings.roblox.security_level {
                    SecurityLevel::None => roblox::SECURITY_LEVEL_NONE,
                    SecurityLevel::RobloxScript => roblox::SECURITY_LEVEL_ROBLOX_SCRIPT,
                    SecurityLevel::LocalUser => roblox::SECURITY_LEVEL_LOCAL_USER,
                    SecurityLevel::Plugin => roblox::SECURITY_LEVEL_PLUGIN,
                };

                let definitions_file_name = roblox::get_definitions_file_for_level(security_level);

                if !is_file(&definitions_file_name) {
                    roblox::download_definitions(security_level)?;
                }
                args.push(
                    format!(
                        "--definitions={}/{}",
                        &current_dir_str, definitions_file_name
                    )
                    .into(),
                );
            }
        }

        // Handle documentation and definition settings.
        // Happens after handling Roblox settings because we want these to be added after the
        // Roblox definition files are, because otherwise they can't depend on the Roblox types.
        {
            fn get_prefix<'a>(path: &str, proj_root_str: &'a str) -> &'a str {
                match is_path_absolute(path) {
                    true => "",
                    false => proj_root_str,
                }
            }

            let proj_root_str = &format!("{}/", worktree.root_path());

            for def in &settings.definitions {
                let prefix = get_prefix(&def, &proj_root_str);
                args.push(format!("--definitions={prefix}{def}").into());
            }

            for doc in &settings.documentation {
                let prefix = get_prefix(&doc, &proj_root_str);
                args.push(format!("--docs={prefix}{doc}").into());
            }
        }

        for arg in &settings.binary.args {
            args.push(arg.into());
        }

        let command = if settings.plugin.enabled {
            self.proxy_binary_path(language_server_id, &settings)?
        } else {
            binary_path.path.clone()
        };

        Ok(zed::Command {
            command,
            args,
            env: Default::default(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let lsp_settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;
        let initialization_options = lsp_settings.initialization_options;
        Ok(initialization_options)
    }

    fn label_for_completion(
        &self,
        _language_server_id: &LanguageServerId,
        completion: zed::lsp::Completion,
    ) -> Option<CodeLabel> {
        match completion.kind? {
            CompletionKind::Method | CompletionKind::Function => {
                let name_len = completion.label.find('(').unwrap_or(completion.label.len());
                Some(CodeLabel {
                    spans: vec![CodeLabelSpan::code_range(0..completion.label.len())],
                    filter_range: (0..name_len).into(),
                    code: completion.label,
                })
            }
            CompletionKind::Field => Some(CodeLabel {
                spans: vec![CodeLabelSpan::literal(
                    completion.label.clone(),
                    Some("property".into()),
                )],
                filter_range: (0..completion.label.len()).into(),
                code: Default::default(),
            }),
            _ => None,
        }
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), _worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings.clone())
            .unwrap_or_default();
        Ok(Some(serde_json::json!(settings)))
    }
}

zed::register_extension!(LuauExtension);
