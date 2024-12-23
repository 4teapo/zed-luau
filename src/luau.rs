use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use zed::lsp::CompletionKind;
use zed::settings::LspSettings;
use zed::{serde_json, CodeLabel, CodeLabelSpan, LanguageServerId};
use zed_extension_api::{self as zed, Result};

mod roblox;

const FFLAG_URL: &str =
    "https://clientsettingscdn.roblox.com/v1/settings/application?applicationName=PCDesktopClient";
const FFLAG_PREFIXES: &[&str] = &["FFlag", "FInt", "DFFlag", "DFInt"];
const FFLAG_FILE_NAME: &str = "fflags.json";
const LUAU_LSP_BINARY_DIR_NAME: &str = "luau-lsp-binaries";

#[derive(Debug, Deserialize)]
struct ExtSettings {
    #[serde(default)]
    roblox: ExtRobloxSettings,
    #[serde(default)]
    fflags: ExtFFlagsSettings,
    #[serde(default)]
    binary: ExtBinarySettings,
    #[serde(default)]
    definitions: Vec<String>,
    #[serde(default)]
    documentation: Vec<String>,
}

impl Default for ExtSettings {
    fn default() -> Self {
        ExtSettings {
            roblox: Default::default(),
            fflags: Default::default(),
            binary: Default::default(),
            definitions: Default::default(),
            documentation: Default::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ExtRobloxSettings {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    security_level: SecurityLevel,
}

impl Default for ExtRobloxSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            security_level: SecurityLevel::Plugin,
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

impl Default for SecurityLevel {
    fn default() -> Self {
        SecurityLevel::RobloxScript
    }
}

#[derive(Debug, Deserialize)]
struct ExtFFlagsSettings {
    #[serde(default)]
    enable_by_default: bool,
    #[serde(default)]
    sync: bool,
    #[serde(default, rename = "override")]
    overrides: HashMap<String, String>,
}

impl Default for ExtFFlagsSettings {
    fn default() -> Self {
        Self {
            enable_by_default: false,
            sync: true,
            overrides: Default::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ExtBinarySettings {
    #[serde(default)]
    ignore_system_version: bool,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    args: Vec<String>,
}

impl Default for ExtBinarySettings {
    fn default() -> Self {
        Self {
            ignore_system_version: false,
            path: None,
            args: Default::default(),
        }
    }
}

struct LuauExtension {
    cached_binary_path: Option<String>,
}

fn is_file(path: &str) -> bool {
    fs::metadata(path).map_or(false, |stat| stat.is_file())
}

fn is_dir(path: &str) -> bool {
    fs::metadata(path).map_or(false, |stat| stat.is_dir())
}

fn get_ext_settings(settings_val: Option<serde_json::Value>) -> Result<ExtSettings> {
    let Some(mut settings_val) = settings_val else {
        return Ok(ExtSettings::default());
    };

    let Some(settings) = settings_val.as_object_mut() else {
        return Err("invalid luau-lsp settings: `settings` must be an object, but isn't.".into());
    };

    let Some(ext_settings_val) = settings.remove("ext") else {
        return Ok(ExtSettings::default());
    };

    serde_path_to_error::deserialize(ext_settings_val).map_err(|e| e.to_string())
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

impl LuauExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
        settings: &ExtSettings,
    ) -> Result<String> {
        if let Some(path) = &settings.binary.path {
            return Ok(path.clone());
        }

        if !settings.binary.ignore_system_version {
            if let Some(path) = worktree.which("luau-lsp") {
                return Ok(path);
            }
        }

        if let Some(path) = &self.cached_binary_path {
            if is_file(path) {
                return Ok(path.clone());
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
                .map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                if entry.file_name().to_str() != Some(&dir_name) {
                    fs::remove_dir_all(&entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());

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

        let settings = get_ext_settings(lsp_settings.settings)?;

        let mut args = vec!["lsp".into()];

        fn is_path_absolute(path: &str) -> bool {
            let (platform, _) = zed::current_platform();
            match platform {
                zed::Os::Windows => path.contains(':'),
                _ => path.starts_with('/'),
            }
        }

        // Handle documentation and definition settings.
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

            for (name, value) in fflags.iter() {
                args.push(format!("--flag:{}={}", name, value).into());
            }
        }

        if settings.roblox.enabled {
            if !is_file(roblox::API_DOCS_FILE_NAME) {
                roblox::download_api_docs()?;
            }

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

            let current_dir = std::env::current_dir().unwrap();
            let current_dir_str = 'outer: {
                let (platform, _) = zed::current_platform();
                if platform == zed::Os::Windows {
                    // Remove the '/' at the beginning of the path, as Windows paths don't
                    // have it. (Since we're in WASM, it always begins with a '/'.)
                    if let Ok(path) = current_dir.strip_prefix("/") {
                        break 'outer path.display();
                    }
                }
                current_dir.display()
            };
            args.push(format!("--docs={}/{}", &current_dir_str, roblox::API_DOCS_FILE_NAME).into());
            args.push(
                format!(
                    "--definitions={}/{}",
                    &current_dir_str, definitions_file_name
                )
                .into(),
            );
        }

        for arg in &settings.binary.args {
            args.push(arg.into());
        }

        let binary_path =
            self.language_server_binary_path(language_server_id, worktree, &settings)?;
        Ok(zed::Command {
            command: binary_path,
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
