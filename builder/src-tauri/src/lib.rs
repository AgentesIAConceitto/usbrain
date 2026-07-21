pub mod commands;
use serde_json::json;

fn maybe_run_factory_qa_on_startup(app_handle: tauri::AppHandle) {
    let Ok(target_mount_path) = std::env::var("USBRAIN_FACTORY_QA_TARGET") else {
        return;
    };
    let trimmed_target = target_mount_path.trim().to_string();
    if trimmed_target.is_empty() {
        return;
    }

    std::thread::spawn(move || {
        let request = serde_json::from_value(json!({
            "targetMountPath": trimmed_target,
            "buildMode": "offline-first",
            "languages": ["pt-BR", "en", "es"],
            "allowGpu": true,
            "allowCloudFallback": false,
            "creatorLabel": "Fabiano Costa",
            "wipeTargetIfNeeded": true
        }))
        .unwrap_or_default();

        let result = commands::run_factory_pipeline(app_handle, Some(request));
        eprintln!("FACTORY_QA_STATUS={}", result.overall_status);
        eprintln!("FACTORY_QA_REPORT={}", result.report_path);
        if let Some(assembly_error) = result.assembly_error {
            eprintln!("FACTORY_QA_ASSEMBLY_ERROR={}", assembly_error);
        }
    });
}

#[cfg(target_os = "macos")]
fn prime_webkit_data_directories() {
    if let Ok(home_dir) = std::env::var("HOME") {
        let webkit_root = std::path::Path::new(&home_dir)
            .join("Library")
            .join("WebKit")
            .join("usbrain-builder")
            .join("WebsiteData");

        for relative_path in [
            "",
            "MediaKeys/v1",
            "IndexedDB",
            "LocalStorage",
            "DeviceIdHashSalts",
            "MediaKeysHashSalts",
            "ResourceLoadStatistics",
            "Default",
            "SearchHistory",
            "ResourceMonitorThrottler",
        ] {
            let target = if relative_path.is_empty() {
                webkit_root.clone()
            } else {
                webkit_root.join(relative_path)
            };

            let _ = std::fs::create_dir_all(target);
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn prime_webkit_data_directories() {}

pub fn run() {
    prime_webkit_data_directories();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            maybe_run_factory_qa_on_startup(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_overview,
            commands::list_schemas,
            commands::get_workspace_snapshot,
            commands::list_candidate_targets,
            commands::get_model_catalog,
            commands::refresh_model_catalog,
            commands::download_catalog_model,
            commands::generate_build_manifest_preview,
            commands::generate_build_plan,
            commands::prepare_builder_library,
            commands::list_builder_library,
            commands::delete_builder_library_artifact,
            commands::download_builder_library_artifact,
            commands::export_build_plan,
            commands::assemble_build_plan,
            commands::run_factory_pipeline,
            commands::eject_target,
            commands::validate_assembled_build,
            commands::run_assembled_smoke_test,
            commands::run_runtime_probe,
            commands::run_end_to_end_probe,
            commands::run_full_local_llm_probe,
            commands::generate_build_readiness_report,
            commands::prepare_distribution_bundle,
            commands::prepare_release_package,
        ])
        .run(tauri::generate_context!())
        .expect("error while running usbrain builder");
}
