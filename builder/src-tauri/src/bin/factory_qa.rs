use std::{env, process::ExitCode};

use serde_json::json;
use usbrain_builder_lib::commands::{
    assemble_build_plan_for_qa, generate_build_plan, run_assembled_smoke_test, run_full_local_llm_probe,
    validate_assembled_build, ManifestPreviewRequest,
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
        "wipeTargetIfNeeded": true
    }))
    .unwrap_or_default();

    let build_plan = generate_build_plan(Some(request.clone()));
    let build_plan_value = serde_json::to_value(&build_plan).unwrap_or_else(|_| json!({"serialization": "failed"}));
    eprintln!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "stage": "plan",
            "buildPlan": build_plan_value,
        }))
        .unwrap_or_else(|_| "{\"stage\":\"plan-error\"}".to_string())
    );

    let assembly = match assemble_build_plan_for_qa(Some(request.clone())) {
        Ok(result) => result,
        Err(error) => {
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "stage": "assembly",
                    "status": "failed",
                    "error": error,
                }))
                .unwrap_or_else(|_| "{\"stage\":\"assembly\",\"status\":\"failed\"}".to_string())
            );
            return ExitCode::from(1);
        }
    };

    let validation = validate_assembled_build(Some(request.clone()));
    let smoke = run_assembled_smoke_test(Some(request.clone()));
    let full_probe = run_full_local_llm_probe(Some(request));

    let qa_result = json!({
        "stage": "factory-qa",
        "assembly": assembly,
        "validation": validation,
        "smoke": smoke,
        "fullLocalLlmProbe": full_probe,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&qa_result)
            .unwrap_or_else(|_| "{\"stage\":\"factory-qa-serialize-error\"}".to_string())
    );

    if qa_result["validation"]["validationStatus"] == "failed"
        || qa_result["smoke"]["smokeStatus"] == "failed"
        || qa_result["fullLocalLlmProbe"]["probeStatus"] == "failed"
    {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}
