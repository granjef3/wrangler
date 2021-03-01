mod modules_worker;
mod plain_text;
mod project_assets;
mod service_worker;
mod text_blob;
mod wasm_module;

use reqwest::blocking::multipart::Form;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use ignore::overrides::{Override, OverrideBuilder};
use ignore::WalkBuilder;

use crate::settings::binding;
use crate::settings::toml::{Builder, ScriptFormat, Target};
use crate::sites::AssetManifest;

use plain_text::PlainText;
use project_assets::{ModulesAssets, ServiceWorkerAssets};
use text_blob::TextBlob;
use wasm_module::WasmModule;

// TODO: https://github.com/cloudflare/wrangler/issues/1083
use super::Package;

use self::project_assets::Module;

pub fn build(
    target: &Target,
    asset_manifest: Option<AssetManifest>,
    session_config: Option<serde_json::Value>,
) -> Result<Form, failure::Error> {
    let kv_namespaces = &target.kv_namespaces;
    let mut text_blobs: Vec<TextBlob> = Vec::new();
    let mut plain_texts: Vec<PlainText> = Vec::new();

    // TODO upload these in custom builds
    let wasm_modules: Vec<WasmModule> = Vec::new();

    if let Some(blobs) = &target.text_blobs {
        for (key, blob_path) in blobs.iter() {
            let blob = fs::read_to_string(blob_path)?;
            text_blobs.push(TextBlob::new(blob, key.clone())?);
        }
    }

    if let Some(vars) = &target.vars {
        for (key, value) in vars.iter() {
            plain_texts.push(PlainText::new(key.clone(), value.clone())?)
        }
    }

    match &target.build {
        Some(config) => match &config.upload_format {
            ScriptFormat::ServiceWorker => {
                log::info!("Plain JavaScript project detected. Publishing...");
                let package_dir = target.package_dir()?;
                let package = Package::new(&package_dir)?;
                let script_path = package.main(&package_dir)?;

                if let Some(asset_manifest) = asset_manifest {
                    log::info!("adding __STATIC_CONTENT_MANIFEST");
                    let binding = "__STATIC_CONTENT_MANIFEST".to_string();
                    let asset_manifest_blob = get_asset_manifest_blob(asset_manifest)?;
                    let text_blob = TextBlob::new(asset_manifest_blob, binding)?;
                    text_blobs.push(text_blob);
                }

                let assets = ServiceWorkerAssets::new(
                    script_path,
                    wasm_modules,
                    kv_namespaces.to_vec(),
                    text_blobs,
                    plain_texts,
                )?;

                service_worker::build_form(&assets, session_config)
            }
            ScriptFormat::Modules => {
                let package_dir = target.package_dir()?;
                let package = Package::new(&package_dir)?;
                let main_module = package.module(&package_dir)?;
                let main_module_name = filename_from_path(&main_module)
                    .ok_or_else(|| failure::err_msg("filename required for main module"))?;

                let ignore = build_ignore(config, &package_dir)?;
                let modules_iter = WalkBuilder::new(config.upload_dir.clone())
                    .standard_filters(false)
                    .hidden(true)
                    .overrides(ignore)
                    .build();

                let mut modules: Vec<Module> = vec![];

                for entry in modules_iter {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() {
                        log::info!("Adding module {}", path.display());
                        modules.push(Module::new(path.to_owned())?);
                    }
                }

                let assets = ModulesAssets::new(
                    main_module_name,
                    modules,
                    kv_namespaces.to_vec(),
                    plain_texts,
                )?;

                modules_worker::build_form(&assets, session_config)
            }
        },
        None => {
            log::info!("Plain JavaScript project detected. Publishing...");
            let package_dir = target.package_dir()?;
            let package = Package::new(&package_dir)?;
            let script_path = package.main(&package_dir)?;

            let assets = ServiceWorkerAssets::new(
                script_path,
                wasm_modules,
                kv_namespaces.to_vec(),
                text_blobs,
                plain_texts,
            )?;

            service_worker::build_form(&assets, session_config)
        }
    }
}

fn get_asset_manifest_blob(asset_manifest: AssetManifest) -> Result<String, failure::Error> {
    let asset_manifest = serde_json::to_string(&asset_manifest)?;
    Ok(asset_manifest)
}

fn filestem_from_path(path: &PathBuf) -> Option<String> {
    path.file_stem()?.to_str().map(|s| s.to_string())
}

fn filename_from_path(path: &PathBuf) -> Option<String> {
    path.file_name()
        .map(|filename| filename.to_string_lossy().into_owned())
}

fn build_ignore(config: &Builder, directory: &Path) -> Result<Override, failure::Error> {
    let mut overrides = OverrideBuilder::new(directory);
    // If `include` present, use it and don't touch the `exclude` field
    if let Some(included) = &config.upload_include {
        for i in included {
            overrides.add(&i)?;
            log::info!("Including {}", i);
        }
    } else if let Some(excluded) = &config.upload_exclude {
        for e in excluded {
            overrides.add(&format!("!{}", e))?;
            log::info!("Ignoring {}", e);
        }
    }

    Ok(overrides.build()?)
}
