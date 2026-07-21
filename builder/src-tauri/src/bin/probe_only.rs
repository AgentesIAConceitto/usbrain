use std::{env, process::ExitCode};

use serde_json::json;
use usbrain_builder_lib::commands::{
    run_assembled_smoke_test, run_full_local_llm_probe, validate_assembled_build, ManifestPreviewRequest,
};

fn parse_selected_model_ids() -> Vec<String> {
    env::args()
        .skip(2)
        .filter_map(|arg| arg.strip_prefix("--model=").map(str::to_string))
        .collect()
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let target_mount_path = args.next().unwrap_or_else(|| "/Volumes/USBRAIN".to_string());
    let selected_model_ids = parse_selected_model_ids();

    let request: ManifestPreviewRequest = serde_json::from_value(json!({
        "targetMountPath": target_mount_path,
        "buildMode": "offline-first",
        "languages": ["pt-BR", "en", "es"],
        "allowGpu": true,
        "allowCloudFallback": false,
        "creatorLabel": "Fabiano Costa",
        "selectedModelIds": if selected_model_ids.is_empty() { json!(null) } else { json!(selected_model_ids) },
        "wipeTargetIfNeeded": false
    }))
    .unwrap_or_default();

    let validation = validate_assembled_build(Some(request.clone()));
    let smoke = run_assembled_smoke_test(Some(request.clone()));
    let full_probe = run_full_local_llm_probe(Some(request));

    let result = json!({
        "stage": "probe-only",
        "validation": validation,
        "smoke": smoke,
        "fullLocalLlmProbe": full_probe,
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&result)
            .unwrap_or_else(|_| "{\"stage\":\"probe-only-serialize-error\"}".to_string())
    );

    if result["validation"]["validationStatus"] == "failed"
        || result["smoke"]["smokeStatus"] == "failed"
        || result["fullLocalLlmProbe"]["probeStatus"] == "failed"
    {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
