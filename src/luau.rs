use std::collections::HashMap;
use std::fs;
// use std::process::{Child, Command};
use zed::lsp::CompletionKind;
use zed::settings::LspSettings;
use zed::{serde_json, CodeLabel, CodeLabelSpan, LanguageServerId};
use zed_extension_api::{self as zed, Result};

mod roblox;

const FFLAG_URL: &str =
    "https://clientsettingscdn.roblox.com/v1/settings/application?applicationName=PCDesktopClient";

const FFLAG_FILE_NAME: &str = "fflags.json";

const LUAU_LSP_BINARY_DIR_NAME: &str = "luau-lsp-binaries";

type JsonObject = serde_json::Map<String, serde_json::Value>;

struct LuauExtension {
    cached_binary_path: Option<String>,
    cached_fflag_file_path: Option<String>,
}

pub(crate) fn is_file(path: &str) -> bool {
    fs::metadata(path).map_or(false, |stat| stat.is_file())
}

pub(crate) fn is_dir(path: &str) -> bool {
    fs::metadata(path).map_or(false, |stat| stat.is_dir())
}

fn get_ext_settings(
    language_server_id: &LanguageServerId,
    worktree: &zed::Worktree,
) -> Option<Result<JsonObject>> {
    let lsp_settings = match LspSettings::for_worktree(language_server_id.as_ref(), worktree) {
        Ok(v) => v,
        Err(e) => return Some(Err(e)),
    };

    let Some(settings_val) = lsp_settings.settings else {
        return None;
    };

    let Some(settings) = settings_val.as_object() else {
        return Some(Err(
            "invalid luau-lsp settings: `settings` must be an object, but isn't.".into(),
        ));
    };

    let Some(ext_settings_val) = settings.get("ext") else {
        return None;
    };

    match ext_settings_val.as_object() {
        Some(v) => Some(Ok(v.clone())),
        None => None,
    }
}

fn get_roblox_settings(ext_settings: JsonObject) -> Result<Option<JsonObject>> {
    if let Some(roblox_settings_val) = ext_settings.get("roblox") {
        let Some(roblox_settings) = roblox_settings_val.as_object() else {
            return Err(
                "invalid luau-lsp settings: `settings.ext.roblox` must be an object, but isn't."
                    .into(),
            );
        };

        if let Some(enabled_val) = roblox_settings.get("enabled") {
            let Some(enabled) = enabled_val.as_bool() else {
                return Err(
                    "invalid luau-lsp settings: `settings.ext.roblox` must be a bool, but isn't."
                        .into(),
                );
            };
            if enabled == false {
                return Ok(None);
            }
        }
        return Ok(Some(roblox_settings.clone()));
    }
    Ok(None)
}

impl LuauExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        if let Some(Ok(ext_settings)) = get_ext_settings(language_server_id, worktree) {
            match ext_settings.get("prefer_worktree_binary") {
                Some(val) => match val.as_bool() {
                    Some(b) => if b {
                        if let Some(path) = worktree.which("luau-lsp") {
                            return Ok(path);
                        }
                    },
                    None => return Err("invalid luau-lsp settings: `settings.ext.prefer_worktree_binary` must be a bool, but isn't.".into())
                },
                None => {}
            }
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
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

    fn fflag_file_path(&mut self) -> Result<String> {
        if let Some(path) = &self.cached_fflag_file_path {
            if is_file(path) {
                return Ok(path.clone());
            }
        }

        let file_path = "fflags.json";
        zed::download_file(FFLAG_URL, file_path, zed::DownloadedFileType::Uncompressed)
            .map_err(|e| format!("failed to download file: {e}"))?;

        self.cached_fflag_file_path = Some(file_path.to_string());

        Ok(file_path.to_string())
    }
}

impl zed::Extension for LuauExtension {
    fn new() -> Self {
        // Try deleting files for definitions, docs & fflags to make sure they are downladed again
        // later, so that they're up to date.
        fs::remove_file(FFLAG_FILE_NAME).ok();
        fs::remove_file(roblox::API_DOCS_FILE_NAME).ok();
        fs::remove_file(roblox::DEFINITIONS_FILE_NAME_ROBLOX_SCRIPT).ok();
        fs::remove_file(roblox::DEFINITIONS_FILE_NAME_LOCAL_USER).ok();
        fs::remove_file(roblox::DEFINITIONS_FILE_NAME_PLUGIN).ok();
        fs::remove_file(roblox::DEFINITIONS_FILE_NAME_NONE).ok();
        Self {
            cached_binary_path: None,
            cached_fflag_file_path: None,
            // sourcemap_gen: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let mut args = vec!["lsp".into()];
        if let Some(Ok(ext_settings)) = get_ext_settings(language_server_id, worktree) {
            let proj_root_str_f = &format!("{}/", worktree.root_path());

            fn is_path_absolute(path: &str) -> bool {
                let (platform, _) = zed::current_platform();
                if platform == zed::Os::Windows {
                    path.contains(':')
                } else {
                    path.starts_with('/')
                }
            }

            if let Some(definitions_settings_val) = ext_settings.get("definitions") {
                let Some(definitions_settings) = definitions_settings_val.as_array() else {
                    return Err("invalid luau-lsp settings: `settings.ext.definitions` must be an array, but isn't.".into());
                };
                for def in definitions_settings {
                    let Some(def_str) = def.as_str() else {
                        return Err("invalid luau-lsp settings: `settings.ext.definitions.*` all elements must be strings, but one or more aren't.".into());
                    };
                    let begin = if is_path_absolute(def_str) {
                        ""
                    } else {
                        proj_root_str_f
                    };
                    args.push(format!("--definitions={begin}{def_str}").into());
                }
            }

            if let Some(doc_settings_val) = ext_settings.get("documentation") {
                let Some(doc_settings) = doc_settings_val.as_array() else {
                    return Err("invalid luau-lsp settings: `settings.ext.documentation` must be an array, but isn't.".into());
                };
                for doc in doc_settings {
                    let Some(doc_str) = doc.as_str() else {
                        return Err("invalid luau-lsp settings: `settings.ext.documentation.*` all elements must be strings, but one or more aren't.".into());
                    };
                    let begin = if is_path_absolute(doc_str) {
                        ""
                    } else {
                        proj_root_str_f
                    };
                    args.push(format!("--docs={begin}{doc_str}").into());
                }
            }

            if let Some(fflags_settings_val) = ext_settings.get("fflags") {
                let Some(fflags_settings) = fflags_settings_val.as_object() else {
                    return Err("invalid luau-lsp settings: `settings.ext.fflags` must be an object, but isn't.".into());
                };

                if let Some(enable_by_default_val) = fflags_settings.get("enable_by_default") {
                    let Some(enable_by_default) = enable_by_default_val.as_bool() else {
                        return Err("invalid luau-lsp settings: `settings.ext.fflags.enable_by_default` must be a bool, but isn't.".into());
                    };
                    if enable_by_default == false {
                        args.push("--no-flags-enabled".into());
                    }
                }

                let mut fflags: HashMap<String, String> = HashMap::new();

                // NOTE: This needs to happen after enable_by_default, so that
                // it overrides those, but before override, so that these can
                // get overridden.
                if let Some(sync_val) = fflags_settings.get("sync") {
                    let Some(sync) = sync_val.as_bool() else {
                        return Err("invalid luau-lsp settings: `settings.ext.fflags.sync` must be a bool, but isn't.".into());
                    };
                    if sync == true {
                        let path = self.fflag_file_path()?;
                        let as_str = fs::read_to_string(path)
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
                        for (k, v) in app_settings.iter() {
                            if !k.starts_with("FFlagLuau") {
                                continue;
                            }
                            let Some(val) = v.as_str() else {
                                return Err("failed to sync fflags: error when reading parsed fflags: json.applicationSettings.* all values must be strings, but one or more aren't.".into());
                            };
                            fflags.insert(k[5..].into(), val.to_string());
                        }
                    }
                }

                if let Some(override_val) = fflags_settings.get("override") {
                    let Some(override_map) = override_val.as_object() else {
                        return Err("invalid luau-lsp settings: `settings.ext.fflags.override` must be an object, but isn't.".into());
                    };
                    for (k, v) in override_map.iter() {
                        let Some(val) = v.as_str() else {
                            return Err("failed to apply fflag overrides: error when reading fflags: fflags.* all values must be strings, but one or more aren't.".into());
                        };
                        fflags.insert(k.clone(), val.to_string());
                    }
                }

                for (k, v) in fflags.iter() {
                    args.push(format!("--flag:{}={}", k, v).into());
                }
            }

            if let Some(roblox_settings) = get_roblox_settings(ext_settings)? {
                if !is_file(roblox::API_DOCS_FILE_NAME) {
                    roblox::download_api_docs()?;
                }

                let definitions_file_name = match roblox_settings.get("security_level") {
                    Some(security_level) => match security_level.as_str() {
                        Some("roblox_script") => {
                            if !is_file(roblox::DEFINITIONS_FILE_NAME_ROBLOX_SCRIPT) {
                                roblox::download_definitions_roblox_script()?;
                            }
                            roblox::DEFINITIONS_FILE_NAME_ROBLOX_SCRIPT
                        },
                        Some("local_user") => {
                            if !is_file(roblox::DEFINITIONS_FILE_NAME_LOCAL_USER) {
                                roblox::download_definitions_local_user()?;
                            }
                            roblox::DEFINITIONS_FILE_NAME_LOCAL_USER
                        },
                        Some("plugin") => {
                            if !is_file(roblox::DEFINITIONS_FILE_NAME_PLUGIN) {
                                roblox::download_definitions_plugin()?;
                            }
                            roblox::DEFINITIONS_FILE_NAME_PLUGIN
                        },
                        Some("none") => {
                            if !is_file(roblox::DEFINITIONS_FILE_NAME_NONE) {
                                roblox::download_definitions_none()?;
                            }
                            roblox::DEFINITIONS_FILE_NAME_NONE
                        },
                        Some(_) => return Err("invalid luau-lsp settings: `settings.ext.roblox.security_level must be `roblox_script`, `local_user`, `plugin` or `none`, but is neither.".into()),
                        None => return Err("invalid luau-lsp settings: `settings.ext.roblox.security_level` must be a string, but isn't.".into()),
                    },
                    None => {
                        if !is_file(roblox::DEFINITIONS_FILE_NAME_ROBLOX_SCRIPT) {
                            roblox::download_definitions_roblox_script()?;
                        }
                        roblox::DEFINITIONS_FILE_NAME_ROBLOX_SCRIPT
                    },
                };

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
                args.push(
                    format!("--docs={}/{}", &current_dir_str, roblox::API_DOCS_FILE_NAME).into(),
                );
                args.push(
                    format!(
                        "--definitions={}/{}",
                        &current_dir_str, definitions_file_name
                    )
                    .into(),
                );
            }
        }

        let binary_path = self.language_server_binary_path(language_server_id, worktree)?;
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

    fn label_for_symbol(
        &self,
        _language_server_id: &LanguageServerId,
        symbol: zed::lsp::Symbol,
    ) -> Option<CodeLabel> {
        let prefix = "let a = ";
        let suffix = match symbol.kind {
            zed::lsp::SymbolKind::Method => "()",
            _ => "",
        };
        let code = format!("{prefix}{}{suffix}", symbol.name);
        Some(CodeLabel {
            spans: vec![CodeLabelSpan::code_range(
                prefix.len()..code.len() - suffix.len(),
            )],
            filter_range: (0..symbol.name.len()).into(),
            code,
        })
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
