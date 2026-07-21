use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter};

const SUPPORTED_BUILD_MODES: &[&str] = &[
    "offline-first",
    "offline-only",
    "local-assisted",
    "hybrid",
];

const SUPPORTED_LANGUAGES: &[&str] = &["pt-BR", "en", "es"];
const ROOT_RUNTIME_FILES: &[&str] = &[
    "main.py",
    "agent.py",
    "personality.md",
    "requirements.txt",
    "README.txt",
    "start.bat",
    "start.sh",
    "llamafile",
    "llamafile-linux",
];
const WINDOWS_PYTHON_REQUIRED_FILES: &[&str] = &[
    "python.exe",
    "python3.dll",
    "python312.dll",
    "vcruntime140.dll",
    "DLLs/_ctypes.pyd",
    "DLLs/libffi-8.dll",
    "Lib/site.py",
    "Lib/site-packages/aiohttp/__init__.py",
    "Lib/site-packages/httpx/__init__.py",
    "Lib/site-packages/numpy/__init__.py",
    "Lib/site-packages/pypdf/__init__.py",
    "Lib/site-packages/sentence_transformers/__init__.py",
    "Lib/site-packages/transformers/__init__.py",
    "Lib/site-packages/torch/__init__.py",
    "Lib/site-packages/sklearn/__init__.py",
    "Lib/site-packages/sklearn/.libs/vcomp140.dll",
    "Lib/site-packages/sklearn/.libs/msvcp140.dll",
];
const BUILDER_MARKER_FILE: &str = ".usbrain-builder-stamp.json";
const ASSEMBLY_PROGRESS_EVENT: &str = "usbrain://assembly-progress";

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AssemblyProgressEvent {
    phase: String,
    percent: u8,
    message: String,
}

fn emit_assembly_progress(app_handle: Option<&AppHandle>, phase: &str, percent: u8, message: impl Into<String>) {
    if let Some(app_handle) = app_handle {
        let _ = app_handle.emit(
            ASSEMBLY_PROGRESS_EVENT,
            AssemblyProgressEvent {
                phase: phase.to_string(),
                percent,
                message: message.into(),
            },
        );
    }
}

// #region debug-point A:factory-session-debug-report
fn debug_report_factory_session(hypothesis_id: &str, location: &str, msg: &str, data: serde_json::Value) {
    let env_path = workspace_root().join(".dbg").join("usbrain-same-error.env");
    let debug_env_path = workspace_root().join(".dbg").join("usb-eject-rag-lag.env");
    let mut debug_server_url = "http://127.0.0.1:7777/event".to_string();
    let mut session_id = "usbrain-same-error".to_string();
    let selected_env_path = if debug_env_path.exists() { debug_env_path } else { env_path };
    if let Ok(contents) = fs::read_to_string(selected_env_path) {
        for line in contents.lines() {
            if let Some(value) = line.strip_prefix("DEBUG_SERVER_URL=") {
                debug_server_url = value.trim().to_string();
            } else if let Some(value) = line.strip_prefix("DEBUG_SESSION_ID=") {
                session_id = value.trim().to_string();
            }
        }
    }
    let Ok(payload) = serde_json::to_vec(&json!({
        "sessionId": session_id,
        "runId": "pre-fix",
        "hypothesisId": hypothesis_id,
        "location": location,
        "msg": format!("[DEBUG] {}", msg),
        "data": data,
        "ts": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|value| value.as_millis())
            .unwrap_or_default()
    })) else {
        return;
    };

    let Some(rest) = debug_server_url.strip_prefix("http://") else {
        return;
    };
    let mut parts = rest.splitn(2, '/');
    let host_port = parts.next().unwrap_or("127.0.0.1:7777");
    let path = format!("/{}", parts.next().unwrap_or("event"));
    let mut host_parts = host_port.splitn(2, ':');
    let host = host_parts.next().unwrap_or("127.0.0.1");
    let port = host_parts
        .next()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(7777);
    let Ok(address) = format!("{}:{}", host, port).parse::<SocketAddr>() else {
        return;
    };
    let Ok(mut stream) = TcpStream::connect_timeout(&address, Duration::from_millis(400)) else {
        return;
    };
    let request = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        path,
        host_port,
        payload.len()
    );
    let _ = stream.write_all(request.as_bytes());
    let _ = stream.write_all(&payload);
}
// #endregion

// #region debug-point A:full-probe-debug-report
fn debug_report_full_probe(hypothesis_id: &str, location: &str, msg: &str, data: serde_json::Value) {
    let env_path = workspace_root().join(".dbg").join("llm-probe-gate.env");
    let mut debug_server_url = "http://127.0.0.1:7777/event".to_string();
    let mut session_id = "llm-probe-gate".to_string();
    if let Ok(contents) = fs::read_to_string(env_path) {
        for line in contents.lines() {
            if let Some(value) = line.strip_prefix("DEBUG_SERVER_URL=") {
                debug_server_url = value.trim().to_string();
            } else if let Some(value) = line.strip_prefix("DEBUG_SESSION_ID=") {
                session_id = value.trim().to_string();
            }
        }
    }
    let Ok(payload) = serde_json::to_vec(&json!({
        "sessionId": session_id,
        "runId": "pre-fix",
        "hypothesisId": hypothesis_id,
        "location": location,
        "msg": format!("[DEBUG] {}", msg),
        "data": data,
        "ts": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|value| value.as_millis())
            .unwrap_or_default()
    })) else {
        return;
    };

    let Some(rest) = debug_server_url.strip_prefix("http://") else {
        return;
    };
    let mut parts = rest.splitn(2, '/');
    let host_port = parts.next().unwrap_or("127.0.0.1:7777");
    let path = format!("/{}", parts.next().unwrap_or("event"));
    let mut host_parts = host_port.splitn(2, ':');
    let host = host_parts.next().unwrap_or("127.0.0.1");
    let port = host_parts
        .next()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(7777);
    let Ok(address) = format!("{}:{}", host, port).parse::<SocketAddr>() else {
        return;
    };
    let Ok(mut stream) = TcpStream::connect_timeout(&address, Duration::from_millis(400)) else {
        return;
    };
    let request = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        path,
        host_port,
        payload.len()
    );
    let _ = stream.write_all(request.as_bytes());
    let _ = stream.write_all(&payload);
}
// #endregion

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppOverview {
    app_name: String,
    app_version: String,
    workspace_root: String,
    schema_count: usize,
    document_count: usize,
    tauri_backend_available: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaSummary {
    file_name: String,
    title: String,
    schema_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSnapshot {
    root_path: String,
    schema_directory: String,
    key_documents: Vec<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CandidateTarget {
    label: String,
    mount_path: String,
    source: String,
    filesystem: String,
    capacity_gb: Option<f64>,
    available_gb: Option<f64>,
    usage_percent: Option<f64>,
    removable: bool,
    recommended_model_tier: String,
    estimated_model_slots: u8,
    warnings: Vec<String>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ManifestPreviewRequest {
    target_mount_path: Option<String>,
    target_label: Option<String>,
    build_mode: Option<String>,
    languages: Option<Vec<String>>,
    allow_gpu: Option<bool>,
    allow_cloud_fallback: Option<bool>,
    creator_label: Option<String>,
    selected_model_ids: Option<Vec<String>>,
    wipe_target_if_needed: Option<bool>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlannedModel {
    installed_model_id: String,
    display_name: String,
    role: String,
    disk_size_gb: f64,
    recommended_ram_gb: f64,
    runtime_family: String,
    source_file_name: String,
    source_absolute_path: String,
    relative_target_path: String,
    available_locally: bool,
    availability_status: String,
    availability_label: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PendingModelDownload {
    model_id: String,
    display_name: String,
    disk_size_gb: f64,
    recommended_ram_gb: f64,
    homepage_or_source: String,
    download_locations: Vec<String>,
    checksums: Vec<String>,
    local_file_name: String,
    relative_target_path: String,
    availability_status: String,
    availability_label: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CatalogBenchmarkSummaryRecord {
    source_label: String,
    summary: String,
    composite_score: Option<f64>,
    rank_label: Option<String>,
    last_verified_at: Option<String>,
    #[serde(default)]
    match_origin: String,
    #[serde(default)]
    confidence_label: String,
    #[serde(default)]
    confidence_level: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CatalogBenchmarkMetricRecord {
    label: String,
    value: f64,
    unit: Option<String>,
    benchmark_suite: Option<String>,
    higher_is_better: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CatalogTechnicalProfileRecord {
    context_window_tokens: Option<u32>,
    estimated_prompt_tokens_per_second: Option<f64>,
    estimated_generation_tokens_per_second: Option<f64>,
    estimated_peak_memory_gb: Option<f64>,
    recommended_gpu_class: Option<String>,
    notes: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CatalogModelRecord {
    model_id: String,
    display_name: String,
    family: String,
    parameter_scale: String,
    variant: String,
    disk_size_gb: f64,
    ram_min_gb: f64,
    ram_recommended_gb: f64,
    host_ram_tier_gb: u16,
    languages: Vec<String>,
    domains: Vec<String>,
    capabilities: Vec<String>,
    quality_score: u8,
    cpu_only_viable: bool,
    gpu_optional: bool,
    license: String,
    homepage_or_source: String,
    description: String,
    rationale: String,
    priority: u8,
    #[serde(default)]
    download_locations: Vec<String>,
    #[serde(default)]
    checksums: Vec<String>,
    #[serde(default)]
    provider_or_origin: Option<String>,
    #[serde(default)]
    variant_id: Option<String>,
    #[serde(default)]
    quantization: Option<String>,
    #[serde(default)]
    file_format: Option<String>,
    #[serde(default)]
    editorial_recommendation: Option<String>,
    #[serde(default)]
    benchmark_summary: Option<CatalogBenchmarkSummaryRecord>,
    #[serde(default)]
    benchmark_metrics: Vec<CatalogBenchmarkMetricRecord>,
    #[serde(default)]
    technical_profile: Option<CatalogTechnicalProfileRecord>,
    local_file_name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ModelCatalogSnapshot {
    catalog_version: String,
    last_updated: String,
    source: String,
    models: Vec<CatalogModelRecord>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct CatalogSourceRecord {
    label: String,
    #[serde(alias = "snapshot_url")]
    snapshot_url: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct CatalogSourcesConfig {
    sources: Vec<CatalogSourceRecord>,
}

#[derive(Clone)]
struct CatalogSourceCandidate {
    label: String,
    url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HuggingFaceSearchRecord {
    id: Option<String>,
    model_id: Option<String>,
    downloads: Option<u64>,
    likes: Option<u64>,
    #[serde(default)]
    tags: Vec<String>,
    pipeline_tag: Option<String>,
    private: Option<bool>,
    gated: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HuggingFaceModelInfoRecord {
    id: String,
    author: Option<String>,
    downloads: Option<u64>,
    likes: Option<u64>,
    #[serde(default)]
    tags: Vec<String>,
    pipeline_tag: Option<String>,
    #[serde(default)]
    siblings: Vec<HuggingFaceSiblingRecord>,
    card_data: Option<HuggingFaceCardDataRecord>,
    gguf: Option<HuggingFaceGgufRecord>,
    private: Option<bool>,
    gated: Option<bool>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct HuggingFaceSiblingRecord {
    rfilename: String,
    size: Option<u64>,
    lfs: Option<HuggingFaceLfsRecord>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct HuggingFaceLfsRecord {
    oid: Option<String>,
    size: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HuggingFaceCardDataRecord {
    license: Option<String>,
    #[serde(default)]
    language: Vec<String>,
    base_model: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HuggingFaceGgufRecord {
    total: Option<u64>,
    total_file_size: Option<u64>,
    architecture: Option<String>,
    context_length: Option<u32>,
}

#[derive(Deserialize)]
struct DatasetServerRowsResponse<T> {
    rows: Vec<DatasetServerRow<T>>,
}

#[derive(Deserialize)]
struct DatasetServerRow<T> {
    row: T,
}

#[derive(Deserialize, Clone)]
struct OpenEvalsBenchmarkRow {
    model_id: String,
    model_name: String,
    provider: Option<String>,
    parameters_billions: Option<f64>,
    license: Option<String>,
    context_window: Option<u32>,
    architecture: Option<String>,
    #[serde(rename = "aime2026_score")]
    aime2026_score: Option<f64>,
    #[serde(rename = "evasionBench_score")]
    evasion_bench_score: Option<f64>,
    #[serde(rename = "gpqa_score")]
    gpqa_score: Option<f64>,
    #[serde(rename = "gsm8k_score")]
    gsm8k_score: Option<f64>,
    #[serde(rename = "hle_score")]
    hle_score: Option<f64>,
    #[serde(rename = "hmmt2026_score")]
    hmmt2026_score: Option<f64>,
    #[serde(rename = "mmluPro_score")]
    mmlu_pro_score: Option<f64>,
    #[serde(rename = "olmOcr_score")]
    olm_ocr_score: Option<f64>,
    #[serde(rename = "swePro_score")]
    swe_pro_score: Option<f64>,
    #[serde(rename = "sweVerified_score")]
    swe_verified_score: Option<f64>,
    #[serde(rename = "terminalBench_score")]
    terminal_bench_score: Option<f64>,
    aggregate_score: Option<f64>,
    coverage_count: Option<u32>,
    coverage_percent: Option<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BenchLmLeaderboardResponse {
    last_updated: Option<String>,
    models: Vec<BenchLmLeaderboardRow>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct BenchLmLeaderboardRow {
    model: String,
    creator: Option<String>,
    source_type: Option<String>,
    overall_score: Option<f64>,
    category_scores: Option<BenchLmCategoryScores>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct BenchLmCategoryScores {
    agentic: Option<f64>,
    coding: Option<f64>,
    reasoning: Option<f64>,
    multimodal_grounded: Option<f64>,
    knowledge: Option<f64>,
    multilingual: Option<f64>,
    instruction_following: Option<f64>,
    math: Option<f64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BenchLmPricingResponse {
    last_updated: Option<String>,
    models: Vec<BenchLmPricingRow>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct BenchLmPricingRow {
    model: String,
    creator: Option<String>,
    context_window: Option<String>,
    source_type: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalogQuery {
    target_mount_path: Option<String>,
    search: Option<String>,
    domain: Option<String>,
    language: Option<String>,
    host_ram_gb: Option<u16>,
    cpu_only_only: Option<bool>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CatalogBenchmarkSummary {
    source_label: String,
    summary: String,
    composite_score: Option<f64>,
    rank_label: Option<String>,
    last_verified_at: Option<String>,
    match_origin: String,
    confidence_label: String,
    confidence_level: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CatalogBenchmarkMetric {
    label: String,
    value: f64,
    unit: Option<String>,
    benchmark_suite: Option<String>,
    higher_is_better: Option<bool>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CatalogTechnicalProfile {
    context_window_tokens: Option<u32>,
    estimated_prompt_tokens_per_second: Option<f64>,
    estimated_generation_tokens_per_second: Option<f64>,
    estimated_peak_memory_gb: Option<f64>,
    recommended_gpu_class: Option<String>,
    notes: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CatalogModelSummary {
    model_id: String,
    display_name: String,
    family: String,
    parameter_scale: String,
    variant: String,
    disk_size_gb: f64,
    ram_min_gb: f64,
    ram_recommended_gb: f64,
    host_ram_tier_gb: u16,
    languages: Vec<String>,
    domains: Vec<String>,
    capabilities: Vec<String>,
    quality_score: u8,
    cpu_only_viable: bool,
    gpu_optional: bool,
    license: String,
    homepage_or_source: String,
    download_locations: Vec<String>,
    checksums: Vec<String>,
    provider_or_origin: Option<String>,
    variant_id: Option<String>,
    quantization: Option<String>,
    file_format: Option<String>,
    editorial_recommendation: Option<String>,
    benchmark_summary: Option<CatalogBenchmarkSummary>,
    benchmark_metrics: Vec<CatalogBenchmarkMetric>,
    technical_profile: Option<CatalogTechnicalProfile>,
    description: String,
    rationale: String,
    local_file_name: Option<String>,
    available_locally: bool,
    availability_status: String,
    availability_label: String,
    fits_selected_target: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalogResponse {
    catalog_version: String,
    last_updated: String,
    source: String,
    install_budget_gb: Option<f64>,
    domain_options: Vec<String>,
    models: Vec<CatalogModelSummary>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RefreshModelCatalogRequest {
    source_url: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshModelCatalogResult {
    status: String,
    source_url: String,
    cache_path: String,
    catalog_version: String,
    last_updated: String,
    model_count: usize,
    warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestedCatalogModel {
    model_id: String,
    display_name: String,
    estimated_disk_size_gb: f64,
    target_host_ram_gb: u16,
    rationale: String,
    available_locally: bool,
    availability_status: String,
    availability_label: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostRamRecommendation {
    host_ram_gb: u16,
    label: String,
    summary: String,
    suggested_model: SuggestedCatalogModel,
    alternatives: Vec<SuggestedCatalogModel>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedPack {
    installed_pack_id: String,
    name: String,
    pack_type: String,
    activation_mode: String,
    category: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedManifest {
    file_name: String,
    title: String,
    contents: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPlanSummary {
    build_id: String,
    target_label: String,
    target_mount_path: String,
    filesystem: String,
    build_mode: String,
    profile_mode: String,
    privacy_mode: String,
    capacity_gb: Option<f64>,
    available_gb: Option<f64>,
    install_budget_gb: Option<f64>,
    estimated_model_slots: u8,
    selected_model_count: usize,
    pending_model_download_count: usize,
    selected_pack_count: usize,
    allow_gpu: bool,
    allow_cloud_fallback: bool,
    primary_language: String,
    languages: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPlanResponse {
    summary: BuildPlanSummary,
    models: Vec<PlannedModel>,
    pending_model_downloads: Vec<PendingModelDownload>,
    host_ram_recommendations: Vec<HostRamRecommendation>,
    packs: Vec<PlannedPack>,
    manifests: Vec<GeneratedManifest>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExportedFileSummary {
    file_name: String,
    absolute_path: String,
    relative_path: String,
    size_bytes: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPlanExportResult {
    build_id: String,
    export_root: String,
    manifests_directory: String,
    summary_file: String,
    exported_files: Vec<ExportedFileSummary>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildDistributionGateResult {
    build_id: String,
    export_root: String,
    distribution_status: String,
    distribution_ready: bool,
    message: String,
    readiness_report_file: String,
    distribution_summary_file: String,
    exported_files: Vec<ExportedFileSummary>,
    readiness_report: BuildReadinessReport,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseArtifactChecksum {
    file_name: String,
    absolute_path: String,
    relative_path: String,
    size_bytes: u64,
    sha256: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildReleasePackageResult {
    build_id: String,
    export_root: String,
    release_status: String,
    release_ready: bool,
    message: String,
    recommended_release_tag: String,
    release_manifest_file: String,
    github_release_notes_file: String,
    usbrain_release_summary_file: String,
    checksums_file: String,
    exported_files: Vec<ExportedFileSummary>,
    artifact_checksums: Vec<ReleaseArtifactChecksum>,
    distribution_result: BuildDistributionGateResult,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssembledDirectorySummary {
    relative_path: String,
    file_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildAssemblyResult {
    build_id: String,
    target_root: String,
    marker_file: String,
    manifest_directory: String,
    selected_models_directory: String,
    exported_files: Vec<ExportedFileSummary>,
    assembled_directories: Vec<AssembledDirectorySummary>,
    warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuilderLibraryPreparationResult {
    build_id: String,
    library_root: String,
    model_library_directory: String,
    download_manifest_directory: String,
    copied_models_count: usize,
    already_available_count: usize,
    written_manifests_count: usize,
    exported_files: Vec<ExportedFileSummary>,
    pending_downloads: Vec<PendingModelDownload>,
    warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuilderLibraryArtifactSummary {
    artifact_id: String,
    artifact_type: String,
    display_name: String,
    source_label: String,
    local_path: String,
    size_bytes: u64,
    integrity_status: String,
    downloaded_at: Option<String>,
    source_url: Option<String>,
    removable: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuilderLibraryInventory {
    library_root: String,
    artifact_count: usize,
    total_size_bytes: u64,
    artifacts: Vec<BuilderLibraryArtifactSummary>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteBuilderLibraryArtifactRequest {
    artifact_type: String,
    artifact_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteBuilderLibraryArtifactResult {
    library_root: String,
    artifact_id: String,
    artifact_type: String,
    deleted_path: String,
    recovered_size_bytes: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadBuilderLibraryArtifactRequest {
    artifact_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadCatalogModelRequest {
    model_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadBuilderLibraryArtifactResult {
    status: String,
    library_root: String,
    artifact_id: String,
    display_name: String,
    source_url: String,
    downloaded_path: String,
    removed_manifest_path: String,
    size_bytes: u64,
    sha256: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationCheck {
    id: String,
    status: String,
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildValidationResult {
    build_id: String,
    target_root: String,
    validation_status: String,
    checks: Vec<ValidationCheck>,
    errors: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmokeTestCheck {
    id: String,
    status: String,
    message: String,
    command: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildSmokeTestResult {
    build_id: String,
    target_root: String,
    smoke_status: String,
    validation_status: String,
    host_python_command: String,
    checks: Vec<SmokeTestCheck>,
    errors: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeProbeCheck {
    id: String,
    status: String,
    message: String,
    command: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeProbeResult {
    build_id: String,
    target_root: String,
    probe_status: String,
    host_python_command: String,
    endpoint_url: String,
    web_port: u16,
    bge_port: u16,
    llm_port: u16,
    duration_ms: u128,
    stdout_log_path: String,
    stderr_log_path: String,
    checks: Vec<RuntimeProbeCheck>,
    errors: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EndToEndProbeCheck {
    id: String,
    status: String,
    message: String,
    command: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EndToEndProbeResult {
    build_id: String,
    target_root: String,
    probe_status: String,
    host_python_command: String,
    stats_endpoint_url: String,
    chat_endpoint_url: String,
    probe_prompt: String,
    response_preview: String,
    web_port: u16,
    bge_port: u16,
    llm_port: u16,
    duration_ms: u128,
    stdout_log_path: String,
    stderr_log_path: String,
    checks: Vec<EndToEndProbeCheck>,
    errors: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullLocalLlmProbeCheck {
    id: String,
    status: String,
    message: String,
    command: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullLocalLlmProbeResult {
    build_id: String,
    target_root: String,
    probe_status: String,
    host_python_command: String,
    stats_endpoint_url: String,
    chat_endpoint_url: String,
    probe_prompt: String,
    response_preview: String,
    selected_model_count: usize,
    web_port: u16,
    bge_port: u16,
    llm_port: u16,
    duration_ms: u128,
    stdout_log_path: String,
    stderr_log_path: String,
    checks: Vec<FullLocalLlmProbeCheck>,
    errors: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FactoryPipelineResult {
    pub build_id: String,
    pub target_root: String,
    pub overall_status: String,
    pub report_path: String,
    pub assembly_result: Option<BuildAssemblyResult>,
    pub assembly_error: Option<String>,
    pub validation_result: Option<BuildValidationResult>,
    pub smoke_test_result: Option<BuildSmokeTestResult>,
    pub full_local_llm_probe_result: Option<FullLocalLlmProbeResult>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadinessSection {
    id: String,
    label: String,
    status: String,
    required: bool,
    summary: String,
    error_count: usize,
    warning_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildReadinessReport {
    build_id: String,
    target_root: String,
    readiness_status: String,
    readiness_score: u8,
    distribution_ready: bool,
    blocked_reason: Option<String>,
    generated_at: String,
    sections: Vec<ReadinessSection>,
    blocking_issues: Vec<String>,
    advisory_issues: Vec<String>,
}

#[derive(Clone)]
struct AvailableLocalModel {
    file_name: String,
    absolute_path: String,
    normalized_name: String,
    display_name: String,
    disk_size_gb: f64,
    recommended_ram_gb: f64,
    priority: i32,
    role: String,
    source_label: String,
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("failed to resolve workspace root")
}

fn bundled_resources_dir() -> Option<PathBuf> {
    let executable_path = env::current_exe().ok()?;
    let resources_dir = executable_path.parent()?.parent()?.join("Resources");
    resources_dir.exists().then_some(resources_dir)
}

fn bundled_app_root() -> Option<PathBuf> {
    let app_root = bundled_resources_dir()?.join("app");
    app_root.exists().then_some(app_root)
}

fn source_app_root() -> PathBuf {
    bundled_app_root().unwrap_or_else(workspace_root)
}

fn schema_dir() -> PathBuf {
    bundled_resources_dir()
        .map(|resources| resources.join("schemas"))
        .filter(|path| path.exists())
        .unwrap_or_else(|| workspace_root().join("schemas"))
}

fn builder_root() -> PathBuf {
    workspace_root().join("builder")
}

fn builder_output_dir() -> PathBuf {
    writable_cache_dir().join("builder-output")
}

fn builder_library_root() -> PathBuf {
    writable_cache_root().join("library")
}

fn builder_library_models_dir() -> PathBuf {
    builder_library_root().join("models")
}

fn builder_library_manifests_dir() -> PathBuf {
    builder_library_root().join("manifests")
}

fn builder_library_download_manifests_dir() -> PathBuf {
    builder_library_manifests_dir().join("model-downloads")
}

fn source_cache_root() -> PathBuf {
    bundled_resources_dir()
        .map(|resources| resources.join("app-cache"))
        .filter(|path| path.exists())
        .unwrap_or_else(|| workspace_root().join("cache"))
}

fn writable_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .map(|path| path.join("org.usbrain.builder"))
        .unwrap_or_else(|| builder_root().join("cache"))
}

fn cache_dir() -> PathBuf {
    source_cache_root()
}

fn writable_cache_root() -> PathBuf {
    writable_cache_dir()
}

fn models_dir() -> PathBuf {
    source_app_root()
        .join("models")
        .exists()
        .then(|| source_app_root().join("models"))
        .unwrap_or_else(|| workspace_root().join("models"))
}

#[cfg(test)]
fn bundled_model_catalog_path() -> PathBuf {
    source_app_root().join("catalog").join("model-catalog.snapshot.json")
}

fn bundled_catalog_sources_path() -> PathBuf {
    source_app_root().join("catalog").join("catalog-sources.json")
}

fn cached_model_catalog_path() -> PathBuf {
    writable_cache_root().join("model-catalog").join("catalog.snapshot.json")
}

fn write_model_catalog_snapshot_to_path(
    snapshot: &ModelCatalogSnapshot,
    path: &Path,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Falha ao criar diretorio {}: {}", parent.display(), error))?;
    }

    let contents = serde_json::to_string_pretty(snapshot)
        .map_err(|error| format!("Falha ao serializar snapshot do catalogo: {}", error))?;
    fs::write(path, contents)
        .map_err(|error| format!("Falha ao escrever snapshot do catalogo em {}: {}", path.display(), error))?;
    Ok(())
}

fn load_catalog_sources_config() -> CatalogSourcesConfig {
    if !bundled_catalog_sources_path().exists() {
        return CatalogSourcesConfig::default();
    }

    let contents = match fs::read_to_string(bundled_catalog_sources_path()) {
        Ok(contents) => contents,
        Err(_) => return CatalogSourcesConfig::default(),
    };

    let parsed: CatalogSourcesConfig = match serde_json::from_str(&contents) {
        Ok(parsed) => parsed,
        Err(_) => return CatalogSourcesConfig::default(),
    };
    parsed
}

fn load_model_catalog_snapshot() -> Result<ModelCatalogSnapshot, String> {
    let candidate_path = cached_model_catalog_path();
    if !candidate_path.exists() {
        return Err(
            "Nenhum catalogo real foi sincronizado ainda. Atualize o catalogo vivo a partir de uma fonte remota verificavel antes de usar o Builder."
                .to_string(),
        );
    }

    let contents = fs::read_to_string(&candidate_path)
        .map_err(|error| format!("Falha ao ler snapshot real do catalogo {}: {}", candidate_path.display(), error))?;

    let snapshot: ModelCatalogSnapshot = serde_json::from_str(&contents)
        .map_err(|error| format!("Falha ao interpretar snapshot real do catalogo {}: {}", candidate_path.display(), error))?;

    if snapshot.source.trim().is_empty() || snapshot.source == "bundled-snapshot" {
        return Err(
            "O cache local do catalogo ainda nao aponta para uma fonte real verificavel. Force uma nova sincronizacao remota antes de continuar."
                .to_string(),
        );
    }

    if snapshot.models.is_empty() {
        return Err("O catalogo real sincronizado nao trouxe nenhum modelo utilizavel.".to_string());
    }

    Ok(snapshot)
}

fn candidate_remote_catalog_sources(request: &RefreshModelCatalogRequest) -> Vec<CatalogSourceCandidate> {
    let mut candidates = Vec::new();

    if let Some(source_url) = request.source_url.as_ref().map(|value| value.trim()).filter(|value| !value.is_empty()) {
        candidates.push(CatalogSourceCandidate {
            label: "request-source".to_string(),
            url: source_url.to_string(),
        });
    }

    if let Ok(env_url) = env::var("USBRAIN_MODEL_CATALOG_URL") {
        let trimmed = env_url.trim();
        if !trimmed.is_empty() && !candidates.iter().any(|value: &CatalogSourceCandidate| value.url == trimmed) {
            candidates.push(CatalogSourceCandidate {
                label: "env-usbrain-model-catalog-url".to_string(),
                url: trimmed.to_string(),
            });
        }
    }

    for source in load_catalog_sources_config().sources {
        let trimmed = source.snapshot_url.trim();
        if source.label.trim().is_empty() || trimmed.is_empty() {
            continue;
        }
        if !candidates.iter().any(|value| value.url == trimmed) {
            candidates.push(CatalogSourceCandidate {
                label: source.label.trim().to_string(),
                url: trimmed.to_string(),
            });
        }
    }

    candidates
}

fn merge_unique_strings(target: &mut Vec<String>, incoming: Vec<String>) {
    let mut seen = target.iter().cloned().collect::<HashSet<_>>();
    for value in incoming {
        if !value.trim().is_empty() && seen.insert(value.clone()) {
            target.push(value);
        }
    }
}

fn benchmark_summary_source_rank(summary: &CatalogBenchmarkSummaryRecord) -> u8 {
    match summary.source_label.as_str() {
        "OpenEvals official leaderboard" => 3,
        "Data from BenchLM.ai" => 2,
        _ => 1,
    }
}

fn benchmark_summary_confidence_rank(summary: &CatalogBenchmarkSummaryRecord) -> u8 {
    match summary.confidence_level.as_str() {
        "high" => 3,
        "medium" => 2,
        "guarded" => 1,
        _ => 0,
    }
}

fn benchmark_confidence_metadata(
    match_origin: BenchmarkMatchOrigin,
    variant_specific: bool,
) -> (&'static str, &'static str, &'static str) {
    match (match_origin, variant_specific) {
        (BenchmarkMatchOrigin::Exact, true) => ("variant-specific", "Benchmark proprio", "high"),
        (BenchmarkMatchOrigin::InheritedBase, _) => ("inherited-base", "Benchmark herdado", "guarded"),
        (BenchmarkMatchOrigin::Exact, false) => ("direct-comparable", "Benchmark comparavel", "medium"),
    }
}

fn choose_preferred_benchmark_summary(
    existing: Option<CatalogBenchmarkSummaryRecord>,
    incoming: Option<CatalogBenchmarkSummaryRecord>,
) -> Option<CatalogBenchmarkSummaryRecord> {
    match (existing, incoming) {
        (None, None) => None,
        (Some(summary), None) | (None, Some(summary)) => Some(summary),
        (Some(existing), Some(incoming)) => {
            let existing_confidence = benchmark_summary_confidence_rank(&existing);
            let incoming_confidence = benchmark_summary_confidence_rank(&incoming);
            if incoming_confidence > existing_confidence {
                return Some(incoming);
            }
            if incoming_confidence < existing_confidence {
                return Some(existing);
            }
            let existing_rank = benchmark_summary_source_rank(&existing);
            let incoming_rank = benchmark_summary_source_rank(&incoming);
            if incoming_rank > existing_rank {
                Some(incoming)
            } else if incoming_rank < existing_rank {
                Some(existing)
            } else if incoming.composite_score.unwrap_or_default() > existing.composite_score.unwrap_or_default() {
                Some(incoming)
            } else {
                Some(existing)
            }
        }
    }
}

fn merge_benchmark_metrics(
    target: &mut Vec<CatalogBenchmarkMetricRecord>,
    incoming: Vec<CatalogBenchmarkMetricRecord>,
) {
    let mut seen = target
        .iter()
        .map(|metric| {
            format!(
                "{}::{}",
                metric.benchmark_suite.as_deref().unwrap_or_default(),
                metric.label
            )
        })
        .collect::<HashSet<_>>();
    for metric in incoming {
        let key = format!(
            "{}::{}",
            metric.benchmark_suite.as_deref().unwrap_or_default(),
            metric.label
        );
        if seen.insert(key) {
            target.push(metric);
        }
    }
}

fn catalog_model_benchmark_confidence_rank(model: &CatalogModelRecord) -> u8 {
    match model
        .benchmark_summary
        .as_ref()
        .map(|summary| summary.confidence_level.as_str())
    {
        Some("high") => 3,
        Some("medium") => 2,
        Some("guarded") => 1,
        _ => 0,
    }
}

fn merge_notes(existing: Option<String>, incoming: Option<String>) -> Option<String> {
    match (existing, incoming) {
        (None, None) => None,
        (Some(note), None) | (None, Some(note)) => Some(note),
        (Some(existing), Some(incoming)) => {
            if existing == incoming {
                Some(existing)
            } else {
                Some(format!("{} {}", existing, incoming))
            }
        }
    }
}

fn merge_technical_profile(
    existing: Option<CatalogTechnicalProfileRecord>,
    incoming: Option<CatalogTechnicalProfileRecord>,
) -> Option<CatalogTechnicalProfileRecord> {
    match (existing, incoming) {
        (None, None) => None,
        (Some(profile), None) | (None, Some(profile)) => Some(profile),
        (Some(mut existing), Some(incoming)) => {
            existing.context_window_tokens = existing.context_window_tokens.or(incoming.context_window_tokens);
            existing.estimated_prompt_tokens_per_second = existing
                .estimated_prompt_tokens_per_second
                .or(incoming.estimated_prompt_tokens_per_second);
            existing.estimated_generation_tokens_per_second = existing
                .estimated_generation_tokens_per_second
                .or(incoming.estimated_generation_tokens_per_second);
            existing.estimated_peak_memory_gb =
                existing.estimated_peak_memory_gb.or(incoming.estimated_peak_memory_gb);
            existing.recommended_gpu_class =
                existing.recommended_gpu_class.or(incoming.recommended_gpu_class);
            existing.notes = merge_notes(existing.notes, incoming.notes);
            Some(existing)
        }
    }
}

fn merge_catalog_model_records(
    mut existing: CatalogModelRecord,
    incoming: CatalogModelRecord,
) -> CatalogModelRecord {
    existing.quality_score = existing.quality_score.max(incoming.quality_score);
    existing.priority = existing.priority.max(incoming.priority);
    existing.cpu_only_viable = existing.cpu_only_viable || incoming.cpu_only_viable;
    existing.gpu_optional = existing.gpu_optional || incoming.gpu_optional;
    existing.ram_min_gb = existing.ram_min_gb.min(incoming.ram_min_gb);
    existing.ram_recommended_gb = existing.ram_recommended_gb.min(incoming.ram_recommended_gb);
    existing.host_ram_tier_gb = existing.host_ram_tier_gb.min(incoming.host_ram_tier_gb);

    if existing.provider_or_origin.is_none() {
        existing.provider_or_origin = incoming.provider_or_origin;
    }
    if existing.variant_id.is_none() {
        existing.variant_id = incoming.variant_id;
    }
    if existing.quantization.is_none() {
        existing.quantization = incoming.quantization;
    }
    if existing.file_format.is_none() {
        existing.file_format = incoming.file_format;
    }
    if existing.editorial_recommendation.is_none() {
        existing.editorial_recommendation = incoming.editorial_recommendation;
    }
    if existing.local_file_name.is_none() {
        existing.local_file_name = incoming.local_file_name;
    }
    if existing.family.trim().is_empty() {
        existing.family = incoming.family;
    }
    if existing.parameter_scale == "desconhecido" && incoming.parameter_scale != "desconhecido" {
        existing.parameter_scale = incoming.parameter_scale;
    }
    if existing.license.trim().is_empty() || existing.license == "desconhecida" {
        existing.license = incoming.license;
    }
    if existing.homepage_or_source.trim().is_empty() {
        existing.homepage_or_source = incoming.homepage_or_source;
    }
    if existing.description.trim().is_empty() {
        existing.description = incoming.description;
    }
    if existing.rationale.trim().is_empty() {
        existing.rationale = incoming.rationale;
    }

    merge_unique_strings(&mut existing.languages, incoming.languages);
    merge_unique_strings(&mut existing.domains, incoming.domains);
    merge_unique_strings(&mut existing.capabilities, incoming.capabilities);
    merge_unique_strings(&mut existing.download_locations, incoming.download_locations);
    merge_unique_strings(&mut existing.checksums, incoming.checksums);
    merge_benchmark_metrics(&mut existing.benchmark_metrics, incoming.benchmark_metrics);
    existing.benchmark_summary =
        choose_preferred_benchmark_summary(existing.benchmark_summary, incoming.benchmark_summary);
    existing.technical_profile = merge_technical_profile(existing.technical_profile, incoming.technical_profile);

    existing
}

fn catalog_model_preference_cmp(left: &CatalogModelRecord, right: &CatalogModelRecord) -> std::cmp::Ordering {
    right
        .benchmark_summary
        .is_some()
        .cmp(&left.benchmark_summary.is_some())
        .then_with(|| {
            catalog_model_benchmark_confidence_rank(right)
                .cmp(&catalog_model_benchmark_confidence_rank(left))
        })
        .then_with(|| right.quality_score.cmp(&left.quality_score))
        .then_with(|| right.priority.cmp(&left.priority))
        .then_with(|| right.download_locations.len().cmp(&left.download_locations.len()))
        .then_with(|| {
            left.disk_size_gb
                .partial_cmp(&right.disk_size_gb)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn choose_preferred_catalog_model_record(
    existing: CatalogModelRecord,
    incoming: CatalogModelRecord,
) -> CatalogModelRecord {
    match catalog_model_preference_cmp(&existing, &incoming) {
        std::cmp::Ordering::Less => merge_catalog_model_records(incoming, existing),
        _ => merge_catalog_model_records(existing, incoming),
    }
}

fn sort_catalog_models(models: &mut [CatalogModelRecord]) {
    models.sort_by(catalog_model_preference_cmp);
}

fn aggregate_remote_model_catalog_snapshot(
    sources: &[CatalogSourceCandidate],
) -> Result<(ModelCatalogSnapshot, Vec<String>, String), String> {
    let mut warnings = Vec::new();
    let mut merged_models = HashMap::<String, CatalogModelRecord>::new();
    let mut successful_labels = Vec::new();
    let mut successful_urls = Vec::new();

    for source in sources {
        match fetch_remote_model_catalog_snapshot(&source.url) {
            Ok(snapshot) => {
                successful_labels.push(source.label.clone());
                successful_urls.push(source.url.clone());
                for model in snapshot.models {
                    let model_id = model.model_id.clone();
                    if let Some(existing) = merged_models.remove(&model_id) {
                        merged_models.insert(model_id, merge_catalog_model_records(existing, model));
                    } else {
                        merged_models.insert(model_id, model);
                    }
                }
            }
            Err(error) => {
                warnings.push(format!("{} ({}): {}", source.label, source.url, error))
            }
        }
    }

    if merged_models.is_empty() {
        return Err(format!(
            "Nao foi possivel atualizar o catalogo vivo a partir das fontes configuradas. {}",
            warnings.join(" | ")
        ));
    }

    let mut models = merged_models.into_values().collect::<Vec<_>>();
    sort_catalog_models(&mut models);

    let last_updated = now_utc_string();
    let source_label = format!(
        "fontes reais agregadas ({}){}",
        successful_labels.len(),
        if successful_labels.is_empty() {
            String::new()
        } else {
            format!(": {}", successful_labels.join(", "))
        }
    );
    Ok((
        ModelCatalogSnapshot {
            catalog_version: format!("multi-source-live-{}", last_updated),
            last_updated: last_updated.clone(),
            source: format!("multi-source:{}", successful_urls.join(" | ")),
            models,
        },
        warnings,
        source_label,
    ))
}

fn huggingface_api_token() -> Option<String> {
    [
        "USBRAIN_HUGGINGFACE_API_KEY",
        "HUGGINGFACE_API_KEY",
        "HUGGINGFACE_HUB_TOKEN",
        "HF_TOKEN",
    ]
    .iter()
    .find_map(|name| env::var(name).ok())
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty())
}

fn http_fetch_text(url: &str) -> Result<String, String> {
    let mut request = ureq::get(url)
        .set("Accept", "application/json")
        .set("User-Agent", "USBRAIN-Builder/0.1");
    if let Some(token) = huggingface_api_token() {
        request = request.set("Authorization", &format!("Bearer {}", token));
    }

    let response = request
        .call()
        .map_err(|error| format!("Falha ao baixar recurso remoto {}: {}", url, error))?;
    response
        .into_string()
        .map_err(|error| format!("Falha ao ler resposta remota {}: {}", url, error))
}

fn is_huggingface_models_api_url(url: &str) -> bool {
    url.starts_with("https://huggingface.co/api/models")
}

fn open_evals_rows_url() -> &'static str {
    "https://datasets-server.huggingface.co/rows?dataset=OpenEvals%2Fleaderboard-data&config=default&split=train&offset=0&length=200"
}

fn benchlm_leaderboard_url() -> &'static str {
    "https://benchlm.ai/api/data/leaderboard?limit=200"
}

fn benchlm_pricing_url() -> &'static str {
    "https://benchlm.ai/api/data/pricing?limit=200"
}

fn slug_from_repo_id(repo_id: &str) -> String {
    repo_id
        .split('/')
        .next_back()
        .unwrap_or(repo_id)
        .replace("-GGUF", "")
        .replace("-gguf", "")
}

fn title_case_slug(slug: &str) -> String {
    slug.split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            match characters.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), characters.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn infer_parameter_scale(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    for token in lower.split(|character: char| !character.is_ascii_alphanumeric() && character != '.') {
        if token.ends_with('b') && token.len() > 1 {
            let numeric = &token[..token.len() - 1];
            if numeric.chars().all(|character| character.is_ascii_digit() || character == '.') {
                return Some(format!("{}B", numeric.to_ascii_uppercase()));
            }
        }
    }
    None
}

fn normalize_model_lookup_key(text: &str) -> String {
    text.chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_lowercase())
        .collect()
}

fn trim_lookup_candidate(text: &str) -> String {
    text.trim()
        .trim_matches(|character: char| matches!(character, '-' | '_' | '/' | ' ' | '(' | ')'))
        .to_string()
}

fn strip_lookup_suffix_after_marker(text: &str) -> Option<String> {
    let lowercase = text.to_ascii_lowercase();
    for marker in [
        "-gguf",
        "_gguf",
        "-awq",
        "_awq",
        "-gptq",
        "_gptq",
        "-exl2",
        "_exl2",
        "-mlx",
        "_mlx",
        "-onnx",
        "_onnx",
        "-q",
        "_q",
        "-iq",
        "_iq",
        "-fp16",
        "_fp16",
        "-bf16",
        "_bf16",
    ] {
        if let Some(index) = lowercase.find(marker) {
            let stripped = trim_lookup_candidate(&text[..index]);
            if !stripped.is_empty() {
                return Some(stripped);
            }
        }
    }
    None
}

fn strip_trailing_release_suffix(text: &str) -> Option<String> {
    let trimmed = trim_lookup_candidate(text);
    let split_index = trimmed.rfind(|character| matches!(character, '-' | '_' | ' '))?;
    let token = trimmed.get(split_index + 1..)?.trim();
    let looks_like_numeric_release = token.len() == 4 && token.chars().all(|character| character.is_ascii_digit());
    let looks_like_version_suffix = token.len() > 1
        && token.starts_with('v')
        && token[1..]
            .chars()
            .all(|character| character.is_ascii_digit() || character == '.');
    if !looks_like_numeric_release && !looks_like_version_suffix {
        return None;
    }
    let stripped = trim_lookup_candidate(&trimmed[..split_index]);
    if stripped.is_empty() {
        None
    } else {
        Some(stripped)
    }
}

fn insert_lookup_separator_before_keyword(text: &str, keyword: &str) -> String {
    let mut result = String::new();
    let mut remaining = text;
    while let Some(index) = remaining.find(keyword) {
        let (before, after_keyword_start) = remaining.split_at(index);
        result.push_str(before);
        if result.chars().last().map(|character| character.is_ascii_alphanumeric()).unwrap_or(false) {
            result.push('-');
        }
        let (_, after_keyword) = after_keyword_start.split_at(keyword.len());
        result.push_str(keyword);
        remaining = after_keyword;
    }
    result.push_str(remaining);
    result
}

fn normalize_lookup_candidate_separators(text: &str) -> String {
    let mut normalized = text.replace('_', "-");
    for keyword in ["Instruct", "Chat", "Coder"] {
        normalized = insert_lookup_separator_before_keyword(&normalized, keyword);
    }
    normalized
}

fn alias_candidates_for_lookup_text(text: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut pending = vec![text.trim().to_string()];
    let mut seen_raw = HashSet::new();
    while let Some(candidate) = pending.pop() {
        let candidate = trim_lookup_candidate(&candidate);
        if candidate.is_empty() {
            continue;
        }
        if !seen_raw.insert(candidate.to_ascii_lowercase()) {
            continue;
        }
        candidates.push(candidate.clone());

        if let Some((_, suffix)) = candidate.split_once('/') {
            pending.push(suffix.to_string());
        }
        if let Some(stripped) = strip_lookup_suffix_after_marker(&candidate) {
            pending.push(stripped);
        }
        if let Some(stripped) = strip_trailing_release_suffix(&candidate) {
            pending.push(stripped);
        }

        let normalized = normalize_lookup_candidate_separators(&candidate);
        if normalized != candidate {
            pending.push(normalized);
        }
    }

    let mut unique = Vec::new();
    let mut seen = HashSet::new();
    for candidate in candidates {
        let normalized = normalize_model_lookup_key(&candidate);
        if !normalized.is_empty() && seen.insert(normalized.clone()) {
            unique.push(normalized);
        }
    }
    unique
}

fn specialization_profile_from_text(text: &str) -> Vec<String> {
    let lower = text.to_ascii_lowercase();
    let mut profile = Vec::new();
    if contains_any_marker(&lower, &["coder", "coding", "codegen"]) {
        profile.push("coding".to_string());
    }
    if contains_any_marker(&lower, &["reasoning", "thinking", "reasoner"]) {
        profile.push("reasoning".to_string());
    }
    if contains_any_marker(
        &lower,
        &["tool-use", "tool_use", "tool use", "function-calling", "function calling", "agentic"],
    ) || (lower.contains("tool") && lower.contains("agent"))
    {
        profile.push("tool-use".to_string());
    }
    if contains_any_marker(&lower, &["legal", "law"]) {
        profile.push("legal".to_string());
    }
    if contains_any_marker(&lower, &["medical", "medicine", "clinical"]) {
        profile.push("medical".to_string());
    }
    if contains_any_marker(&lower, &["multilingual", "bilingual"]) {
        profile.push("multilingual".to_string());
    }
    profile.sort();
    profile.dedup();
    profile
}

fn remove_specialization_markers_from_text(text: &str) -> String {
    let filtered_tokens = text
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '.')
        .filter(|token| !token.is_empty())
        .filter(|token| {
            let token = token.to_ascii_lowercase();
            !matches!(
                token.as_str(),
                "coder"
                    | "coding"
                    | "codegen"
                    | "reasoning"
                    | "thinking"
                    | "reasoner"
                    | "tool"
                    | "tools"
                    | "use"
                    | "function"
                    | "calling"
                    | "agent"
                    | "agentic"
                    | "legal"
                    | "law"
                    | "medical"
                    | "medicine"
                    | "clinical"
                    | "multilingual"
                    | "bilingual"
            )
        })
        .collect::<Vec<_>>();
    filtered_tokens.join(" ")
}

fn semantic_lookup_keys_for_text(text: &str) -> Vec<String> {
    let mut unique = Vec::new();
    let mut seen = HashSet::new();

    let mut raw_candidates = vec![text.trim().to_string()];
    let base_text = remove_specialization_markers_from_text(text);
    if !base_text.trim().is_empty() && normalize_model_lookup_key(&base_text) != normalize_model_lookup_key(text) {
        raw_candidates.push(base_text.clone());
    }

    let profile = specialization_profile_from_text(text);
    if !profile.is_empty() {
        let profile_suffix = profile.join(" ");
        raw_candidates.push(format!("{} {}", text.trim(), profile_suffix));
        if !base_text.trim().is_empty() {
            raw_candidates.push(format!("{} {}", base_text, profile_suffix));
        }
    }

    for candidate in raw_candidates {
        for alias in alias_candidates_for_lookup_text(&candidate) {
            if seen.insert(alias.clone()) {
                unique.push(alias);
            }
        }
    }
    unique
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BenchmarkMatchOrigin {
    Exact,
    InheritedBase,
}

struct BenchmarkLookupPlan {
    exact_keys: Vec<String>,
    inherited_base_keys: Vec<String>,
    requires_exact_variant_benchmark: bool,
}

fn collect_semantic_lookup_keys(candidates: Vec<String>) -> Vec<String> {
    let mut unique = Vec::new();
    let mut seen = HashSet::new();
    for candidate in candidates {
        for alias in semantic_lookup_keys_for_text(&candidate) {
            if seen.insert(alias.clone()) {
                unique.push(alias);
            }
        }
    }
    unique
}

fn collect_exact_lookup_keys(candidates: Vec<String>) -> Vec<String> {
    let mut unique = Vec::new();
    let mut seen = HashSet::new();
    for candidate in candidates {
        for alias in alias_candidates_for_lookup_text(&candidate) {
            if seen.insert(alias.clone()) {
                unique.push(alias);
            }
        }
    }
    unique
}

fn benchmark_lookup_plan_for_huggingface_model(
    record: &HuggingFaceModelInfoRecord,
    slug: &str,
) -> BenchmarkLookupPlan {
    let languages = infer_languages(record);
    let domains = infer_domains(&record.tags, &record.id);
    let capabilities = infer_capabilities(&record.tags, &languages);
    let specialization_profile =
        specialization_profile_from_model_traits(&record.id, &record.tags, &languages, &domains, &capabilities);
    let requires_exact_variant_benchmark =
        !huggingface_repo_penalty_flags(&record.id, &record.tags).is_empty() || !specialization_profile.is_empty();
    let specialization_suffix = (!specialization_profile.is_empty()).then(|| specialization_profile.join(" "));
    let repo_candidate = record.id.replace("-GGUF", "").replace("-gguf", "");
    let slug_candidate = slug.replace('_', "-");

    let mut exact_candidates = vec![repo_candidate.clone(), slug_candidate.clone()];
    if let Some(suffix) = specialization_suffix.as_ref() {
        if let Some(base_model) = preferred_huggingface_base_model_text(record) {
            exact_candidates.push(format!("{} {}", base_model, suffix));
        }
        if let Some(base_model_tag) = first_base_model_tag(&record.tags) {
            exact_candidates.push(format!("{} {}", base_model_tag, suffix));
        }
        exact_candidates.push(format!("{} {}", repo_candidate, suffix));
        exact_candidates.push(format!("{} {}", slug_candidate, suffix));
    } else if !requires_exact_variant_benchmark {
        if let Some(base_model) = preferred_huggingface_base_model_text(record) {
            exact_candidates.push(base_model);
        }
        if let Some(base_model_tag) = first_base_model_tag(&record.tags) {
            exact_candidates.push(base_model_tag);
        }
    }

    let mut inherited_base_candidates = Vec::new();
    if let Some(base_model) = preferred_huggingface_base_model_text(record) {
        inherited_base_candidates.push(base_model);
    }
    if let Some(base_model_tag) = first_base_model_tag(&record.tags) {
        inherited_base_candidates.push(base_model_tag);
    }
    inherited_base_candidates.push(repo_candidate);
    inherited_base_candidates.push(slug_candidate);

    BenchmarkLookupPlan {
        exact_keys: collect_exact_lookup_keys(exact_candidates),
        inherited_base_keys: collect_semantic_lookup_keys(inherited_base_candidates),
        requires_exact_variant_benchmark,
    }
}

fn find_benchmark_match<T: Clone>(
    plan: &BenchmarkLookupPlan,
    benchmark_index: &HashMap<String, T>,
) -> Option<(T, BenchmarkMatchOrigin)> {
    if let Some(row) = plan
        .exact_keys
        .iter()
        .find_map(|key| benchmark_index.get(key))
        .cloned()
    {
        return Some((row, BenchmarkMatchOrigin::Exact));
    }
    if plan.requires_exact_variant_benchmark {
        return None;
    }
    plan.inherited_base_keys
        .iter()
        .find_map(|key| benchmark_index.get(key))
        .cloned()
        .map(|row| (row, BenchmarkMatchOrigin::InheritedBase))
}

fn first_base_model_tag(tags: &[String]) -> Option<String> {
    tags.iter()
        .find_map(|tag| tag.strip_prefix("base_model:"))
        .filter(|value| !value.starts_with("quantized:"))
        .map(|value| value.to_string())
}

fn is_open_weight_source(source_type: Option<&str>) -> bool {
    source_type
        .map(|value| value.eq_ignore_ascii_case("Open Weight"))
        .unwrap_or(false)
}

fn parse_context_window_tokens(value: &str) -> Option<u32> {
    let trimmed = value.trim().trim_end_matches('+').to_ascii_uppercase();
    if let Some(number) = trimmed.strip_suffix('K') {
        return number.trim().parse::<f64>().ok().map(|parsed| (parsed * 1_000.0) as u32);
    }
    if let Some(number) = trimmed.strip_suffix('M') {
        return number.trim().parse::<f64>().ok().map(|parsed| (parsed * 1_000_000.0) as u32);
    }
    trimmed.parse::<u32>().ok()
}

fn fetch_open_evals_benchmark_index() -> Result<HashMap<String, OpenEvalsBenchmarkRow>, String> {
    let contents = http_fetch_text(open_evals_rows_url())?;
    let response: DatasetServerRowsResponse<OpenEvalsBenchmarkRow> = serde_json::from_str(&contents)
        .map_err(|error| format!("Falha ao interpretar leaderboard OpenEvals: {}", error))?;

    let mut index = HashMap::new();
    for entry in response.rows {
        let row = entry.row;
        if row.aggregate_score.is_none() && row.mmlu_pro_score.is_none() && row.gpqa_score.is_none() {
            continue;
        }
        for alias in semantic_lookup_keys_for_text(&row.model_name) {
            index.entry(alias).or_insert_with(|| row.clone());
        }
        for alias in semantic_lookup_keys_for_text(&row.model_id) {
            index.entry(alias).or_insert_with(|| row.clone());
        }
    }

    Ok(index)
}

fn fetch_benchlm_pricing_index() -> Result<(HashMap<String, BenchLmPricingRow>, Option<String>), String> {
    let contents = http_fetch_text(benchlm_pricing_url())?;
    let response: BenchLmPricingResponse = serde_json::from_str(&contents)
        .map_err(|error| format!("Falha ao interpretar pricing do BenchLM: {}", error))?;

    let mut index = HashMap::new();
    for row in response.models {
        if !is_open_weight_source(row.source_type.as_deref()) {
            continue;
        }
        for alias in semantic_lookup_keys_for_text(&row.model) {
            index.entry(alias).or_insert_with(|| row.clone());
        }
    }

    Ok((index, response.last_updated))
}

fn fetch_benchlm_benchmark_index() -> Result<(HashMap<String, BenchLmLeaderboardRow>, Option<String>), String> {
    let contents = http_fetch_text(benchlm_leaderboard_url())?;
    let response: BenchLmLeaderboardResponse = serde_json::from_str(&contents)
        .map_err(|error| format!("Falha ao interpretar leaderboard do BenchLM: {}", error))?;

    let mut index = HashMap::new();
    for row in response.models {
        if !is_open_weight_source(row.source_type.as_deref()) {
            continue;
        }
        if row.overall_score.is_none() && row.category_scores.is_none() {
            continue;
        }
        for alias in semantic_lookup_keys_for_text(&row.model) {
            index.entry(alias).or_insert_with(|| BenchLmLeaderboardRow {
                model: row.model.clone(),
                creator: row.creator.clone(),
                source_type: row.source_type.clone(),
                overall_score: row.overall_score,
                category_scores: row.category_scores.clone(),
            });
        }
    }

    Ok((index, response.last_updated))
}

#[cfg_attr(not(test), allow(dead_code))]
fn benchmark_lookup_keys_for_huggingface_model(record: &HuggingFaceModelInfoRecord, slug: &str) -> Vec<String> {
    let plan = benchmark_lookup_plan_for_huggingface_model(record, slug);
    let mut unique = plan.exact_keys;
    for key in plan.inherited_base_keys {
        if !unique.iter().any(|existing| existing == &key) {
            unique.push(key);
        }
    }
    unique
}

fn build_open_evals_benchmark_metrics(row: &OpenEvalsBenchmarkRow) -> Vec<CatalogBenchmarkMetricRecord> {
    let mut metrics = Vec::new();
    for (label, value) in [
        ("Aggregate", row.aggregate_score),
        ("MMLU-Pro", row.mmlu_pro_score),
        ("GPQA", row.gpqa_score),
        ("AIME 2026", row.aime2026_score),
        ("EvasionBench", row.evasion_bench_score),
        ("HLE", row.hle_score),
        ("SWE-Pro", row.swe_pro_score),
        ("SWE Verified", row.swe_verified_score),
        ("TerminalBench", row.terminal_bench_score),
        ("GSM8K", row.gsm8k_score),
        ("HMMT 2026", row.hmmt2026_score),
        ("olmOCR", row.olm_ocr_score),
    ] {
        if let Some(value) = value {
            metrics.push(CatalogBenchmarkMetricRecord {
                label: label.to_string(),
                value,
                unit: Some("pts".to_string()),
                benchmark_suite: Some("OpenEvals".to_string()),
                higher_is_better: Some(true),
            });
        }
    }
    metrics
}

fn build_open_evals_benchmark_summary(
    row: &OpenEvalsBenchmarkRow,
    match_origin: BenchmarkMatchOrigin,
    variant_specific: bool,
) -> Option<CatalogBenchmarkSummaryRecord> {
    let (match_origin_label, confidence_label, confidence_level) =
        benchmark_confidence_metadata(match_origin, variant_specific);
    let composite_score = row.aggregate_score;
    let highlights = [
        row.mmlu_pro_score.map(|value| format!("MMLU-Pro {:.1}", value)),
        row.gpqa_score.map(|value| format!("GPQA {:.1}", value)),
        row.swe_verified_score.map(|value| format!("SWE Verified {:.1}", value)),
        row.terminal_bench_score.map(|value| format!("TerminalBench {:.1}", value)),
    ]
    .into_iter()
    .flatten()
    .take(3)
    .collect::<Vec<_>>();

    if composite_score.is_none() && highlights.is_empty() {
        return None;
    }

    let coverage_label = match (row.coverage_count, row.coverage_percent) {
        (Some(count), Some(percent)) => Some(format!("cobertura {} benchmarks ({:.0}%)", count, percent)),
        (Some(count), None) => Some(format!("cobertura {} benchmarks", count)),
        _ => None,
    };
    let summary = if highlights.is_empty() {
        match (match_origin, variant_specific) {
            (BenchmarkMatchOrigin::Exact, true) => {
                "Pontuacao real agregada carregada do OpenEvals para a variante especializada correspondente.".to_string()
            }
            (BenchmarkMatchOrigin::InheritedBase, _) => {
                "Pontuacao real agregada carregada do OpenEvals para o modelo base correspondente; esta variante usa heranca explicita desse benchmark publico.".to_string()
            }
            _ => "Pontuacao real agregada carregada do OpenEvals para o modelo base correspondente.".to_string(),
        }
    } else {
        match (match_origin, variant_specific) {
            (BenchmarkMatchOrigin::Exact, true) => format!(
                "Benchmarks reais do OpenEvals para a variante especializada correspondente: {}.",
                highlights.join(" · ")
            ),
            (BenchmarkMatchOrigin::InheritedBase, _) => format!(
                "Benchmarks reais do OpenEvals para o modelo base correspondente, herdados explicitamente por esta variante: {}.",
                highlights.join(" · ")
            ),
            _ => format!(
                "Benchmarks reais do OpenEvals para o modelo base correspondente: {}.",
                highlights.join(" · ")
            ),
        }
    };

    Some(CatalogBenchmarkSummaryRecord {
        source_label: "OpenEvals official leaderboard".to_string(),
        summary,
        composite_score,
        rank_label: coverage_label,
        last_verified_at: None,
        match_origin: match_origin_label.to_string(),
        confidence_label: confidence_label.to_string(),
        confidence_level: confidence_level.to_string(),
    })
}

fn build_benchlm_benchmark_metrics(row: &BenchLmLeaderboardRow) -> Vec<CatalogBenchmarkMetricRecord> {
    let mut metrics = Vec::new();
    if let Some(overall_score) = row.overall_score {
        metrics.push(CatalogBenchmarkMetricRecord {
            label: "Overall".to_string(),
            value: overall_score,
            unit: Some("pts".to_string()),
            benchmark_suite: Some("BenchLM".to_string()),
            higher_is_better: Some(true),
        });
    }
    if let Some(category_scores) = row.category_scores.as_ref() {
        for (label, value) in [
            ("Agentic", category_scores.agentic),
            ("Coding", category_scores.coding),
            ("Reasoning", category_scores.reasoning),
            ("Knowledge", category_scores.knowledge),
            ("Math", category_scores.math),
            ("Instruction Following", category_scores.instruction_following),
            ("Multilingual", category_scores.multilingual),
            ("Multimodal Grounded", category_scores.multimodal_grounded),
        ] {
            if let Some(value) = value {
                metrics.push(CatalogBenchmarkMetricRecord {
                    label: label.to_string(),
                    value,
                    unit: Some("pts".to_string()),
                    benchmark_suite: Some("BenchLM".to_string()),
                    higher_is_better: Some(true),
                });
            }
        }
    }
    metrics
}

fn build_benchlm_benchmark_summary(
    row: &BenchLmLeaderboardRow,
    last_updated: Option<&String>,
    match_origin: BenchmarkMatchOrigin,
    variant_specific: bool,
) -> Option<CatalogBenchmarkSummaryRecord> {
    let (match_origin_label, confidence_label, confidence_level) =
        benchmark_confidence_metadata(match_origin, variant_specific);
    let mut highlights = Vec::new();
    if let Some(category_scores) = row.category_scores.as_ref() {
        for (label, value) in [
            ("coding", category_scores.coding),
            ("reasoning", category_scores.reasoning),
            ("knowledge", category_scores.knowledge),
            ("math", category_scores.math),
            ("multilingual", category_scores.multilingual),
        ] {
            if let Some(value) = value {
                highlights.push(format!("{} {:.1}", label, value));
            }
        }
    }

    if row.overall_score.is_none() && highlights.is_empty() {
        return None;
    }

    let summary = if highlights.is_empty() {
        match (match_origin, variant_specific) {
            (BenchmarkMatchOrigin::Exact, true) => format!(
                "Score real agregado do BenchLM para a variante especializada/open-weight correspondente `{}`.",
                row.model
            ),
            (BenchmarkMatchOrigin::InheritedBase, _) => format!(
                "Score real agregado do BenchLM para o modelo base open-weight correspondente `{}`; esta variante herda explicitamente esse benchmark.",
                row.model
            ),
            _ => format!(
                "Score real agregado do BenchLM para o modelo open-weight correspondente `{}`.",
                row.model
            ),
        }
    } else {
        let details = highlights.into_iter().take(3).collect::<Vec<_>>().join(" · ");
        match (match_origin, variant_specific) {
            (BenchmarkMatchOrigin::Exact, true) => format!(
                "Dados reais do BenchLM para a variante especializada/open-weight correspondente `{}`: {}.",
                row.model, details
            ),
            (BenchmarkMatchOrigin::InheritedBase, _) => format!(
                "Dados reais do BenchLM para o modelo base open-weight correspondente `{}`; esta variante herda explicitamente esse benchmark: {}.",
                row.model, details
            ),
            _ => format!(
                "Dados reais do BenchLM para o modelo open-weight correspondente `{}`: {}.",
                row.model, details
            ),
        }
    };

    Some(CatalogBenchmarkSummaryRecord {
        source_label: "Data from BenchLM.ai".to_string(),
        summary,
        composite_score: row.overall_score,
        rank_label: Some("leaderboard open-weight".to_string()),
        last_verified_at: last_updated.cloned(),
        match_origin: match_origin_label.to_string(),
        confidence_label: confidence_label.to_string(),
        confidence_level: confidence_level.to_string(),
    })
}

fn enrich_model_with_open_evals_benchmarks(
    model: &mut CatalogModelRecord,
    record: &HuggingFaceModelInfoRecord,
    slug: &str,
    benchmark_index: &HashMap<String, OpenEvalsBenchmarkRow>,
) {
    let lookup_plan = benchmark_lookup_plan_for_huggingface_model(record, slug);
    let variant_specific = lookup_plan.requires_exact_variant_benchmark;
    let Some((benchmark_row, match_origin)) = find_benchmark_match(&lookup_plan, benchmark_index) else {
        if variant_specific {
            let note = "O USBRAIN encontrou uma variante/fork especializado, mas nao herdou benchmark do modelo-base porque ainda nao ha benchmark publico proprio suficientemente aderente para esta especializacao.";
            model.editorial_recommendation = Some(match model.editorial_recommendation.take() {
                Some(existing) => format!("{} {}", existing, note),
                None => note.to_string(),
            });
        }
        return;
    };

    model.benchmark_summary =
        build_open_evals_benchmark_summary(&benchmark_row, match_origin, variant_specific);
    model.benchmark_metrics = build_open_evals_benchmark_metrics(&benchmark_row);
    if let Some(aggregate_score) = benchmark_row.aggregate_score {
        let benchmark_quality = aggregate_score.clamp(0.0, 95.0).round() as u8;
        model.quality_score = model.quality_score.max(benchmark_quality);
        model.priority = model.priority.max(benchmark_quality);
    }

    let benchmark_note = match (match_origin, variant_specific) {
        (BenchmarkMatchOrigin::Exact, true) => format!(
            "Benchmark real carregado do OpenEvals para a variante/fork correspondente `{}`. Parametros {:.1}B · arquitetura {} · licenca {}.",
            benchmark_row.model_name,
            benchmark_row.parameters_billions.unwrap_or_default(),
            benchmark_row.architecture.clone().unwrap_or_else(|| "nao informada".to_string()),
            benchmark_row.license.clone().unwrap_or_else(|| "nao informada".to_string())
        ),
        (BenchmarkMatchOrigin::InheritedBase, _) => format!(
            "Benchmark real carregado do OpenEvals para o modelo base correspondente `{}`; esta variante herda explicitamente esse benchmark publico. Parametros {:.1}B · arquitetura {} · licenca {}.",
            benchmark_row.model_name,
            benchmark_row.parameters_billions.unwrap_or_default(),
            benchmark_row.architecture.clone().unwrap_or_else(|| "nao informada".to_string()),
            benchmark_row.license.clone().unwrap_or_else(|| "nao informada".to_string())
        ),
        _ => format!(
            "Benchmark real carregado do OpenEvals para `{}`; a comparacao e por modelo base, enquanto a variante GGUF continua especifica do Hugging Face. Parametros {:.1}B · arquitetura {} · licenca {}.",
            benchmark_row.model_name,
            benchmark_row.parameters_billions.unwrap_or_default(),
            benchmark_row.architecture.clone().unwrap_or_else(|| "nao informada".to_string()),
            benchmark_row.license.clone().unwrap_or_else(|| "nao informada".to_string())
        ),
    };
    match model.technical_profile.as_mut() {
        Some(profile) => {
            if profile.context_window_tokens.is_none() {
                profile.context_window_tokens = benchmark_row.context_window.filter(|value| *value > 0);
            }
            profile.notes = Some(match profile.notes.as_ref() {
                Some(existing) => format!("{} {}", existing, benchmark_note),
                None => benchmark_note,
            });
        }
        None => {
            model.technical_profile = Some(CatalogTechnicalProfileRecord {
                context_window_tokens: benchmark_row.context_window.filter(|value| *value > 0),
                estimated_prompt_tokens_per_second: None,
                estimated_generation_tokens_per_second: None,
                estimated_peak_memory_gb: Some(model.ram_recommended_gb),
                recommended_gpu_class: None,
                notes: Some(benchmark_note),
            });
        }
    }

    if model.provider_or_origin.is_none() {
        model.provider_or_origin = benchmark_row.provider.clone();
    }
    if model.family.trim().is_empty() {
        if let Some(architecture) = benchmark_row.architecture.clone() {
            model.family = architecture;
        }
    }
    if model.parameter_scale == "desconhecido" {
        if let Some(parameters_billions) = benchmark_row.parameters_billions {
            model.parameter_scale = format!("{:.1}B", parameters_billions);
        }
    }
    if model.editorial_recommendation.is_none() {
        model.editorial_recommendation = Some(match (match_origin, variant_specific) {
            (BenchmarkMatchOrigin::Exact, true) => {
                "O USBRAIN encontrou benchmark publico aderente a esta variante especializada; compare esta proposta com o modelo-base e com outras especializacoes antes de decidir a quantizacao GGUF.".to_string()
            }
            _ => "O USBRAIN encontrou benchmark publico do modelo base correspondente; compare esta familia com outras antes de decidir a quantizacao GGUF."
                .to_string(),
        });
    }
}

fn enrich_model_with_benchlm_benchmarks(
    model: &mut CatalogModelRecord,
    record: &HuggingFaceModelInfoRecord,
    slug: &str,
    benchmark_index: &HashMap<String, BenchLmLeaderboardRow>,
    pricing_index: &HashMap<String, BenchLmPricingRow>,
    last_updated: Option<&String>,
) {
    let lookup_plan = benchmark_lookup_plan_for_huggingface_model(record, slug);
    let variant_specific = lookup_plan.requires_exact_variant_benchmark;
    let benchmark_row = find_benchmark_match(&lookup_plan, benchmark_index);
    let pricing_row = find_benchmark_match(&lookup_plan, pricing_index).map(|(row, _)| row);

    if let Some((benchmark_row, match_origin)) = benchmark_row {
        let benchlm_metrics = build_benchlm_benchmark_metrics(&benchmark_row);
        for metric in benchlm_metrics {
            let already_present = model.benchmark_metrics.iter().any(|existing| {
                existing.label == metric.label && existing.benchmark_suite == metric.benchmark_suite
            });
            if !already_present {
                model.benchmark_metrics.push(metric);
            }
        }

        if model.benchmark_summary.is_none() {
            model.benchmark_summary =
                build_benchlm_benchmark_summary(&benchmark_row, last_updated, match_origin, variant_specific);
        } else if let Some(profile) = model.technical_profile.as_mut() {
            let note = match (match_origin, variant_specific) {
                (BenchmarkMatchOrigin::Exact, true) => format!(
                    "Complemento de benchmark do BenchLM para a variante/fork correspondente `{}` com atribuicao visivel exigida pela fonte.",
                    benchmark_row.model
                ),
                (BenchmarkMatchOrigin::InheritedBase, _) => format!(
                    "Complemento de benchmark do BenchLM para o modelo base correspondente `{}`; esta variante herda explicitamente esse benchmark com atribuicao visivel exigida pela fonte.",
                    benchmark_row.model
                ),
                _ => format!(
                    "Complemento de benchmark do BenchLM para `{}` com atribuicao visivel exigida pela fonte.",
                    benchmark_row.model
                ),
            };
            profile.notes = Some(match profile.notes.as_ref() {
                Some(existing) => format!("{} {}", existing, note),
                None => note,
            });
        }

        if let Some(overall_score) = benchmark_row.overall_score {
            let benchmark_quality = overall_score.clamp(0.0, 95.0).round() as u8;
            model.quality_score = model.quality_score.max(benchmark_quality);
            model.priority = model.priority.max(benchmark_quality);
        }

        if model.provider_or_origin.is_none() {
            model.provider_or_origin = benchmark_row.creator.clone();
        }
    } else if variant_specific {
        let note = "A variante/fork especializado nao recebeu fallback automatico de benchmark do modelo-base no BenchLM; o USBRAIN exige benchmark publico proprio para evitar comparacao otimista demais.";
        model.editorial_recommendation = Some(match model.editorial_recommendation.take() {
            Some(existing) => format!("{} {}", existing, note),
            None => note.to_string(),
        });
    }

    if let Some(pricing_row) = pricing_row {
        if let Some(profile) = model.technical_profile.as_mut() {
            if profile.context_window_tokens.is_none() {
                profile.context_window_tokens = pricing_row
                    .context_window
                    .as_deref()
                    .and_then(parse_context_window_tokens);
            }
        } else {
            model.technical_profile = Some(CatalogTechnicalProfileRecord {
                context_window_tokens: pricing_row
                    .context_window
                    .as_deref()
                    .and_then(parse_context_window_tokens),
                estimated_prompt_tokens_per_second: None,
                estimated_generation_tokens_per_second: None,
                estimated_peak_memory_gb: Some(model.ram_recommended_gb),
                recommended_gpu_class: None,
                notes: Some(
                    "Dados reais de contexto carregados do BenchLM para o modelo open-weight correspondente."
                        .to_string(),
                ),
            });
        }
        if model.provider_or_origin.is_none() {
            model.provider_or_origin = pricing_row.creator.clone();
        }
    }
}

fn infer_languages(record: &HuggingFaceModelInfoRecord) -> Vec<String> {
    let mut languages = record
        .card_data
        .as_ref()
        .map(|card| card.language.clone())
        .unwrap_or_default();
    if languages.is_empty() && record.tags.iter().any(|tag| tag.eq_ignore_ascii_case("multilingual")) {
        languages = vec!["pt-BR".to_string(), "en".to_string(), "es".to_string()];
    }
    if languages.is_empty() {
        languages.push("en".to_string());
    }
    languages
}

fn infer_domains(tags: &[String], repo_id: &str) -> Vec<String> {
    let joined = format!("{} {}", repo_id.to_ascii_lowercase(), tags.join(" ").to_ascii_lowercase());
    let mut domains = Vec::new();
    if joined.contains("code") || joined.contains("coder") {
        domains.push("engineering".to_string());
        domains.push("coding".to_string());
    }
    if joined.contains("law") || joined.contains("legal") {
        domains.push("law".to_string());
    }
    if joined.contains("med") || joined.contains("bio") || joined.contains("health") {
        domains.push("medicine".to_string());
    }
    if domains.is_empty() {
        domains.push("general".to_string());
    }
    domains
}

fn infer_capabilities(tags: &[String], languages: &[String]) -> Vec<String> {
    let joined = tags.join(" ").to_ascii_lowercase();
    let mut capabilities = vec!["offline".to_string()];
    if joined.contains("chat") || joined.contains("conversational") {
        capabilities.push("chat".to_string());
    }
    if joined.contains("tool") {
        capabilities.push("tool-use".to_string());
    }
    if joined.contains("reason") {
        capabilities.push("reasoning".to_string());
    }
    if languages.len() > 1 {
        capabilities.push("multilingual".to_string());
    }
    capabilities.sort();
    capabilities.dedup();
    capabilities
}

fn infer_quality_score(downloads: u64, likes: u64) -> u8 {
    let download_component = ((downloads.min(500_000) as f64 / 500_000.0) * 35.0).round() as u8;
    let likes_component = ((likes.min(2_000) as f64 / 2_000.0) * 25.0).round() as u8;
    40u8.saturating_add(download_component).saturating_add(likes_component).min(95)
}

fn tag_matches_exact(tags: &[String], expected: &str) -> bool {
    tags.iter().any(|tag| tag.eq_ignore_ascii_case(expected))
}

fn tag_matches_prefix(tags: &[String], prefix: &str) -> bool {
    let prefix_lower = prefix.to_ascii_lowercase();
    tags.iter()
        .any(|tag| tag.to_ascii_lowercase().starts_with(&prefix_lower))
}

fn contains_any_marker(text: &str, markers: &[&str]) -> bool {
    let lower = text.to_ascii_lowercase();
    markers.iter().any(|marker| lower.contains(marker))
}

fn has_multimodal_companion_assets(record: &HuggingFaceModelInfoRecord) -> bool {
    record.siblings.iter().any(|sibling| {
        let lower = sibling.rfilename.to_ascii_lowercase();
        lower.contains("mmproj") || lower.contains("projector")
    })
}

fn is_probably_multimodal_repo(record: &HuggingFaceModelInfoRecord) -> bool {
    let pipeline = record.pipeline_tag.as_deref().unwrap_or_default();
    if matches!(
        pipeline,
        "image-text-to-text" | "image-to-text" | "video-text-to-text" | "image-text-to-image"
    ) {
        return true;
    }
    tag_matches_exact(&record.tags, "multimodal")
        || tag_matches_exact(&record.tags, "vision")
        || tag_matches_exact(&record.tags, "image-text-to-text")
        || tag_matches_exact(&record.tags, "image-to-text")
        || tag_matches_exact(&record.tags, "video-text-to-text")
        || tag_matches_exact(&record.tags, "image-text-to-image")
        || has_multimodal_companion_assets(record)
}

fn huggingface_repo_penalty_flags(repo_id: &str, tags: &[String]) -> Vec<&'static str> {
    let joined = format!("{} {}", repo_id.to_ascii_lowercase(), tags.join(" ").to_ascii_lowercase());
    let mut flags = Vec::new();
    if tag_matches_prefix(tags, "base_model:adapter:") || contains_any_marker(&joined, &[" adapter "]) {
        flags.push("adapter");
    }
    if contains_any_marker(&joined, &["lora", "qlora"]) {
        flags.push("lora");
    }
    if contains_any_marker(&joined, &["merged", "mergekit"]) {
        flags.push("merged");
    }
    if contains_any_marker(&joined, &["distilled", "distill"]) {
        flags.push("distilled");
    }
    if contains_any_marker(&joined, &["uncensored", "abliterated"]) {
        flags.push("uncensored");
    }
    flags
}

fn huggingface_repo_quality_penalty(repo_id: &str, tags: &[String]) -> u8 {
    huggingface_repo_penalty_flags(repo_id, tags)
        .into_iter()
        .map(|flag| match flag {
            "adapter" => 18,
            "lora" => 18,
            "merged" => 10,
            "distilled" => 8,
            "uncensored" => 14,
            _ => 0,
        })
        .sum::<u8>()
        .min(40)
}

fn build_huggingface_repo_editorial_note(repo_id: &str, tags: &[String]) -> Option<String> {
    let flags = huggingface_repo_penalty_flags(repo_id, tags);
    if flags.is_empty() {
        return None;
    }
    Some(format!(
        "Repositorio derivado detectado na descoberta automatica ({}); o USBRAIN mantem este candidato visivel, mas reduz sua prioridade para privilegiar pesos mais estaveis e comparaveis.",
        flags.join(", ")
    ))
}

fn preferred_huggingface_base_model_text(record: &HuggingFaceModelInfoRecord) -> Option<String> {
    record
        .card_data
        .as_ref()
        .and_then(|card| card.base_model.as_ref())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .or_else(|| first_base_model_tag(&record.tags))
}

fn simplify_huggingface_family_text(text: &str) -> String {
    let mut candidate = text.split('/').next_back().unwrap_or(text).to_string();
    if let Some(stripped) = strip_lookup_suffix_after_marker(&candidate) {
        candidate = stripped;
    }
    while let Some(stripped) = strip_trailing_release_suffix(&candidate) {
        candidate = stripped;
    }
    trim_lookup_candidate(&candidate)
}

fn slugify_catalog_identifier(text: &str) -> String {
    let mut output = String::new();
    let mut last_was_dash = false;
    for character in text.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            output.push('-');
            last_was_dash = true;
        }
    }
    output.trim_matches('-').to_string()
}

fn generic_huggingface_family_source_text(record: &HuggingFaceModelInfoRecord, slug: &str) -> String {
    if huggingface_repo_penalty_flags(&record.id, &record.tags).is_empty() {
        if let Some(base_model) = preferred_huggingface_base_model_text(record) {
            return simplify_huggingface_family_text(&base_model);
        }
    }
    simplify_huggingface_family_text(slug)
}

fn specialization_profile_from_model_traits(
    repo_id: &str,
    tags: &[String],
    languages: &[String],
    domains: &[String],
    capabilities: &[String],
) -> Vec<String> {
    let joined = format!("{} {}", repo_id.to_ascii_lowercase(), tags.join(" ").to_ascii_lowercase());
    let mut profile = Vec::new();
    if domains.iter().any(|domain| domain == "coding")
        || contains_any_marker(&joined, &["coder", "codegen", "coding"])
    {
        profile.push("coding".to_string());
    }
    if capabilities.iter().any(|capability| capability == "reasoning")
        || contains_any_marker(&joined, &["reasoning", "thinking"])
    {
        profile.push("reasoning".to_string());
    }
    if capabilities.iter().any(|capability| capability == "tool-use")
        || contains_any_marker(&joined, &["tool-use", "tool_use", "function-calling", "function calling", "agent"])
    {
        profile.push("tool-use".to_string());
    }
    if domains.iter().any(|domain| domain == "law") {
        profile.push("legal".to_string());
    }
    if domains.iter().any(|domain| domain == "medicine") {
        profile.push("medical".to_string());
    }
    if tag_matches_exact(tags, "multilingual") || capabilities.iter().any(|capability| capability == "multilingual" && languages.len() > 2) {
        profile.push("multilingual".to_string());
    }
    profile.sort();
    profile.dedup();
    profile
}

fn specialization_profile_key(profile: &[String]) -> Option<String> {
    let key = slugify_catalog_identifier(&profile.join("-"));
    if key.is_empty() {
        None
    } else {
        Some(key)
    }
}

fn specialization_profile_display_suffix(profile: &[String]) -> Option<String> {
    let labels = profile
        .iter()
        .map(|token| match token.as_str() {
            "coding" => "Coder".to_string(),
            "reasoning" => "Reasoning".to_string(),
            "tool-use" => "Tool Use".to_string(),
            "legal" => "Legal".to_string(),
            "medical" => "Medical".to_string(),
            "multilingual" => "Multilingual".to_string(),
            _ => title_case_slug(token),
        })
        .collect::<Vec<_>>();
    if labels.is_empty() {
        None
    } else {
        Some(labels.join(" "))
    }
}

fn canonical_huggingface_group_key(
    record: &HuggingFaceModelInfoRecord,
    slug: &str,
    quantization: Option<&str>,
    specialization_profile: &[String],
) -> String {
    let family_source = generic_huggingface_family_source_text(record, slug);
    let family_key = slugify_catalog_identifier(&family_source);
    let specialization_key = specialization_profile_key(specialization_profile);
    let quant_key = quantization
        .map(slugify_catalog_identifier)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "gguf".to_string());
    match specialization_key {
        Some(specialization_key) => format!("{}-{}-{}", family_key, specialization_key, quant_key),
        None => format!("{}-{}", family_key, quant_key),
    }
}

fn huggingface_search_candidate_score(record: &HuggingFaceSearchRecord) -> i32 {
    let repo_id = record
        .model_id
        .as_deref()
        .or(record.id.as_deref())
        .unwrap_or_default();
    let mut score = infer_quality_score(record.downloads.unwrap_or_default(), record.likes.unwrap_or_default()) as i32;
    if tag_matches_prefix(&record.tags, "base_model:") {
        score += 8;
    }
    if tag_matches_exact(&record.tags, "endpoints_compatible") {
        score += 4;
    }
    if record
        .pipeline_tag
        .as_deref()
        .map(|value| value.eq_ignore_ascii_case("text-generation"))
        .unwrap_or(false)
        || tag_matches_exact(&record.tags, "text-generation")
        || tag_matches_exact(&record.tags, "conversational")
    {
        score += 4;
    }
    if record
        .pipeline_tag
        .as_deref()
        .map(|value| {
            matches!(
                value,
                "image-text-to-text" | "image-to-text" | "video-text-to-text" | "image-text-to-image"
            )
        })
        .unwrap_or(false)
        || tag_matches_exact(&record.tags, "multimodal")
        || tag_matches_exact(&record.tags, "vision")
    {
        score -= 24;
    }
    score -= huggingface_repo_quality_penalty(repo_id, &record.tags) as i32;
    score
}

fn infer_ram_requirements_gb(disk_size_gb: f64) -> (f64, f64, u16) {
    let ram_min = round_gb((disk_size_gb * 1.8).max(4.0));
    let ram_recommended = round_gb((disk_size_gb * 2.8).max(ram_min + 2.0));
    let host_tier = if ram_recommended <= 8.0 {
        8
    } else if ram_recommended <= 16.0 {
        16
    } else if ram_recommended <= 32.0 {
        32
    } else {
        64
    };
    (ram_min, ram_recommended, host_tier)
}

fn preferred_quant_rank(file_name: &str) -> Option<usize> {
    let lower = file_name.to_ascii_lowercase();
    [
        "q4_k_m.gguf",
        "q4_0.gguf",
        "q5_k_m.gguf",
        "q5_0.gguf",
        "q6_k.gguf",
        "q8_0.gguf",
        "q3_k_m.gguf",
        "q2_k.gguf",
    ]
    .iter()
    .position(|candidate| lower.ends_with(candidate))
}

fn extract_quantization_from_file_name(file_name: &str) -> Option<String> {
    let lower = file_name.to_ascii_lowercase();
    for marker in ["q2_k", "q3_k_m", "q4_k_m", "q4_0", "q5_k_m", "q5_0", "q6_k", "q8_0", "bf16", "fp16"] {
        if lower.contains(marker) {
            return Some(marker.to_ascii_uppercase());
        }
    }
    None
}

fn choose_huggingface_gguf_file(record: &HuggingFaceModelInfoRecord) -> Option<HuggingFaceSiblingRecord> {
    let file_names = record
        .siblings
        .iter()
        .filter(|sibling| sibling.rfilename.to_ascii_lowercase().ends_with(".gguf"))
        .map(|sibling| sibling.rfilename.clone())
        .collect::<HashSet<_>>();

    record
        .siblings
        .iter()
        .filter(|sibling| sibling.rfilename.to_ascii_lowercase().ends_with(".gguf"))
        .filter(|sibling| preferred_quant_rank(&sibling.rfilename).is_some())
        .filter(|sibling| {
            let lower = sibling.rfilename.to_ascii_lowercase();
            !(lower.contains("-00001-of-") || lower.contains("-00002-of-") || lower.contains("mmproj"))
        })
        .filter(|sibling| {
            if let Some(quant) = extract_quantization_from_file_name(&sibling.rfilename) {
                let shard_candidate = sibling
                    .rfilename
                    .to_ascii_lowercase()
                    .replace(".gguf", "")
                    .replace(&quant.to_ascii_lowercase(), &quant.to_ascii_lowercase());
                !file_names.iter().any(|name| {
                    name.to_ascii_lowercase().starts_with(&format!("{}-", shard_candidate))
                        && name.to_ascii_lowercase().contains("-00001-of-")
                })
            } else {
                true
            }
        })
        .min_by(|left, right| {
            let left_rank = preferred_quant_rank(&left.rfilename).unwrap_or(usize::MAX);
            let right_rank = preferred_quant_rank(&right.rfilename).unwrap_or(usize::MAX);
            left_rank
                .cmp(&right_rank)
                .then_with(|| left.rfilename.cmp(&right.rfilename))
        })
        .cloned()
}

fn normalize_huggingface_model_record(
    record: HuggingFaceModelInfoRecord,
    benchmark_index: &HashMap<String, OpenEvalsBenchmarkRow>,
    benchlm_benchmark_index: &HashMap<String, BenchLmLeaderboardRow>,
    benchlm_pricing_index: &HashMap<String, BenchLmPricingRow>,
    benchlm_last_updated: Option<&String>,
) -> Option<CatalogModelRecord> {
    if record.private.unwrap_or(false) || record.gated.unwrap_or(false) {
        return None;
    }
    if is_probably_multimodal_repo(&record) {
        return None;
    }
    let pipeline_tag = record.pipeline_tag.clone().unwrap_or_default();
    if !(pipeline_tag.eq_ignore_ascii_case("text-generation")
        || record.tags.iter().any(|tag| tag.eq_ignore_ascii_case("text-generation"))
        || record.tags.iter().any(|tag| tag.eq_ignore_ascii_case("chat"))
        || record.tags.iter().any(|tag| tag.eq_ignore_ascii_case("conversational")))
    {
        return None;
    }

    let chosen_file = choose_huggingface_gguf_file(&record)?;
    let file_size_bytes = chosen_file
        .lfs
        .as_ref()
        .and_then(|lfs| lfs.size)
        .or(chosen_file.size)
        .or_else(|| record.gguf.as_ref().and_then(|gguf| gguf.total))
        .or_else(|| record.gguf.as_ref().and_then(|gguf| gguf.total_file_size))?;
    let disk_size_gb = round_gb(file_size_bytes as f64 / (1024.0 * 1024.0 * 1024.0));
    let (ram_min_gb, ram_recommended_gb, host_ram_tier_gb) = infer_ram_requirements_gb(disk_size_gb);
    let slug = slug_from_repo_id(&record.id);
    let family_source_text = generic_huggingface_family_source_text(&record, &slug);
    let parameter_scale = infer_parameter_scale(&record.id)
        .or_else(|| record.card_data.as_ref().and_then(|card| card.base_model.as_ref()).and_then(|base| infer_parameter_scale(base)))
        .unwrap_or_else(|| "desconhecido".to_string());
    let languages = infer_languages(&record);
    let domains = infer_domains(&record.tags, &record.id);
    let capabilities = infer_capabilities(&record.tags, &languages);
    let specialization_profile =
        specialization_profile_from_model_traits(&record.id, &record.tags, &languages, &domains, &capabilities);
    let display_name = match specialization_profile_display_suffix(&specialization_profile) {
        Some(suffix) => format!("{} {}", title_case_slug(&family_source_text.replace('_', "-")), suffix),
        None => title_case_slug(&family_source_text.replace('_', "-")),
    };
    let downloads = record.downloads.unwrap_or_default();
    let likes = record.likes.unwrap_or_default();
    let quality_penalty = huggingface_repo_quality_penalty(&record.id, &record.tags);
    let base_quality_score = infer_quality_score(downloads, likes);
    let quantization = extract_quantization_from_file_name(&chosen_file.rfilename);
    let canonical_group_key =
        canonical_huggingface_group_key(&record, &slug, quantization.as_deref(), &specialization_profile);
    let architecture = record
        .gguf
        .as_ref()
        .and_then(|gguf| gguf.architecture.clone())
        .unwrap_or_else(|| slug.clone());

    let mut model = CatalogModelRecord {
        model_id: canonical_group_key,
        display_name,
        family: architecture,
        parameter_scale,
        variant: quantization.clone().unwrap_or_else(|| "GGUF".to_string()),
        disk_size_gb,
        ram_min_gb,
        ram_recommended_gb,
        host_ram_tier_gb,
        languages,
        domains,
        capabilities,
        quality_score: base_quality_score.saturating_sub(quality_penalty),
        cpu_only_viable: disk_size_gb <= 10.0,
        gpu_optional: true,
        license: record
            .card_data
            .as_ref()
            .and_then(|card| card.license.clone())
            .unwrap_or_else(|| "See upstream".to_string()),
        homepage_or_source: format!("https://huggingface.co/{}", record.id),
        description: format!(
            "Repositorio GGUF real do Hugging Face com {} download(s) e {} curtida(s).",
            downloads, likes
        ),
        rationale: format!(
            "Familia {} com variante {} selecionada automaticamente a partir dos arquivos GGUF publicos do repositorio {}.",
            family_source_text,
            quantization.clone().unwrap_or_else(|| "GGUF".to_string()),
            record.id
        ),
        priority: base_quality_score.saturating_sub(quality_penalty),
        download_locations: vec![format!(
            "https://huggingface.co/{}/resolve/main/{}",
            record.id, chosen_file.rfilename
        )],
        checksums: chosen_file
            .lfs
            .as_ref()
            .and_then(|lfs| lfs.oid.clone())
            .map(|oid| vec![format!("sha256:{}", oid)])
            .unwrap_or_default(),
        provider_or_origin: record.author.clone(),
        variant_id: Some(chosen_file.rfilename.clone()),
        quantization,
        file_format: Some("GGUF".to_string()),
        editorial_recommendation: build_huggingface_repo_editorial_note(&record.id, &record.tags),
        benchmark_summary: None,
        benchmark_metrics: vec![],
        technical_profile: Some(CatalogTechnicalProfileRecord {
            context_window_tokens: record.gguf.as_ref().and_then(|gguf| gguf.context_length),
            estimated_prompt_tokens_per_second: None,
            estimated_generation_tokens_per_second: None,
            estimated_peak_memory_gb: Some(ram_recommended_gb),
            recommended_gpu_class: None,
            notes: Some(
                "Descoberta automatica via Hugging Face Hub; metricas de throughput ainda dependem de benchmark real por hardware."
                    .to_string(),
            ),
        }),
        local_file_name: Path::new(&chosen_file.rfilename)
            .file_name()
            .map(|value| value.to_string_lossy().to_string()),
    };

    enrich_model_with_open_evals_benchmarks(&mut model, &record, &slug, benchmark_index);
    enrich_model_with_benchlm_benchmarks(
        &mut model,
        &record,
        &slug,
        benchlm_benchmark_index,
        benchlm_pricing_index,
        benchlm_last_updated,
    );
    Some(model)
}

fn fetch_huggingface_model_info(repo_id: &str) -> Result<HuggingFaceModelInfoRecord, String> {
    let url = format!("https://huggingface.co/api/models/{}", repo_id);
    let contents = http_fetch_text(&url)?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("Falha ao interpretar modelo do Hugging Face {}: {}", repo_id, error))
}

fn fetch_huggingface_catalog_snapshot(url: &str, contents: &str) -> Result<ModelCatalogSnapshot, String> {
    let search_results: Vec<HuggingFaceSearchRecord> = serde_json::from_str(contents)
        .map_err(|error| format!("Falha ao interpretar busca do Hugging Face {}: {}", url, error))?;
    if search_results.is_empty() {
        return Err(format!("A busca do Hugging Face {} nao retornou repositorios GGUF.", url));
    }

    let benchmark_index = fetch_open_evals_benchmark_index().unwrap_or_default();
    let (benchlm_benchmark_index, benchlm_last_updated) =
        fetch_benchlm_benchmark_index().unwrap_or_else(|_| (HashMap::new(), None));
    let (benchlm_pricing_index, _) =
        fetch_benchlm_pricing_index().unwrap_or_else(|_| (HashMap::new(), None));
    let mut grouped_models = HashMap::<String, CatalogModelRecord>::new();
    let mut ranked_search_results = search_results;
    ranked_search_results.sort_by(|left, right| {
        huggingface_search_candidate_score(right)
            .cmp(&huggingface_search_candidate_score(left))
            .then_with(|| right.downloads.unwrap_or_default().cmp(&left.downloads.unwrap_or_default()))
            .then_with(|| right.likes.unwrap_or_default().cmp(&left.likes.unwrap_or_default()))
    });
    for search_result in ranked_search_results.into_iter().take(16) {
        if search_result.private.unwrap_or(false) || search_result.gated.unwrap_or(false) {
            continue;
        }
        let repo_id = search_result
            .model_id
            .or(search_result.id)
            .ok_or_else(|| "Resposta do Hugging Face sem modelId.".to_string())?;
        let info = fetch_huggingface_model_info(&repo_id)?;
        if let Some(model) = normalize_huggingface_model_record(
            info,
            &benchmark_index,
            &benchlm_benchmark_index,
            &benchlm_pricing_index,
            benchlm_last_updated.as_ref(),
        ) {
            let key = model.model_id.clone();
            if let Some(existing) = grouped_models.remove(&key) {
                grouped_models.insert(key, choose_preferred_catalog_model_record(existing, model));
            } else {
                grouped_models.insert(key, model);
            }
        }
    }

    let mut models = grouped_models.into_values().collect::<Vec<_>>();

    if models.is_empty() {
        return Err(format!(
            "A busca do Hugging Face {} nao conseguiu normalizar nenhum modelo GGUF utilizavel.",
            url
        ));
    }
    sort_catalog_models(&mut models);

    Ok(ModelCatalogSnapshot {
        catalog_version: format!("hf-live-{}", now_utc_string()),
        last_updated: now_utc_string(),
        source: format!("huggingface-search:{}", url),
        models,
    })
}

fn fetch_remote_model_catalog_snapshot(url: &str) -> Result<ModelCatalogSnapshot, String> {
    let contents = http_fetch_text(url)?;
    if is_huggingface_models_api_url(url) {
        return fetch_huggingface_catalog_snapshot(url, &contents);
    }

    let mut snapshot: ModelCatalogSnapshot = serde_json::from_str(&contents)
        .map_err(|error| format!("Falha ao interpretar catalogo remoto {}: {}", url, error))?;

    if snapshot.models.is_empty() {
        return Err(format!("Catalogo remoto {} nao trouxe nenhum modelo.", url));
    }

    snapshot.source = format!("remote-feed:{}", url);
    Ok(snapshot)
}

fn find_target_for_mount_path(target_mount_path: Option<&str>) -> Option<CandidateTarget> {
    target_mount_path.and_then(|mount_path| {
        let mount_path = mount_path.to_string();
        list_candidate_targets()
            .into_iter()
            .find(|candidate| candidate.mount_path == mount_path)
    })
}

fn round_gb(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn command_stdout(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

// #region debug-point A:debug-server-reporting
fn debug_server_report(hypothesis_id: &str, location: &str, msg: &str, data: serde_json::Value) {
    let env_path = workspace_root().join(".dbg").join("usb-eject-upload-web.env");
    let debug_env_path = workspace_root().join(".dbg").join("usb-eject-rag-lag.env");
    let mut url = "http://127.0.0.1:7777/event".to_string();
    let mut session_id = "usb-eject-upload-web".to_string();
    let selected_env_path = if debug_env_path.exists() { debug_env_path } else { env_path };
    if let Ok(contents) = fs::read_to_string(&selected_env_path) {
        for line in contents.lines() {
            if let Some(value) = line.strip_prefix("DEBUG_SERVER_URL=") {
                if !value.trim().is_empty() {
                    url = value.trim().to_string();
                }
            } else if let Some(value) = line.strip_prefix("DEBUG_SESSION_ID=") {
                if !value.trim().is_empty() {
                    session_id = value.trim().to_string();
                }
            }
        }
    }
    let payload = json!({
        "sessionId": session_id,
        "runId": "pre-fix",
        "hypothesisId": hypothesis_id,
        "location": location,
        "msg": format!("[DEBUG] {msg}"),
        "data": data,
        "ts": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|value| value.as_millis() as u64)
            .unwrap_or(0),
    })
    .to_string();
    let debug_dir = workspace_root().join(".dbg");
    let log_file_path = debug_dir.join(format!("trae-debug-log-{}.ndjson", session_id));
    let _ = fs::create_dir_all(&debug_dir);
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
    {
        let _ = writeln!(file, "{}", payload);
    }
    let _ = Command::new("curl")
        .args(["-s", "-X", "POST", "-H", "Content-Type: application/json", "--data", &payload, &url])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}
// #endregion

fn disk_metrics(path: &Path) -> Option<(f64, f64, f64)> {
    let path_str = path.to_str()?;
    let stdout = command_stdout("df", &["-k", path_str])?;
    let line = stdout
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())?;

    let columns: Vec<&str> = line.split_whitespace().collect();
    if columns.len() < 6 {
        return None;
    }

    let total_kb = columns.get(1)?.parse::<f64>().ok()?;
    let available_kb = columns.get(3)?.parse::<f64>().ok()?;
    let usage_percent = columns
        .get(4)?
        .trim_end_matches('%')
        .parse::<f64>()
        .ok()?;

    Some((
        round_gb(total_kb / 1024.0 / 1024.0),
        round_gb(available_kb / 1024.0 / 1024.0),
        usage_percent,
    ))
}

#[cfg(target_os = "macos")]
fn filesystem_name(path: &Path) -> Option<String> {
    command_stdout("stat", &["-f", "%T", path.to_str()?])
}

#[cfg(target_os = "linux")]
fn filesystem_name(path: &Path) -> Option<String> {
    command_stdout("stat", &["-f", "-c", "%T", path.to_str()?])
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn filesystem_name(_path: &Path) -> Option<String> {
    None
}

fn installation_budget_gb(capacity_gb: Option<f64>, available_gb: Option<f64>) -> Option<f64> {
    let capacity_budget = capacity_gb.map(|capacity| round_gb((capacity - 12.0).max(0.0)));

    match (capacity_budget, available_gb) {
        (Some(capacity_budget), Some(available_gb)) => Some(round_gb(capacity_budget.min(available_gb))),
        (Some(capacity_budget), None) => Some(capacity_budget),
        (None, Some(available_gb)) => Some(round_gb((available_gb - 10.0).max(0.0))),
        (None, None) => None,
    }
}

fn recommendation_from_capacity_gb(capacity_gb: Option<f64>) -> (String, u8, Vec<String>) {
    let Some(capacity_gb) = capacity_gb else {
        return (
            "Capacidade ainda nao analisada".to_string(),
            0,
            vec!["O Builder ainda nao conseguiu medir a capacidade total deste pendrive.".to_string()],
        );
    };

    if capacity_gb < 16.0 {
        return (
            "Espaco insuficiente para um build completo".to_string(),
            0,
            vec![
                "Recomendado usar um pendrive com pelo menos 16 GB de capacidade total.".to_string(),
                "Considere outro pendrive antes de baixar modelos e packs.".to_string(),
            ],
        );
    }

    if capacity_gb < 32.0 {
        return (
            "Build enxuto com um modelo 3B quantizado".to_string(),
            1,
            vec!["Capacidade viavel para um USBRAIN compacto focado em CPU-only.".to_string()],
        );
    }

    if capacity_gb < 48.0 {
        return (
            "Build padrao com um modelo 3B e packs essenciais".to_string(),
            1,
            vec!["Ja comporta runtime, embeddings e um conjunto inicial de packs.".to_string()],
        );
    }

    if capacity_gb < 80.0 {
        return (
            "Build completo com lineup para PCs de 8 GB, 16 GB, 32 GB e 64 GB".to_string(),
            4,
            vec!["Ha espaco para combinar modelos leves, intermediarios e avancados no mesmo pendrive.".to_string()],
        );
    }

    (
        "Build avancado com lineup multi-hardware e packs especialistas".to_string(),
        5,
        vec!["Espaco confortavel para multiplos perfis de hardware, packs maiores e modelos de qualidade superior.".to_string()],
    )
}

#[cfg(target_os = "macos")]
fn plist_bool_value(plist: &str, key: &str) -> Option<bool> {
    let mut lines = plist.lines();

    while let Some(line) = lines.next() {
        if line.trim() == format!("<key>{key}</key>") {
            let next_line = lines.next()?.trim();
            return match next_line {
                "<true/>" => Some(true),
                "<false/>" => Some(false),
                _ => None,
            };
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn plist_string_value(plist: &str, key: &str) -> Option<String> {
    let mut lines = plist.lines();

    while let Some(line) = lines.next() {
        if line.trim() == format!("<key>{key}</key>") {
            let next_line = lines.next()?.trim();
            if next_line.starts_with("<string>") && next_line.ends_with("</string>") {
                return Some(
                    next_line
                        .trim_start_matches("<string>")
                        .trim_end_matches("</string>")
                        .to_string(),
                );
            }
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn macos_usb_source_for_path(path: &Path) -> Option<String> {
    let plist = command_stdout("diskutil", &["info", "-plist", path.to_str()?])?;
    let is_internal = plist_bool_value(&plist, "Internal").unwrap_or(true);
    let removable = plist_bool_value(&plist, "Removable").unwrap_or(false)
        || plist_bool_value(&plist, "RemovableMedia").unwrap_or(false)
        || plist_bool_value(&plist, "Ejectable").unwrap_or(false);
    let bus_protocol = plist_string_value(&plist, "BusProtocol").unwrap_or_default();

    if is_internal || !removable || !bus_protocol.eq_ignore_ascii_case("usb") {
        return None;
    }

    let device_identifier = plist_string_value(&plist, "DeviceIdentifier").unwrap_or_else(|| "usb-device".to_string());
    Some(format!("macos-usb:{device_identifier}"))
}

#[cfg(target_os = "macos")]
fn macos_eject_target(target_root: &Path) -> Result<String, String> {
    let started_at = Instant::now();
    let target_root_str = target_root
        .to_str()
        .ok_or_else(|| "Destino invalido para ejeção.".to_string())?;
    let plist = command_stdout(
        "diskutil",
        &["info", "-plist", target_root_str],
    )
    .ok_or_else(|| format!("Falha ao consultar o volume {} via diskutil.", target_root.display()))?;
    let device_identifier = plist_string_value(&plist, "ParentWholeDisk")
        .or_else(|| plist_string_value(&plist, "DeviceIdentifier"))
        .ok_or_else(|| format!("Nao foi possivel determinar o disco fisico de {}.", target_root.display()))?;
    debug_server_report(
        "A",
        "commands.rs:macos_eject_target:start",
        "Starting macOS eject flow",
        json!({
            "target_root": target_root.display().to_string(),
            "device_identifier": device_identifier,
        }),
    );

    let _ = Command::new("sync").status();
    let mut failure_details = String::new();
    for attempt in 1..=3 {
        let unmount_disk_result = Command::new("diskutil")
            .args(["unmountDisk", "force", &device_identifier])
            .output();
        let unmount_mount_result = Command::new("diskutil")
            .args(["unmount", "force", target_root_str])
            .output();
        debug_server_report(
            "A",
            "commands.rs:macos_eject_target:pre-eject",
            "Executed pre-eject unmount commands",
            json!({
                "attempt": attempt,
                "target_root": target_root.display().to_string(),
                "device_identifier": device_identifier,
                "unmount_disk_ok": unmount_disk_result.as_ref().map(|output| output.status.success()).unwrap_or(false),
                "unmount_mount_ok": unmount_mount_result.as_ref().map(|output| output.status.success()).unwrap_or(false),
            }),
        );
        thread::sleep(Duration::from_millis(500));
        let output = Command::new("diskutil")
            .args(["eject", &device_identifier])
            .output()
            .map_err(|error| format!("Falha ao executar diskutil eject {}: {}", device_identifier, error))?;
        if output.status.success() {
            debug_server_report(
                "B",
                "commands.rs:macos_eject_target:success",
                "macOS eject flow succeeded",
                json!({
                    "attempt": attempt,
                    "target_root": target_root.display().to_string(),
                    "device_identifier": device_identifier,
                    "elapsed_ms": started_at.elapsed().as_millis(),
                }),
            );
            return Ok(format!(
                "Pendrive {} ejetado automaticamente via diskutil ({}).",
                target_root.display(),
                device_identifier
            ));
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        failure_details = if !stderr.is_empty() { stderr } else { stdout };
        debug_server_report(
            "A",
            "commands.rs:macos_eject_target:retry",
            "diskutil eject attempt failed",
            json!({
                "target_root": target_root.display().to_string(),
                "device_identifier": device_identifier,
                "attempt": attempt,
                "details": failure_details,
            }),
        );
        let _ = Command::new("diskutil")
            .args(["unmountDisk", "force", &device_identifier])
            .output();
        let _ = Command::new("diskutil")
            .args(["eject", target_root_str])
            .output();
        thread::sleep(Duration::from_millis((attempt as u64) * 700));
    }

    debug_server_report(
        "B",
        "commands.rs:macos_eject_target:final-failure",
        "macOS eject flow exhausted all retries",
        json!({
            "target_root": target_root.display().to_string(),
            "device_identifier": device_identifier,
            "failure_details": failure_details,
            "elapsed_ms": started_at.elapsed().as_millis(),
        }),
    );

    Err(format!(
        "Falha ao ejetar o pendrive {} ({}): {}",
        target_root.display(),
        device_identifier,
        failure_details
    ))
}

#[cfg(not(target_os = "macos"))]
fn macos_eject_target(_target_root: &Path) -> Result<String, String> {
    Err("Ejeção automática indisponível fora do macOS.".to_string())
}

#[cfg(not(target_os = "macos"))]
fn macos_usb_source_for_path(_path: &Path) -> Option<String> {
    None
}

#[tauri::command]
pub fn eject_target(target_mount_path: String) -> Result<String, String> {
    let target_root = PathBuf::from(&target_mount_path);
    debug_server_report(
        "B",
        "commands.rs:eject_target:entry",
        "Dedicated eject command invoked",
        json!({
            "target_mount_path": target_mount_path,
            "target_exists": target_root.exists(),
        }),
    );
    if !target_root.exists() {
        return Ok(format!(
            "O pendrive {} ja nao esta montado; nenhuma ejeção adicional foi necessaria.",
            target_root.display()
        ));
    }
    macos_eject_target(&target_root)
}

fn analyze_target(path: PathBuf, label: String, source: &str, removable: bool) -> CandidateTarget {
    let filesystem = filesystem_name(&path).unwrap_or_else(|| "desconhecido".to_string());
    let (capacity_gb, available_gb, usage_percent) = disk_metrics(&path)
        .map(|(capacity, available, usage)| (Some(capacity), Some(available), Some(usage)))
        .unwrap_or((None, None, None));
    let (recommended_model_tier, estimated_model_slots, mut warnings) =
        recommendation_from_capacity_gb(capacity_gb);
    let install_budget_gb = installation_budget_gb(capacity_gb, available_gb);

    let filesystem_lc = filesystem.to_ascii_lowercase();
    if !(filesystem_lc.contains("exfat")
        || filesystem_lc.contains("msdos")
        || filesystem_lc.contains("fat")
        || filesystem_lc.contains("apfs")
        || filesystem_lc.contains("ext"))
    {
        warnings.push(format!(
            "Filesystem '{}' pode exigir validacao extra para compatibilidade entre hosts.",
            filesystem
        ));
    }

    if matches!((available_gb, install_budget_gb), (Some(available), Some(install_budget)) if available < install_budget) {
        warnings.push(
            "O espaco livre atual esta abaixo do orcamento ideal calculado pela capacidade total do pendrive.".to_string(),
        );
    }

    CandidateTarget {
        label,
        mount_path: path.display().to_string(),
        source: source.to_string(),
        filesystem,
        capacity_gb,
        available_gb,
        usage_percent,
        removable,
        recommended_model_tier,
        estimated_model_slots,
        warnings,
    }
}

fn fallback_target() -> CandidateTarget {
    let mut target = analyze_target(
        PathBuf::from("/Volumes/USBRAIN"),
        "USB target preview".to_string(),
        "fallback-preview",
        true,
    );
    target
        .warnings
        .push("Selecione um pendrive USB real para habilitar a criacao do USBRAIN.".to_string());
    target
}

fn slugify(input: &str) -> String {
    let slug = input
        .to_ascii_lowercase()
        .chars()
        .map(|ch| match ch {
            'a'..='z' | '0'..='9' => ch,
            _ => '-',
        })
        .collect::<String>();
    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "usbrain".to_string()
    } else {
        slug
    }
}

fn normalize_model_name(name: &str) -> String {
    name.to_ascii_lowercase()
        .chars()
        .map(|ch| match ch {
            'a'..='z' | '0'..='9' => ch,
            _ => '-',
        })
        .collect::<String>()
}

fn build_id_from_target(target_label: &str) -> String {
    format!("preview-{}", slugify(target_label))
}

fn primary_language(languages: &[String]) -> String {
    languages
        .first()
        .cloned()
        .unwrap_or_else(|| "pt-BR".to_string())
}

fn profile_mode_from_build_mode(build_mode: &str) -> String {
    match build_mode {
        "offline-only" => "offline",
        "local-assisted" => "local-assisted",
        "hybrid" => "hybrid",
        _ => "offline",
    }
    .to_string()
}

fn privacy_mode_from_build_mode(build_mode: &str) -> String {
    match build_mode {
        "offline-only" => "strict-local",
        "hybrid" => "hybrid-explicit",
        "local-assisted" => "local-first",
        _ => "local-first",
    }
    .to_string()
}

fn fallback_strategy(allow_cloud_fallback: bool, build_mode: &str) -> String {
    if !allow_cloud_fallback {
        return "local-only".to_string();
    }

    match build_mode {
        "hybrid" => "local-then-cloud",
        "local-assisted" => "user-confirmed-cloud",
        _ => "local-only",
    }
    .to_string()
}

fn priority_mode(build_mode: &str, capacity_gb: Option<f64>) -> String {
    if matches!(capacity_gb, Some(value) if value < 32.0) {
        return "speed".to_string();
    }

    match build_mode {
        "offline-only" => "offline-max",
        "hybrid" => "balanced",
        "local-assisted" => "rag-oriented",
        _ => "balanced",
    }
    .to_string()
}

fn quantization_from_file_name(file_name: &str) -> String {
    let normalized = file_name.to_ascii_uppercase();
    for token in ["Q6_K", "Q5_K_M", "Q5_K", "Q4_K_M", "Q4_K", "Q3_K_M", "Q3_K"] {
        if normalized.contains(token) {
            return token.to_string();
        }
    }
    "unknown".to_string()
}

fn classify_local_model_with_source(
    model_path: &Path,
    source_label: &str,
) -> AvailableLocalModel {
    let file_name = model_path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown.gguf".to_string());
    let normalized_name = normalize_model_name(&file_name);
    let disk_size_gb = fs::metadata(model_path)
        .ok()
        .map(|metadata| round_gb(metadata.len() as f64 / 1024.0 / 1024.0 / 1024.0))
        .unwrap_or(0.0);

    let (display_name, recommended_ram_gb, priority, role) = if normalized_name.contains("phi-3-5-mini") {
        ("Phi-3.5 Mini".to_string(), 8.0, 100, "fast-small".to_string())
    } else if normalized_name.contains("llama-3-2-3b") {
        ("Llama 3.2 3B".to_string(), 8.0, 95, "fast-small".to_string())
    } else if normalized_name.contains("qwen2-5-3b") {
        ("Qwen2.5 3B".to_string(), 8.0, 92, "fast-small".to_string())
    } else if normalized_name.contains("gemma-2-2b") {
        ("Gemma 2 2B".to_string(), 8.0, 85, "fast-small".to_string())
    } else if normalized_name.contains("qwen2-5-7b") {
        ("Qwen2.5 7B".to_string(), 16.0, 80, "primary".to_string())
    } else if normalized_name.contains("phi-4-14b") {
        ("Phi-4 14B".to_string(), 24.0, 75, "quality-large".to_string())
    } else if normalized_name.contains("qwen-qwen3-8b") || normalized_name.contains("qwen3-8b") {
        ("Qwen3 8B".to_string(), 16.0, 70, "quality-large".to_string())
    } else if disk_size_gb >= 10.0 {
        (Path::new(&file_name).file_stem().map(|value| value.to_string_lossy().to_string()).unwrap_or_else(|| file_name.clone()), 24.0, 50, "quality-large".to_string())
    } else if disk_size_gb >= 6.0 {
        (Path::new(&file_name).file_stem().map(|value| value.to_string_lossy().to_string()).unwrap_or_else(|| file_name.clone()), 16.0, 45, "primary".to_string())
    } else {
        (Path::new(&file_name).file_stem().map(|value| value.to_string_lossy().to_string()).unwrap_or_else(|| file_name.clone()), 8.0, 40, "fast-small".to_string())
    };

    AvailableLocalModel {
        file_name,
        absolute_path: model_path.display().to_string(),
        normalized_name,
        display_name,
        disk_size_gb,
        recommended_ram_gb,
        priority,
        role,
        source_label: source_label.to_string(),
    }
}

fn is_candidate_local_model_path(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let Some(file_name) = path.file_name().map(|value| value.to_string_lossy().to_string()) else {
        return false;
    };
    if should_ignore_path_name(&file_name) {
        return false;
    }
    path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("gguf"))
}

fn discover_local_models() -> Vec<AvailableLocalModel> {
    let mut unique_models = HashMap::<String, AvailableLocalModel>::new();
    let model_sources = [
        (builder_library_models_dir(), "Biblioteca local"),
        (models_dir(), "Legado em models/"),
    ];

    for (directory, source_label) in model_sources {
        for model in fs::read_dir(directory)
            .ok()
            .into_iter()
            .flat_map(|entries| entries.filter_map(Result::ok))
            .map(|entry| entry.path())
            .filter(|path| is_candidate_local_model_path(path))
            .map(|path| classify_local_model_with_source(&path, source_label))
        {
            unique_models.entry(model.file_name.clone()).or_insert(model);
        }
    }

    let mut models = unique_models.into_values().collect::<Vec<_>>();

    models.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left.disk_size_gb.partial_cmp(&right.disk_size_gb).unwrap_or(std::cmp::Ordering::Equal))
            .then_with(|| left.normalized_name.cmp(&right.normalized_name))
    });
    models
}

fn catalog_model_availability(
    catalog_model: &CatalogModelRecord,
    available_local_models: &[AvailableLocalModel],
) -> (bool, String, String) {
    if let Some(local_model) = catalog_model_local_source(catalog_model, available_local_models) {
        return (
            true,
            "available-local".to_string(),
            local_model.source_label,
        );
    }

    (
        false,
        "requires-download".to_string(),
        "Precisa baixar".to_string(),
    )
}

fn catalog_model_local_source(
    catalog_model: &CatalogModelRecord,
    available_local_models: &[AvailableLocalModel],
) -> Option<AvailableLocalModel> {
    if let Some(file_name) = catalog_model.local_file_name.as_ref() {
        if let Some(local_model) = available_local_models
            .iter()
            .find(|model| &model.file_name == file_name)
        {
            return Some(local_model.clone());
        }
    }

    available_local_models
        .iter()
        .find(|model| model.normalized_name.contains(&normalize_model_name(&catalog_model.model_id)))
        .cloned()
}

fn catalog_model_to_summary(
    catalog_model: &CatalogModelRecord,
    available_local_models: &[AvailableLocalModel],
    install_budget_gb: Option<f64>,
) -> CatalogModelSummary {
    let fits_selected_target = install_budget_gb
        .map(|budget| catalog_model.disk_size_gb <= budget)
        .unwrap_or(true);
    let (available_locally, availability_status, availability_label) =
        catalog_model_availability(catalog_model, available_local_models);
    let benchmark_summary = catalog_model.benchmark_summary.as_ref().map(|benchmark| CatalogBenchmarkSummary {
        source_label: benchmark.source_label.clone(),
        summary: benchmark.summary.clone(),
        composite_score: benchmark.composite_score,
        rank_label: benchmark.rank_label.clone(),
        last_verified_at: benchmark.last_verified_at.clone(),
        match_origin: benchmark.match_origin.clone(),
        confidence_label: benchmark.confidence_label.clone(),
        confidence_level: benchmark.confidence_level.clone(),
    });
    let benchmark_metrics = catalog_model
        .benchmark_metrics
        .iter()
        .map(|metric| CatalogBenchmarkMetric {
            label: metric.label.clone(),
            value: metric.value,
            unit: metric.unit.clone(),
            benchmark_suite: metric.benchmark_suite.clone(),
            higher_is_better: metric.higher_is_better,
        })
        .collect::<Vec<_>>();
    let technical_profile = catalog_model.technical_profile.as_ref().map(|profile| CatalogTechnicalProfile {
        context_window_tokens: profile.context_window_tokens,
        estimated_prompt_tokens_per_second: profile.estimated_prompt_tokens_per_second,
        estimated_generation_tokens_per_second: profile.estimated_generation_tokens_per_second,
        estimated_peak_memory_gb: profile.estimated_peak_memory_gb,
        recommended_gpu_class: profile.recommended_gpu_class.clone(),
        notes: profile.notes.clone(),
    });

    CatalogModelSummary {
        model_id: catalog_model.model_id.clone(),
        display_name: catalog_model.display_name.clone(),
        family: catalog_model.family.clone(),
        parameter_scale: catalog_model.parameter_scale.clone(),
        variant: catalog_model.variant.clone(),
        disk_size_gb: catalog_model.disk_size_gb,
        ram_min_gb: catalog_model.ram_min_gb,
        ram_recommended_gb: catalog_model.ram_recommended_gb,
        host_ram_tier_gb: catalog_model.host_ram_tier_gb,
        languages: catalog_model.languages.clone(),
        domains: catalog_model.domains.clone(),
        capabilities: catalog_model.capabilities.clone(),
        quality_score: catalog_model.quality_score,
        cpu_only_viable: catalog_model.cpu_only_viable,
        gpu_optional: catalog_model.gpu_optional,
        license: catalog_model.license.clone(),
        homepage_or_source: catalog_model.homepage_or_source.clone(),
        download_locations: catalog_model.download_locations.clone(),
        checksums: catalog_model.checksums.clone(),
        provider_or_origin: catalog_model.provider_or_origin.clone(),
        variant_id: catalog_model.variant_id.clone(),
        quantization: catalog_model.quantization.clone(),
        file_format: catalog_model.file_format.clone(),
        editorial_recommendation: catalog_model.editorial_recommendation.clone(),
        benchmark_summary,
        benchmark_metrics,
        technical_profile,
        description: catalog_model.description.clone(),
        rationale: catalog_model.rationale.clone(),
        local_file_name: catalog_model.local_file_name.clone(),
        available_locally,
        availability_status,
        availability_label,
        fits_selected_target,
    }
}

fn catalog_model_to_recommendation(
    catalog_model: &CatalogModelRecord,
    available_local_models: &[AvailableLocalModel],
) -> SuggestedCatalogModel {
    let (available_locally, availability_status, availability_label) =
        catalog_model_availability(catalog_model, available_local_models);

    SuggestedCatalogModel {
        model_id: catalog_model.model_id.clone(),
        display_name: catalog_model.display_name.clone(),
        estimated_disk_size_gb: catalog_model.disk_size_gb,
        target_host_ram_gb: catalog_model.host_ram_tier_gb,
        rationale: catalog_model.rationale.clone(),
        available_locally,
        availability_status,
        availability_label,
    }
}

fn host_ram_recommendations(
    target: &CandidateTarget,
    available_local_models: &[AvailableLocalModel],
    catalog_models: &[CatalogModelRecord],
) -> Vec<HostRamRecommendation> {
    let install_budget_gb = installation_budget_gb(target.capacity_gb, target.available_gb).unwrap_or(0.0);

    [8_u16, 16, 32, 64]
        .into_iter()
        .filter_map(|host_ram_gb| {
            let mut candidates = catalog_models
                .iter()
                .filter(|model| model.host_ram_tier_gb == host_ram_gb)
                .filter(|model| model.disk_size_gb <= install_budget_gb)
                .collect::<Vec<_>>();

            candidates.sort_by(|left, right| {
                catalog_model_preference_cmp(left, right).then_with(|| {
                    let left_available = catalog_model_local_source(left, available_local_models).is_some();
                    let right_available = catalog_model_local_source(right, available_local_models).is_some();
                    right_available.cmp(&left_available)
                })
            });

            let suggested_model = candidates.first()?;
            let alternatives = candidates
                .iter()
                .skip(1)
                .take(2)
                .map(|model| catalog_model_to_recommendation(model, available_local_models))
                .collect::<Vec<_>>();

            Some(HostRamRecommendation {
                host_ram_gb,
                label: format!("PC com {host_ram_gb} GB de RAM"),
                summary: format!(
                    "Sugestao pensada para um pendrive de {} com orcamento de aproximadamente {} para modelos.",
                    target
                        .capacity_gb
                        .map(|value| format!("{value:.1} GB"))
                        .unwrap_or_else(|| "capacidade indefinida".to_string()),
                    install_budget_gb
                ),
                suggested_model: catalog_model_to_recommendation(suggested_model, available_local_models),
                alternatives,
            })
        })
        .collect()
}

fn context_strategy(capacity_gb: Option<f64>) -> String {
    match capacity_gb {
        Some(value) if value < 32.0 => "conservative".to_string(),
        Some(value) if value > 96.0 => "max-quality".to_string(),
        _ => "adaptive".to_string(),
    }
}

fn boot_timeout_profile(filesystem: &str) -> String {
    let filesystem_lc = filesystem.to_ascii_lowercase();
    if filesystem_lc.contains("fat") || filesystem_lc.contains("exfat") {
        "slow-storage".to_string()
    } else {
        "balanced".to_string()
    }
}

fn planned_models(
    target: &CandidateTarget,
    languages: &[String],
    _allow_gpu: bool,
    selected_model_ids: &[String],
    catalog_models: &[CatalogModelRecord],
) -> (Vec<PlannedModel>, Vec<PendingModelDownload>, Vec<String>) {
    let mut selected = Vec::new();
    let mut pending_downloads = Vec::new();
    let mut warnings = Vec::new();
    let available_local_models = discover_local_models();
    let mut remaining_budget_gb = installation_budget_gb(target.capacity_gb, target.available_gb).unwrap_or(0.0);
    let slot_limit = usize::from(target.estimated_model_slots.max(1)).min(3);

    if !selected_model_ids.is_empty() {
        for model_id in selected_model_ids {
            if selected.len() + pending_downloads.len() >= slot_limit {
                warnings.push(format!(
                    "A selecao manual excedeu o limite de {} modelos para este pendrive.",
                    slot_limit
                ));
                break;
            }

            let Some(catalog_model) = catalog_models.iter().find(|model| &model.model_id == model_id) else {
                warnings.push(format!(
                    "O modelo selecionado {} nao foi encontrado no catalogo atual.",
                    model_id
                ));
                continue;
            };

            if remaining_budget_gb < catalog_model.disk_size_gb {
                warnings.push(format!(
                    "{} nao cabe no orcamento restante do pendrive.",
                    catalog_model.display_name
                ));
                continue;
            }

            let Some(local_model) = catalog_model_local_source(catalog_model, &available_local_models) else {
                let local_file_name = catalog_model
                    .local_file_name
                    .clone()
                    .unwrap_or_else(|| format!("{}.gguf", slugify(&catalog_model.model_id)));
                pending_downloads.push(PendingModelDownload {
                    model_id: catalog_model.model_id.clone(),
                    display_name: catalog_model.display_name.clone(),
                    disk_size_gb: catalog_model.disk_size_gb,
                    recommended_ram_gb: catalog_model.ram_recommended_gb,
                    homepage_or_source: catalog_model.homepage_or_source.clone(),
                    download_locations: catalog_model.download_locations.clone(),
                    checksums: catalog_model.checksums.clone(),
                    local_file_name: local_file_name.clone(),
                    relative_target_path: format!("models/{}", local_file_name),
                    availability_status: "requires-download".to_string(),
                    availability_label: "Precisa baixar".to_string(),
                });
                warnings.push(format!(
                    "{} ainda nao esta na biblioteca local; o download sob demanda sera necessario antes da montagem offline.",
                    catalog_model.display_name
                ));
                remaining_budget_gb -= catalog_model.disk_size_gb;
                continue;
            };

            selected.push(PlannedModel {
                installed_model_id: catalog_model.model_id.clone(),
                display_name: catalog_model.display_name.clone(),
                role: "manual-selection".to_string(),
                disk_size_gb: local_model.disk_size_gb,
                recommended_ram_gb: catalog_model.ram_recommended_gb,
                runtime_family: "llama.cpp".to_string(),
                source_file_name: local_model.file_name.clone(),
                source_absolute_path: local_model.absolute_path.clone(),
                relative_target_path: format!("models/{}", local_model.file_name),
                available_locally: true,
                availability_status: "available-local".to_string(),
                availability_label: local_model.source_label.clone(),
            });
            remaining_budget_gb -= local_model.disk_size_gb;
        }
    } else {
        for model in &available_local_models {
            if selected.len() >= slot_limit {
                break;
            }

            if remaining_budget_gb >= model.disk_size_gb {
                selected.push(PlannedModel {
                    installed_model_id: slugify(&model.file_name),
                    display_name: if languages.iter().any(|lang| lang == "pt-BR") {
                        model.display_name.clone()
                    } else {
                        format!("{} Intl", model.display_name)
                    },
                    role: model.role.clone(),
                    disk_size_gb: model.disk_size_gb,
                    recommended_ram_gb: model.recommended_ram_gb,
                    runtime_family: "llama.cpp".to_string(),
                    source_file_name: model.file_name.clone(),
                    source_absolute_path: model.absolute_path.clone(),
                    relative_target_path: format!("models/{}", model.file_name),
                    available_locally: true,
                    availability_status: "available-local".to_string(),
                    availability_label: model.source_label.clone(),
                });
                remaining_budget_gb -= model.disk_size_gb;
            }
        }
    }

    if selected.is_empty() && selected_model_ids.is_empty() {
        if let Some(model) = available_local_models.first() {
            selected.push(PlannedModel {
                installed_model_id: slugify(&model.file_name),
                display_name: model.display_name.clone(),
                role: "fallback".to_string(),
                disk_size_gb: model.disk_size_gb,
                recommended_ram_gb: model.recommended_ram_gb,
                runtime_family: "llama.cpp".to_string(),
                source_file_name: model.file_name.clone(),
                source_absolute_path: model.absolute_path.clone(),
                relative_target_path: format!("models/{}", model.file_name),
                available_locally: true,
                availability_status: "available-local".to_string(),
                availability_label: model.source_label.clone(),
            });
        }
    }

    (selected, pending_downloads, warnings)
}

fn planned_packs(languages: &[String]) -> Vec<PlannedPack> {
    let mut packs = vec![
        PlannedPack {
            installed_pack_id: "pack-core-runtime".to_string(),
            name: "USBRAIN Core Runtime Pack".to_string(),
            pack_type: "behavior".to_string(),
            activation_mode: "always".to_string(),
            category: "core".to_string(),
        },
        PlannedPack {
            installed_pack_id: "pack-semantic-bge".to_string(),
            name: "USBRAIN Semantic BGE-M3 Pack".to_string(),
            pack_type: "tool".to_string(),
            activation_mode: "always".to_string(),
            category: "semantic-search".to_string(),
        },
    ];

    if languages.len() > 1 {
        packs.push(PlannedPack {
            installed_pack_id: "pack-multilingual-assist".to_string(),
            name: "USBRAIN Multilingual Assist Pack".to_string(),
            pack_type: "knowledge".to_string(),
            activation_mode: "language-matched".to_string(),
            category: "language".to_string(),
        });
    }

    packs
}

fn manifest_json<T: Serialize>(payload: &T) -> String {
    serde_json::to_string_pretty(payload).unwrap_or_else(|_| "{}".to_string())
}

fn write_export_file(
    path: &Path,
    contents: &str,
    export_root: &Path,
) -> Result<ExportedFileSummary, String> {
    fs::write(path, contents).map_err(|error| format!("Falha ao escrever {}: {}", path.display(), error))?;
    let metadata = fs::metadata(path)
        .map_err(|error| format!("Falha ao ler metadata de {}: {}", path.display(), error))?;
    let relative_path = path
        .strip_prefix(export_root)
        .map(|value| value.display().to_string())
        .unwrap_or_else(|_| path.display().to_string());

    Ok(ExportedFileSummary {
        file_name: path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| relative_path.clone()),
        absolute_path: path.display().to_string(),
        relative_path,
        size_bytes: metadata.len(),
    })
}

fn sha256_hex_for_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path)
        .map_err(|error| format!("Falha ao abrir {} para checksum: {}", path.display(), error))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|error| format!("Falha ao ler {} para checksum: {}", path.display(), error))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn build_release_artifact_checksums(
    exported_files: &[ExportedFileSummary],
) -> Result<Vec<ReleaseArtifactChecksum>, String> {
    exported_files
        .iter()
        .map(|file| {
            let path = Path::new(&file.absolute_path);
            Ok(ReleaseArtifactChecksum {
                file_name: file.file_name.clone(),
                absolute_path: file.absolute_path.clone(),
                relative_path: file.relative_path.clone(),
                size_bytes: file.size_bytes,
                sha256: sha256_hex_for_file(path)?,
            })
        })
        .collect()
}

fn sanitize_release_tag_fragment(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn recommended_release_tag(build_id: &str) -> String {
    let sanitized_build_id = sanitize_release_tag_fragment(build_id);
    if sanitized_build_id.is_empty() {
        format!("usbrain-builder-v{}", env!("CARGO_PKG_VERSION"))
    } else {
        format!(
            "usbrain-builder-v{}-{}",
            env!("CARGO_PKG_VERSION"),
            sanitized_build_id
        )
    }
}

fn render_release_checksums_file(artifact_checksums: &[ReleaseArtifactChecksum]) -> String {
    let mut lines = artifact_checksums
        .iter()
        .map(|artifact| format!("{}  {}", artifact.sha256, artifact.relative_path))
        .collect::<Vec<_>>();
    lines.push(String::new());
    lines.join("\n")
}

fn render_github_release_notes(
    build_plan: &BuildPlanResponse,
    distribution_result: &BuildDistributionGateResult,
    release_status: &str,
    recommended_tag: &str,
    artifact_checksums: &[ReleaseArtifactChecksum],
) -> String {
    let checklist = if distribution_result.distribution_ready {
        "- Distribution gate liberado\n- Probe com LLM local aprovado\n- Build pronto para publicacao open source"
            .to_string()
    } else {
        "- Distribution gate bloqueado\n- Artefatos de release gerados para auditoria\n- Corrigir bloqueios antes de publicar"
            .to_string()
    };
    let artifact_lines = artifact_checksums
        .iter()
        .map(|artifact| {
            format!(
                "- `{}` (`{}`) - {} bytes - sha256 `{}`",
                artifact.file_name, artifact.relative_path, artifact.size_bytes, artifact.sha256
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "# USBRAIN Builder Release Package\n\n## Release Summary\n- Suggested tag: `{}`\n- Release status: `{}`\n- Distribution ready: `{}`\n- Build ID: `{}`\n- Target: `{}`\n- Build mode: `{}`\n- Languages: `{}`\n- Models selected: `{}`\n- Packs selected: `{}`\n\n## Publication Checklist\n{}\n\n## Distribution Gate\n- Status: `{}`\n- Message: {}\n- Readiness score: {}/100\n\n## Exported Artifacts\n{}\n",
        recommended_tag,
        release_status,
        distribution_result.distribution_ready,
        build_plan.summary.build_id,
        build_plan.summary.target_label,
        build_plan.summary.build_mode,
        build_plan.summary.languages.join(", "),
        build_plan.models.len(),
        build_plan.packs.len(),
        checklist,
        distribution_result.distribution_status,
        distribution_result.message,
        distribution_result.readiness_report.readiness_score,
        artifact_lines
    )
}

fn render_usbrain_release_summary(
    build_plan: &BuildPlanResponse,
    distribution_result: &BuildDistributionGateResult,
    release_status: &str,
    recommended_tag: &str,
    artifact_checksums: &[ReleaseArtifactChecksum],
) -> String {
    manifest_json(&json!({
        "build_id": build_plan.summary.build_id,
        "builder_version": env!("CARGO_PKG_VERSION"),
        "release_status": release_status,
        "release_ready": distribution_result.distribution_ready,
        "recommended_release_tag": recommended_tag,
        "target_label": build_plan.summary.target_label,
        "build_mode": build_plan.summary.build_mode,
        "languages": build_plan.summary.languages,
        "selected_model_count": build_plan.models.len(),
        "selected_pack_count": build_plan.packs.len(),
        "distribution_status": distribution_result.distribution_status,
        "distribution_message": distribution_result.message,
        "readiness_score": distribution_result.readiness_report.readiness_score,
        "generated_at": now_utc_string(),
        "artifacts": artifact_checksums,
    }))
}

fn should_ignore_path_name(name: &str) -> bool {
    matches!(name, ".DS_Store" | ".Spotlight-V100" | ".Trashes" | ".fseventsd" | "__MACOSX")
        || name.starts_with("._")
}

fn copy_file_contents(source: &Path, destination: &Path) -> Result<(), String> {
    let mut source_file =
        fs::File::open(source).map_err(|error| format!("Falha ao abrir {}: {}", source.display(), error))?;
    let source_metadata = source_file
        .metadata()
        .map_err(|error| format!("Falha ao ler metadata de {}: {}", source.display(), error))?;

    let mut destination_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(destination)
        .map_err(|error| format!("Falha ao abrir {} para escrita: {}", destination.display(), error))?;

    io::copy(&mut source_file, &mut destination_file).map_err(|error| {
        format!(
            "Falha ao copiar conteudo de {} para {}: {}",
            source.display(),
            destination.display(),
            error
        )
    })?;
    destination_file
        .flush()
        .map_err(|error| format!("Falha ao finalizar escrita de {}: {}", destination.display(), error))?;

    fs::set_permissions(destination, source_metadata.permissions()).map_err(|error| {
        format!(
            "Falha ao aplicar permissoes de {} em {}: {}",
            source.display(),
            destination.display(),
            error
        )
    })?;

    Ok(())
}

fn copy_file_with_summary(
    source: &Path,
    destination: &Path,
    target_root: &Path,
) -> Result<ExportedFileSummary, String> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Falha ao criar diretorio {}: {}", parent.display(), error))?;
    }

    copy_file_contents(source, destination)?;

    let metadata = fs::metadata(destination)
        .map_err(|error| format!("Falha ao ler metadata de {}: {}", destination.display(), error))?;

    Ok(ExportedFileSummary {
        file_name: destination
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| destination.display().to_string()),
        absolute_path: destination.display().to_string(),
        relative_path: destination
            .strip_prefix(target_root)
            .map(|value| value.display().to_string())
            .unwrap_or_else(|_| destination.display().to_string()),
        size_bytes: metadata.len(),
    })
}

fn write_json_file_with_summary(
    destination: &Path,
    contents: &str,
    target_root: &Path,
) -> Result<ExportedFileSummary, String> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Falha ao criar diretorio {}: {}", parent.display(), error))?;
    }

    fs::write(destination, contents)
        .map_err(|error| format!("Falha ao escrever {}: {}", destination.display(), error))?;

    let metadata = fs::metadata(destination)
        .map_err(|error| format!("Falha ao ler metadata de {}: {}", destination.display(), error))?;

    Ok(ExportedFileSummary {
        file_name: destination
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| destination.display().to_string()),
        absolute_path: destination.display().to_string(),
        relative_path: destination
            .strip_prefix(target_root)
            .map(|value| value.display().to_string())
            .unwrap_or_else(|_| destination.display().to_string()),
        size_bytes: metadata.len(),
    })
}

fn prepare_builder_library_with_root(
    build_plan: &BuildPlanResponse,
    library_root: &Path,
) -> Result<BuilderLibraryPreparationResult, String> {
    let model_library_directory = library_root.join("models");
    let download_manifest_directory = builder_library_download_manifests_dir()
        .strip_prefix(builder_library_root())
        .map(|relative| library_root.join(relative))
        .unwrap_or_else(|_| library_root.join("manifests").join("model-downloads"));
    fs::create_dir_all(&model_library_directory)
        .map_err(|error| format!("Falha ao criar diretorio {}: {}", model_library_directory.display(), error))?;
    fs::create_dir_all(&download_manifest_directory)
        .map_err(|error| format!("Falha ao criar diretorio {}: {}", download_manifest_directory.display(), error))?;

    let mut exported_files = Vec::new();
    let mut warnings = Vec::new();
    let mut copied_models_count = 0;
    let mut already_available_count = 0;
    let mut written_manifests_count = 0;

    for model in &build_plan.models {
        let source = PathBuf::from(&model.source_absolute_path);
        let destination = model_library_directory.join(&model.source_file_name);

        if destination == source || destination.exists() {
            already_available_count += 1;
            continue;
        }

        if !source.exists() {
            warnings.push(format!(
                "Nao foi possivel materializar {} porque a origem nao existe mais em {}.",
                model.display_name, model.source_absolute_path
            ));
            continue;
        }

        exported_files.push(copy_file_with_summary(&source, &destination, library_root)?);
        copied_models_count += 1;
    }

    for pending in &build_plan.pending_model_downloads {
        let destination = download_manifest_directory.join(format!("{}.json", pending.model_id));
        let contents = manifest_json(&json!({
            "schema_version": "1.0",
            "kind": "model-download-request",
            "build_id": build_plan.summary.build_id,
            "generated_at": now_utc_string(),
            "model_id": pending.model_id,
            "display_name": pending.display_name,
            "disk_size_gb": pending.disk_size_gb,
            "recommended_ram_gb": pending.recommended_ram_gb,
            "homepage_or_source": pending.homepage_or_source,
            "download_locations": pending.download_locations,
            "checksums": pending.checksums,
            "local_file_name": pending.local_file_name,
            "relative_target_path": pending.relative_target_path,
            "availability_status": pending.availability_status,
            "availability_label": pending.availability_label,
            "notes": [
                "Catalogo atual ainda nao fornece URL direta de download do artefato.",
                "Manifesto gerado para futura aquisicao sob demanda a partir de fonte resolvida."
            ]
        }));
        exported_files.push(write_json_file_with_summary(&destination, &contents, library_root)?);
        written_manifests_count += 1;
    }

    Ok(BuilderLibraryPreparationResult {
        build_id: build_plan.summary.build_id.clone(),
        library_root: library_root.display().to_string(),
        model_library_directory: model_library_directory.display().to_string(),
        download_manifest_directory: download_manifest_directory.display().to_string(),
        copied_models_count,
        already_available_count,
        written_manifests_count,
        exported_files,
        pending_downloads: build_plan.pending_model_downloads.clone(),
        warnings,
    })
}

fn format_system_time(value: SystemTime) -> Option<String> {
    value
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs().to_string())
}

fn builder_library_inventory_with_root(library_root: &Path) -> Result<BuilderLibraryInventory, String> {
    let models_dir = library_root.join("models");
    let download_manifests_dir = library_root.join("manifests").join("model-downloads");

    fs::create_dir_all(&models_dir)
        .map_err(|error| format!("Falha ao criar diretorio da biblioteca local: {}", error))?;
    fs::create_dir_all(&download_manifests_dir)
        .map_err(|error| format!("Falha ao criar diretorio de manifestos da biblioteca local: {}", error))?;

    let mut artifacts = Vec::new();
    let mut total_size_bytes = 0_u64;

    for entry in fs::read_dir(&models_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
    {
        let path = entry.path();
        if !path.is_file() || !is_candidate_local_model_path(&path) {
            continue;
        }
        let metadata = fs::metadata(&path)
            .map_err(|error| format!("Falha ao ler metadata de {}: {}", path.display(), error))?;
        let size_bytes = metadata.len();
        total_size_bytes += size_bytes;
        let file_name = path
            .file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown.gguf".to_string());
        let display_name = file_name
            .strip_suffix(".gguf")
            .unwrap_or(&file_name)
            .replace(['-', '_'], " ");

        artifacts.push(BuilderLibraryArtifactSummary {
            artifact_id: file_name.clone(),
            artifact_type: "model".to_string(),
            display_name,
            source_label: "Biblioteca local".to_string(),
            local_path: path.display().to_string(),
            size_bytes,
            integrity_status: "present".to_string(),
            downloaded_at: metadata.modified().ok().and_then(format_system_time),
            source_url: None,
            removable: true,
        });
    }

    for entry in fs::read_dir(&download_manifests_dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
    {
        let path = entry.path();
        if !path.is_file() || path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        let metadata = fs::metadata(&path)
            .map_err(|error| format!("Falha ao ler metadata de {}: {}", path.display(), error))?;
        let size_bytes = metadata.len();
        total_size_bytes += size_bytes;
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("Falha ao ler manifesto {}: {}", path.display(), error))?;
        let manifest: serde_json::Value = serde_json::from_str(&contents)
            .map_err(|error| format!("Falha ao interpretar manifesto {}: {}", path.display(), error))?;
        let artifact_id = manifest
            .get("model_id")
            .and_then(|value| value.as_str())
            .map(str::to_string)
            .or_else(|| path.file_stem().map(|value| value.to_string_lossy().to_string()))
            .unwrap_or_else(|| "unknown-request".to_string());
        let display_name = manifest
            .get("display_name")
            .and_then(|value| value.as_str())
            .map(str::to_string)
            .unwrap_or_else(|| artifact_id.clone());
        let source_url = manifest
            .get("homepage_or_source")
            .and_then(|value| value.as_str())
            .map(str::to_string);

        artifacts.push(BuilderLibraryArtifactSummary {
            artifact_id,
            artifact_type: "download-request".to_string(),
            display_name,
            source_label: "Fila de aquisicao".to_string(),
            local_path: path.display().to_string(),
            size_bytes,
            integrity_status: "pending-download".to_string(),
            downloaded_at: metadata.modified().ok().and_then(format_system_time),
            source_url,
            removable: true,
        });
    }

    artifacts.sort_by(|left, right| left.display_name.cmp(&right.display_name));

    Ok(BuilderLibraryInventory {
        library_root: library_root.display().to_string(),
        artifact_count: artifacts.len(),
        total_size_bytes,
        artifacts,
    })
}

fn delete_builder_library_artifact_with_root(
    library_root: &Path,
    request: DeleteBuilderLibraryArtifactRequest,
) -> Result<DeleteBuilderLibraryArtifactResult, String> {
    let models_dir = library_root.join("models");
    let download_manifests_dir = library_root.join("manifests").join("model-downloads");
    let (target_path, normalized_type) = match request.artifact_type.as_str() {
        "model" => (models_dir.join(&request.artifact_id), "model".to_string()),
        "download-request" => (
            download_manifests_dir.join(format!("{}.json", request.artifact_id)),
            "download-request".to_string(),
        ),
        other => {
            return Err(format!("Tipo de artefato nao suportado para exclusao: {}", other));
        }
    };

    let canonical_root = library_root
        .canonicalize()
        .unwrap_or_else(|_| library_root.to_path_buf());

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Falha ao preparar diretorio {}: {}", parent.display(), error))?;
    }

    let canonical_target = target_path
        .canonicalize()
        .unwrap_or_else(|_| target_path.clone());

    if !canonical_target.starts_with(&canonical_root) {
        return Err("Tentativa de exclusao fora da biblioteca local foi bloqueada.".to_string());
    }

    if !target_path.exists() {
        return Err(format!("Artefato {} nao existe mais na biblioteca local.", target_path.display()));
    }

    let recovered_size_bytes = fs::metadata(&target_path)
        .map_err(|error| format!("Falha ao ler metadata de {}: {}", target_path.display(), error))?
        .len();

    fs::remove_file(&target_path)
        .map_err(|error| format!("Falha ao excluir {}: {}", target_path.display(), error))?;

    Ok(DeleteBuilderLibraryArtifactResult {
        library_root: library_root.display().to_string(),
        artifact_id: request.artifact_id,
        artifact_type: normalized_type,
        deleted_path: target_path.display().to_string(),
        recovered_size_bytes,
    })
}

fn first_non_empty_json_string(manifest: &serde_json::Value, key: &str) -> Option<String> {
    manifest
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn json_string_array(manifest: &serde_json::Value, key: &str) -> Vec<String> {
    manifest
        .get(key)
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn first_sha256_checksum(checksums: &[String]) -> Option<String> {
    checksums.iter().find_map(|checksum| {
        let trimmed = checksum.trim();
        trimmed
            .strip_prefix("sha256:")
            .or(Some(trimmed))
            .map(str::trim)
            .filter(|value| value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit()))
            .map(|value| value.to_ascii_lowercase())
    })
}

fn sanitized_library_file_name(requested: Option<String>, fallback_artifact_id: &str) -> String {
    requested
        .as_deref()
        .map(Path::new)
        .and_then(|path| path.file_name())
        .map(|value| value.to_string_lossy().trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| format!("{}.gguf", fallback_artifact_id))
}

fn download_url_to_file(url: &str, destination: &Path) -> Result<(), String> {
    let response = ureq::get(url)
        .call()
        .map_err(|error| format!("Falha ao baixar artefato {}: {}", url, error))?;
    let mut reader = response.into_reader();

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Falha ao criar diretorio {}: {}", parent.display(), error))?;
    }

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(destination)
        .map_err(|error| format!("Falha ao abrir {} para escrita: {}", destination.display(), error))?;

    io::copy(&mut reader, &mut file)
        .map_err(|error| format!("Falha ao gravar download em {}: {}", destination.display(), error))?;
    file.flush()
        .map_err(|error| format!("Falha ao finalizar download em {}: {}", destination.display(), error))?;

    Ok(())
}

fn download_builder_library_artifact_with_root(
    library_root: &Path,
    request: DownloadBuilderLibraryArtifactRequest,
) -> Result<DownloadBuilderLibraryArtifactResult, String> {
    let models_dir = library_root.join("models");
    let manifests_dir = library_root.join("manifests").join("model-downloads");
    fs::create_dir_all(&models_dir)
        .map_err(|error| format!("Falha ao criar diretorio {}: {}", models_dir.display(), error))?;
    fs::create_dir_all(&manifests_dir)
        .map_err(|error| format!("Falha ao criar diretorio {}: {}", manifests_dir.display(), error))?;

    let manifest_path = manifests_dir.join(format!("{}.json", request.artifact_id));
    if !manifest_path.exists() {
        return Err(format!(
            "Manifesto de download {} nao existe mais na biblioteca local.",
            manifest_path.display()
        ));
    }

    let contents = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("Falha ao ler manifesto {}: {}", manifest_path.display(), error))?;
    let manifest: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|error| format!("Falha ao interpretar manifesto {}: {}", manifest_path.display(), error))?;

    let artifact_id = first_non_empty_json_string(&manifest, "model_id").unwrap_or(request.artifact_id);
    let display_name = first_non_empty_json_string(&manifest, "display_name").unwrap_or_else(|| artifact_id.clone());
    let source_url = json_string_array(&manifest, "download_locations")
        .into_iter()
        .next()
        .ok_or_else(|| format!("O manifesto {} nao possui URL direta de download.", manifest_path.display()))?;
    let local_file_name =
        sanitized_library_file_name(first_non_empty_json_string(&manifest, "local_file_name"), &artifact_id);
    let destination_path = models_dir.join(&local_file_name);
    let temporary_path = models_dir.join(format!("{}.partial", local_file_name));

    let already_available = destination_path.exists();
    let sha256 = if already_available {
        sha256_hex_for_file(&destination_path)?
    } else {
        if temporary_path.exists() {
            fs::remove_file(&temporary_path).map_err(|error| {
                format!(
                    "Falha ao limpar download temporario anterior {}: {}",
                    temporary_path.display(),
                    error
                )
            })?;
        }

        download_url_to_file(&source_url, &temporary_path)?;
        let downloaded_sha256 = sha256_hex_for_file(&temporary_path)?;
        if let Some(expected_sha256) = first_sha256_checksum(&json_string_array(&manifest, "checksums")) {
            if downloaded_sha256 != expected_sha256 {
                fs::remove_file(&temporary_path).ok();
                return Err(format!(
                    "Checksum invalido para {}. Esperado sha256 {}, obtido {}.",
                    display_name, expected_sha256, downloaded_sha256
                ));
            }
        }

        fs::rename(&temporary_path, &destination_path).map_err(|error| {
            format!(
                "Falha ao mover download temporario {} para {}: {}",
                temporary_path.display(),
                destination_path.display(),
                error
            )
        })?;
        downloaded_sha256
    };

    let size_bytes = fs::metadata(&destination_path)
        .map_err(|error| format!("Falha ao ler metadata de {}: {}", destination_path.display(), error))?
        .len();
    fs::remove_file(&manifest_path)
        .map_err(|error| format!("Falha ao remover manifesto {}: {}", manifest_path.display(), error))?;

    Ok(DownloadBuilderLibraryArtifactResult {
        status: if already_available {
            "already-available".to_string()
        } else {
            "downloaded".to_string()
        },
        library_root: library_root.display().to_string(),
        artifact_id,
        display_name,
        source_url,
        downloaded_path: destination_path.display().to_string(),
        removed_manifest_path: manifest_path.display().to_string(),
        size_bytes,
        sha256,
    })
}

fn download_catalog_record_with_root(
    library_root: &Path,
    catalog_model: CatalogModelRecord,
) -> Result<DownloadBuilderLibraryArtifactResult, String> {
    if catalog_model.download_locations.is_empty() {
        return Err(format!(
            "O modelo {} ainda nao possui URL direta resolvida para download.",
            catalog_model.display_name
        ));
    }

    let manifests_dir = library_root.join("manifests").join("model-downloads");
    fs::create_dir_all(&manifests_dir)
        .map_err(|error| format!("Falha ao criar diretorio {}: {}", manifests_dir.display(), error))?;
    let manifest_path = manifests_dir.join(format!("{}.json", catalog_model.model_id));
    let manifest_contents = manifest_json(&json!({
        "schema_version": "1.0",
        "kind": "model-download-request",
        "generated_at": now_utc_string(),
        "model_id": catalog_model.model_id,
        "display_name": catalog_model.display_name,
        "disk_size_gb": catalog_model.disk_size_gb,
        "recommended_ram_gb": catalog_model.ram_recommended_gb,
        "homepage_or_source": catalog_model.homepage_or_source,
        "download_locations": catalog_model.download_locations,
        "checksums": catalog_model.checksums,
        "local_file_name": catalog_model.local_file_name,
        "relative_target_path": format!(
            "models/{}",
            catalog_model
                .local_file_name
                .clone()
                .unwrap_or_else(|| format!("{}.gguf", catalog_model.model_id))
        ),
        "availability_status": "requires-download",
        "availability_label": "Precisa baixar",
        "provider_or_origin": catalog_model.provider_or_origin,
        "variant_id": catalog_model.variant_id,
        "quantization": catalog_model.quantization,
        "file_format": catalog_model.file_format
    }));
    fs::write(&manifest_path, manifest_contents)
        .map_err(|error| format!("Falha ao escrever manifesto {}: {}", manifest_path.display(), error))?;

    download_builder_library_artifact_with_root(
        library_root,
        DownloadBuilderLibraryArtifactRequest {
            artifact_id: catalog_model.model_id,
        },
    )
}

fn download_catalog_model_with_root(
    library_root: &Path,
    request: DownloadCatalogModelRequest,
) -> Result<DownloadBuilderLibraryArtifactResult, String> {
    let snapshot = load_model_catalog_snapshot()?;
    let catalog_model = snapshot
        .models
        .iter()
        .find(|model| model.model_id == request.model_id)
        .ok_or_else(|| format!("Modelo {} nao foi encontrado no catalogo vivo atual.", request.model_id))?;
    download_catalog_record_with_root(library_root, catalog_model.clone())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<usize, String> {
    if !source.exists() {
        return Ok(0);
    }

    fs::create_dir_all(destination)
        .map_err(|error| format!("Falha ao criar diretorio {}: {}", destination.display(), error))?;

    let mut copied_files = 0usize;
    let mut stack = vec![source.to_path_buf()];
    while let Some(current) = stack.pop() {
        let entries = fs::read_dir(&current)
            .map_err(|error| format!("Falha ao ler diretorio {}: {}", current.display(), error))?;

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let Some(name) = path.file_name().map(|value| value.to_string_lossy().to_string()) else {
                continue;
            };
            if should_ignore_path_name(&name) {
                continue;
            }
            let relative = path
                .strip_prefix(source)
                .map_err(|error| format!("Falha ao calcular caminho relativo: {}", error))?;
            let target_path = destination.join(relative);

            if path.is_dir() {
                fs::create_dir_all(&target_path)
                    .map_err(|error| format!("Falha ao criar diretorio {}: {}", target_path.display(), error))?;
                stack.push(path);
            } else if path.is_file() {
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|error| format!("Falha ao criar diretorio {}: {}", parent.display(), error))?;
                }
                copy_file_contents(&path, &target_path)?;
                copied_files += 1;
            }
        }
    }

    Ok(copied_files)
}

fn missing_required_relative_paths(root: &Path, required_paths: &[&str]) -> Vec<String> {
    required_paths
        .iter()
        .filter(|relative| !root.join(relative).exists())
        .map(|relative| (*relative).to_string())
        .collect()
}

fn manifest_matches_validation_expectation(file_name: &str, expected: &str, actual: &str) -> bool {
    if actual == expected {
        return true;
    }

    let Ok(expected_json) = serde_json::from_str::<serde_json::Value>(expected) else {
        return false;
    };
    let Ok(actual_json) = serde_json::from_str::<serde_json::Value>(actual) else {
        return false;
    };

    match file_name {
        // These manifests are intentionally dynamic across planning/validation runs.
        "build-manifest.json" => {
            actual_json.get("schema_version") == expected_json.get("schema_version")
                && actual_json.get("build_id") == expected_json.get("build_id")
                && actual_json.get("build_mode") == expected_json.get("build_mode")
                && actual_json.get("languages") == expected_json.get("languages")
                && actual_json.get("profile_ref") == expected_json.get("profile_ref")
                && actual_json.get("execution_policy_ref") == expected_json.get("execution_policy_ref")
                && actual_json.get("installed_models_ref") == expected_json.get("installed_models_ref")
                && actual_json.get("installed_packs_ref") == expected_json.get("installed_packs_ref")
                && actual_json.get("runtime_config_ref") == expected_json.get("runtime_config_ref")
                && actual_json.get("provider_config_ref") == expected_json.get("provider_config_ref")
                && actual_json.get("validation_report_ref") == expected_json.get("validation_report_ref")
        }
        "validation-report.json" => {
            actual_json.get("schema_version") == expected_json.get("schema_version")
                && actual_json.get("builder_version") == expected_json.get("builder_version")
                && actual_json.get("validation_status") == expected_json.get("validation_status")
        }
        "usbrain-profile.json" => {
            actual_json.get("schema_version") == expected_json.get("schema_version")
                && actual_json.get("profile_id") == expected_json.get("profile_id")
                && actual_json.get("primary_language") == expected_json.get("primary_language")
                && actual_json.get("supported_languages") == expected_json.get("supported_languages")
                && actual_json.get("mode") == expected_json.get("mode")
                && actual_json.get("privacy_mode") == expected_json.get("privacy_mode")
                && actual_json.get("gpu_preference") == expected_json.get("gpu_preference")
                && actual_json.get("personality_profile") == expected_json.get("personality_profile")
                && actual_json.get("agent_profile") == expected_json.get("agent_profile")
                && actual_json.get("default_pack_collection") == expected_json.get("default_pack_collection")
        }
        _ => false,
    }
}

fn count_files_clean(path: &Path) -> usize {
    if !path.exists() {
        return 0;
    }

    let mut file_count = 0usize;
    let mut stack = vec![path.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(entries) = fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            let entry_path = entry.path();
            let Some(name) = entry_path.file_name().map(|value| value.to_string_lossy().to_string()) else {
                continue;
            };
            if should_ignore_path_name(&name) {
                continue;
            }
            if entry_path.is_dir() {
                stack.push(entry_path);
            } else if entry_path.is_file() {
                file_count += 1;
            }
        }
    }

    file_count
}

fn appledouble_compileall_error_only(error: &str) -> bool {
    let trimmed = error.trim();
    !trimmed.is_empty()
        && trimmed.lines().all(|line| {
            let candidate = line.trim();
            candidate.is_empty()
                || candidate.starts_with("*** Error compiling '")
                    && candidate.contains("'")
                    && (candidate.contains("/._") || candidate.contains("'._"))
                || candidate.contains("ValueError: source code string cannot contain null bytes")
        })
}

fn target_meaningful_entries(target_root: &Path) -> Vec<PathBuf> {
    fs::read_dir(target_root)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .map(|value| !should_ignore_path_name(&value.to_string_lossy()))
                .unwrap_or(false)
        })
        .collect()
}

fn is_builder_managed_target(target_root: &Path) -> bool {
    let marker_path = target_root.join(BUILDER_MARKER_FILE);
    let legacy_manifest_paths = [
        target_root.join("manifests").join("build-manifest.json"),
        target_root.join("manifests").join("runtime-config.json"),
        target_root.join("manifests").join("installed-models.json"),
    ];
    let runtime_signature_paths = [
        target_root.join("main.py"),
        target_root.join("agent.py"),
        target_root.join("start.bat"),
        target_root.join("start.sh"),
        target_root.join("models"),
    ];
    let marker_exists = marker_path.exists();
    let legacy_exists = legacy_manifest_paths.iter().any(|path| path.exists());
    let runtime_signature_count = runtime_signature_paths.iter().filter(|path| path.exists()).count();
    let runtime_signature_exists = runtime_signature_count >= 4;

    if std::env::var_os("USBRAIN_FACTORY_QA_TARGET").is_some() {
        eprintln!(
            "FACTORY_QA_MANAGED_CHECK target={} markerExists={} buildManifest={} runtimeConfig={} installedModels={} runtimeSignatureCount={}",
            target_root.display(),
            marker_exists,
            legacy_manifest_paths[0].exists(),
            legacy_manifest_paths[1].exists(),
            legacy_manifest_paths[2].exists(),
            runtime_signature_count
        );
    }

    marker_exists || legacy_exists || runtime_signature_exists
}

fn wipe_target_meaningful_entries(target_root: &Path) -> Result<(), String> {
    for path in target_meaningful_entries(target_root) {
        if !path.exists() {
            continue;
        }
        if path.is_dir() {
            fs::remove_dir_all(&path)
                .map_err(|error| format!("Falha ao apagar diretorio {}: {}", path.display(), error))?;
        } else {
            fs::remove_file(&path)
                .map_err(|error| format!("Falha ao apagar arquivo {}: {}", path.display(), error))?;
        }
    }
    Ok(())
}

fn remove_path_recursively(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("Falha ao ler metadata de {}: {}", path.display(), error))?;
    if metadata.file_type().is_dir() {
        wipe_directory_contents_preserving_root(path)?;
        fs::remove_dir(path).or_else(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(error)
            }
        })
        .map_err(|error| format!("Falha ao apagar diretorio {}: {}", path.display(), error))?;
    } else {
        fs::remove_file(path).or_else(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(error)
            }
        })
        .map_err(|error| format!("Falha ao apagar arquivo {}: {}", path.display(), error))?;
    }

    Ok(())
}

fn wipe_directory_contents_preserving_root(root: &Path) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }
    if !root.is_dir() {
        return Err(format!(
            "Falha ao limpar diretorio {}: o caminho nao e um diretorio.",
            root.display()
        ));
    }

    let entries: Vec<PathBuf> = fs::read_dir(root)
        .map_err(|error| format!("Falha ao ler diretorio {}: {}", root.display(), error))?
        .map(|entry| entry.map(|value| value.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Falha ao enumerar diretorio {}: {}", root.display(), error))?;
    for entry_path in entries {
        let Some(name) = entry_path.file_name().map(|value| value.to_string_lossy().to_string()) else {
            continue;
        };
        if should_ignore_path_name(&name) {
            let _ = remove_path_recursively(&entry_path);
            continue;
        }
        remove_path_recursively(&entry_path)?;
    }

    let remaining_entries = fs::read_dir(root)
        .map_err(|error| format!("Falha ao reler diretorio {}: {}", root.display(), error))?;
    let remaining_meaningful: Vec<String> = remaining_entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter_map(|path| {
            path.file_name()
                .map(|value| value.to_string_lossy().to_string())
                .filter(|name| !should_ignore_path_name(name))
                .map(|_| path.display().to_string())
        })
        .collect();
    if !remaining_meaningful.is_empty() {
        return Err(format!(
            "Falha ao limpar runtime Python anterior {}: ainda restam entradas: {}",
            root.display(),
            remaining_meaningful.join(", ")
        ));
    }

    Ok(())
}

fn ensure_target_is_ready_for_assembly(target_root: &Path, wipe_target_if_needed: bool) -> Result<(), String> {
    if !target_root.exists() {
        return Err(format!("Destino {} nao existe.", target_root.display()));
    }
    if !target_root.is_dir() {
        return Err(format!("Destino {} nao e um diretorio valido.", target_root.display()));
    }

    let entries = target_meaningful_entries(target_root);
    if entries.is_empty() {
        return Ok(());
    }

    if is_builder_managed_target(target_root) {
        return Ok(());
    }

    if wipe_target_if_needed {
        return wipe_target_meaningful_entries(target_root);
    }

    Err(format!(
        "Destino {} ja contem arquivos. Confirme a limpeza completa do pendrive para o Builder apagar os dados atuais e criar o USBRAIN.",
        target_root.display()
    ))
}

fn assemble_build_plan_with_sources_and_options(
    build_plan: &BuildPlanResponse,
    target_root: &Path,
    app_root: &Path,
    cache_root: &Path,
    wipe_target_if_needed: bool,
    app_handle: Option<&AppHandle>,
) -> Result<BuildAssemblyResult, String> {
    // #region debug-point B:assembly-source-root
    debug_report_factory_session(
        "B",
        "commands.rs:assemble_build_plan_with_sources_and_options",
        "Assembly started with explicit roots",
        json!({
            "target_root": target_root.display().to_string(),
            "app_root": app_root.display().to_string(),
            "cache_root": cache_root.display().to_string(),
            "webui_server_exists": app_root.join("webui").join("server.py").exists(),
            "webui_template_exists": app_root.join("webui").join("templates").join("index.html").exists(),
            "bundled_app_root": bundled_app_root().map(|path| path.display().to_string()),
            "source_app_root": source_app_root().display().to_string(),
        }),
    );
    // #endregion
    emit_assembly_progress(
        app_handle,
        "prepare-target",
        6,
        format!("Preparando o destino {} para criar o USBRAIN.", target_root.display()),
    );
    ensure_target_is_ready_for_assembly(target_root, wipe_target_if_needed)?;
    emit_assembly_progress(
        app_handle,
        "copy-runtime-files",
        14,
        "Copiando os arquivos principais do runtime para o pendrive.",
    );
    let mut exported_files = Vec::new();
    let mut assembled_directories = Vec::new();
    let mut warnings = build_plan.summary.warnings.clone();

    for file_name in ROOT_RUNTIME_FILES {
        let source = app_root.join(file_name);
        if source.exists() {
            exported_files.push(copy_file_with_summary(&source, &target_root.join(file_name), target_root)?);
        }
    }

    let optional_cache_files = [
        (cache_root.join("llamafile.exe"), target_root.join("llamafile.exe")),
        (cache_root.join("python_runtime.zip"), target_root.join("python_runtime.zip")),
    ];
    emit_assembly_progress(
        app_handle,
        "copy-optional-assets",
        26,
        "Copiando assets auxiliares e runtime complementar do USBRAIN.",
    );
    for (source, destination) in optional_cache_files {
        if source.exists() {
            exported_files.push(copy_file_with_summary(&source, &destination, target_root)?);
        } else {
            warnings.push(format!("Asset opcional ausente para montagem: {}", source.display()));
        }
    }

    let directory_sources = [
        ("tools", app_root.join("tools")),
        ("skills", app_root.join("skills")),
        ("rag", app_root.join("rag")),
        ("webui", app_root.join("webui")),
        ("knowledge", app_root.join("knowledge")),
        ("python", cache_root.join("python")),
    ];
    emit_assembly_progress(
        app_handle,
        "copy-directories",
        42,
        "Copiando ferramentas, skills, webui e dependencias do USBRAIN.",
    );
    for (relative, source) in directory_sources {
        if source.exists() {
            let destination = target_root.join(relative);
            if relative == "python" && destination.exists() {
                wipe_directory_contents_preserving_root(&destination)?;
            }
            let file_count = copy_tree(&source, &destination)?;
            assembled_directories.push(AssembledDirectorySummary {
                relative_path: relative.to_string(),
                file_count,
            });
        } else if relative != "knowledge" {
            warnings.push(format!("Diretorio esperado nao encontrado para montagem: {}", source.display()));
        }
    }
    // #region debug-point B:webui-copy-result
    debug_report_factory_session(
        "B",
        "commands.rs:assemble_build_plan_with_sources_and_options",
        "Assembly copied runtime directories",
        json!({
            "target_webui_server_exists": target_root.join("webui").join("server.py").exists(),
            "target_webui_template_exists": target_root.join("webui").join("templates").join("index.html").exists(),
            "target_webui_flat_index_exists": target_root.join("webui").join("index.html").exists(),
        }),
    );
    // #endregion

    let bge_source = cache_root.join("bge-m3-model");
    emit_assembly_progress(
        app_handle,
        "copy-rag-assets",
        58,
        "Gravando o BGE-M3 e os ativos de RAG no pendrive.",
    );
    if bge_source.exists() {
        let bge_target = target_root.join("rag").join("bge-m3-model");
        let file_count = copy_tree(&bge_source, &bge_target)?;
        assembled_directories.push(AssembledDirectorySummary {
            relative_path: "rag/bge-m3-model".to_string(),
            file_count,
        });
    } else {
        warnings.push(format!("BGE-M3 offline nao encontrado em {}", bge_source.display()));
    }

    let target_models_root = target_root.join("models");
    if target_models_root.exists() {
        fs::remove_dir_all(&target_models_root).map_err(|error| {
            format!(
                "Falha ao limpar diretorio de modelos anterior {}: {}",
                target_models_root.display(),
                error
            )
        })?;
    }
    fs::create_dir_all(&target_models_root)
        .map_err(|error| format!("Falha ao criar diretorio de modelos: {}", error))?;
    emit_assembly_progress(
        app_handle,
        "copy-models",
        74,
        "Gravando os modelos selecionados no pendrive.",
    );
    for model in &build_plan.models {
        let source = PathBuf::from(&model.source_absolute_path);
        if source.exists() {
            exported_files.push(copy_file_with_summary(
                &source,
                &target_root.join(&model.relative_target_path),
                target_root,
            )?);
        } else {
            warnings.push(format!("Modelo planejado ausente no workspace: {}", model.source_absolute_path));
        }
    }

    let manifests_root = target_root.join("manifests");
    fs::create_dir_all(&manifests_root)
        .map_err(|error| format!("Falha ao criar diretorio de manifestos: {}", error))?;
    emit_assembly_progress(
        app_handle,
        "write-manifests",
        86,
        "Gerando os manifestos e contratos do USBRAIN no pendrive.",
    );
    for manifest in &build_plan.manifests {
        let manifest_path = manifests_root.join(&manifest.file_name);
        exported_files.push(write_export_file(&manifest_path, &manifest.contents, target_root)?);
    }

    emit_assembly_progress(
        app_handle,
        "finalize-runtime",
        94,
        "Finalizando a estrutura de runtime, logs e arquivos de suporte.",
    );
    for relative in ["packs", "runtime", "runtime/logs", "runtime/tmp"] {
        let path = target_root.join(relative);
        fs::create_dir_all(&path)
            .map_err(|error| format!("Falha ao criar diretorio {}: {}", path.display(), error))?;
    }

    let marker_path = target_root.join(BUILDER_MARKER_FILE);
    let marker_payload = json!({
        "build_id": build_plan.summary.build_id,
        "target_label": build_plan.summary.target_label,
        "assembled_at": now_utc_string(),
        "builder_version": env!("CARGO_PKG_VERSION"),
        "model_count": build_plan.models.len(),
        "pack_count": build_plan.packs.len()
    });
    exported_files.push(write_export_file(
        &marker_path,
        &manifest_json(&marker_payload),
        target_root,
    )?);
    emit_assembly_progress(
        app_handle,
        "assembly-finished",
        100,
        format!("Montagem concluida em {}.", target_root.display()),
    );

    Ok(BuildAssemblyResult {
        build_id: build_plan.summary.build_id.clone(),
        target_root: target_root.display().to_string(),
        marker_file: marker_path.display().to_string(),
        manifest_directory: manifests_root.display().to_string(),
        selected_models_directory: target_root.join("models").display().to_string(),
        exported_files,
        assembled_directories,
        warnings,
    })
}

#[cfg(test)]
fn assemble_build_plan_with_sources(
    build_plan: &BuildPlanResponse,
    target_root: &Path,
    app_root: &Path,
    cache_root: &Path,
) -> Result<BuildAssemblyResult, String> {
    assemble_build_plan_with_sources_and_options(build_plan, target_root, app_root, cache_root, false, None)
}

fn assemble_build_plan_to_target(
    build_plan: &BuildPlanResponse,
    target_root: &Path,
    wipe_target_if_needed: bool,
    app_handle: Option<&AppHandle>,
) -> Result<BuildAssemblyResult, String> {
    assemble_build_plan_with_sources_and_options(
        build_plan,
        target_root,
        &source_app_root(),
        &cache_dir(),
        wipe_target_if_needed,
        app_handle,
    )
}

fn validate_assembled_target_with_sources(
    build_plan: &BuildPlanResponse,
    target_root: &Path,
    app_root: &Path,
    cache_root: &Path,
) -> BuildValidationResult {
    let mut checks = Vec::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let target_python_root = target_root.join("python");

    let marker_path = target_root.join(BUILDER_MARKER_FILE);
    if marker_path.exists() {
        checks.push(ValidationCheck {
            id: "builder-marker".to_string(),
            status: "passed".to_string(),
            message: format!("Marcador do Builder encontrado em {}.", marker_path.display()),
        });
    } else {
        errors.push(format!("Marcador do Builder ausente: {}", marker_path.display()));
        checks.push(ValidationCheck {
            id: "builder-marker".to_string(),
            status: "failed".to_string(),
            message: "O destino nao contem o marcador de montagem do USBRAIN Builder.".to_string(),
        });
    }

    let mut missing_root_files = Vec::new();
    for file_name in ROOT_RUNTIME_FILES {
        let source = app_root.join(file_name);
        if !source.exists() {
            continue;
        }
        let target_file = target_root.join(file_name);
        if !target_file.exists() {
            missing_root_files.push(file_name.to_string());
        }
    }
    if missing_root_files.is_empty() {
        checks.push(ValidationCheck {
            id: "root-runtime-files".to_string(),
            status: "passed".to_string(),
            message: "Arquivos principais do runtime foram encontrados no destino.".to_string(),
        });
    } else {
        errors.push(format!(
            "Arquivos principais ausentes: {}",
            missing_root_files.join(", ")
        ));
        checks.push(ValidationCheck {
            id: "root-runtime-files".to_string(),
            status: "failed".to_string(),
            message: "Nem todos os arquivos principais do runtime foram copiados.".to_string(),
        });
    }

    let required_directories = [
        "tools",
        "skills",
        "rag",
        "webui",
        "manifests",
        "models",
        "packs",
        "runtime",
        "runtime/logs",
    ];
    let mut missing_directories = Vec::new();
    for relative in required_directories {
        if !target_root.join(relative).exists() {
            missing_directories.push(relative.to_string());
        }
    }
    if missing_directories.is_empty() {
        checks.push(ValidationCheck {
            id: "required-directories".to_string(),
            status: "passed".to_string(),
            message: "Diretorios essenciais do USBRAIN foram encontrados.".to_string(),
        });
    } else {
        errors.push(format!(
            "Diretorios essenciais ausentes: {}",
            missing_directories.join(", ")
        ));
        checks.push(ValidationCheck {
            id: "required-directories".to_string(),
            status: "failed".to_string(),
            message: "A estrutura de diretorios do USBRAIN esta incompleta.".to_string(),
        });
    }

    let source_directory_pairs = [
        ("tools", app_root.join("tools"), target_root.join("tools")),
        ("skills", app_root.join("skills"), target_root.join("skills")),
        ("rag", app_root.join("rag"), target_root.join("rag")),
        ("webui", app_root.join("webui"), target_root.join("webui")),
        ("python", cache_root.join("python"), target_root.join("python")),
    ];
    for (label, source, target) in source_directory_pairs {
        if !source.exists() || !target.exists() {
            continue;
        }
        let source_count = count_files_clean(&source);
        let target_count = count_files_clean(&target);
        let (status, message) = if label == "python" {
            if target_count == 0 {
                errors.push("Runtime Python expandido ausente no destino.".to_string());
                (
                    "failed".to_string(),
                    "Runtime Python expandido nao foi encontrado no destino.".to_string(),
                )
            } else if target_count < source_count {
                warnings.push(format!(
                    "Contagem divergente em {}: origem={} destino={}. O runtime Python expandido existe, mas o cache empacotado contem mais arquivos do que o necessario para o pendrive.",
                    label, source_count, target_count
                ));
                (
                    "warning".to_string(),
                    format!(
                        "{} arquivos em {} (origem={} destino={}). O runtime expandido foi encontrado, com contagem reduzida tolerada.",
                        label, label, source_count, target_count
                    ),
                )
            } else {
                (
                    "passed".to_string(),
                    format!("{} arquivos em {} (origem={} destino={}).", label, label, source_count, target_count),
                )
            }
        } else if target_count < source_count {
            errors.push(format!(
                "Contagem divergente em {}: origem={} destino={}",
                label, source_count, target_count
            ));
            (
                "failed".to_string(),
                format!("{} arquivos em {} (origem={} destino={}).", label, label, source_count, target_count),
            )
        } else {
            (
                "passed".to_string(),
                format!("{} arquivos em {} (origem={} destino={}).", label, label, source_count, target_count),
            )
        };
        checks.push(ValidationCheck {
            id: format!("dir-count-{}", label),
            status,
            message,
        });
    }

    let missing_python_runtime_files =
        missing_required_relative_paths(&target_python_root, WINDOWS_PYTHON_REQUIRED_FILES);
    if missing_python_runtime_files.is_empty() {
        checks.push(ValidationCheck {
            id: "python-runtime-layout".to_string(),
            status: "passed".to_string(),
            message: "Runtime Python Windows validado com os caminhos criticos esperados.".to_string(),
        });
    } else {
        errors.push(format!(
            "Runtime Python Windows incompleto no destino: {}",
            missing_python_runtime_files.join(", ")
        ));
        checks.push(ValidationCheck {
            id: "python-runtime-layout".to_string(),
            status: "failed".to_string(),
            message: "O runtime Python Windows nao contem todos os caminhos criticos esperados.".to_string(),
        });
    }

    let flattened_python_runtime_files = ["site.py", "_ctypes.pyd"]
        .into_iter()
        .filter(|relative| target_python_root.join(relative).exists())
        .collect::<Vec<_>>();
    if !flattened_python_runtime_files.is_empty() {
        errors.push(format!(
            "Runtime Python achatado no destino: {}",
            flattened_python_runtime_files.join(", ")
        ));
        checks.push(ValidationCheck {
            id: "python-runtime-flattened-layout".to_string(),
            status: "failed".to_string(),
            message: "Arquivos criticos do runtime apareceram achatados na raiz de `python/`, indicando montagem invalida.".to_string(),
        });
    }

    let bge_source = cache_root.join("bge-m3-model");
    let bge_target = target_root.join("rag").join("bge-m3-model");
    if bge_source.exists() {
        let source_count = count_files_clean(&bge_source);
        let target_count = count_files_clean(&bge_target);
        let bge_required_files = [
            "pytorch_model.bin",
            "sentencepiece.bpe.model",
            "colbert_linear.pt",
            "sparse_linear.pt",
        ];
        let missing_bge_files = bge_required_files
            .into_iter()
            .filter(|file_name| !bge_target.join(file_name).exists())
            .collect::<Vec<_>>();
        if !missing_bge_files.is_empty() {
            errors.push(format!(
                "BGE-M3 incompleto: arquivos obrigatorios ausentes ({})",
                missing_bge_files.join(", ")
            ));
            checks.push(ValidationCheck {
                id: "bge-m3-assets".to_string(),
                status: "failed".to_string(),
                message: "Nem todos os artefatos essenciais do BGE-M3 foram encontrados.".to_string(),
            });
        } else if target_count < source_count {
            warnings.push(format!(
                "BGE-M3 com contagem divergente: origem={} destino={}. Os arquivos essenciais existem no destino.",
                source_count, target_count
            ));
            checks.push(ValidationCheck {
                id: "bge-m3-assets".to_string(),
                status: "warning".to_string(),
                message: format!(
                    "BGE-M3 validado pelos artefatos essenciais, com contagem reduzida tolerada (origem={} destino={}).",
                    source_count, target_count
                ),
            });
        } else {
            checks.push(ValidationCheck {
                id: "bge-m3-assets".to_string(),
                status: "passed".to_string(),
                message: format!("BGE-M3 validado com {} arquivo(s).", target_count),
            });
        }
    } else {
        warnings.push(format!(
            "Cache do BGE-M3 nao encontrado em {}; a checagem foi ignorada.",
            bge_source.display()
        ));
        checks.push(ValidationCheck {
            id: "bge-m3-assets".to_string(),
            status: "warning".to_string(),
            message: "Cache do BGE-M3 ausente no workspace; checagem ignorada.".to_string(),
        });
    }

    let optional_assets = [
        ("llamafile.exe", cache_root.join("llamafile.exe"), target_root.join("llamafile.exe")),
        ("python_runtime.zip", cache_root.join("python_runtime.zip"), target_root.join("python_runtime.zip")),
    ];
    for (label, source, target) in optional_assets {
        if !source.exists() {
            warnings.push(format!("Asset opcional ausente no cache: {}", source.display()));
            checks.push(ValidationCheck {
                id: format!("asset-{}", label),
                status: "warning".to_string(),
                message: format!("Asset opcional {} ausente no cache; checagem ignorada.", label),
            });
            continue;
        }

        if !target.exists() {
            errors.push(format!("Asset esperado ausente no destino: {}", target.display()));
            checks.push(ValidationCheck {
                id: format!("asset-{}", label),
                status: "failed".to_string(),
                message: format!("O asset {} nao foi encontrado no destino.", label),
            });
            continue;
        }

        let source_size = fs::metadata(&source).ok().map(|metadata| metadata.len()).unwrap_or(0);
        let target_size = fs::metadata(&target).ok().map(|metadata| metadata.len()).unwrap_or(0);
        if source_size != target_size {
            errors.push(format!(
                "Tamanho divergente para {}: origem={} destino={}",
                label, source_size, target_size
            ));
            checks.push(ValidationCheck {
                id: format!("asset-{}", label),
                status: "failed".to_string(),
                message: format!("O asset {} existe, mas com tamanho divergente.", label),
            });
        } else {
            checks.push(ValidationCheck {
                id: format!("asset-{}", label),
                status: "passed".to_string(),
                message: format!("Asset {} validado.", label),
            });
        }
    }

    for model in &build_plan.models {
        let source = PathBuf::from(&model.source_absolute_path);
        let target = target_root.join(&model.relative_target_path);
        if !target.exists() {
            errors.push(format!("Modelo ausente no destino: {}", model.relative_target_path));
            checks.push(ValidationCheck {
                id: format!("model-{}", model.installed_model_id),
                status: "failed".to_string(),
                message: format!("O modelo {} nao foi encontrado no destino.", model.source_file_name),
            });
            continue;
        }

        let source_size = fs::metadata(&source).ok().map(|metadata| metadata.len()).unwrap_or(0);
        let target_size = fs::metadata(&target).ok().map(|metadata| metadata.len()).unwrap_or(0);
        if source.exists() && source_size != target_size {
            errors.push(format!(
                "Modelo divergente {}: origem={} destino={}",
                model.source_file_name, source_size, target_size
            ));
            checks.push(ValidationCheck {
                id: format!("model-{}", model.installed_model_id),
                status: "failed".to_string(),
                message: format!("O modelo {} foi encontrado, mas com tamanho divergente.", model.source_file_name),
            });
        } else {
            checks.push(ValidationCheck {
                id: format!("model-{}", model.installed_model_id),
                status: "passed".to_string(),
                message: format!("Modelo {} validado.", model.source_file_name),
            });
        }
    }

    for manifest in &build_plan.manifests {
        let target_manifest = target_root.join("manifests").join(&manifest.file_name);
        if !target_manifest.exists() {
            errors.push(format!("Manifesto ausente no destino: {}", manifest.file_name));
            checks.push(ValidationCheck {
                id: format!("manifest-{}", manifest.file_name),
                status: "failed".to_string(),
                message: format!("O manifesto {} nao foi encontrado no destino.", manifest.file_name),
            });
            continue;
        }

        let contents = fs::read_to_string(&target_manifest).unwrap_or_default();
        if !manifest_matches_validation_expectation(&manifest.file_name, &manifest.contents, &contents) {
            errors.push(format!("Conteudo divergente no manifesto {}", manifest.file_name));
            checks.push(ValidationCheck {
                id: format!("manifest-{}", manifest.file_name),
                status: "failed".to_string(),
                message: format!("O manifesto {} existe, mas o conteudo diverge do plano atual.", manifest.file_name),
            });
        } else {
            checks.push(ValidationCheck {
                id: format!("manifest-{}", manifest.file_name),
                status: "passed".to_string(),
                message: format!("Manifesto {} validado.", manifest.file_name),
            });
        }
    }

    BuildValidationResult {
        build_id: build_plan.summary.build_id.clone(),
        target_root: target_root.display().to_string(),
        validation_status: if errors.is_empty() {
            if warnings.is_empty() {
                "passed".to_string()
            } else {
                "passed-with-warnings".to_string()
            }
        } else {
            "failed".to_string()
        },
        checks,
        errors,
        warnings,
    }
}

fn validate_assembled_target(
    build_plan: &BuildPlanResponse,
    target_root: &Path,
) -> BuildValidationResult {
    validate_assembled_target_with_sources(build_plan, target_root, &source_app_root(), &cache_dir())
}

fn host_python_command_candidates() -> [(&'static str, &'static [&'static str]); 10] {
    [
        ("/usr/bin/python3", &[]),
        ("/opt/homebrew/bin/python3", &[]),
        ("/Library/Frameworks/Python.framework/Versions/Current/bin/python3", &[]),
        ("python3", &[]),
        ("python", &[]),
        ("py", &["-3"]),
        ("/bin/sh", &["-lc", "exec /usr/bin/python3 \"$@\"", "python3"]),
        ("/bin/sh", &["-lc", "exec /opt/homebrew/bin/python3 \"$@\"", "python3"]),
        (
            "/bin/sh",
            &["-lc", "exec /Library/Frameworks/Python.framework/Versions/Current/bin/python3 \"$@\"", "python3"],
        ),
        ("/bin/sh", &["-lc", "exec python3 \"$@\"", "python3"]),
    ]
}

fn detect_host_python_command() -> Option<(String, Vec<String>)> {
    let candidates = host_python_command_candidates();
    for (program, prefix_args) in candidates {
        let mut args = prefix_args.to_vec();
        args.push("--version");
        if let Ok(output) = Command::new(program).args(&args).output() {
            if output.status.success() {
                return Some((
                    program.to_string(),
                    prefix_args.iter().map(|value| (*value).to_string()).collect(),
                ));
            }
        }
    }

    None
}

fn debug_host_python_detection_attempts() -> Vec<String> {
    let mut attempts = Vec::new();

    for (program, prefix_args) in host_python_command_candidates() {
        let mut args = prefix_args.to_vec();
        args.push("--version");
        match Command::new(program).args(&args).output() {
            Ok(output) if output.status.success() => attempts.push(format!(
                "{} {} => success",
                program,
                args.join(" ")
            )),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                attempts.push(format!(
                    "{} {} => status={:?} stdout='{}' stderr='{}'",
                    program,
                    args.join(" "),
                    output.status.code(),
                    stdout,
                    stderr
                ));
            }
            Err(error) => attempts.push(format!("{} {} => error={}", program, args.join(" "), error)),
        }
    }

    attempts
}

fn bundled_macos_python_roots() -> Vec<PathBuf> {
    let primary_root = cache_dir().join("python-macos");
    let workspace_fallback = workspace_root()
        .join("cache")
        .join("macos-python-standalone")
        .join("extracted")
        .join("python");

    if workspace_fallback == primary_root {
        vec![primary_root]
    } else {
        vec![primary_root, workspace_fallback]
    }
}

fn bundled_macos_python_candidate_paths_in_roots(python_roots: &[PathBuf]) -> Vec<PathBuf> {
    python_roots
        .into_iter()
        .flat_map(|python_root| {
            [
                python_root.join("bin").join("python3.10"),
                python_root.join("bin").join("python3"),
            ]
        })
        .collect()
}

fn bundled_python_version_output(path: &Path, python_roots: &[PathBuf]) -> std::io::Result<std::process::Output> {
    Command::new(path)
        .envs(python_command_env_in_roots(&path.display().to_string(), python_roots))
        .arg("--version")
        .output()
}

fn tolerate_runtime_binary_preflight_permission_denied(error: &std::io::Error) -> bool {
    error.raw_os_error() == Some(1) && bundled_resources_dir().is_some()
}

fn detect_bundled_macos_python_command_in_roots(python_roots: &[PathBuf]) -> Option<(String, Vec<String>)> {
    for path in bundled_macos_python_candidate_paths_in_roots(python_roots) {
        if !path.is_file() {
            continue;
        }

        if let Ok(output) = bundled_python_version_output(&path, python_roots) {
            if output.status.success() {
                return Some((path.display().to_string(), Vec::new()));
            }
        }
    }

    None
}

fn detect_bundled_macos_python_command() -> Option<(String, Vec<String>)> {
    detect_bundled_macos_python_command_in_roots(&bundled_macos_python_roots())
}

fn detect_probe_python_command() -> Option<(String, Vec<String>)> {
    detect_bundled_macos_python_command().or_else(detect_host_python_command)
}

fn debug_bundled_macos_python_detection_attempts_in_roots(python_roots: &[PathBuf]) -> Vec<String> {
    let mut attempts = Vec::new();

    for path in bundled_macos_python_candidate_paths_in_roots(python_roots) {
        if !path.exists() {
            attempts.push(format!("{} --version => missing", path.display()));
            continue;
        }

        match bundled_python_version_output(&path, python_roots) {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let version = if !stdout.is_empty() {
                    stdout
                } else if !stderr.is_empty() {
                    stderr
                } else {
                    "success".to_string()
                };
                attempts.push(format!("{} --version => success {}", path.display(), version));
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                attempts.push(format!(
                    "{} --version => status={:?} stdout='{}' stderr='{}'",
                    path.display(),
                    output.status.code(),
                    stdout,
                    stderr
                ));
            }
            Err(error) => attempts.push(format!("{} --version => error={}", path.display(), error)),
        }
    }

    attempts
}

fn debug_bundled_macos_python_detection_attempts() -> Vec<String> {
    debug_bundled_macos_python_detection_attempts_in_roots(&bundled_macos_python_roots())
}

fn debug_probe_python_detection_attempts() -> Vec<String> {
    let mut attempts = debug_bundled_macos_python_detection_attempts();
    attempts.extend(debug_host_python_detection_attempts());
    attempts
}

fn python_command_source_in_roots(program: &str, python_roots: &[PathBuf]) -> &'static str {
    let path = Path::new(program);
    if path.is_absolute() && python_roots.iter().any(|python_root| path.starts_with(python_root)) {
        "bundled-macos"
    } else {
        "host"
    }
}

fn python_command_source(program: &str) -> &'static str {
    python_command_source_in_roots(program, &bundled_macos_python_roots())
}

fn bundled_macos_python_root_for_program_in_roots(program: &str, python_roots: &[PathBuf]) -> Option<PathBuf> {
    let path = Path::new(program);
    path.is_absolute()
        .then_some(())
        .and_then(|_| python_roots.iter().find(|python_root| path.starts_with(python_root)).cloned())
}

fn python_command_env_in_roots(program: &str, python_roots: &[PathBuf]) -> Vec<(String, String)> {
    bundled_macos_python_root_for_program_in_roots(program, python_roots)
        .map(|python_root| vec![("PYTHONHOME".to_string(), python_root.display().to_string())])
        .unwrap_or_default()
}

fn python_command_env(program: &str) -> Vec<(String, String)> {
    python_command_env_in_roots(program, &bundled_macos_python_roots())
}

fn run_python_smoke_command(
    program: &str,
    prefix_args: &[String],
    extra_args: &[&str],
    cwd: &Path,
) -> Result<String, String> {
    let python_env = python_command_env(program);
    let mut command = Command::new(program);
    command.args(prefix_args);
    command.args(extra_args);
    command.current_dir(cwd);
    command.envs(python_env);
    let output = command
        .output()
        .map_err(|error| format!("Falha ao executar {}: {}", program, error))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if output.status.success() {
        let combined = if !stdout.is_empty() {
            stdout
        } else if !stderr.is_empty() {
            stderr
        } else {
            "comando executado com sucesso".to_string()
        };
        Ok(combined)
    } else {
        Err(if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            format!("Comando finalizou com status {:?}", output.status.code())
        })
    }
}

fn http_get_text_via_python(
    program: &str,
    prefix_args: &[String],
    url: &str,
    timeout: Duration,
    cwd: &Path,
) -> Result<String, String> {
    let timeout_secs = format!("{:.1}", timeout.as_secs_f64());
    let script = "import sys, urllib.request; url=sys.argv[1]; timeout=float(sys.argv[2]); req=urllib.request.Request(url, method='GET'); resp=urllib.request.urlopen(req, timeout=timeout); sys.stdout.write(resp.read().decode('utf-8', 'replace'))";
    let args = ["-c", script, url, timeout_secs.as_str()];
    run_python_smoke_command(program, prefix_args, &args, cwd)
}

fn http_post_json_text_via_python(
    program: &str,
    prefix_args: &[String],
    url: &str,
    body: &str,
    timeout: Duration,
    cwd: &Path,
) -> Result<String, String> {
    let timeout_secs = format!("{:.1}", timeout.as_secs_f64());
    let script = "import sys, urllib.request; url=sys.argv[1]; body=sys.argv[2].encode('utf-8'); timeout=float(sys.argv[3]); req=urllib.request.Request(url, data=body, headers={'Content-Type': 'application/json'}, method='POST'); resp=urllib.request.urlopen(req, timeout=timeout); sys.stdout.write(resp.read().decode('utf-8', 'replace'))";
    let args = ["-c", script, url, body, timeout_secs.as_str()];
    run_python_smoke_command(program, prefix_args, &args, cwd)
}

fn smoke_command_label(program: &str, prefix_args: &[String], extra_args: &[&str]) -> String {
    prefix_args
        .iter()
        .map(String::as_str)
        .chain(extra_args.iter().copied())
        .fold(program.to_string(), |mut acc, value| {
            acc.push(' ');
            acc.push_str(value);
            acc
        })
}

fn clip_smoke_message(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() <= 220 {
        trimmed.to_string()
    } else {
        format!("{}...", &trimmed[..220])
    }
}

fn run_assembled_smoke_test_with_sources(
    build_plan: &BuildPlanResponse,
    target_root: &Path,
    app_root: &Path,
    cache_root: &Path,
) -> BuildSmokeTestResult {
    let validation = validate_assembled_target_with_sources(build_plan, target_root, app_root, cache_root);
    let mut checks = Vec::new();
    let mut errors = validation.errors.clone();
    let mut warnings = validation.warnings.clone();

    checks.push(SmokeTestCheck {
        id: "validation-gate".to_string(),
        status: if validation.validation_status == "failed" {
            "failed".to_string()
        } else {
            "passed".to_string()
        },
        message: format!(
            "Validacao estrutural anterior retornou {}.",
            validation.validation_status
        ),
        command: None,
    });

    let host_python = detect_probe_python_command();
    let host_python_command = host_python
        .as_ref()
        .map(|(program, prefix_args)| smoke_command_label(program, prefix_args, &[]))
        .unwrap_or_else(|| "indisponivel".to_string());
    let python_source = host_python
        .as_ref()
        .map(|(program, _)| python_command_source(program))
        .unwrap_or("unavailable");
    let python_env = host_python
        .as_ref()
        .map(|(program, _)| python_command_env(program))
        .unwrap_or_default();
    // #region debug-point A:host-python
    debug_report_full_probe(
        "A",
        "commands.rs:run_full_local_llm_probe_with_sources:host-python",
        "host python detection finished",
        json!({
            "hostPythonCommand": host_python_command,
            "found": host_python.is_some(),
            "pythonCommandSource": python_source,
            "pythonCommandEnv": python_env
        }),
    );
    // #endregion

    if !target_root.join("main.py").exists() {
        errors.push(format!("Arquivo principal ausente para smoke test: {}", target_root.join("main.py").display()));
        checks.push(SmokeTestCheck {
            id: "main-entrypoint".to_string(),
            status: "failed".to_string(),
            message: "O arquivo main.py nao foi encontrado no destino montado.".to_string(),
            command: None,
        });
    }

    if let Some((program, prefix_args)) = host_python {
        let compile_args = ["-m", "compileall", "-q", "main.py", "agent.py", "rag", "webui", "tools", "skills"];
        let compile_label = smoke_command_label(&program, &prefix_args, &compile_args);
        match run_python_smoke_command(&program, &prefix_args, &compile_args, target_root) {
            Ok(output) => checks.push(SmokeTestCheck {
                id: "python-compileall".to_string(),
                status: "passed".to_string(),
                message: format!("Compilacao Python valida no destino: {}.", clip_smoke_message(&output)),
                command: Some(compile_label),
            }),
            Err(error) => {
                if appledouble_compileall_error_only(&error) {
                    warnings.push(
                        "Compileall encontrou apenas arquivos auxiliares AppleDouble do macOS; o resultado foi tolerado."
                            .to_string(),
                    );
                    checks.push(SmokeTestCheck {
                        id: "python-compileall".to_string(),
                        status: "warning".to_string(),
                        message: "Compileall encontrou apenas arquivos AppleDouble ignoraveis do macOS.".to_string(),
                        command: Some(compile_label),
                    });
                } else {
                    errors.push(format!("Falha no compileall do runtime montado: {}", error));
                    checks.push(SmokeTestCheck {
                        id: "python-compileall".to_string(),
                        status: "failed".to_string(),
                        message: clip_smoke_message(&error),
                        command: Some(compile_label),
                    });
                }
            }
        }

        let help_args = ["main.py", "--help"];
        let help_label = smoke_command_label(&program, &prefix_args, &help_args);
        match run_python_smoke_command(&program, &prefix_args, &help_args, target_root) {
            Ok(output) => checks.push(SmokeTestCheck {
                id: "main-help".to_string(),
                status: "passed".to_string(),
                message: format!("Entrypoint respondeu ao help: {}.", clip_smoke_message(&output)),
                command: Some(help_label),
            }),
            Err(error) => {
                errors.push(format!("Falha ao executar main.py --help no destino: {}", error));
                checks.push(SmokeTestCheck {
                    id: "main-help".to_string(),
                    status: "failed".to_string(),
                    message: clip_smoke_message(&error),
                    command: Some(help_label),
                });
            }
        }

        let manifest_probe_script = "import json, pathlib; base=pathlib.Path('manifests'); names=['build-manifest.json','runtime-config.json','installed-models.json']; [json.loads((base/name).read_text(encoding='utf-8')) for name in names]; print('manifest probe ok')";
        let manifest_args = ["-c", manifest_probe_script];
        let manifest_label = smoke_command_label(&program, &prefix_args, &manifest_args);
        match run_python_smoke_command(&program, &prefix_args, &manifest_args, target_root) {
            Ok(output) => checks.push(SmokeTestCheck {
                id: "manifest-probe".to_string(),
                status: "passed".to_string(),
                message: format!("Manifestos essenciais foram lidos com sucesso: {}.", clip_smoke_message(&output)),
                command: Some(manifest_label),
            }),
            Err(error) => {
                errors.push(format!("Falha ao ler manifestos do destino: {}", error));
                checks.push(SmokeTestCheck {
                    id: "manifest-probe".to_string(),
                    status: "failed".to_string(),
                    message: clip_smoke_message(&error),
                    command: Some(manifest_label),
                });
            }
        }

        let start_bat = target_root.join("start.bat");
        if start_bat.exists() {
            let start_contents = fs::read_to_string(&start_bat).unwrap_or_default().to_ascii_lowercase();
            let status = if start_contents.contains("main.py") {
                "passed"
            } else {
                warnings.push("start.bat foi encontrado, mas nao menciona main.py explicitamente.".to_string());
                "warning"
            };
            checks.push(SmokeTestCheck {
                id: "start-bat".to_string(),
                status: status.to_string(),
                message: if status == "passed" {
                    "Script start.bat referencia o entrypoint principal.".to_string()
                } else {
                    "Script start.bat existe, mas a referencia ao entrypoint principal nao foi confirmada.".to_string()
                },
                command: None,
            });
        } else {
            warnings.push("start.bat ausente no destino montado.".to_string());
            checks.push(SmokeTestCheck {
                id: "start-bat".to_string(),
                status: "warning".to_string(),
                message: "Script start.bat nao foi encontrado no destino.".to_string(),
                command: None,
            });
        }
    } else {
        warnings.push("Nenhum Python hospedeiro foi encontrado para executar o smoke test operacional.".to_string());
        checks.push(SmokeTestCheck {
            id: "host-python".to_string(),
            status: "warning".to_string(),
            message: "Nenhum Python hospedeiro disponivel; checks executaveis foram ignorados.".to_string(),
            command: None,
        });
    }

    BuildSmokeTestResult {
        build_id: build_plan.summary.build_id.clone(),
        target_root: target_root.display().to_string(),
        smoke_status: if errors.is_empty() {
            if warnings.is_empty() {
                "passed".to_string()
            } else {
                "passed-with-warnings".to_string()
            }
        } else {
            "failed".to_string()
        },
        validation_status: validation.validation_status,
        host_python_command,
        checks,
        errors,
        warnings,
    }
}

fn execute_assembled_smoke_test(
    build_plan: &BuildPlanResponse,
    target_root: &Path,
) -> BuildSmokeTestResult {
    run_assembled_smoke_test_with_sources(build_plan, target_root, &source_app_root(), &cache_dir())
}

fn reserve_local_port() -> Result<u16, String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|error| format!("Falha ao reservar porta local: {}", error))?;
    let port = listener
        .local_addr()
        .map_err(|error| format!("Falha ao ler porta reservada: {}", error))?
        .port();
    drop(listener);
    Ok(port)
}

#[cfg(test)]
fn http_get_text(host: &str, port: u16, path: &str, timeout: Duration) -> Result<String, String> {
    let address: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|error| format!("Endereco HTTP invalido: {}", error))?;
    let mut stream = TcpStream::connect_timeout(&address, timeout)
        .map_err(|error| format!("Falha ao conectar em {}:{}: {}", host, port, error))?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|error| format!("Falha ao configurar timeout de leitura: {}", error))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|error| format!("Falha ao configurar timeout de escrita: {}", error))?;

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\n\r\n",
        path, host, port
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("Falha ao enviar requisicao HTTP: {}", error))?;
    let _ = stream.shutdown(std::net::Shutdown::Write);

    let response = read_http_response(&mut stream, "GET")?;
    let mut sections = response.split("\r\n\r\n");
    let headers = sections.next().unwrap_or_default();
    let body = sections.collect::<Vec<_>>().join("\r\n\r\n");
    let status_line = headers.lines().next().unwrap_or_default();
    if !status_line.contains(" 200 ") {
        return Err(format!(
            "Resposta HTTP inesperada de {}:{}{}: {}",
            host, port, path, status_line
        ));
    }
    Ok(body)
}

#[cfg(test)]
fn read_http_response(stream: &mut TcpStream, label: &str) -> Result<String, String> {
    let mut response = Vec::new();
    let mut chunk = [0u8; 8192];
    let mut expected_total_len = None;

    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read) => {
                response.extend_from_slice(&chunk[..read]);
                if expected_total_len.is_none() {
                    if let Some(header_end) = response.windows(4).position(|window| window == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&response[..header_end + 4]);
                        if let Some(content_length) = headers.lines().find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            if name.trim().eq_ignore_ascii_case("content-length") {
                                value.trim().parse::<usize>().ok()
                            } else {
                                None
                            }
                        }) {
                            expected_total_len = Some(header_end + 4 + content_length);
                        }
                    }
                }
                if let Some(expected_total_len) = expected_total_len {
                    if response.len() >= expected_total_len {
                        break;
                    }
                }
            }
            Err(error) if matches!(error.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut) => {
                if response.is_empty() {
                    return Err(format!("Falha ao ler resposta HTTP {}: {}", label, error));
                }
                break;
            }
            Err(error) => return Err(format!("Falha ao ler resposta HTTP {}: {}", label, error)),
        }
    }

    Ok(String::from_utf8_lossy(&response).to_string())
}

fn read_log_excerpt(path: &Path) -> String {
    let contents = fs::read_to_string(path).unwrap_or_default();
    clip_smoke_message(&contents)
}

fn terminate_probe_process(child: &mut Child) {
    if child.try_wait().ok().flatten().is_some() {
        return;
    }

    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-TERM", &child.id().to_string()])
            .status();
    }

    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &child.id().to_string(), "/T"])
            .status();
    }

    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        if child.try_wait().ok().flatten().is_some() {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }

    let _ = child.kill();
    let _ = child.wait();
}

fn run_runtime_probe_with_sources(
    build_plan: &BuildPlanResponse,
    target_root: &Path,
    app_root: &Path,
    cache_root: &Path,
) -> RuntimeProbeResult {
    let mut checks = Vec::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let started_at = Instant::now();

    let host_python = detect_probe_python_command();
    let host_python_command = host_python
        .as_ref()
        .map(|(program, prefix_args)| smoke_command_label(program, prefix_args, &[]))
        .unwrap_or_else(|| "indisponivel".to_string());

    let web_port = reserve_local_port().unwrap_or(39800);
    let bge_port = reserve_local_port().unwrap_or(39801);
    let llm_port = reserve_local_port().unwrap_or(39802);
    let endpoint_url = format!("http://127.0.0.1:{}/api/stats", web_port);
    let probe_tmp_dir = writable_cache_root()
        .join("probe-runtime")
        .join(&build_plan.summary.build_id);
    let _ = fs::create_dir_all(&probe_tmp_dir);
    let stdout_log_path = probe_tmp_dir.join("runtime-probe.stdout.log");
    let stderr_log_path = probe_tmp_dir.join("runtime-probe.stderr.log");
    let probe_db_path = probe_tmp_dir.join("runtime-probe.db");

    let validation = validate_assembled_target_with_sources(build_plan, target_root, app_root, cache_root);
    checks.push(RuntimeProbeCheck {
        id: "validation-gate".to_string(),
        status: if validation.validation_status == "failed" {
            "failed".to_string()
        } else {
            "passed".to_string()
        },
        message: format!("Validacao estrutural retornou {}.", validation.validation_status),
        command: None,
    });
    if validation.validation_status == "failed" {
        errors.extend(validation.errors);
    } else {
        warnings.extend(validation.warnings);
    }

    let Some((program, prefix_args)) = host_python else {
        errors.push("Nenhum Python disponivel foi encontrado para executar o runtime probe.".to_string());
        return RuntimeProbeResult {
            build_id: build_plan.summary.build_id.clone(),
            target_root: target_root.display().to_string(),
            probe_status: "failed".to_string(),
            host_python_command,
            endpoint_url,
            web_port,
            bge_port,
            llm_port,
            duration_ms: started_at.elapsed().as_millis(),
            stdout_log_path: stdout_log_path.display().to_string(),
            stderr_log_path: stderr_log_path.display().to_string(),
            checks,
            errors,
            warnings,
        };
    };

    let stdout_file = fs::File::create(&stdout_log_path);
    let stderr_file = fs::File::create(&stderr_log_path);
    if stdout_file.is_err() || stderr_file.is_err() {
        errors.push("Nao foi possivel criar os arquivos de log do runtime probe.".to_string());
        return RuntimeProbeResult {
            build_id: build_plan.summary.build_id.clone(),
            target_root: target_root.display().to_string(),
            probe_status: "failed".to_string(),
            host_python_command,
            endpoint_url,
            web_port,
            bge_port,
            llm_port,
            duration_ms: started_at.elapsed().as_millis(),
            stdout_log_path: stdout_log_path.display().to_string(),
            stderr_log_path: stderr_log_path.display().to_string(),
            checks,
            errors,
            warnings,
        };
    }

    let runtime_args = [
        "main.py".to_string(),
        "--no-llm".to_string(),
        "--host".to_string(),
        "127.0.0.1".to_string(),
        "--port".to_string(),
        web_port.to_string(),
        "--bge-port".to_string(),
        bge_port.to_string(),
        "--llm-port".to_string(),
        llm_port.to_string(),
        "--db".to_string(),
        probe_db_path.display().to_string(),
    ];
    let command_args = prefix_args
        .iter()
        .cloned()
        .chain(runtime_args.iter().cloned())
        .collect::<Vec<_>>();
    let command_label = smoke_command_label(
        &program,
        &prefix_args,
        &runtime_args.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    let python_env = python_command_env(&program);

    let mut command = Command::new(&program);
    command
        .args(&prefix_args)
        .args(&runtime_args)
        .current_dir(target_root)
        .env("USBRAIN_DISABLE_BROWSER_OPEN", "1")
        .envs(python_env)
        .stdout(Stdio::from(stdout_file.expect("stdout file already checked")))
        .stderr(Stdio::from(stderr_file.expect("stderr file already checked")));

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            errors.push(format!("Falha ao iniciar o runtime probe: {}", error));
            checks.push(RuntimeProbeCheck {
                id: "runtime-spawn".to_string(),
                status: "failed".to_string(),
                message: format!("Nao foi possivel iniciar o runtime de teste: {}", error),
                command: Some(command_label),
            });
            return RuntimeProbeResult {
                build_id: build_plan.summary.build_id.clone(),
                target_root: target_root.display().to_string(),
                probe_status: "failed".to_string(),
                host_python_command,
                endpoint_url,
                web_port,
                bge_port,
                llm_port,
                duration_ms: started_at.elapsed().as_millis(),
                stdout_log_path: stdout_log_path.display().to_string(),
                stderr_log_path: stderr_log_path.display().to_string(),
                checks,
                errors,
                warnings,
            };
        }
    };

    checks.push(RuntimeProbeCheck {
        id: "runtime-spawn".to_string(),
        status: "passed".to_string(),
        message: format!("Runtime de teste iniciado com PID {}.", child.id()),
        command: Some(command_label),
    });

    let timeout = Duration::from_secs(45);
    let poll_started_at = Instant::now();
    let mut endpoint_body = None;
    while poll_started_at.elapsed() < timeout {
        if let Ok(Some(status)) = child.try_wait() {
            let stderr_excerpt = read_log_excerpt(&stderr_log_path);
            let stdout_excerpt = read_log_excerpt(&stdout_log_path);
            errors.push(format!(
                "O runtime probe encerrou antes do endpoint responder. status={:?} stderr={} stdout={}",
                status.code(),
                stderr_excerpt,
                stdout_excerpt
            ));
            break;
        }

        match http_get_text_via_python(&program, &prefix_args, &endpoint_url, Duration::from_secs(2), target_root) {
            Ok(body) => {
                endpoint_body = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(250)),
        }
    }

    if let Some(body) = endpoint_body {
        checks.push(RuntimeProbeCheck {
            id: "http-endpoint".to_string(),
            status: "passed".to_string(),
            message: format!("Endpoint /api/stats respondeu: {}.", clip_smoke_message(&body)),
            command: Some(format!("GET {}", endpoint_url)),
        });
    } else if errors.is_empty() {
        let stderr_excerpt = read_log_excerpt(&stderr_log_path);
        errors.push(format!(
            "O endpoint {} nao respondeu dentro do timeout. stderr={}",
            endpoint_url, stderr_excerpt
        ));
        checks.push(RuntimeProbeCheck {
            id: "http-endpoint".to_string(),
            status: "failed".to_string(),
            message: "O endpoint /api/stats nao respondeu dentro do tempo esperado.".to_string(),
            command: Some(format!("GET {}", endpoint_url)),
        });
    } else {
        checks.push(RuntimeProbeCheck {
            id: "http-endpoint".to_string(),
            status: "failed".to_string(),
            message: "O endpoint nao respondeu porque o runtime encerrou prematuramente.".to_string(),
            command: Some(format!("GET {}", endpoint_url)),
        });
    }

    terminate_probe_process(&mut child);
    checks.push(RuntimeProbeCheck {
        id: "runtime-shutdown".to_string(),
        status: "passed".to_string(),
        message: "Processo do probe encerrado automaticamente ao final da verificacao.".to_string(),
        command: None,
    });

    let _ = command_args; // keeps the assembled command arguments explicit during maintenance

    RuntimeProbeResult {
        build_id: build_plan.summary.build_id.clone(),
        target_root: target_root.display().to_string(),
        probe_status: if errors.is_empty() {
            if warnings.is_empty() {
                "passed".to_string()
            } else {
                "passed-with-warnings".to_string()
            }
        } else {
            "failed".to_string()
        },
        host_python_command,
        endpoint_url,
        web_port,
        bge_port,
        llm_port,
        duration_ms: started_at.elapsed().as_millis(),
        stdout_log_path: stdout_log_path.display().to_string(),
        stderr_log_path: stderr_log_path.display().to_string(),
        checks,
        errors,
        warnings,
    }
}

fn execute_runtime_probe(build_plan: &BuildPlanResponse, target_root: &Path) -> RuntimeProbeResult {
    run_runtime_probe_with_sources(build_plan, target_root, &source_app_root(), &cache_dir())
}

fn run_end_to_end_probe_with_sources(
    build_plan: &BuildPlanResponse,
    target_root: &Path,
    app_root: &Path,
    cache_root: &Path,
) -> EndToEndProbeResult {
    let mut checks = Vec::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let started_at = Instant::now();
    let probe_prompt = "USBRAIN probe: responda com OK".to_string();

    let host_python = detect_probe_python_command();
    let host_python_command = host_python
        .as_ref()
        .map(|(program, prefix_args)| smoke_command_label(program, prefix_args, &[]))
        .unwrap_or_else(|| "indisponivel".to_string());

    let web_port = reserve_local_port().unwrap_or(39900);
    let bge_port = reserve_local_port().unwrap_or(39901);
    let llm_port = reserve_local_port().unwrap_or(39902);
    let stats_endpoint_url = format!("http://127.0.0.1:{}/api/stats", web_port);
    let chat_endpoint_url = format!("http://127.0.0.1:{}/api/chat", web_port);
    let probe_tmp_dir = writable_cache_root()
        .join("probe-runtime")
        .join(&build_plan.summary.build_id);
    let _ = fs::create_dir_all(&probe_tmp_dir);
    let stdout_log_path = probe_tmp_dir.join("e2e-probe.stdout.log");
    let stderr_log_path = probe_tmp_dir.join("e2e-probe.stderr.log");
    let probe_db_path = probe_tmp_dir.join("e2e-probe.db");
    let mut response_preview = String::new();

    let validation = validate_assembled_target_with_sources(build_plan, target_root, app_root, cache_root);
    checks.push(EndToEndProbeCheck {
        id: "validation-gate".to_string(),
        status: if validation.validation_status == "failed" {
            "failed".to_string()
        } else {
            "passed".to_string()
        },
        message: format!("Validacao estrutural retornou {}.", validation.validation_status),
        command: None,
    });
    if validation.validation_status == "failed" {
        errors.extend(validation.errors);
    } else {
        warnings.extend(validation.warnings);
    }

    let Some((program, prefix_args)) = host_python else {
        errors.push("Nenhum Python disponivel foi encontrado para executar o probe ponta a ponta.".to_string());
        return EndToEndProbeResult {
            build_id: build_plan.summary.build_id.clone(),
            target_root: target_root.display().to_string(),
            probe_status: "failed".to_string(),
            host_python_command,
            stats_endpoint_url,
            chat_endpoint_url,
            probe_prompt,
            response_preview,
            web_port,
            bge_port,
            llm_port,
            duration_ms: started_at.elapsed().as_millis(),
            stdout_log_path: stdout_log_path.display().to_string(),
            stderr_log_path: stderr_log_path.display().to_string(),
            checks,
            errors,
            warnings,
        };
    };

    let stdout_file = fs::File::create(&stdout_log_path);
    let stderr_file = fs::File::create(&stderr_log_path);
    if stdout_file.is_err() || stderr_file.is_err() {
        errors.push("Nao foi possivel criar os arquivos de log do probe ponta a ponta.".to_string());
        return EndToEndProbeResult {
            build_id: build_plan.summary.build_id.clone(),
            target_root: target_root.display().to_string(),
            probe_status: "failed".to_string(),
            host_python_command,
            stats_endpoint_url,
            chat_endpoint_url,
            probe_prompt,
            response_preview,
            web_port,
            bge_port,
            llm_port,
            duration_ms: started_at.elapsed().as_millis(),
            stdout_log_path: stdout_log_path.display().to_string(),
            stderr_log_path: stderr_log_path.display().to_string(),
            checks,
            errors,
            warnings,
        };
    }

    let runtime_args = [
        "main.py".to_string(),
        "--no-llm".to_string(),
        "--host".to_string(),
        "127.0.0.1".to_string(),
        "--port".to_string(),
        web_port.to_string(),
        "--bge-port".to_string(),
        bge_port.to_string(),
        "--llm-port".to_string(),
        llm_port.to_string(),
        "--db".to_string(),
        probe_db_path.display().to_string(),
    ];
    let command_label = smoke_command_label(
        &program,
        &prefix_args,
        &runtime_args.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    let python_env = python_command_env(&program);

    let mut command = Command::new(&program);
    command
        .args(&prefix_args)
        .args(&runtime_args)
        .current_dir(target_root)
        .env("USBRAIN_DISABLE_BROWSER_OPEN", "1")
        .envs(python_env)
        .stdout(Stdio::from(stdout_file.expect("stdout file already checked")))
        .stderr(Stdio::from(stderr_file.expect("stderr file already checked")));

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            errors.push(format!("Falha ao iniciar o probe ponta a ponta: {}", error));
            checks.push(EndToEndProbeCheck {
                id: "runtime-spawn".to_string(),
                status: "failed".to_string(),
                message: format!("Nao foi possivel iniciar o runtime para o probe ponta a ponta: {}", error),
                command: Some(command_label),
            });
            return EndToEndProbeResult {
                build_id: build_plan.summary.build_id.clone(),
                target_root: target_root.display().to_string(),
                probe_status: "failed".to_string(),
                host_python_command,
                stats_endpoint_url,
                chat_endpoint_url,
                probe_prompt,
                response_preview,
                web_port,
                bge_port,
                llm_port,
                duration_ms: started_at.elapsed().as_millis(),
                stdout_log_path: stdout_log_path.display().to_string(),
                stderr_log_path: stderr_log_path.display().to_string(),
                checks,
                errors,
                warnings,
            };
        }
    };

    checks.push(EndToEndProbeCheck {
        id: "runtime-spawn".to_string(),
        status: "passed".to_string(),
        message: format!("Runtime de teste iniciado com PID {}.", child.id()),
        command: Some(command_label),
    });

    let timeout = Duration::from_secs(45);
    let poll_started_at = Instant::now();
    let mut stats_body = None;
    while poll_started_at.elapsed() < timeout {
        if let Ok(Some(status)) = child.try_wait() {
            let stderr_excerpt = read_log_excerpt(&stderr_log_path);
            let stdout_excerpt = read_log_excerpt(&stdout_log_path);
            errors.push(format!(
                "O probe ponta a ponta encerrou antes do endpoint responder. status={:?} stderr={} stdout={}",
                status.code(),
                stderr_excerpt,
                stdout_excerpt
            ));
            break;
        }

        match http_get_text_via_python(&program, &prefix_args, &stats_endpoint_url, Duration::from_secs(2), target_root) {
            Ok(body) => {
                stats_body = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(250)),
        }
    }

    if let Some(body) = stats_body {
        checks.push(EndToEndProbeCheck {
            id: "stats-endpoint".to_string(),
            status: "passed".to_string(),
            message: format!("Endpoint /api/stats respondeu: {}.", clip_smoke_message(&body)),
            command: Some(format!("GET {}", stats_endpoint_url)),
        });
    } else if errors.is_empty() {
        let stderr_excerpt = read_log_excerpt(&stderr_log_path);
        errors.push(format!(
            "O endpoint {} nao respondeu dentro do timeout. stderr={}",
            stats_endpoint_url, stderr_excerpt
        ));
        checks.push(EndToEndProbeCheck {
            id: "stats-endpoint".to_string(),
            status: "failed".to_string(),
            message: "O endpoint /api/stats nao respondeu dentro do tempo esperado.".to_string(),
            command: Some(format!("GET {}", stats_endpoint_url)),
        });
    } else {
        checks.push(EndToEndProbeCheck {
            id: "stats-endpoint".to_string(),
            status: "failed".to_string(),
            message: "O endpoint /api/stats nao respondeu porque o runtime encerrou prematuramente.".to_string(),
            command: Some(format!("GET {}", stats_endpoint_url)),
        });
    }

    if errors.is_empty() {
        let request_body = manifest_json(&json!({
            "message": probe_prompt
        }));
        match http_post_json_text_via_python(
            &program,
            &prefix_args,
            &chat_endpoint_url,
            &request_body,
            Duration::from_secs(10),
            target_root,
        ) {
            Ok(body) => {
                response_preview = clip_smoke_message(&body);
                let parsed = serde_json::from_str::<serde_json::Value>(&body);
                match parsed {
                    Ok(value) => {
                        let response_text = value
                            .get("response")
                            .and_then(|item| item.as_str())
                            .unwrap_or_default()
                            .trim()
                            .to_string();
                        let model_text = value
                            .get("model")
                            .and_then(|item| item.as_str())
                            .unwrap_or_default()
                            .trim()
                            .to_string();

                        if response_text.is_empty() {
                            errors.push("O endpoint /api/chat respondeu, mas sem campo response utilizavel.".to_string());
                            checks.push(EndToEndProbeCheck {
                                id: "chat-endpoint".to_string(),
                                status: "failed".to_string(),
                                message: "A resposta JSON nao trouxe um campo response valido.".to_string(),
                                command: Some(format!("POST {}", chat_endpoint_url)),
                            });
                        } else if response_text.starts_with("Erro:") {
                            warnings.push(format!(
                                "O endpoint /api/chat respondeu em modo degradado sem LLM ativa: {}",
                                response_text
                            ));
                            checks.push(EndToEndProbeCheck {
                                id: "chat-endpoint".to_string(),
                                status: "warning".to_string(),
                                message: format!(
                                    "O contrato HTTP do chat respondeu, mas em modo degradado: {}.",
                                    clip_smoke_message(&response_text)
                                ),
                                command: Some(format!("POST {}", chat_endpoint_url)),
                            });
                        } else {
                            checks.push(EndToEndProbeCheck {
                                id: "chat-endpoint".to_string(),
                                status: "passed".to_string(),
                                message: format!(
                                    "O endpoint /api/chat respondeu com payload valido: {}.",
                                    clip_smoke_message(&response_text)
                                ),
                                command: Some(format!("POST {}", chat_endpoint_url)),
                            });
                        }

                        if model_text.is_empty() {
                            warnings.push("O endpoint /api/chat nao retornou o campo model.".to_string());
                            checks.push(EndToEndProbeCheck {
                                id: "chat-model-field".to_string(),
                                status: "warning".to_string(),
                                message: "A resposta do chat nao trouxe o campo model.".to_string(),
                                command: None,
                            });
                        } else {
                            checks.push(EndToEndProbeCheck {
                                id: "chat-model-field".to_string(),
                                status: "passed".to_string(),
                                message: format!("A resposta do chat retornou o modelo {}.", model_text),
                                command: None,
                            });
                        }
                    }
                    Err(error) => {
                        errors.push(format!("A resposta do endpoint /api/chat nao e JSON valido: {}", error));
                        checks.push(EndToEndProbeCheck {
                            id: "chat-endpoint".to_string(),
                            status: "failed".to_string(),
                            message: "A resposta do endpoint /api/chat nao pode ser interpretada como JSON.".to_string(),
                            command: Some(format!("POST {}", chat_endpoint_url)),
                        });
                    }
                }
            }
            Err(error) => {
                errors.push(format!("Falha ao chamar o endpoint /api/chat: {}", error));
                checks.push(EndToEndProbeCheck {
                    id: "chat-endpoint".to_string(),
                    status: "failed".to_string(),
                    message: clip_smoke_message(&error),
                    command: Some(format!("POST {}", chat_endpoint_url)),
                });
            }
        }
    }

    terminate_probe_process(&mut child);
    checks.push(EndToEndProbeCheck {
        id: "runtime-shutdown".to_string(),
        status: "passed".to_string(),
        message: "Processo do probe ponta a ponta encerrado automaticamente ao final da verificacao.".to_string(),
        command: None,
    });

    EndToEndProbeResult {
        build_id: build_plan.summary.build_id.clone(),
        target_root: target_root.display().to_string(),
        probe_status: if errors.is_empty() {
            if warnings.is_empty() {
                "passed".to_string()
            } else {
                "passed-with-warnings".to_string()
            }
        } else {
            "failed".to_string()
        },
        host_python_command,
        stats_endpoint_url,
        chat_endpoint_url,
        probe_prompt,
        response_preview,
        web_port,
        bge_port,
        llm_port,
        duration_ms: started_at.elapsed().as_millis(),
        stdout_log_path: stdout_log_path.display().to_string(),
        stderr_log_path: stderr_log_path.display().to_string(),
        checks,
        errors,
        warnings,
    }
}

fn execute_end_to_end_probe(build_plan: &BuildPlanResponse, target_root: &Path) -> EndToEndProbeResult {
    run_end_to_end_probe_with_sources(build_plan, target_root, &source_app_root(), &cache_dir())
}

fn run_full_local_llm_probe_with_sources(
    build_plan: &BuildPlanResponse,
    target_root: &Path,
    app_root: &Path,
    cache_root: &Path,
) -> FullLocalLlmProbeResult {
    // #region debug-point A:full-probe-entry
    debug_report_full_probe(
        "A",
        "commands.rs:run_full_local_llm_probe_with_sources:entry",
        "full local llm probe started",
        json!({
            "targetRoot": target_root.display().to_string(),
            "appRoot": app_root.display().to_string(),
            "cacheRoot": cache_root.display().to_string(),
            "plannedModels": build_plan.models.iter().map(|model| model.installed_model_id.clone()).collect::<Vec<_>>()
        }),
    );
    // #endregion
    let mut checks = Vec::new();
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let started_at = Instant::now();
    let probe_prompt = "Responda exatamente: OK".to_string();

    let host_python = detect_probe_python_command();
    let host_python_command = host_python
        .as_ref()
        .map(|(program, prefix_args)| smoke_command_label(program, prefix_args, &[]))
        .unwrap_or_else(|| "indisponivel".to_string());
    let python_source = host_python
        .as_ref()
        .map(|(program, _)| python_command_source(program))
        .unwrap_or("unavailable");
    let python_env = host_python
        .as_ref()
        .map(|(program, _)| python_command_env(program))
        .unwrap_or_default();
    // #region debug-point A:host-python-full-probe
    debug_report_full_probe(
        "A",
        "commands.rs:run_full_local_llm_probe_with_sources:host-python",
        "host python detection finished for full probe",
        json!({
            "hostPythonCommand": host_python_command,
            "found": host_python.is_some(),
            "pythonCommandSource": python_source,
            "pythonCommandEnv": python_env,
            "bundledMacosPythonCandidates": debug_bundled_macos_python_detection_attempts()
        }),
    );
    // #endregion

    let web_port = reserve_local_port().unwrap_or(40000);
    let bge_port = reserve_local_port().unwrap_or(40001);
    let llm_port = reserve_local_port().unwrap_or(40002);
    let stats_endpoint_url = format!("http://127.0.0.1:{}/api/stats", web_port);
    let chat_endpoint_url = format!("http://127.0.0.1:{}/api/chat", web_port);
    let probe_tmp_dir = writable_cache_root()
        .join("probe-runtime")
        .join(&build_plan.summary.build_id);
    let _ = fs::create_dir_all(&probe_tmp_dir);
    let stdout_log_path = probe_tmp_dir.join("full-llm-probe.stdout.log");
    let stderr_log_path = probe_tmp_dir.join("full-llm-probe.stderr.log");
    let probe_db_path = probe_tmp_dir.join("full-llm-probe.db");
    let mut response_preview = String::new();

    let validation = validate_assembled_target_with_sources(build_plan, target_root, app_root, cache_root);
    // #region debug-point D:validation-gate
    debug_report_full_probe(
        "D",
        "commands.rs:run_full_local_llm_probe_with_sources:validation",
        "validation gate evaluated before full probe",
        json!({
            "validationStatus": validation.validation_status,
            "errorCount": validation.errors.len(),
            "warningCount": validation.warnings.len(),
            "firstError": validation.errors.first().cloned(),
            "firstWarning": validation.warnings.first().cloned()
        }),
    );
    // #endregion
    checks.push(FullLocalLlmProbeCheck {
        id: "validation-gate".to_string(),
        status: if validation.validation_status == "failed" {
            "failed".to_string()
        } else {
            "passed".to_string()
        },
        message: format!("Validacao estrutural retornou {}.", validation.validation_status),
        command: None,
    });
    if validation.validation_status == "failed" {
        errors.extend(validation.errors);
    } else {
        warnings.extend(validation.warnings);
    }

    if build_plan.models.is_empty() {
        // #region debug-point B:no-models
        debug_report_full_probe(
            "B",
            "commands.rs:run_full_local_llm_probe_with_sources:planned-models",
            "full probe aborted because no planned models were found",
            json!({ "plannedModelCount": 0 }),
        );
        // #endregion
        errors.push("Nao ha modelos planejados para executar o probe completo com LLM local.".to_string());
        checks.push(FullLocalLlmProbeCheck {
            id: "planned-models".to_string(),
            status: "failed".to_string(),
            message: "O build atual nao possui modelos GGUF selecionados.".to_string(),
            command: None,
        });
    } else {
        checks.push(FullLocalLlmProbeCheck {
            id: "planned-models".to_string(),
            status: "passed".to_string(),
            message: format!("{} modelo(s) planejado(s) para o probe completo.", build_plan.models.len()),
            command: None,
        });
    }

    let Some((program, prefix_args)) = host_python else {
        let host_python_attempts = debug_probe_python_detection_attempts();
        // #region debug-point A:no-host-python
        debug_report_full_probe(
            "A",
            "commands.rs:run_full_local_llm_probe_with_sources:no-host-python",
            "full local llm probe aborted because no host python was found",
            json!({
                "attempts": host_python_attempts,
            }),
        );
        // #endregion
        errors.push("Nenhum Python disponivel foi encontrado para executar o probe completo com LLM.".to_string());
        warnings.extend(host_python_attempts.into_iter().map(|attempt| format!("Deteccao de Python: {}", attempt)));
        let result = FullLocalLlmProbeResult {
            build_id: build_plan.summary.build_id.clone(),
            target_root: target_root.display().to_string(),
            probe_status: "failed".to_string(),
            host_python_command,
            stats_endpoint_url,
            chat_endpoint_url,
            probe_prompt,
            response_preview,
            selected_model_count: build_plan.models.len(),
            web_port,
            bge_port,
            llm_port,
            duration_ms: started_at.elapsed().as_millis(),
            stdout_log_path: stdout_log_path.display().to_string(),
            stderr_log_path: stderr_log_path.display().to_string(),
            checks,
            errors,
            warnings,
        };
        write_full_local_llm_probe_report(&result);
        return result;
    };

    if !errors.is_empty() {
        let result = FullLocalLlmProbeResult {
            build_id: build_plan.summary.build_id.clone(),
            target_root: target_root.display().to_string(),
            probe_status: "failed".to_string(),
            host_python_command,
            stats_endpoint_url,
            chat_endpoint_url,
            probe_prompt,
            response_preview,
            selected_model_count: build_plan.models.len(),
            web_port,
            bge_port,
            llm_port,
            duration_ms: started_at.elapsed().as_millis(),
            stdout_log_path: stdout_log_path.display().to_string(),
            stderr_log_path: stderr_log_path.display().to_string(),
            checks,
            errors,
            warnings,
        };
        write_full_local_llm_probe_report(&result);
        return result;
    }

    // #region debug-point A:preflight-binary-start
    debug_report_full_probe(
        "A",
        "commands.rs:run_full_local_llm_probe_with_sources:preflight-binary-start",
        "starting host runtime binary preflight",
        json!({
            "targetRoot": target_root.display().to_string()
        }),
    );
    // #endregion
    match preflight_current_host_runtime_binary(target_root) {
        Ok(Some(binary_path)) => {
            // #region debug-point A:preflight-binary-ok
            debug_report_full_probe(
                "A",
                "commands.rs:run_full_local_llm_probe_with_sources:preflight-binary-ok",
                "host runtime binary preflight passed",
                json!({
                    "binaryPath": binary_path
                }),
            );
            // #endregion
            checks.push(FullLocalLlmProbeCheck {
                id: "host-runtime-binary".to_string(),
                status: "passed".to_string(),
                message: format!("Binario llamafile validado para o host atual em {}.", binary_path),
                command: None,
            })
        }
        Ok(None) => {
            // #region debug-point A:preflight-binary-none
            debug_report_full_probe(
                "A",
                "commands.rs:run_full_local_llm_probe_with_sources:preflight-binary-none",
                "host runtime binary preflight returned none",
                json!({}),
            );
            // #endregion
            warnings.push("Nao foi possivel validar um binario llamafile especifico para o host atual neste ambiente.".to_string())
        }
        Err(error) => {
            // #region debug-point A:binary-preflight-failed
            debug_report_full_probe(
                "A",
                "commands.rs:run_full_local_llm_probe_with_sources:preflight-binary",
                "host runtime binary preflight failed",
                json!({
                    "targetRoot": target_root.display().to_string(),
                    "error": error
                }),
            );
            // #endregion
            errors.push(error.clone());
            checks.push(FullLocalLlmProbeCheck {
                id: "host-runtime-binary".to_string(),
                status: "failed".to_string(),
                message: clip_smoke_message(&error),
                command: None,
            });
            let result = FullLocalLlmProbeResult {
                build_id: build_plan.summary.build_id.clone(),
                target_root: target_root.display().to_string(),
                probe_status: "failed".to_string(),
                host_python_command,
                stats_endpoint_url,
                chat_endpoint_url,
                probe_prompt,
                response_preview,
                selected_model_count: build_plan.models.len(),
                web_port,
                bge_port,
                llm_port,
                duration_ms: started_at.elapsed().as_millis(),
                stdout_log_path: stdout_log_path.display().to_string(),
                stderr_log_path: stderr_log_path.display().to_string(),
                checks,
                errors,
                warnings,
            };
            write_full_local_llm_probe_report(&result);
            return result;
        }
    }

    let stdout_file = fs::File::create(&stdout_log_path);
    let stderr_file = fs::File::create(&stderr_log_path);
    // #region debug-point A:probe-log-files
    debug_report_full_probe(
        "A",
        "commands.rs:run_full_local_llm_probe_with_sources:probe-log-files",
        "attempted to create probe log files",
        json!({
            "stdoutLogPath": stdout_log_path.display().to_string(),
            "stderrLogPath": stderr_log_path.display().to_string(),
            "stdoutOk": stdout_file.is_ok(),
            "stderrOk": stderr_file.is_ok(),
            "stdoutErr": stdout_file.as_ref().err().map(|e| e.to_string()),
            "stderrErr": stderr_file.as_ref().err().map(|e| e.to_string())
        }),
    );
    // #endregion
    if stdout_file.is_err() || stderr_file.is_err() {
        errors.push("Nao foi possivel criar os arquivos de log do probe completo com LLM.".to_string());
        let result = FullLocalLlmProbeResult {
            build_id: build_plan.summary.build_id.clone(),
            target_root: target_root.display().to_string(),
            probe_status: "failed".to_string(),
            host_python_command,
            stats_endpoint_url,
            chat_endpoint_url,
            probe_prompt,
            response_preview,
            selected_model_count: build_plan.models.len(),
            web_port,
            bge_port,
            llm_port,
            duration_ms: started_at.elapsed().as_millis(),
            stdout_log_path: stdout_log_path.display().to_string(),
            stderr_log_path: stderr_log_path.display().to_string(),
            checks,
            errors,
            warnings,
        };
        write_full_local_llm_probe_report(&result);
        return result;
    }

    let runtime_args = [
        "main.py".to_string(),
        "--host".to_string(),
        "127.0.0.1".to_string(),
        "--port".to_string(),
        web_port.to_string(),
        "--bge-port".to_string(),
        bge_port.to_string(),
        "--llm-port".to_string(),
        llm_port.to_string(),
        "--db".to_string(),
        probe_db_path.display().to_string(),
    ];
    let command_label = smoke_command_label(
        &program,
        &prefix_args,
        &runtime_args.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    let python_env = python_command_env(&program);

    let mut command = Command::new(&program);
    command
        .args(&prefix_args)
        .args(&runtime_args)
        .current_dir(target_root)
        .env("USBRAIN_DISABLE_BROWSER_OPEN", "1")
        .envs(python_env)
        .stdout(Stdio::from(stdout_file.expect("stdout file already checked")))
        .stderr(Stdio::from(stderr_file.expect("stderr file already checked")));

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            // #region debug-point A:runtime-spawn-failed
            debug_report_full_probe(
                "A",
                "commands.rs:run_full_local_llm_probe_with_sources:spawn",
                "failed to spawn runtime for full local llm probe",
                json!({
                    "command": command_label,
                    "error": error.to_string(),
                    "stdoutLogPath": stdout_log_path.display().to_string(),
                    "stderrLogPath": stderr_log_path.display().to_string()
                }),
            );
            // #endregion
            errors.push(format!("Falha ao iniciar o probe completo com LLM: {}", error));
            checks.push(FullLocalLlmProbeCheck {
                id: "runtime-spawn".to_string(),
                status: "failed".to_string(),
                message: format!("Nao foi possivel iniciar o runtime com LLM local: {}", error),
                command: Some(command_label),
            });
            let result = FullLocalLlmProbeResult {
                build_id: build_plan.summary.build_id.clone(),
                target_root: target_root.display().to_string(),
                probe_status: "failed".to_string(),
                host_python_command,
                stats_endpoint_url,
                chat_endpoint_url,
                probe_prompt,
                response_preview,
                selected_model_count: build_plan.models.len(),
                web_port,
                bge_port,
                llm_port,
                duration_ms: started_at.elapsed().as_millis(),
                stdout_log_path: stdout_log_path.display().to_string(),
                stderr_log_path: stderr_log_path.display().to_string(),
                checks,
                errors,
                warnings,
            };
            write_full_local_llm_probe_report(&result);
            return result;
        }
    };

    checks.push(FullLocalLlmProbeCheck {
        id: "runtime-spawn".to_string(),
        status: "passed".to_string(),
        message: format!("Runtime completo iniciado com PID {}.", child.id()),
        command: Some(command_label),
    });
    // #region debug-point C:runtime-spawned
    debug_report_full_probe(
        "C",
        "commands.rs:run_full_local_llm_probe_with_sources:spawned",
        "runtime spawned for full local llm probe",
        json!({
            "pid": child.id(),
            "webPort": web_port,
            "bgePort": bge_port,
            "llmPort": llm_port,
            "stdoutLogPath": stdout_log_path.display().to_string(),
            "stderrLogPath": stderr_log_path.display().to_string()
        }),
    );
    // #endregion

    let timeout = Duration::from_secs(180);
    let poll_started_at = Instant::now();
    let mut stats_body = None;
    while poll_started_at.elapsed() < timeout {
        if let Ok(Some(status)) = child.try_wait() {
            let stderr_excerpt = read_log_excerpt(&stderr_log_path);
            let stdout_excerpt = read_log_excerpt(&stdout_log_path);
            // #region debug-point C:runtime-exited-before-stats
            debug_report_full_probe(
                "C",
                "commands.rs:run_full_local_llm_probe_with_sources:stats-wait",
                "runtime exited before stats endpoint answered",
                json!({
                    "statusCode": status.code(),
                    "stderrExcerpt": stderr_excerpt,
                    "stdoutExcerpt": stdout_excerpt
                }),
            );
            // #endregion
            errors.push(format!(
                "O probe completo com LLM encerrou antes do endpoint responder. status={:?} stderr={} stdout={}",
                status.code(),
                stderr_excerpt,
                stdout_excerpt
            ));
            break;
        }

        match http_get_text_via_python(&program, &prefix_args, &stats_endpoint_url, Duration::from_secs(2), target_root) {
            Ok(body) => {
                stats_body = Some(body);
                break;
            }
            Err(_) => thread::sleep(Duration::from_millis(300)),
        }
    }

    if let Some(body) = stats_body {
        // #region debug-point C:stats-ready
        debug_report_full_probe(
            "C",
            "commands.rs:run_full_local_llm_probe_with_sources:stats-ready",
            "stats endpoint answered during full local llm probe",
            json!({
                "statsBody": clip_smoke_message(&body),
                "webPort": web_port
            }),
        );
        // #endregion
        checks.push(FullLocalLlmProbeCheck {
            id: "stats-endpoint".to_string(),
            status: "passed".to_string(),
            message: format!("Endpoint /api/stats respondeu: {}.", clip_smoke_message(&body)),
            command: Some(format!("GET {}", stats_endpoint_url)),
        });
    } else if errors.is_empty() {
        let stderr_excerpt = read_log_excerpt(&stderr_log_path);
        // #region debug-point C:stats-timeout
        debug_report_full_probe(
            "C",
            "commands.rs:run_full_local_llm_probe_with_sources:stats-timeout",
            "stats endpoint timed out during full local llm probe",
            json!({
                "statsEndpointUrl": stats_endpoint_url,
                "stderrExcerpt": stderr_excerpt
            }),
        );
        // #endregion
        errors.push(format!(
            "O endpoint {} nao respondeu dentro do timeout do probe completo. stderr={}",
            stats_endpoint_url, stderr_excerpt
        ));
        checks.push(FullLocalLlmProbeCheck {
            id: "stats-endpoint".to_string(),
            status: "failed".to_string(),
            message: "O endpoint /api/stats nao respondeu dentro do tempo esperado para boot completo.".to_string(),
            command: Some(format!("GET {}", stats_endpoint_url)),
        });
    } else {
        checks.push(FullLocalLlmProbeCheck {
            id: "stats-endpoint".to_string(),
            status: "failed".to_string(),
            message: "O endpoint /api/stats nao respondeu porque o runtime completo encerrou prematuramente.".to_string(),
            command: Some(format!("GET {}", stats_endpoint_url)),
        });
    }

    if errors.is_empty() {
        let request_body = manifest_json(&json!({
            "message": probe_prompt
        }));
        match http_post_json_text_via_python(
            &program,
            &prefix_args,
            &chat_endpoint_url,
            &request_body,
            Duration::from_secs(120),
            target_root,
        ) {
            Ok(body) => {
                response_preview = clip_smoke_message(&body);
                // #region debug-point C:chat-response
                debug_report_full_probe(
                    "C",
                    "commands.rs:run_full_local_llm_probe_with_sources:chat-response",
                    "chat endpoint answered during full local llm probe",
                    json!({
                        "responsePreview": response_preview,
                        "rawLength": body.len()
                    }),
                );
                // #endregion
                let parsed = serde_json::from_str::<serde_json::Value>(&body);
                match parsed {
                    Ok(value) => {
                        let response_text = value
                            .get("response")
                            .and_then(|item| item.as_str())
                            .unwrap_or_default()
                            .trim()
                            .to_string();
                        let model_text = value
                            .get("model")
                            .and_then(|item| item.as_str())
                            .unwrap_or_default()
                            .trim()
                            .to_string();

                        if response_text.is_empty() {
                            errors.push("O endpoint /api/chat respondeu, mas sem campo response utilizavel.".to_string());
                            checks.push(FullLocalLlmProbeCheck {
                                id: "chat-endpoint".to_string(),
                                status: "failed".to_string(),
                                message: "A resposta JSON nao trouxe um campo response valido.".to_string(),
                                command: Some(format!("POST {}", chat_endpoint_url)),
                            });
                        } else if response_text.starts_with("Erro:") {
                            errors.push(format!(
                                "O endpoint /api/chat retornou modo degradado durante o probe completo: {}",
                                response_text
                            ));
                            checks.push(FullLocalLlmProbeCheck {
                                id: "chat-endpoint".to_string(),
                                status: "failed".to_string(),
                                message: format!(
                                    "O chat respondeu em modo degradado, indicando falha funcional da LLM local: {}.",
                                    clip_smoke_message(&response_text)
                                ),
                                command: Some(format!("POST {}", chat_endpoint_url)),
                            });
                        } else {
                            checks.push(FullLocalLlmProbeCheck {
                                id: "chat-endpoint".to_string(),
                                status: "passed".to_string(),
                                message: format!(
                                    "O endpoint /api/chat respondeu com payload funcional: {}.",
                                    clip_smoke_message(&response_text)
                                ),
                                command: Some(format!("POST {}", chat_endpoint_url)),
                            });
                        }

                        if model_text.is_empty() {
                            warnings.push("O endpoint /api/chat nao retornou o campo model no probe completo.".to_string());
                            checks.push(FullLocalLlmProbeCheck {
                                id: "chat-model-field".to_string(),
                                status: "warning".to_string(),
                                message: "A resposta do chat nao trouxe o campo model.".to_string(),
                                command: None,
                            });
                        } else {
                            checks.push(FullLocalLlmProbeCheck {
                                id: "chat-model-field".to_string(),
                                status: "passed".to_string(),
                                message: format!("A resposta do chat retornou o modelo {}.", model_text),
                                command: None,
                            });
                        }
                    }
                    Err(error) => {
                        errors.push(format!("A resposta do endpoint /api/chat nao e JSON valido: {}", error));
                        checks.push(FullLocalLlmProbeCheck {
                            id: "chat-endpoint".to_string(),
                            status: "failed".to_string(),
                            message: "A resposta do endpoint /api/chat nao pode ser interpretada como JSON.".to_string(),
                            command: Some(format!("POST {}", chat_endpoint_url)),
                        });
                    }
                }
            }
            Err(error) => {
                // #region debug-point C:chat-failed
                debug_report_full_probe(
                    "C",
                    "commands.rs:run_full_local_llm_probe_with_sources:chat-failed",
                    "chat endpoint failed during full local llm probe",
                    json!({
                        "error": error.to_string(),
                        "chatEndpointUrl": chat_endpoint_url
                    }),
                );
                // #endregion
                errors.push(format!("Falha ao chamar o endpoint /api/chat com LLM local: {}", error));
                checks.push(FullLocalLlmProbeCheck {
                    id: "chat-endpoint".to_string(),
                    status: "failed".to_string(),
                    message: clip_smoke_message(&error),
                    command: Some(format!("POST {}", chat_endpoint_url)),
                });
            }
        }
    }

    terminate_probe_process(&mut child);
    checks.push(FullLocalLlmProbeCheck {
        id: "runtime-shutdown".to_string(),
        status: "passed".to_string(),
        message: "Processo do probe completo com LLM encerrado automaticamente ao final da verificacao.".to_string(),
        command: None,
    });

    let result = FullLocalLlmProbeResult {
        build_id: build_plan.summary.build_id.clone(),
        target_root: target_root.display().to_string(),
        probe_status: if errors.is_empty() {
            if warnings.is_empty() {
                "passed".to_string()
            } else {
                "passed-with-warnings".to_string()
            }
        } else {
            "failed".to_string()
        },
        host_python_command,
        stats_endpoint_url,
        chat_endpoint_url,
        probe_prompt,
        response_preview,
        selected_model_count: build_plan.models.len(),
        web_port,
        bge_port,
        llm_port,
        duration_ms: started_at.elapsed().as_millis(),
        stdout_log_path: stdout_log_path.display().to_string(),
        stderr_log_path: stderr_log_path.display().to_string(),
        checks,
        errors,
        warnings,
    };
    write_full_local_llm_probe_report(&result);
    result
}

fn execute_full_local_llm_probe(
    build_plan: &BuildPlanResponse,
    target_root: &Path,
) -> FullLocalLlmProbeResult {
    run_full_local_llm_probe_with_sources(build_plan, target_root, &source_app_root(), &cache_dir())
}

fn full_local_llm_probe_report_path() -> PathBuf {
    writable_cache_root().join("full-local-llm-probe-last.json")
}

fn smoke_test_report_path() -> PathBuf {
    writable_cache_root().join("smoke-test-last.json")
}

fn write_smoke_test_report(result: &BuildSmokeTestResult) {
    let report_path = smoke_test_report_path();
    if let Some(parent) = report_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(contents) = serde_json::to_string_pretty(result) {
        let _ = fs::write(report_path, contents);
    }
}

fn write_full_local_llm_probe_report(result: &FullLocalLlmProbeResult) {
    let report_path = full_local_llm_probe_report_path();
    if let Some(parent) = report_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(contents) = serde_json::to_string_pretty(result) {
        let _ = fs::write(report_path, contents);
    }
}

fn factory_pipeline_report_path() -> PathBuf {
    writable_cache_root().join("factory-qa-last.json")
}

fn write_factory_pipeline_report(result: &FactoryPipelineResult) {
    let report_path = factory_pipeline_report_path();
    if let Some(parent) = report_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(contents) = serde_json::to_string_pretty(result) {
        let _ = fs::write(report_path, contents);
    }
}

fn derive_factory_pipeline_status(
    validation_result: &BuildValidationResult,
    smoke_test_result: &BuildSmokeTestResult,
    full_local_llm_probe_result: &FullLocalLlmProbeResult,
) -> String {
    if validation_result.validation_status == "failed"
        || smoke_test_result.smoke_status == "failed"
        || full_local_llm_probe_result.probe_status == "failed"
    {
        "failed".to_string()
    } else if validation_result.validation_status == "passed-with-warnings"
        || smoke_test_result.smoke_status == "passed-with-warnings"
        || full_local_llm_probe_result.probe_status == "passed-with-warnings"
    {
        "passed-with-warnings".to_string()
    } else {
        "passed".to_string()
    }
}

fn run_factory_pipeline_with_sources_and_options(
    request: ManifestPreviewRequest,
    app_handle: Option<&AppHandle>,
) -> FactoryPipelineResult {
    let wipe_target_if_needed = request.wipe_target_if_needed.unwrap_or(false);
    let build_plan = build_plan(request);
    let target_root = PathBuf::from(&build_plan.summary.target_mount_path);
    let report_path = factory_pipeline_report_path().display().to_string();
    // #region debug-point C:factory-pipeline-start
    debug_report_factory_session(
        "C",
        "commands.rs:run_factory_pipeline",
        "Factory pipeline starting",
        json!({
            "target_root": target_root.display().to_string(),
            "report_path": report_path,
            "source_app_root": source_app_root().display().to_string(),
            "source_template_exists": source_app_root().join("webui").join("templates").join("index.html").exists(),
            "wipe_target_if_needed": wipe_target_if_needed,
        }),
    );
    // #endregion

    let assembly_result = match assemble_build_plan_to_target(&build_plan, &target_root, wipe_target_if_needed, app_handle) {
        Ok(result) => result,
        Err(error) => {
            // #region debug-point D:factory-assembly-error
            debug_report_factory_session(
                "D",
                "commands.rs:run_factory_pipeline",
                "Factory pipeline failed during assembly",
                json!({
                    "error": error,
                    "target_root": target_root.display().to_string(),
                }),
            );
            // #endregion
            let failed_result = FactoryPipelineResult {
                build_id: build_plan.summary.build_id.clone(),
                target_root: target_root.display().to_string(),
                overall_status: "failed".to_string(),
                report_path,
                assembly_result: None,
                assembly_error: Some(error),
                validation_result: None,
                smoke_test_result: None,
                full_local_llm_probe_result: None,
            };
            write_factory_pipeline_report(&failed_result);
            return failed_result;
        }
    };
    // #region debug-point C:factory-after-assembly
    debug_report_factory_session(
        "C",
        "commands.rs:run_factory_pipeline",
        "Factory assembly completed",
        json!({
            "webui_template_exists_on_target": target_root.join("webui").join("templates").join("index.html").exists(),
            "webui_flat_index_exists_on_target": target_root.join("webui").join("index.html").exists(),
        }),
    );
    // #endregion

    let validation_result = validate_assembled_target_with_sources(&build_plan, &target_root, &source_app_root(), &cache_dir());
    let smoke_test_result = run_assembled_smoke_test_with_sources(&build_plan, &target_root, &source_app_root(), &cache_dir());
    let full_local_llm_probe_result =
        run_full_local_llm_probe_with_sources(&build_plan, &target_root, &source_app_root(), &cache_dir());
    // #region debug-point E:factory-stage-results
    debug_report_factory_session(
        "E",
        "commands.rs:run_factory_pipeline",
        "Factory pipeline stage results collected",
        json!({
            "validation_status": validation_result.validation_status,
            "validation_error_count": validation_result.errors.len(),
            "validation_warning_count": validation_result.warnings.len(),
            "smoke_status": smoke_test_result.smoke_status,
            "smoke_error_count": smoke_test_result.errors.len(),
            "smoke_warning_count": smoke_test_result.warnings.len(),
            "full_probe_status": full_local_llm_probe_result.probe_status,
            "full_probe_error_count": full_local_llm_probe_result.errors.len(),
            "full_probe_warning_count": full_local_llm_probe_result.warnings.len(),
        }),
    );
    // #endregion
    let result = FactoryPipelineResult {
        build_id: build_plan.summary.build_id.clone(),
        target_root: target_root.display().to_string(),
        overall_status: derive_factory_pipeline_status(
            &validation_result,
            &smoke_test_result,
            &full_local_llm_probe_result,
        ),
        report_path,
        assembly_result: Some(assembly_result),
        assembly_error: None,
        validation_result: Some(validation_result),
        smoke_test_result: Some(smoke_test_result),
        full_local_llm_probe_result: Some(full_local_llm_probe_result),
    };

    if result.overall_status != "failed" && cfg!(target_os = "macos") {
        debug_server_report(
            "A",
            "commands.rs:run_factory_pipeline:auto-eject-deferred",
            "Factory pipeline completed; waiting for dedicated eject command from the UI",
            json!({
                "target_root": target_root.display().to_string(),
                "overall_status": result.overall_status,
            }),
        );
    }

    write_factory_pipeline_report(&result);
    result
}

fn current_host_runtime_binary_candidates(target_root: &Path) -> Vec<PathBuf> {
    if cfg!(target_os = "windows") {
        vec![target_root.join("llamafile.exe")]
    } else if cfg!(target_os = "linux") {
        vec![target_root.join("llamafile-linux"), target_root.join("llamafile")]
    } else if cfg!(target_os = "macos") {
        vec![target_root.join("llamafile")]
    } else {
        Vec::new()
    }
}

fn preflight_current_host_runtime_binary(target_root: &Path) -> Result<Option<String>, String> {
    let candidates = current_host_runtime_binary_candidates(target_root);
    if candidates.is_empty() {
        return Ok(None);
    }

    let Some(binary_path) = candidates.iter().find(|path| path.exists()).cloned() else {
        return Err(format!(
            "Nenhum binario llamafile compativel com o host atual foi encontrado no destino. Esperado um destes caminhos: {}",
            candidates
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    };

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;

        let metadata = fs::metadata(&binary_path)
            .map_err(|error| format!("Falha ao ler o binario {}: {}", binary_path.display(), error))?;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(format!(
                "O binario {} existe, mas nao possui permissao de execucao.",
                binary_path.display()
            ));
        }
    }

    let direct_run = Command::new(&binary_path)
        .arg("--help")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if let Err(error) = direct_run {
        if error.raw_os_error() == Some(8) {
            Command::new("/bin/sh")
                .arg(&binary_path)
                .arg("--help")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map_err(|shell_error| {
                    format!(
                        "O binario {} nao pode ser executado diretamente no host atual ({}) e o fallback via /bin/sh tambem falhou: {}",
                        binary_path.display(),
                        error,
                        shell_error
                    )
                })?;
        } else if tolerate_runtime_binary_preflight_permission_denied(&error) {
            return Ok(None);
        } else {
            return Err(format!(
                "O binario {} nao pode ser executado no host atual: {}",
                binary_path.display(),
                error
            ));
        }
    }

    Ok(Some(binary_path.display().to_string()))
}

fn normalize_section_status(status: &str) -> String {
    match status {
        "failed" => "failed".to_string(),
        "warning" | "passed-with-warnings" => "warning".to_string(),
        "passed" => "passed".to_string(),
        other => other.to_string(),
    }
}

fn build_readiness_report_with_sources(
    build_plan: &BuildPlanResponse,
    target_root: &Path,
    app_root: &Path,
    cache_root: &Path,
) -> BuildReadinessReport {
    let validation = validate_assembled_target_with_sources(build_plan, target_root, app_root, cache_root);
    let smoke = run_assembled_smoke_test_with_sources(build_plan, target_root, app_root, cache_root);
    let runtime_probe = run_runtime_probe_with_sources(build_plan, target_root, app_root, cache_root);
    let end_to_end_probe = run_end_to_end_probe_with_sources(build_plan, target_root, app_root, cache_root);
    let full_local_llm_probe =
        run_full_local_llm_probe_with_sources(build_plan, target_root, app_root, cache_root);

    let sections = vec![
        ReadinessSection {
            id: "validation".to_string(),
            label: "Validacao estrutural".to_string(),
            status: normalize_section_status(&validation.validation_status),
            required: true,
            summary: format!(
                "{} checks, {} erro(s), {} alerta(s).",
                validation.checks.len(),
                validation.errors.len(),
                validation.warnings.len()
            ),
            error_count: validation.errors.len(),
            warning_count: validation.warnings.len(),
        },
        ReadinessSection {
            id: "smoke-test".to_string(),
            label: "Smoke test".to_string(),
            status: normalize_section_status(&smoke.smoke_status),
            required: true,
            summary: format!(
                "{} checks, {} erro(s), {} alerta(s).",
                smoke.checks.len(),
                smoke.errors.len(),
                smoke.warnings.len()
            ),
            error_count: smoke.errors.len(),
            warning_count: smoke.warnings.len(),
        },
        ReadinessSection {
            id: "runtime-probe".to_string(),
            label: "Runtime probe".to_string(),
            status: normalize_section_status(&runtime_probe.probe_status),
            required: true,
            summary: format!(
                "{} checks, {} erro(s), {} alerta(s), {} ms.",
                runtime_probe.checks.len(),
                runtime_probe.errors.len(),
                runtime_probe.warnings.len(),
                runtime_probe.duration_ms
            ),
            error_count: runtime_probe.errors.len(),
            warning_count: runtime_probe.warnings.len(),
        },
        ReadinessSection {
            id: "e2e-probe".to_string(),
            label: "Probe E2E degradado".to_string(),
            status: normalize_section_status(&end_to_end_probe.probe_status),
            required: false,
            summary: format!(
                "{} checks, {} erro(s), {} alerta(s), {} ms.",
                end_to_end_probe.checks.len(),
                end_to_end_probe.errors.len(),
                end_to_end_probe.warnings.len(),
                end_to_end_probe.duration_ms
            ),
            error_count: end_to_end_probe.errors.len(),
            warning_count: end_to_end_probe.warnings.len(),
        },
        ReadinessSection {
            id: "full-local-llm-probe".to_string(),
            label: "Probe completo com LLM local".to_string(),
            status: normalize_section_status(&full_local_llm_probe.probe_status),
            required: true,
            summary: format!(
                "{} checks, {} erro(s), {} alerta(s), {} ms.",
                full_local_llm_probe.checks.len(),
                full_local_llm_probe.errors.len(),
                full_local_llm_probe.warnings.len(),
                full_local_llm_probe.duration_ms
            ),
            error_count: full_local_llm_probe.errors.len(),
            warning_count: full_local_llm_probe.warnings.len(),
        },
    ];

    let mut blocking_issues = Vec::new();
    let mut advisory_issues = Vec::new();
    let mut score: i32 = 100;

    if validation.validation_status == "failed" {
        blocking_issues.push("A validacao estrutural falhou.".to_string());
        blocking_issues.extend(validation.errors.clone());
        score -= 35;
    } else if !validation.warnings.is_empty() {
        advisory_issues.extend(validation.warnings.clone());
        score -= 5;
    }

    if smoke.smoke_status == "failed" {
        blocking_issues.push("O smoke test falhou.".to_string());
        blocking_issues.extend(smoke.errors.clone());
        score -= 15;
    } else if !smoke.warnings.is_empty() {
        advisory_issues.extend(smoke.warnings.clone());
        score -= 5;
    }

    if runtime_probe.probe_status == "failed" {
        blocking_issues.push("O runtime probe falhou.".to_string());
        blocking_issues.extend(runtime_probe.errors.clone());
        score -= 20;
    } else if !runtime_probe.warnings.is_empty() {
        advisory_issues.extend(runtime_probe.warnings.clone());
        score -= 5;
    }

    if end_to_end_probe.probe_status == "failed" {
        advisory_issues.push("O probe E2E degradado falhou.".to_string());
        advisory_issues.extend(end_to_end_probe.errors.clone());
        score -= 5;
    } else if !end_to_end_probe.warnings.is_empty() {
        advisory_issues.extend(end_to_end_probe.warnings.clone());
        score -= 3;
    }

    if full_local_llm_probe.probe_status == "failed" {
        blocking_issues.push("A LLM local falhou no probe completo.".to_string());
        blocking_issues.extend(full_local_llm_probe.errors.clone());
        score -= 25;
    } else if !full_local_llm_probe.warnings.is_empty() {
        advisory_issues.extend(full_local_llm_probe.warnings.clone());
        score -= 5;
    }

    let distribution_ready = blocking_issues.is_empty();
    let readiness_score = score.clamp(0, 100) as u8;
    let readiness_status = if !distribution_ready {
        "blocked"
    } else if advisory_issues.is_empty() {
        "ready"
    } else {
        "ready-with-warnings"
    }
    .to_string();

    let blocked_reason = if validation.validation_status == "failed" {
        Some("Distribuicao bloqueada: a validacao estrutural falhou.".to_string())
    } else if smoke.smoke_status == "failed" {
        Some("Distribuicao bloqueada: o smoke test falhou.".to_string())
    } else if runtime_probe.probe_status == "failed" {
        Some("Distribuicao bloqueada: o runtime probe falhou.".to_string())
    } else if full_local_llm_probe.probe_status == "failed" {
        Some("Distribuicao bloqueada: a LLM local falhou no probe completo.".to_string())
    } else {
        None
    };

    BuildReadinessReport {
        build_id: build_plan.summary.build_id.clone(),
        target_root: target_root.display().to_string(),
        readiness_status,
        readiness_score,
        distribution_ready,
        blocked_reason,
        generated_at: now_utc_string(),
        sections,
        blocking_issues,
        advisory_issues,
    }
}

fn execute_build_readiness_report(
    build_plan: &BuildPlanResponse,
    target_root: &Path,
) -> BuildReadinessReport {
    build_readiness_report_with_sources(build_plan, target_root, &source_app_root(), &cache_dir())
}

fn prepare_distribution_bundle_to_dir(
    build_plan: &BuildPlanResponse,
    output_dir: &Path,
    app_root: &Path,
    cache_root: &Path,
) -> Result<BuildDistributionGateResult, String> {
    let export = export_build_plan_to_dir(build_plan, output_dir)?;
    let readiness_report = build_readiness_report_with_sources(
        build_plan,
        Path::new(&build_plan.summary.target_mount_path),
        app_root,
        cache_root,
    );

    let BuildPlanExportResult {
        build_id,
        export_root,
        manifests_directory: _,
        summary_file: _,
        mut exported_files,
    } = export;

    let export_root_path = PathBuf::from(&export_root);
    let readiness_report_path = export_root_path.join("readiness-report.json");
    exported_files.push(write_export_file(
        &readiness_report_path,
        &manifest_json(&readiness_report),
        &export_root_path,
    )?);

    let distribution_status = if readiness_report.distribution_ready {
        "ready".to_string()
    } else {
        "blocked".to_string()
    };
    let message = if readiness_report.distribution_ready {
        format!(
            "Distribuicao liberada para o build {} com nota {}/100.",
            build_id, readiness_report.readiness_score
        )
    } else {
        readiness_report
            .blocked_reason
            .clone()
            .unwrap_or_else(|| "Distribuicao bloqueada pelo gate de prontidao.".to_string())
    };
    let distribution_summary_path = export_root_path.join("distribution-summary.json");
    let distribution_summary = json!({
        "build_id": build_id,
        "distribution_status": distribution_status,
        "distribution_ready": readiness_report.distribution_ready,
        "message": message,
        "readiness_score": readiness_report.readiness_score,
        "generated_at": readiness_report.generated_at,
        "blocked_reason": readiness_report.blocked_reason,
        "blocking_issues": readiness_report.blocking_issues,
        "advisory_issues": readiness_report.advisory_issues,
    });
    exported_files.push(write_export_file(
        &distribution_summary_path,
        &manifest_json(&distribution_summary),
        &export_root_path,
    )?);

    Ok(BuildDistributionGateResult {
        build_id,
        export_root,
        distribution_status,
        distribution_ready: readiness_report.distribution_ready,
        message,
        readiness_report_file: readiness_report_path.display().to_string(),
        distribution_summary_file: distribution_summary_path.display().to_string(),
        exported_files,
        readiness_report,
    })
}

fn prepare_release_package_to_dir(
    build_plan: &BuildPlanResponse,
    output_dir: &Path,
    app_root: &Path,
    cache_root: &Path,
) -> Result<BuildReleasePackageResult, String> {
    let distribution_result =
        prepare_distribution_bundle_to_dir(build_plan, output_dir, app_root, cache_root)?;
    let export_root_path = PathBuf::from(&distribution_result.export_root);
    let artifact_checksums = build_release_artifact_checksums(&distribution_result.exported_files)?;
    let recommended_release_tag = recommended_release_tag(&distribution_result.build_id);
    let release_status = if distribution_result.distribution_ready {
        "ready".to_string()
    } else {
        "blocked".to_string()
    };
    let message = if distribution_result.distribution_ready {
        format!(
            "Pacote de release pronto para publicacao com tag sugerida {}.",
            recommended_release_tag
        )
    } else {
        format!(
            "Pacote de release gerado com bloqueios. Revise o gate antes de publicar. Tag sugerida: {}.",
            recommended_release_tag
        )
    };
    let checksums_path = export_root_path.join("SHA256SUMS.txt");
    let github_release_notes_path = export_root_path.join("github-release-notes.md");
    let usbrain_release_summary_path = export_root_path.join("usbrain-release-summary.json");
    let release_manifest_path = export_root_path.join("release-package.json");
    let mut exported_files = distribution_result.exported_files.clone();

    exported_files.push(write_export_file(
        &checksums_path,
        &render_release_checksums_file(&artifact_checksums),
        &export_root_path,
    )?);
    exported_files.push(write_export_file(
        &github_release_notes_path,
        &render_github_release_notes(
            build_plan,
            &distribution_result,
            &release_status,
            &recommended_release_tag,
            &artifact_checksums,
        ),
        &export_root_path,
    )?);
    exported_files.push(write_export_file(
        &usbrain_release_summary_path,
        &render_usbrain_release_summary(
            build_plan,
            &distribution_result,
            &release_status,
            &recommended_release_tag,
            &artifact_checksums,
        ),
        &export_root_path,
    )?);

    let release_manifest = json!({
        "build_id": distribution_result.build_id,
        "builder_version": env!("CARGO_PKG_VERSION"),
        "release_status": release_status,
        "release_ready": distribution_result.distribution_ready,
        "message": message,
        "recommended_release_tag": recommended_release_tag,
        "release_manifest_file": release_manifest_path.display().to_string(),
        "github_release_notes_file": github_release_notes_path.display().to_string(),
        "usbrain_release_summary_file": usbrain_release_summary_path.display().to_string(),
        "checksums_file": checksums_path.display().to_string(),
        "distribution_result": distribution_result,
        "artifact_checksums": artifact_checksums,
        "generated_at": now_utc_string(),
    });
    exported_files.push(write_export_file(
        &release_manifest_path,
        &manifest_json(&release_manifest),
        &export_root_path,
    )?);

    Ok(BuildReleasePackageResult {
        build_id: build_plan.summary.build_id.clone(),
        export_root: export_root_path.display().to_string(),
        release_status,
        release_ready: distribution_result.distribution_ready,
        message,
        recommended_release_tag,
        release_manifest_file: release_manifest_path.display().to_string(),
        github_release_notes_file: github_release_notes_path.display().to_string(),
        usbrain_release_summary_file: usbrain_release_summary_path.display().to_string(),
        checksums_file: checksums_path.display().to_string(),
        exported_files,
        artifact_checksums,
        distribution_result,
    })
}

fn export_build_plan_to_dir(
    build_plan: &BuildPlanResponse,
    output_dir: &Path,
) -> Result<BuildPlanExportResult, String> {
    let export_root = output_dir.join(&build_plan.summary.build_id);
    let manifests_dir = export_root.join("manifests");
    fs::create_dir_all(&manifests_dir)
        .map_err(|error| format!("Falha ao criar diretorio de exportacao {}: {}", manifests_dir.display(), error))?;

    let mut exported_files = Vec::new();
    for manifest in &build_plan.manifests {
        let file_path = manifests_dir.join(&manifest.file_name);
        exported_files.push(write_export_file(&file_path, &manifest.contents, &export_root)?);
    }

    let summary_file_path = export_root.join("build-plan.json");
    let summary_contents = manifest_json(build_plan);
    let summary_file = write_export_file(&summary_file_path, &summary_contents, &export_root)?;
    let summary_file_absolute_path = summary_file.absolute_path.clone();
    exported_files.push(summary_file);

    Ok(BuildPlanExportResult {
        build_id: build_plan.summary.build_id.clone(),
        export_root: export_root.display().to_string(),
        manifests_directory: manifests_dir.display().to_string(),
        summary_file: summary_file_absolute_path,
        exported_files,
    })
}

fn build_plan(request: ManifestPreviewRequest) -> BuildPlanResponse {
    let root = source_app_root();
    let build_mode = sanitize_build_mode(request.build_mode);
    let languages = sanitize_languages(request.languages);
    let allow_gpu = request.allow_gpu.unwrap_or(false);
    let allow_cloud_fallback = request.allow_cloud_fallback.unwrap_or(false);
    let selected_model_ids = request.selected_model_ids.clone().unwrap_or_default();
    let requested_target_label = request
        .target_label
        .clone()
        .filter(|value| !value.trim().is_empty());
    let candidate_targets = list_candidate_targets();
    let mut target = request
        .target_mount_path
        .as_deref()
        .map(PathBuf::from)
        .and_then(|path| {
            candidate_targets
                .iter()
                .find(|candidate| candidate.mount_path == path.display().to_string())
                .cloned()
        })
        .or_else(|| candidate_targets.into_iter().next())
        .unwrap_or_else(fallback_target);
    if target.source == "fallback-preview" {
        if let Some(label) = requested_target_label {
            target.label = label;
        }
    }
    let build_id = build_id_from_target(&target.label);
    let profile_mode = profile_mode_from_build_mode(&build_mode);
    let privacy_mode = privacy_mode_from_build_mode(&build_mode);
    let primary_language = primary_language(&languages);
    let available_local_models = discover_local_models();
    let catalog_snapshot = load_model_catalog_snapshot();
    let catalog_models = catalog_snapshot
        .as_ref()
        .map(|snapshot| snapshot.models.as_slice())
        .unwrap_or(&[]);
    let (models, pending_model_downloads, model_warnings) = planned_models(
        &target,
        &languages,
        allow_gpu,
        &selected_model_ids,
        catalog_models,
    );
    let host_ram_recommendations = host_ram_recommendations(&target, &available_local_models, catalog_models);
    let packs = planned_packs(&languages);
    let max_active_model_count = models.len().min(3) as u64;
    let fallback_policy = fallback_strategy(allow_cloud_fallback, &build_mode);
    let validation_warnings = target
        .warnings
        .iter()
        .map(|warning| {
            json!({
                "code": "target-warning",
                "message": warning,
                "severity": "warning"
            })
        })
        .collect::<Vec<_>>();
    let mut summary_warnings = target.warnings.clone();
    summary_warnings.extend(model_warnings);
    if !pending_model_downloads.is_empty() {
        summary_warnings.push(format!(
            "{} modelo(s) ainda dependem de download para entrar no pendrive offline.",
            pending_model_downloads.len()
        ));
    }
    if models.is_empty() && pending_model_downloads.is_empty() {
        summary_warnings.push(
            "Nenhum modelo GGUF real foi encontrado na biblioteca local do Builder para este build."
                .to_string(),
        );
    }
    if let Err(error) = &catalog_snapshot {
        summary_warnings.push(error.clone());
    }

    let profile_manifest = json!({
        "schema_version": "1.0",
        "profile_id": format!("{}-profile", build_id),
        "name": format!("USBRAIN {}", target.label),
        "primary_language": primary_language,
        "supported_languages": languages,
        "mode": profile_mode,
        "privacy_mode": privacy_mode,
        "gpu_preference": if allow_gpu { "optional" } else { "disabled" },
        "personality_profile": {
            "id": "usbrain-core-assistant",
            "version": "1.0.0",
            "source": "builtin"
        },
        "agent_profile": {
            "id": "portable-rag-agent",
            "version": "1.0.0",
            "source": "builtin"
        },
        "description": "Profile generated by USBRAIN Builder.",
        "default_pack_collection": packs.iter().map(|pack| pack.installed_pack_id.clone()).collect::<Vec<_>>(),
        "owner_label": request.creator_label.clone().filter(|value| !value.trim().is_empty())
    });

    let execution_policy_manifest = json!({
        "schema_version": "1.0",
        "policy_id": format!("{}-policy", build_id),
        "allow_cloud_fallback": allow_cloud_fallback,
        "allow_gpu": allow_gpu,
        "prefer_cpu_only": !allow_gpu,
        "priority_mode": priority_mode(&build_mode, target.capacity_gb),
        "fallback_strategy": fallback_policy,
        "max_active_model_count": max_active_model_count,
        "context_strategy": context_strategy(target.capacity_gb),
        "pack_activation_strategy": if languages.len() > 1 { "language-first" } else { "all-compatible" },
        "max_memory_pressure_level": if matches!(target.capacity_gb, Some(value) if value < 32.0) { "medium" } else { "high" },
        "prefer_low_latency": matches!(target.capacity_gb, Some(value) if value < 32.0),
        "allow_background_downloads": build_mode != "offline-only"
    });

    let installed_models_manifest = json!({
        "schema_version": "1.0",
        "default_model_id": models.first().map(|model| model.installed_model_id.clone()),
        "models": models.iter().map(|model| {
            json!({
                "installed_model_id": model.installed_model_id,
                "catalog_model_id": model.installed_model_id,
                "variant_id": format!("{}-variant", model.installed_model_id),
                "display_name": model.display_name,
                "relative_path": model.relative_target_path,
                "disk_size_gb": model.disk_size_gb,
                "format": "gguf",
                "quantization": quantization_from_file_name(&model.source_file_name),
                "role": model.role,
                "languages": languages,
                "recommended_ram_min_gb": (model.recommended_ram_gb / 2.0).max(4.0),
                "recommended_ram_gb": model.recommended_ram_gb,
                "cpu_only_viable": true,
                "gpu_viable": allow_gpu,
                "runtime_family": model.runtime_family
            })
        }).collect::<Vec<_>>()
    });

    let installed_packs_manifest = json!({
        "schema_version": "1.0",
        "packs": packs.iter().map(|pack| {
            json!({
                "installed_pack_id": pack.installed_pack_id,
                "pack_id": pack.installed_pack_id,
                "name": pack.name,
                "version": "1.0.0",
                "pack_type": pack.pack_type,
                "relative_manifest_path": format!("runtime/packs/{}/pack-manifest.json", pack.installed_pack_id),
                "relative_root_path": format!("runtime/packs/{}", pack.installed_pack_id),
                "languages": languages,
                "category": pack.category,
                "activation_mode": pack.activation_mode,
                "dependencies": [],
                "sensitivity_level": if pack.category == "core" { "medium" } else { "low" }
            })
        }).collect::<Vec<_>>()
    });

    let runtime_config_manifest = json!({
        "schema_version": "1.0",
        "runtime_version": "0.1.0",
        "default_ui_port": 7860,
        "default_api_port": 7861,
        "default_host": "127.0.0.1",
        "runtime_root": ".",
        "models_root": "models",
        "packs_root": "packs",
        "semantic_root": "rag/bge-m3-model",
        "logs_root": "runtime/logs",
        "agent_entrypoint": "main.py",
        "semantic_backend": "bge-m3",
        "llm_runtime_family": "llama.cpp",
        "preferred_log_level": "info",
        "boot_timeout_profile": boot_timeout_profile(&target.filesystem),
        "port_strategy": "prefer-fixed-then-fallback"
    });

    let provider_config_manifest = json!({
        "schema_version": "1.0",
        "providers_enabled": allow_cloud_fallback,
        "fallback_policy": fallback_strategy(allow_cloud_fallback, &build_mode),
        "credential_mode": if allow_cloud_fallback { "portable-encrypted-ref" } else { "none" },
        "providers": [
            {
                "provider_id": "deepseek",
                "enabled": allow_cloud_fallback,
                "credential_ref": "secrets/providers/deepseek.ref",
                "model_preferences": ["deepseek-chat"],
                "usage_policy": if allow_cloud_fallback { "fallback-only" } else { "disabled" },
                "notes": "Prepared as optional fallback."
            },
            {
                "provider_id": "openai",
                "enabled": false,
                "usage_policy": "disabled"
            },
            {
                "provider_id": "anthropic",
                "enabled": false,
                "usage_policy": "disabled"
            },
            {
                "provider_id": "google-gemini",
                "enabled": false,
                "usage_policy": "disabled"
            }
        ]
    });

    let validation_report_manifest = json!({
        "schema_version": "1.0",
        "validation_status": if validation_warnings.is_empty() { "passed" } else { "passed-with-warnings" },
        "validated_at": now_utc_string(),
        "builder_version": env!("CARGO_PKG_VERSION"),
        "checks_performed": [
            {
                "id": "target-detected",
                "status": "passed",
                "message": format!("Target {} resolved for planning.", target.mount_path)
            },
            {
                "id": "models-planned",
                "status": "passed",
                "message": format!("{} model(s) selected for the current target.", models.len())
            },
            {
                "id": "packs-planned",
                "status": "passed",
                "message": format!("{} pack(s) selected for the current target.", packs.len())
            }
        ],
        "warnings": validation_warnings,
        "errors": []
    });

    let build_manifest = json!({
        "schema_version": "1.0",
        "build_id": build_id,
        "build_timestamp": now_utc_string(),
        "builder_version": env!("CARGO_PKG_VERSION"),
        "runtime_version_min": "0.1.0",
        "target_os_family": ["windows", "linux"],
        "target_device_profile": {
            "label": target.label,
            "capacity_gb": target.capacity_gb.unwrap_or(64.0),
            "filesystem": target.filesystem,
            "removable": target.removable
        },
        "profile_ref": "manifests/usbrain-profile.json",
        "execution_policy_ref": "manifests/execution-policy.json",
        "installed_models_ref": "manifests/installed-models.json",
        "installed_packs_ref": "manifests/installed-packs.json",
        "runtime_config_ref": "manifests/runtime-config.json",
        "provider_config_ref": "manifests/provider-config.json",
        "validation_report_ref": "manifests/validation-report.json",
        "build_mode": build_mode,
        "languages": languages,
        "created_by": request.creator_label.filter(|value| !value.trim().is_empty()),
        "notes": format!("Plan generated from {} for {}", root.display(), target.mount_path),
        "planning_hints": {
            "target_mount_path": target.mount_path,
            "available_gb": target.available_gb,
            "usage_percent": target.usage_percent,
            "recommended_model_tier": target.recommended_model_tier,
            "estimated_model_slots": target.estimated_model_slots,
            "allow_gpu": allow_gpu,
            "allow_cloud_fallback": allow_cloud_fallback,
            "warnings": summary_warnings
        }
    });

    let manifests = vec![
        GeneratedManifest {
            file_name: "build-manifest.json".to_string(),
            title: "Build Manifest".to_string(),
            contents: manifest_json(&build_manifest),
        },
        GeneratedManifest {
            file_name: "usbrain-profile.json".to_string(),
            title: "USBRAIN Profile".to_string(),
            contents: manifest_json(&profile_manifest),
        },
        GeneratedManifest {
            file_name: "execution-policy.json".to_string(),
            title: "Execution Policy".to_string(),
            contents: manifest_json(&execution_policy_manifest),
        },
        GeneratedManifest {
            file_name: "installed-models.json".to_string(),
            title: "Installed Models".to_string(),
            contents: manifest_json(&installed_models_manifest),
        },
        GeneratedManifest {
            file_name: "installed-packs.json".to_string(),
            title: "Installed Packs".to_string(),
            contents: manifest_json(&installed_packs_manifest),
        },
        GeneratedManifest {
            file_name: "runtime-config.json".to_string(),
            title: "Runtime Config".to_string(),
            contents: manifest_json(&runtime_config_manifest),
        },
        GeneratedManifest {
            file_name: "provider-config.json".to_string(),
            title: "Provider Config".to_string(),
            contents: manifest_json(&provider_config_manifest),
        },
        GeneratedManifest {
            file_name: "validation-report.json".to_string(),
            title: "Validation Report".to_string(),
            contents: manifest_json(&validation_report_manifest),
        },
    ];

    BuildPlanResponse {
        summary: BuildPlanSummary {
            build_id: build_id_from_target(&target.label),
            target_label: target.label,
            target_mount_path: target.mount_path,
            filesystem: target.filesystem,
            build_mode,
            profile_mode,
            privacy_mode,
            capacity_gb: target.capacity_gb,
            available_gb: target.available_gb,
            install_budget_gb: installation_budget_gb(target.capacity_gb, target.available_gb),
            estimated_model_slots: target.estimated_model_slots,
            selected_model_count: models.len(),
            pending_model_download_count: pending_model_downloads.len(),
            selected_pack_count: packs.len(),
            allow_gpu,
            allow_cloud_fallback,
            primary_language,
            languages,
            warnings: summary_warnings,
        },
        models,
        pending_model_downloads,
        host_ram_recommendations,
        packs,
        manifests,
    }
}

#[tauri::command]
pub fn get_model_catalog(query: Option<ModelCatalogQuery>) -> Result<ModelCatalogResponse, String> {
    let query = query.unwrap_or_default();
    let snapshot = load_model_catalog_snapshot()?;
    let available_local_models = discover_local_models();
    let target = find_target_for_mount_path(query.target_mount_path.as_deref());
    let install_budget_gb = target
        .as_ref()
        .and_then(|candidate| installation_budget_gb(candidate.capacity_gb, candidate.available_gb));
    let search_term = query.search.unwrap_or_default().trim().to_ascii_lowercase();
    let selected_domain = query
        .domain
        .unwrap_or_else(|| "all".to_string())
        .trim()
        .to_ascii_lowercase();
    let selected_language = query
        .language
        .unwrap_or_else(|| "all".to_string())
        .trim()
        .to_ascii_lowercase();
    let host_ram_filter = query.host_ram_gb;
    let cpu_only_only = query.cpu_only_only.unwrap_or(false);

    let mut domain_options = snapshot
        .models
        .iter()
        .flat_map(|model| model.domains.iter().cloned())
        .collect::<Vec<_>>();
    domain_options.sort();
    domain_options.dedup();

    let mut models = snapshot
        .models
        .iter()
        .filter(|model| {
            if search_term.is_empty() {
                return true;
            }

            let haystack = format!(
                "{} {} {} {} {}",
                model.display_name,
                model.family,
                model.description,
                model.domains.join(" "),
                model.capabilities.join(" ")
            )
            .to_ascii_lowercase();

            haystack.contains(&search_term)
        })
        .filter(|model| {
            selected_domain == "all"
                || model
                    .domains
                    .iter()
                    .any(|domain| domain.eq_ignore_ascii_case(&selected_domain))
        })
        .filter(|model| {
            selected_language == "all"
                || model
                    .languages
                    .iter()
                    .any(|language| language.eq_ignore_ascii_case(&selected_language))
        })
        .filter(|model| host_ram_filter.is_none_or(|host_ram| model.host_ram_tier_gb == host_ram))
        .filter(|model| !cpu_only_only || model.cpu_only_viable)
        .map(|model| catalog_model_to_summary(model, &available_local_models, install_budget_gb))
        .collect::<Vec<_>>();

    models.sort_by(|left, right| {
        right
            .quality_score
            .cmp(&left.quality_score)
            .then_with(|| {
                left.disk_size_gb
                    .partial_cmp(&right.disk_size_gb)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| left.display_name.cmp(&right.display_name))
    });

    Ok(ModelCatalogResponse {
        catalog_version: snapshot.catalog_version,
        last_updated: snapshot.last_updated,
        source: snapshot.source,
        install_budget_gb,
        domain_options,
        models,
    })
}

#[tauri::command]
pub fn refresh_model_catalog(
    request: Option<RefreshModelCatalogRequest>,
) -> Result<RefreshModelCatalogResult, String> {
    let request = request.unwrap_or_default();
    let candidates = candidate_remote_catalog_sources(&request);
    if candidates.is_empty() {
        return Err(
            "Nenhuma fonte remota de catalogo foi configurada. Defina `USBRAIN_MODEL_CATALOG_URL` ou preencha `catalog/catalog-sources.json`."
                .to_string(),
        );
    }

    let (snapshot, warnings, source_label) = aggregate_remote_model_catalog_snapshot(&candidates)?;
    write_model_catalog_snapshot_to_path(&snapshot, &cached_model_catalog_path())?;
    Ok(RefreshModelCatalogResult {
        status: "refreshed".to_string(),
        source_url: source_label,
        cache_path: cached_model_catalog_path().display().to_string(),
        catalog_version: snapshot.catalog_version,
        last_updated: snapshot.last_updated,
        model_count: snapshot.models.len(),
        warnings,
    })
}

fn default_languages() -> Vec<String> {
    SUPPORTED_LANGUAGES.iter().map(|value| (*value).to_string()).collect()
}

fn sanitize_languages(raw_languages: Option<Vec<String>>) -> Vec<String> {
    let mut result = Vec::new();

    for language in raw_languages.unwrap_or_default() {
        if SUPPORTED_LANGUAGES.contains(&language.as_str()) && !result.contains(&language) {
            result.push(language);
        }
    }

    if result.is_empty() {
        default_languages()
    } else {
        result
    }
}

fn sanitize_build_mode(raw_build_mode: Option<String>) -> String {
    let value = raw_build_mode.unwrap_or_else(|| "offline-first".to_string());
    if SUPPORTED_BUILD_MODES.contains(&value.as_str()) {
        value
    } else {
        "offline-first".to_string()
    }
}

fn now_utc_string() -> String {
    command_stdout("date", &["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .unwrap_or_else(|| "2026-06-27T00:00:00Z".to_string())
}

fn title_from_schema(contents: &str) -> String {
    serde_json::from_str::<serde_json::Value>(contents)
        .ok()
        .and_then(|value| {
            value
                .get("title")
                .and_then(|value| value.as_str())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| "Schema sem titulo".to_string())
}

fn id_from_schema(contents: &str) -> String {
    serde_json::from_str::<serde_json::Value>(contents)
        .ok()
        .and_then(|value| {
            value
                .get("$id")
                .and_then(|value| value.as_str())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_app_overview() -> AppOverview {
    let root = workspace_root();
    let schema_count = fs::read_dir(schema_dir())
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
        .count();

    let document_count = [
        "DEFINICAO_PRODUTO_USBRAIN.md",
        "ARQUITETURA_FUNCIONAL_USBRAIN_BUILDER.md",
        "ARQUITETURA_RUNTIME_USBRAIN.md",
        "ROADMAP_USBRAIN.md",
        "CONTRATOS_BUILDER_RUNTIME.md",
        "BACKLOG_INICIAL_BUILDER_TAURI_RUST.md",
        "BACKLOG_INICIAL_RUNTIME.md",
    ]
    .into_iter()
    .filter(|name| root.join(name).exists())
    .count();

    AppOverview {
        app_name: "USBRAIN Builder".into(),
        app_version: env!("CARGO_PKG_VERSION").into(),
        workspace_root: root.display().to_string(),
        schema_count,
        document_count,
        tauri_backend_available: true,
    }
}

#[tauri::command]
pub fn list_schemas() -> Vec<SchemaSummary> {
    let mut schemas: Vec<SchemaSummary> = fs::read_dir(schema_dir())
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "json") {
                return None;
            }

            let contents = fs::read_to_string(&path).ok()?;
            Some(SchemaSummary {
                file_name: path.file_name()?.to_string_lossy().to_string(),
                title: title_from_schema(&contents),
                schema_id: id_from_schema(&contents),
            })
        })
        .collect();

    schemas.sort_by(|left, right| left.file_name.cmp(&right.file_name));
    schemas
}

#[tauri::command]
pub fn get_workspace_snapshot() -> WorkspaceSnapshot {
    let root = workspace_root();
    let documents = [
        "DEFINICAO_PRODUTO_USBRAIN.md",
        "ESPECIFICACAO_CONCEITUAL_USBRAIN.md",
        "ARQUITETURA_FUNCIONAL_USBRAIN_BUILDER.md",
        "ARQUITETURA_RUNTIME_USBRAIN.md",
        "FORMATO_PACK_USBRAIN.md",
        "ESPECIFICACAO_HUB_USBRAIN.md",
        "ESPECIFICACAO_CATALOGO_MODELOS.md",
        "ROADMAP_USBRAIN.md",
        "CONTRATOS_BUILDER_RUNTIME.md",
        "BACKLOG_INICIAL_BUILDER_TAURI_RUST.md",
        "BACKLOG_INICIAL_RUNTIME.md",
    ]
    .into_iter()
    .filter(|name| root.join(name).exists())
    .map(str::to_string)
    .collect();

    WorkspaceSnapshot {
        root_path: root.display().to_string(),
        schema_directory: schema_dir().display().to_string(),
        key_documents: documents,
    }
}

fn push_mount_entries(base: &Path, _source: &str, targets: &mut Vec<CandidateTarget>) {
    let entries = match fs::read_dir(base) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let label = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        #[cfg(target_os = "macos")]
        {
            let Some(macos_source) = macos_usb_source_for_path(&path) else {
                continue;
            };
            targets.push(analyze_target(path, label, &macos_source, true));
        }

        #[cfg(not(target_os = "macos"))]
        {
            targets.push(analyze_target(path, label, source, true));
        }
    }
}

#[tauri::command]
pub fn list_candidate_targets() -> Vec<CandidateTarget> {
    let mut targets = Vec::new();

    if cfg!(target_os = "macos") {
        push_mount_entries(Path::new("/Volumes"), "macos-usb", &mut targets);
    }

    if cfg!(target_os = "linux") {
        for base in ["/media", "/run/media"] {
            push_mount_entries(Path::new(base), "linux-mount", &mut targets);
        }
    }

    targets.sort_by(|left, right| left.mount_path.cmp(&right.mount_path));
    targets.dedup_by(|left, right| left.mount_path == right.mount_path);
    targets
}

#[tauri::command]
pub fn generate_build_manifest_preview(request: Option<ManifestPreviewRequest>) -> String {
    build_plan(request.unwrap_or_default())
        .manifests
        .into_iter()
        .find(|manifest| manifest.file_name == "build-manifest.json")
        .map(|manifest| manifest.contents)
        .unwrap_or_else(|| "{}".to_string())
}

#[tauri::command]
pub fn generate_build_plan(request: Option<ManifestPreviewRequest>) -> BuildPlanResponse {
    build_plan(request.unwrap_or_default())
}

#[tauri::command]
pub fn prepare_builder_library(
    request: Option<ManifestPreviewRequest>,
) -> Result<BuilderLibraryPreparationResult, String> {
    let build_plan = build_plan(request.unwrap_or_default());
    prepare_builder_library_with_root(&build_plan, &builder_library_root())
}

#[tauri::command]
pub fn list_builder_library() -> Result<BuilderLibraryInventory, String> {
    builder_library_inventory_with_root(&builder_library_root())
}

#[tauri::command]
pub fn delete_builder_library_artifact(
    request: DeleteBuilderLibraryArtifactRequest,
) -> Result<DeleteBuilderLibraryArtifactResult, String> {
    delete_builder_library_artifact_with_root(&builder_library_root(), request)
}

#[tauri::command]
pub fn download_builder_library_artifact(
    request: DownloadBuilderLibraryArtifactRequest,
) -> Result<DownloadBuilderLibraryArtifactResult, String> {
    download_builder_library_artifact_with_root(&builder_library_root(), request)
}

#[tauri::command]
pub fn download_catalog_model(request: DownloadCatalogModelRequest) -> Result<DownloadBuilderLibraryArtifactResult, String> {
    download_catalog_model_with_root(&builder_library_root(), request)
}

#[tauri::command]
pub fn export_build_plan(request: Option<ManifestPreviewRequest>) -> Result<BuildPlanExportResult, String> {
    let build_plan = build_plan(request.unwrap_or_default());
    export_build_plan_to_dir(&build_plan, &builder_output_dir())
}

#[tauri::command]
pub fn assemble_build_plan(app_handle: AppHandle, request: Option<ManifestPreviewRequest>) -> Result<BuildAssemblyResult, String> {
    let request = request.unwrap_or_default();
    let wipe_target_if_needed = request.wipe_target_if_needed.unwrap_or(false);
    let build_plan = build_plan(request);
    let target_root = PathBuf::from(&build_plan.summary.target_mount_path);
    assemble_build_plan_to_target(&build_plan, &target_root, wipe_target_if_needed, Some(&app_handle))
}

pub fn assemble_build_plan_for_qa(request: Option<ManifestPreviewRequest>) -> Result<BuildAssemblyResult, String> {
    let request = request.unwrap_or_default();
    let wipe_target_if_needed = request.wipe_target_if_needed.unwrap_or(false);
    let build_plan = build_plan(request);
    let target_root = PathBuf::from(&build_plan.summary.target_mount_path);
    assemble_build_plan_to_target(&build_plan, &target_root, wipe_target_if_needed, None)
}

#[tauri::command]
pub fn run_factory_pipeline(app_handle: AppHandle, request: Option<ManifestPreviewRequest>) -> FactoryPipelineResult {
    run_factory_pipeline_with_sources_and_options(request.unwrap_or_default(), Some(&app_handle))
}

#[tauri::command]
pub fn validate_assembled_build(request: Option<ManifestPreviewRequest>) -> BuildValidationResult {
    let build_plan = build_plan(request.unwrap_or_default());
    let target_root = PathBuf::from(&build_plan.summary.target_mount_path);
    let result = validate_assembled_target(&build_plan, &target_root);
    // #region debug-point D:validate-command-result
    debug_report_factory_session(
        "D",
        "commands.rs:validate_assembled_build",
        "Validation command finished",
        json!({
            "target_root": target_root.display().to_string(),
            "validation_status": result.validation_status,
            "error_count": result.errors.len(),
            "warning_count": result.warnings.len(),
        }),
    );
    // #endregion
    result
}

#[tauri::command]
pub fn run_assembled_smoke_test(request: Option<ManifestPreviewRequest>) -> BuildSmokeTestResult {
    let build_plan = build_plan(request.unwrap_or_default());
    let target_root = PathBuf::from(&build_plan.summary.target_mount_path);
    let result = execute_assembled_smoke_test(&build_plan, &target_root);
    write_smoke_test_report(&result);
    // #region debug-point D:smoke-command-result
    debug_report_factory_session(
        "D",
        "commands.rs:run_assembled_smoke_test",
        "Smoke test command finished",
        json!({
            "target_root": target_root.display().to_string(),
            "smoke_status": result.smoke_status,
            "error_count": result.errors.len(),
            "warning_count": result.warnings.len(),
            "report_path": smoke_test_report_path().display().to_string(),
            "errors": result.errors,
            "warnings": result.warnings,
            "checks": result
                .checks
                .iter()
                .map(|check| json!({
                    "id": check.id,
                    "status": check.status,
                    "message": check.message,
                    "command": check.command,
                }))
                .collect::<Vec<_>>(),
        }),
    );
    // #endregion
    result
}

#[tauri::command]
pub fn run_runtime_probe(request: Option<ManifestPreviewRequest>) -> RuntimeProbeResult {
    let build_plan = build_plan(request.unwrap_or_default());
    let target_root = PathBuf::from(&build_plan.summary.target_mount_path);
    execute_runtime_probe(&build_plan, &target_root)
}

#[tauri::command]
pub fn run_end_to_end_probe(request: Option<ManifestPreviewRequest>) -> EndToEndProbeResult {
    let build_plan = build_plan(request.unwrap_or_default());
    let target_root = PathBuf::from(&build_plan.summary.target_mount_path);
    execute_end_to_end_probe(&build_plan, &target_root)
}

#[tauri::command]
pub fn run_full_local_llm_probe(request: Option<ManifestPreviewRequest>) -> FullLocalLlmProbeResult {
    let build_plan = build_plan(request.unwrap_or_default());
    let target_root = PathBuf::from(&build_plan.summary.target_mount_path);
    let result = execute_full_local_llm_probe(&build_plan, &target_root);
    // #region debug-point E:full-probe-command-result
    debug_report_factory_session(
        "E",
        "commands.rs:run_full_local_llm_probe",
        "Full local LLM probe command finished",
        json!({
            "target_root": target_root.display().to_string(),
            "probe_status": result.probe_status,
            "error_count": result.errors.len(),
            "warning_count": result.warnings.len(),
            "stdout_log_path": result.stdout_log_path,
            "stderr_log_path": result.stderr_log_path,
        }),
    );
    // #endregion
    result
}

#[tauri::command]
pub fn generate_build_readiness_report(request: Option<ManifestPreviewRequest>) -> BuildReadinessReport {
    let build_plan = build_plan(request.unwrap_or_default());
    let target_root = PathBuf::from(&build_plan.summary.target_mount_path);
    execute_build_readiness_report(&build_plan, &target_root)
}

#[tauri::command]
pub fn prepare_distribution_bundle(request: Option<ManifestPreviewRequest>) -> Result<BuildDistributionGateResult, String> {
    let build_plan = build_plan(request.unwrap_or_default());
    prepare_distribution_bundle_to_dir(&build_plan, &builder_output_dir(), &source_app_root(), &cache_dir())
}

#[tauri::command]
pub fn prepare_release_package(request: Option<ManifestPreviewRequest>) -> Result<BuildReleasePackageResult, String> {
    let build_plan = build_plan(request.unwrap_or_default());
    prepare_release_package_to_dir(&build_plan, &builder_output_dir(), &source_app_root(), &cache_dir())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn sample_catalog_model(model_id: &str) -> CatalogModelRecord {
        CatalogModelRecord {
            model_id: model_id.to_string(),
            display_name: "Sample Model".to_string(),
            family: "qwen".to_string(),
            parameter_scale: "7B".to_string(),
            variant: "Instruct".to_string(),
            disk_size_gb: 4.2,
            ram_min_gb: 8.0,
            ram_recommended_gb: 12.0,
            host_ram_tier_gb: 16,
            languages: vec!["en".to_string()],
            domains: vec!["general".to_string()],
            capabilities: vec!["chat".to_string()],
            quality_score: 70,
            cpu_only_viable: true,
            gpu_optional: false,
            license: "apache-2.0".to_string(),
            homepage_or_source: "https://huggingface.co/example/model".to_string(),
            description: "sample".to_string(),
            rationale: "sample rationale".to_string(),
            priority: 70,
            download_locations: vec!["https://example.invalid/model.gguf".to_string()],
            checksums: vec!["sha256:abc".to_string()],
            provider_or_origin: Some("Example".to_string()),
            variant_id: Some("q4km".to_string()),
            quantization: Some("Q4_K_M".to_string()),
            file_format: Some("GGUF".to_string()),
            editorial_recommendation: None,
            benchmark_summary: None,
            benchmark_metrics: vec![],
            technical_profile: None,
            local_file_name: Some("model.gguf".to_string()),
        }
    }

    fn sample_huggingface_model_info(repo_id: &str) -> HuggingFaceModelInfoRecord {
        HuggingFaceModelInfoRecord {
            id: repo_id.to_string(),
            author: Some("ExampleOrg".to_string()),
            downloads: Some(500_000),
            likes: Some(2_000),
            tags: vec![
                "gguf".to_string(),
                "text-generation".to_string(),
                "conversational".to_string(),
                "base_model:Qwen/Qwen3-8B-Instruct".to_string(),
                "base_model:quantized:Qwen/Qwen3-8B-Instruct".to_string(),
                "license:apache-2.0".to_string(),
            ],
            pipeline_tag: Some("text-generation".to_string()),
            siblings: vec![HuggingFaceSiblingRecord {
                rfilename: "Qwen3-8B-Instruct-Q4_K_M.gguf".to_string(),
                size: Some(4_500_000_000),
                lfs: Some(HuggingFaceLfsRecord {
                    oid: Some("deadbeef".to_string()),
                    size: Some(4_500_000_000),
                }),
            }],
            card_data: Some(HuggingFaceCardDataRecord {
                license: Some("apache-2.0".to_string()),
                language: vec!["en".to_string(), "pt".to_string()],
                base_model: Some("Qwen/Qwen3-8B-Instruct".to_string()),
            }),
            gguf: Some(HuggingFaceGgufRecord {
                total: Some(4_500_000_000),
                total_file_size: Some(4_500_000_000),
                architecture: Some("qwen3".to_string()),
                context_length: Some(32768),
            }),
            private: Some(false),
            gated: Some(false),
        }
    }

    fn sample_open_evals_row(model_name: &str) -> OpenEvalsBenchmarkRow {
        OpenEvalsBenchmarkRow {
            model_id: model_name.to_string(),
            model_name: model_name.to_string(),
            provider: Some("OpenRouter".to_string()),
            parameters_billions: Some(8.0),
            license: Some("apache-2.0".to_string()),
            context_window: Some(32768),
            architecture: Some("qwen3".to_string()),
            aime2026_score: None,
            evasion_bench_score: None,
            gpqa_score: Some(61.0),
            gsm8k_score: None,
            hle_score: None,
            hmmt2026_score: None,
            mmlu_pro_score: Some(68.0),
            olm_ocr_score: None,
            swe_pro_score: None,
            swe_verified_score: None,
            terminal_bench_score: None,
            aggregate_score: Some(74.0),
            coverage_count: Some(2),
            coverage_percent: Some(20.0),
        }
    }

    #[test]
    fn alias_candidates_strip_quantization_and_release_suffixes() {
        let aliases = alias_candidates_for_lookup_text("unsloth/Qwen3-30B-A3B-Instruct-2507-Q4_K_M-GGUF");

        assert!(aliases.contains(&"qwen330ba3binstruct2507q4km".to_string()));
        assert!(aliases.contains(&"qwen330ba3binstruct2507".to_string()));
        assert!(aliases.contains(&"qwen330ba3binstruct".to_string()));
    }

    #[test]
    fn semantic_lookup_keys_converge_for_specialized_names_with_different_token_order() {
        let left = semantic_lookup_keys_for_text("Qwen3-Coder-8B-Instruct");
        let right = semantic_lookup_keys_for_text("Qwen3-8B-Instruct coding");

        let shared = left.into_iter().collect::<HashSet<_>>();
        assert!(shared.contains("qwen38binstructcoding"));
        assert!(right.into_iter().any(|key| key == "qwen38binstructcoding"));
    }

    #[test]
    fn merge_catalog_model_records_unifies_downloads_and_prefers_stronger_benchmark_summary() {
        let mut existing = sample_catalog_model("qwen/sample");
        existing.benchmark_summary = Some(CatalogBenchmarkSummaryRecord {
            source_label: "Data from BenchLM.ai".to_string(),
            summary: "BenchLM".to_string(),
            composite_score: Some(72.0),
            rank_label: None,
            last_verified_at: Some("2026-07-04T10:00:00Z".to_string()),
            match_origin: "inherited-base".to_string(),
            confidence_label: "Benchmark herdado".to_string(),
            confidence_level: "guarded".to_string(),
        });
        existing.benchmark_metrics = vec![CatalogBenchmarkMetricRecord {
            label: "Overall".to_string(),
            value: 72.0,
            unit: Some("pts".to_string()),
            benchmark_suite: Some("BenchLM".to_string()),
            higher_is_better: Some(true),
        }];

        let mut incoming = sample_catalog_model("qwen/sample");
        incoming.quality_score = 82;
        incoming.priority = 82;
        incoming.download_locations = vec!["https://mirror.invalid/model.gguf".to_string()];
        incoming.benchmark_summary = Some(CatalogBenchmarkSummaryRecord {
            source_label: "OpenEvals official leaderboard".to_string(),
            summary: "OpenEvals".to_string(),
            composite_score: Some(68.0),
            rank_label: None,
            last_verified_at: None,
            match_origin: "direct-comparable".to_string(),
            confidence_label: "Benchmark comparavel".to_string(),
            confidence_level: "medium".to_string(),
        });
        incoming.benchmark_metrics = vec![CatalogBenchmarkMetricRecord {
            label: "MMLU-Pro".to_string(),
            value: 61.5,
            unit: Some("pts".to_string()),
            benchmark_suite: Some("OpenEvals".to_string()),
            higher_is_better: Some(true),
        }];

        let merged = merge_catalog_model_records(existing, incoming);

        assert_eq!(merged.quality_score, 82);
        assert_eq!(merged.priority, 82);
        assert_eq!(merged.download_locations.len(), 2);
        assert_eq!(
            merged.benchmark_summary.as_ref().map(|summary| summary.source_label.as_str()),
            Some("OpenEvals official leaderboard")
        );
        assert_eq!(merged.benchmark_metrics.len(), 2);
    }

    #[test]
    fn merge_catalog_model_records_prefers_more_confident_benchmark_summary() {
        let mut existing = sample_catalog_model("qwen/sample");
        existing.benchmark_summary = Some(CatalogBenchmarkSummaryRecord {
            source_label: "OpenEvals official leaderboard".to_string(),
            summary: "Benchmark herdado".to_string(),
            composite_score: Some(90.0),
            rank_label: None,
            last_verified_at: None,
            match_origin: "inherited-base".to_string(),
            confidence_label: "Benchmark herdado".to_string(),
            confidence_level: "guarded".to_string(),
        });

        let mut incoming = sample_catalog_model("qwen/sample");
        incoming.benchmark_summary = Some(CatalogBenchmarkSummaryRecord {
            source_label: "Data from BenchLM.ai".to_string(),
            summary: "Benchmark proprio".to_string(),
            composite_score: Some(70.0),
            rank_label: None,
            last_verified_at: Some("2026-07-04T10:00:00Z".to_string()),
            match_origin: "variant-specific".to_string(),
            confidence_label: "Benchmark proprio".to_string(),
            confidence_level: "high".to_string(),
        });

        let merged = merge_catalog_model_records(existing, incoming);

        assert_eq!(
            merged
                .benchmark_summary
                .as_ref()
                .map(|summary| summary.confidence_label.as_str()),
            Some("Benchmark proprio")
        );
        assert_eq!(
            merged
                .benchmark_summary
                .as_ref()
                .map(|summary| summary.source_label.as_str()),
            Some("Data from BenchLM.ai")
        );
    }

    #[test]
    fn normalize_huggingface_model_record_rejects_multimodal_repos_with_mmproj_assets() {
        let mut record = sample_huggingface_model_info("unsloth/Qwen3-VL-2B-Instruct-GGUF");
        record.pipeline_tag = Some("image-text-to-text".to_string());
        record.tags.push("multimodal".to_string());
        record.tags.push("vision".to_string());
        record.siblings.push(HuggingFaceSiblingRecord {
            rfilename: "mmproj-model-f16.gguf".to_string(),
            size: Some(1_000_000_000),
            lfs: None,
        });

        let model = normalize_huggingface_model_record(
            record,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            None,
        );

        assert!(model.is_none());
    }

    #[test]
    fn normalize_huggingface_model_record_penalizes_derived_repos() {
        let mut record = sample_huggingface_model_info("example/Qwen3-8B-Instruct-Lora-Merged-GGUF");
        record.tags.push("lora".to_string());
        record.tags.push("merged".to_string());
        record.tags.push("base_model:adapter:Qwen/Qwen3-8B-Instruct".to_string());

        let model = normalize_huggingface_model_record(
            record,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            None,
        )
        .expect("derived gguf should still be normalized");

        assert!(model.quality_score < infer_quality_score(500_000, 2_000));
        assert!(model.priority < infer_quality_score(500_000, 2_000));
        assert!(
            model
                .editorial_recommendation
                .as_deref()
                .unwrap_or_default()
                .contains("Repositorio derivado detectado")
        );
    }

    #[test]
    fn normalize_huggingface_model_record_falls_back_to_gguf_total_when_sibling_sizes_are_missing() {
        let mut record = sample_huggingface_model_info("bartowski/Llama-3.2-1B-Instruct-GGUF");
        record.siblings = vec![HuggingFaceSiblingRecord {
            rfilename: "Llama-3.2-1B-Instruct-Q4_K_M.gguf".to_string(),
            size: None,
            lfs: None,
        }];
        record.gguf = Some(HuggingFaceGgufRecord {
            total: Some(1_235_814_432),
            total_file_size: Some(2_479_595_360),
            architecture: Some("llama".to_string()),
            context_length: Some(131_072),
        });

        let model = normalize_huggingface_model_record(
            record,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            None,
        )
        .expect("gguf total fallback should allow normalization");

        assert_eq!(model.family, "llama");
        assert_eq!(model.variant, "Q4_K_M");
        assert!(model.disk_size_gb > 1.0);
    }

    #[test]
    fn normalize_huggingface_model_record_groups_generic_quantized_mirrors_by_base_model() {
        let record_a = sample_huggingface_model_info("bartowski/Qwen3-8B-Instruct-GGUF");
        let record_b = sample_huggingface_model_info("lmstudio-community/Qwen3-8B-Instruct-GGUF");

        let model_a = normalize_huggingface_model_record(
            record_a,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            None,
        )
        .expect("first generic mirror should normalize");
        let model_b = normalize_huggingface_model_record(
            record_b,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            None,
        )
        .expect("second generic mirror should normalize");

        assert_eq!(model_a.model_id, model_b.model_id);
        assert_eq!(model_a.display_name, "Qwen3 8B Instruct");
        assert_eq!(model_b.display_name, "Qwen3 8B Instruct");
    }

    #[test]
    fn normalize_huggingface_model_record_keeps_derived_repo_in_distinct_group() {
        let base_record = sample_huggingface_model_info("bartowski/Qwen3-8B-Instruct-GGUF");
        let mut derived_record = sample_huggingface_model_info("example/Qwen3-8B-Instruct-Lora-Merged-GGUF");
        derived_record.tags.push("lora".to_string());
        derived_record.tags.push("merged".to_string());
        derived_record.tags.push("base_model:adapter:Qwen/Qwen3-8B-Instruct".to_string());

        let base_model = normalize_huggingface_model_record(
            base_record,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            None,
        )
        .expect("base mirror should normalize");
        let derived_model = normalize_huggingface_model_record(
            derived_record,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            None,
        )
        .expect("derived model should normalize");

        assert_ne!(base_model.model_id, derived_model.model_id);
    }

    #[test]
    fn normalize_huggingface_model_record_keeps_specialized_generic_repo_in_distinct_group() {
        let base_record = sample_huggingface_model_info("bartowski/Qwen3-8B-Instruct-GGUF");
        let mut specialized_record = sample_huggingface_model_info("example/Qwen3-8B-Instruct-Coder-GGUF");
        specialized_record.tags.push("coder".to_string());
        specialized_record.tags.push("tool-use".to_string());

        let base_model = normalize_huggingface_model_record(
            base_record,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            None,
        )
        .expect("base mirror should normalize");
        let specialized_model = normalize_huggingface_model_record(
            specialized_record,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            None,
        )
        .expect("specialized mirror should normalize");

        assert_ne!(base_model.model_id, specialized_model.model_id);
        assert_eq!(specialized_model.display_name, "Qwen3 8B Instruct Coder Tool Use");
    }

    #[test]
    fn benchmark_lookup_keys_include_semantic_specialization_aliases() {
        let mut record = sample_huggingface_model_info("example/Qwen3-8B-Instruct-Coder-GGUF");
        record.tags.push("coder".to_string());
        record.tags.push("tool-use".to_string());

        let aliases = benchmark_lookup_keys_for_huggingface_model(&record, "Qwen3-8B-Instruct-Coder-GGUF");

        assert!(aliases.contains(&"qwen38binstructcodingtooluse".to_string()));
    }

    #[test]
    fn generic_mirror_can_still_use_base_model_benchmark() {
        let mut model = sample_catalog_model("qwen3-8b-instruct-q4km");
        let record = sample_huggingface_model_info("bartowski/Qwen3-8B-Instruct-GGUF");
        let benchmark_row = sample_open_evals_row("Qwen3-8B-Instruct");
        let mut benchmark_index = HashMap::new();
        benchmark_index.insert("qwen38binstruct".to_string(), benchmark_row);

        enrich_model_with_open_evals_benchmarks(&mut model, &record, "Qwen3-8B-Instruct", &benchmark_index);

        assert!(model.benchmark_summary.is_some());
        assert!(
            model
                .benchmark_summary
                .as_ref()
                .map(|summary| summary.summary.contains("modelo base correspondente"))
                .unwrap_or(false)
        );
    }

    #[test]
    fn specialized_variant_does_not_inherit_base_model_benchmark_without_own_match() {
        let mut model = sample_catalog_model("qwen3-8b-instruct-coder-q4km");
        let mut record = sample_huggingface_model_info("example/Qwen3-8B-Instruct-Coder-GGUF");
        record.tags.push("coder".to_string());
        record.tags.push("tool-use".to_string());
        let benchmark_row = sample_open_evals_row("Qwen3-8B-Instruct");
        let mut benchmark_index = HashMap::new();
        benchmark_index.insert("qwen38binstruct".to_string(), benchmark_row);

        enrich_model_with_open_evals_benchmarks(&mut model, &record, "Qwen3-8B-Instruct-Coder", &benchmark_index);

        assert!(model.benchmark_summary.is_none());
        assert!(
            model
                .editorial_recommendation
                .as_deref()
                .unwrap_or_default()
                .contains("nao herdou benchmark do modelo-base")
        );
    }

    #[test]
    fn specialized_variant_uses_own_benchmark_when_semantic_match_exists() {
        let mut model = sample_catalog_model("qwen3-8b-instruct-coder-q4km");
        let mut record = sample_huggingface_model_info("example/Qwen3-8B-Instruct-Coder-GGUF");
        record.tags.push("coder".to_string());
        record.tags.push("tool-use".to_string());
        let benchmark_row = sample_open_evals_row("Qwen3-8B-Instruct coding tool use");
        let mut benchmark_index = HashMap::new();
        benchmark_index.insert("qwen38binstructcodingtooluse".to_string(), benchmark_row);

        enrich_model_with_open_evals_benchmarks(&mut model, &record, "Qwen3-8B-Instruct-Coder", &benchmark_index);

        assert!(
            model
                .benchmark_summary
                .as_ref()
                .map(|summary| summary.summary.contains("variante especializada correspondente"))
                .unwrap_or(false)
        );
        assert!(
            model
                .editorial_recommendation
                .as_deref()
                .unwrap_or_default()
                .contains("benchmark publico aderente a esta variante especializada")
        );
    }

    #[test]
    fn host_ram_recommendations_prioritize_more_confident_benchmark() {
        let target = CandidateTarget {
            label: "USBRAIN".to_string(),
            mount_path: "/Volumes/USBRAIN".to_string(),
            source: "macos-usb:disk10s1".to_string(),
            filesystem: "exfat".to_string(),
            capacity_gb: Some(64.0),
            available_gb: Some(52.0),
            usage_percent: Some(18.0),
            removable: true,
            recommended_model_tier: "Build completo".to_string(),
            estimated_model_slots: 4,
            warnings: vec![],
        };

        let mut inherited_model = sample_catalog_model("qwen-inherited");
        inherited_model.display_name = "Modelo Herdado".to_string();
        inherited_model.host_ram_tier_gb = 16;
        inherited_model.disk_size_gb = 5.0;
        inherited_model.priority = 90;
        inherited_model.quality_score = 90;
        inherited_model.benchmark_summary = Some(CatalogBenchmarkSummaryRecord {
            source_label: "OpenEvals official leaderboard".to_string(),
            summary: "Benchmark herdado".to_string(),
            composite_score: Some(82.0),
            rank_label: None,
            last_verified_at: None,
            match_origin: "inherited-base".to_string(),
            confidence_label: "Benchmark herdado".to_string(),
            confidence_level: "guarded".to_string(),
        });

        let mut direct_model = sample_catalog_model("qwen-direct");
        direct_model.display_name = "Modelo Comparavel".to_string();
        direct_model.host_ram_tier_gb = 16;
        direct_model.disk_size_gb = 5.4;
        direct_model.priority = 80;
        direct_model.quality_score = 80;
        direct_model.benchmark_summary = Some(CatalogBenchmarkSummaryRecord {
            source_label: "Data from BenchLM.ai".to_string(),
            summary: "Benchmark comparavel".to_string(),
            composite_score: Some(70.0),
            rank_label: None,
            last_verified_at: None,
            match_origin: "direct-comparable".to_string(),
            confidence_label: "Benchmark comparavel".to_string(),
            confidence_level: "medium".to_string(),
        });

        let recommendations = host_ram_recommendations(&target, &[], &[inherited_model, direct_model]);
        let recommendation_16gb = recommendations
            .into_iter()
            .find(|recommendation| recommendation.host_ram_gb == 16)
            .expect("16 GB recommendation should exist");

        assert_eq!(recommendation_16gb.suggested_model.display_name, "Modelo Comparavel");
    }

    #[test]
    fn benchmark_lookup_keys_preserve_and_simplify_versioned_base_model_aliases() {
        let record = HuggingFaceModelInfoRecord {
            id: "unsloth/Qwen3-30B-A3B-Instruct-2507-GGUF".to_string(),
            author: None,
            downloads: None,
            likes: None,
            tags: vec![
                "gguf".to_string(),
                "base_model:Qwen/Qwen3-30B-A3B-Instruct-2507".to_string(),
                "base_model:quantized:Qwen/Qwen3-30B-A3B-Instruct-2507".to_string(),
            ],
            pipeline_tag: None,
            siblings: vec![],
            card_data: Some(HuggingFaceCardDataRecord {
                license: None,
                language: vec![],
                base_model: Some("Qwen/Qwen3-30B-A3B-Instruct-2507".to_string()),
            }),
            gguf: None,
            private: Some(false),
            gated: Some(false),
        };

        let aliases = benchmark_lookup_keys_for_huggingface_model(&record, "Qwen3-30B-A3B-Instruct-2507-GGUF");

        assert!(aliases.contains(&"qwen330ba3binstruct2507".to_string()));
        assert!(aliases.contains(&"qwen330ba3binstruct".to_string()));
    }

    fn write_fake_llamafile_binaries(app_root: &Path) {
        let fake_binary = "#!/bin/sh\necho fake llamafile help\nexit 0\n";
        for file_name in ["llamafile", "llamafile-linux"] {
            let binary_path = app_root.join(file_name);
            fs::write(&binary_path, fake_binary).expect("write fake llamafile");
            #[cfg(not(target_os = "windows"))]
            {
                use std::os::unix::fs::PermissionsExt;

                let mut permissions = fs::metadata(&binary_path).expect("read fake llamafile metadata").permissions();
                permissions.set_mode(0o755);
                fs::set_permissions(&binary_path, permissions).expect("chmod fake llamafile");
            }
        }
    }

    fn write_fake_bge_assets(cache_root: &Path) {
        for file_name in [
            "pytorch_model.bin",
            "sentencepiece.bpe.model",
            "colbert_linear.pt",
            "sparse_linear.pt",
        ] {
            fs::write(cache_root.join("bge-m3-model").join(file_name), "bge").expect("write bge asset");
        }
    }

    #[test]
    fn validation_manifest_matcher_accepts_dynamic_build_manifest_fields() {
        let expected = serde_json::to_string_pretty(&json!({
            "schema_version": "1.0",
            "build_id": "preview-usbrain",
            "build_mode": "offline-first",
            "languages": ["pt-BR", "en", "es"],
            "profile_ref": "manifests/usbrain-profile.json",
            "execution_policy_ref": "manifests/execution-policy.json",
            "installed_models_ref": "manifests/installed-models.json",
            "installed_packs_ref": "manifests/installed-packs.json",
            "runtime_config_ref": "manifests/runtime-config.json",
            "provider_config_ref": "manifests/provider-config.json",
            "validation_report_ref": "manifests/validation-report.json",
            "build_timestamp": "2026-06-30T20:00:00Z",
            "planning_hints": {
                "available_gb": 56.7,
                "usage_percent": 2.0
            }
        }))
        .expect("serialize expected build manifest");
        let actual = serde_json::to_string_pretty(&json!({
            "schema_version": "1.0",
            "build_id": "preview-usbrain",
            "build_mode": "offline-first",
            "languages": ["pt-BR", "en", "es"],
            "profile_ref": "manifests/usbrain-profile.json",
            "execution_policy_ref": "manifests/execution-policy.json",
            "installed_models_ref": "manifests/installed-models.json",
            "installed_packs_ref": "manifests/installed-packs.json",
            "runtime_config_ref": "manifests/runtime-config.json",
            "provider_config_ref": "manifests/provider-config.json",
            "validation_report_ref": "manifests/validation-report.json",
            "build_timestamp": "2026-06-30T20:05:00Z",
            "planning_hints": {
                "available_gb": 12.3,
                "usage_percent": 81.4
            }
        }))
        .expect("serialize actual build manifest");

        assert!(manifest_matches_validation_expectation("build-manifest.json", &expected, &actual));
    }

    #[test]
    fn validation_manifest_matcher_accepts_dynamic_validation_report_timestamp() {
        let expected = serde_json::to_string_pretty(&json!({
            "schema_version": "1.0",
            "validation_status": "passed-with-warnings",
            "validated_at": "2026-06-30T20:00:00Z",
            "builder_version": "0.1.0"
        }))
        .expect("serialize expected validation report");
        let actual = serde_json::to_string_pretty(&json!({
            "schema_version": "1.0",
            "validation_status": "passed-with-warnings",
            "validated_at": "2026-06-30T20:10:00Z",
            "builder_version": "0.1.0"
        }))
        .expect("serialize actual validation report");

        assert!(manifest_matches_validation_expectation("validation-report.json", &expected, &actual));
    }

    #[test]
    fn validation_manifest_matcher_accepts_dynamic_usbrain_profile_fields() {
        let expected = serde_json::to_string_pretty(&json!({
            "schema_version": "1.0",
            "profile_id": "preview-usbrain-profile",
            "name": "USBRAIN USBRAIN",
            "primary_language": "pt-BR",
            "supported_languages": ["pt-BR", "en", "es"],
            "mode": "offline",
            "privacy_mode": "local-first",
            "gpu_preference": "optional",
            "personality_profile": {
                "id": "usbrain-core-assistant",
                "source": "builtin",
                "version": "1.0.0"
            },
            "agent_profile": {
                "id": "portable-rag-agent",
                "source": "builtin",
                "version": "1.0.0"
            },
            "description": "Profile generated by USBRAIN Builder.",
            "default_pack_collection": ["pack-core-runtime", "pack-semantic-bge", "pack-multilingual-assist"],
            "owner_label": "Fabiano Costa"
        }))
        .expect("serialize expected profile");
        let actual = serde_json::to_string_pretty(&json!({
            "schema_version": "1.0",
            "profile_id": "preview-usbrain-profile",
            "name": "USBRAIN USBRAIN",
            "primary_language": "pt-BR",
            "supported_languages": ["pt-BR", "en", "es"],
            "mode": "offline",
            "privacy_mode": "local-first",
            "gpu_preference": "optional",
            "personality_profile": {
                "id": "usbrain-core-assistant",
                "source": "builtin",
                "version": "1.0.0"
            },
            "agent_profile": {
                "id": "portable-rag-agent",
                "source": "builtin",
                "version": "1.0.0"
            },
            "description": "Profile generated by USBRAIN Builder.",
            "default_pack_collection": ["pack-core-runtime", "pack-semantic-bge", "pack-multilingual-assist"],
            "owner_label": "Fabiano"
        }))
        .expect("serialize actual profile");

        assert!(manifest_matches_validation_expectation("usbrain-profile.json", &expected, &actual));
    }

    #[test]
    fn appledouble_compileall_error_only_accepts_only_ignored_paths() {
        let error = "*** Error compiling '/Volumes/USBRAIN/rag/._server.py'...\nValueError: source code string cannot contain null bytes\n*** Error compiling '/Volumes/USBRAIN/webui/._server.py'...\nValueError: source code string cannot contain null bytes";
        assert!(appledouble_compileall_error_only(error));
    }

    #[test]
    fn http_get_text_accepts_response_with_content_length_before_socket_close() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
        let port = listener.local_addr().expect("read local addr").port();

        let server = thread::spawn(move || {
            let (mut socket, _) = listener.accept().expect("accept test client");
            let mut request_buffer = [0u8; 1024];
            let _ = socket.read(&mut request_buffer);
            socket
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 16\r\nConnection: keep-alive\r\n\r\n{\"status\":\"ok\"}",
                )
                .expect("write HTTP response");
            thread::sleep(Duration::from_millis(250));
        });

        let body = http_get_text("127.0.0.1", port, "/api/stats", Duration::from_millis(100))
            .expect("http_get_text should accept a complete response before socket close");
        assert_eq!(body, "{\"status\":\"ok\"}");

        server.join().expect("join test server");
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn run_python_smoke_command_supports_shell_wrapped_python_fallback() {
        use std::os::unix::fs::PermissionsExt;

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-shell-python-wrapper-{}", unique));
        fs::create_dir_all(&test_root).expect("create temp test root");

        let fake_python = test_root.join("python3");
        fs::write(&fake_python, "#!/bin/sh\nprintf 'Python 3.12.0'\n")
            .expect("write fake python");
        let mut permissions = fs::metadata(&fake_python).expect("read fake python metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_python, permissions).expect("chmod fake python");

        let wrapper = format!("exec {} \"$@\"", fake_python.display());
        let prefix_args = vec!["-lc".to_string(), wrapper, "python3".to_string()];
        let output =
            run_python_smoke_command("/bin/sh", &prefix_args, &["--version"], &test_root).expect("shell wrapper should execute fake python");
        assert_eq!(output, "Python 3.12.0");

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn detect_bundled_macos_python_command_prefers_embedded_runtime_layout() {
        use std::os::unix::fs::PermissionsExt;

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-bundled-python-runtime-{}", unique));
        let python_root = test_root.join("python-macos");
        let bin_dir = python_root.join("bin");
        fs::create_dir_all(&bin_dir).expect("create bundled python bin dir");

        let fake_python = bin_dir.join("python3.10");
        fs::write(&fake_python, "#!/bin/sh\nprintf 'Python 3.10.20'\n").expect("write fake bundled python");
        let mut permissions = fs::metadata(&fake_python)
            .expect("read fake bundled python metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake_python, permissions).expect("chmod fake bundled python");

        let roots = vec![python_root.clone()];
        let command =
            detect_bundled_macos_python_command_in_roots(&roots).expect("bundled python should be detected");
        assert_eq!(command.0, fake_python.display().to_string());
        assert!(command.1.is_empty());
        assert_eq!(python_command_source_in_roots(&command.0, &roots), "bundled-macos");

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn python_command_env_sets_pythonhome_for_bundled_runtime() {
        let roots = vec![PathBuf::from("/tmp/usbrain-python-macos")];
        let program = "/tmp/usbrain-python-macos/bin/python3.10";

        assert_eq!(python_command_source_in_roots(program, &roots), "bundled-macos");
        assert_eq!(
            bundled_macos_python_root_for_program_in_roots(program, &roots),
            Some(PathBuf::from("/tmp/usbrain-python-macos"))
        );
        assert_eq!(
            python_command_env_in_roots(program, &roots),
            vec![(
                "PYTHONHOME".to_string(),
                "/tmp/usbrain-python-macos".to_string()
            )]
        );
    }

    #[test]
    fn export_build_plan_writes_manifest_bundle() {
        let request = ManifestPreviewRequest {
            target_label: Some("USBRAIN Test Export".to_string()),
            ..Default::default()
        };
        let build_plan = build_plan(request);
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-test-{}", unique));
        let output_dir = test_root.join("output");

        let export = export_build_plan_to_dir(&build_plan, &output_dir).expect("export should succeed");

        assert!(Path::new(&export.export_root).exists());
        assert!(Path::new(&export.summary_file).exists());
        assert!(export.exported_files.iter().any(|file| file.file_name == "build-plan.json"));
        assert!(export.exported_files.iter().any(|file| file.file_name == "build-manifest.json"));
        assert!(export.exported_files.len() >= build_plan.manifests.len() + 1);

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn assemble_build_plan_writes_runtime_structure() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-assembly-test-{}", unique));
        let app_root = test_root.join("app");
        let cache_root = test_root.join("cache");
        let target_root = test_root.join("target");

        fs::create_dir_all(app_root.join("tools")).expect("create app tools");
        fs::create_dir_all(app_root.join("skills")).expect("create app skills");
        fs::create_dir_all(app_root.join("rag")).expect("create app rag");
        fs::create_dir_all(app_root.join("webui").join("templates")).expect("create app webui");
        fs::create_dir_all(cache_root.join("bge-m3-model")).expect("create bge cache");
        fs::create_dir_all(cache_root.join("python")).expect("create python cache");
        fs::create_dir_all(&target_root).expect("create target root");
        write_fake_llamafile_binaries(&app_root);
        write_fake_llamafile_binaries(&app_root);

        for file_name in ["main.py", "agent.py", "personality.md", "requirements.txt", "README.txt", "start.bat"] {
            fs::write(app_root.join(file_name), format!("test for {}", file_name)).expect("write app file");
        }
        fs::write(app_root.join("tools").join("registry.py"), "tool-registry").expect("write tool");
        fs::write(app_root.join("skills").join("files.py"), "skill-files").expect("write skill");
        fs::write(app_root.join("rag").join("server.py"), "rag-server").expect("write rag");
        fs::write(app_root.join("webui").join("server.py"), "webui-server").expect("write webui");
        fs::write(app_root.join("webui").join("templates").join("index.html"), "<html>ok</html>")
            .expect("write webui template");
        fs::write(cache_root.join("llamafile.exe"), "llama").expect("write llamafile");
        fs::write(cache_root.join("python_runtime.zip"), "zip").expect("write runtime zip");
        fs::write(cache_root.join("python").join("python.exe"), "python").expect("write python exe");
        fs::write(cache_root.join("python").join("python3.dll"), "python3").expect("write python3 dll");
        fs::write(cache_root.join("python").join("python312.dll"), "python312").expect("write python312 dll");
        fs::write(cache_root.join("python").join("vcruntime140.dll"), "vcruntime").expect("write vcruntime");
        fs::create_dir_all(cache_root.join("python").join("DLLs")).expect("create python DLLs");
        fs::create_dir_all(cache_root.join("python").join("Lib").join("site-packages").join("aiohttp"))
            .expect("create aiohttp");
        fs::create_dir_all(cache_root.join("python").join("Lib").join("site-packages").join("httpx"))
            .expect("create httpx");
        fs::create_dir_all(cache_root.join("python").join("Lib").join("site-packages").join("numpy"))
            .expect("create numpy");
        fs::create_dir_all(cache_root.join("python").join("Lib").join("site-packages").join("pypdf"))
            .expect("create pypdf");
        fs::create_dir_all(cache_root.join("python").join("Lib").join("site-packages").join("sentence_transformers"))
            .expect("create sentence_transformers");
        fs::create_dir_all(cache_root.join("python").join("Lib").join("site-packages").join("transformers"))
            .expect("create transformers");
        fs::create_dir_all(cache_root.join("python").join("Lib").join("site-packages").join("torch"))
            .expect("create torch");
        fs::create_dir_all(cache_root.join("python").join("Lib").join("site-packages").join("sklearn").join(".libs"))
            .expect("create sklearn libs");
        fs::write(cache_root.join("python").join("DLLs").join("_ctypes.pyd"), "ctypes").expect("write ctypes");
        fs::write(cache_root.join("python").join("DLLs").join("libffi-8.dll"), "libffi").expect("write libffi");
        fs::write(cache_root.join("python").join("Lib").join("site.py"), "site").expect("write site.py");
        fs::write(
            cache_root.join("python").join("Lib").join("site-packages").join("aiohttp").join("__init__.py"),
            "aiohttp",
        )
        .expect("write aiohttp");
        fs::write(
            cache_root.join("python").join("Lib").join("site-packages").join("httpx").join("__init__.py"),
            "httpx",
        )
        .expect("write httpx");
        fs::write(
            cache_root.join("python").join("Lib").join("site-packages").join("numpy").join("__init__.py"),
            "numpy",
        )
        .expect("write numpy");
        fs::write(
            cache_root.join("python").join("Lib").join("site-packages").join("pypdf").join("__init__.py"),
            "pypdf",
        )
        .expect("write pypdf");
        fs::write(
            cache_root.join("python").join("Lib").join("site-packages").join("sentence_transformers").join("__init__.py"),
            "sentence_transformers",
        )
        .expect("write sentence_transformers");
        fs::write(
            cache_root.join("python").join("Lib").join("site-packages").join("transformers").join("__init__.py"),
            "transformers",
        )
        .expect("write transformers");
        fs::write(
            cache_root.join("python").join("Lib").join("site-packages").join("torch").join("__init__.py"),
            "torch",
        )
        .expect("write torch");
        fs::write(
            cache_root.join("python").join("Lib").join("site-packages").join("sklearn").join("__init__.py"),
            "sklearn",
        )
        .expect("write sklearn");
        fs::write(
            cache_root
                .join("python")
                .join("Lib")
                .join("site-packages")
                .join("sklearn")
                .join(".libs")
                .join("vcomp140.dll"),
            "vcomp",
        )
        .expect("write vcomp");
        fs::write(
            cache_root
                .join("python")
                .join("Lib")
                .join("site-packages")
                .join("sklearn")
                .join(".libs")
                .join("msvcp140.dll"),
            "msvcp",
        )
        .expect("write msvcp");
        write_fake_bge_assets(&cache_root);
        let model_source = test_root.join("model.gguf");
        fs::write(&model_source, "gguf").expect("write model");

        let build_plan = BuildPlanResponse {
            summary: BuildPlanSummary {
                build_id: "test-build".to_string(),
                target_label: "USBRAIN Test".to_string(),
                target_mount_path: target_root.display().to_string(),
                filesystem: "exfat".to_string(),
                build_mode: "offline-first".to_string(),
                profile_mode: "offline".to_string(),
                privacy_mode: "local-first".to_string(),
                capacity_gb: Some(64.0),
                available_gb: Some(64.0),
                install_budget_gb: Some(52.0),
                estimated_model_slots: 2,
                selected_model_count: 1,
                pending_model_download_count: 0,
                selected_pack_count: 1,
                allow_gpu: false,
                allow_cloud_fallback: false,
                primary_language: "pt-BR".to_string(),
                languages: vec!["pt-BR".to_string()],
                warnings: vec![],
            },
            models: vec![PlannedModel {
                installed_model_id: "test-model".to_string(),
                display_name: "Test Model".to_string(),
                role: "primary".to_string(),
                disk_size_gb: 1.0,
                recommended_ram_gb: 8.0,
                runtime_family: "llama.cpp".to_string(),
                source_file_name: "model.gguf".to_string(),
                source_absolute_path: model_source.display().to_string(),
                relative_target_path: "models/model.gguf".to_string(),
                available_locally: true,
                availability_status: "available-local".to_string(),
                availability_label: "Biblioteca local".to_string(),
            }],
            pending_model_downloads: vec![],
            host_ram_recommendations: vec![],
            packs: vec![PlannedPack {
                installed_pack_id: "pack-core-runtime".to_string(),
                name: "USBRAIN Core Runtime Pack".to_string(),
                pack_type: "behavior".to_string(),
                activation_mode: "always".to_string(),
                category: "core".to_string(),
            }],
            manifests: vec![GeneratedManifest {
                file_name: "build-manifest.json".to_string(),
                title: "Build Manifest".to_string(),
                contents: "{\n  \"ok\": true\n}".to_string(),
            }],
        };

        let result = assemble_build_plan_with_sources(&build_plan, &target_root, &app_root, &cache_root)
            .expect("assembly should succeed");

        assert!(target_root.join("main.py").exists());
        assert!(target_root.join("tools").join("registry.py").exists());
        assert!(target_root.join("webui").join("templates").join("index.html").exists());
        assert!(target_root.join("rag").join("bge-m3-model").join("pytorch_model.bin").exists());
        assert!(target_root.join("models").join("model.gguf").exists());
        assert!(target_root.join("manifests").join("build-manifest.json").exists());
        assert!(target_root.join(BUILDER_MARKER_FILE).exists());
        assert!(target_root.join("python").join("Lib").join("site.py").exists());
        assert!(target_root.join("python").join("DLLs").join("_ctypes.pyd").exists());
        assert!(target_root
            .join("python")
            .join("Lib")
            .join("site-packages")
            .join("transformers")
            .join("__init__.py")
            .exists());
        assert!(target_root.join("python_runtime.zip").exists());
        assert_eq!(result.build_id, "test-build");

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn validation_rejects_flattened_python_runtime_layout() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-flat-python-validation-{}", unique));
        let app_root = test_root.join("app");
        let cache_root = test_root.join("cache");
        let target_root = test_root.join("target");

        fs::create_dir_all(app_root.join("tools")).expect("create app tools");
        fs::create_dir_all(app_root.join("skills")).expect("create app skills");
        fs::create_dir_all(app_root.join("rag")).expect("create app rag");
        fs::create_dir_all(app_root.join("webui")).expect("create app webui");
        fs::create_dir_all(cache_root.join("python").join("Lib").join("site-packages").join("transformers"))
            .expect("create cache transformers");
        fs::create_dir_all(cache_root.join("python").join("DLLs")).expect("create cache DLLs");
        fs::create_dir_all(target_root.join("python")).expect("create target python");
        fs::create_dir_all(target_root.join("tools")).expect("create target tools");
        fs::create_dir_all(target_root.join("skills")).expect("create target skills");
        fs::create_dir_all(target_root.join("rag").join("bge-m3-model")).expect("create target bge");
        fs::create_dir_all(target_root.join("webui")).expect("create target webui");
        fs::create_dir_all(target_root.join("manifests")).expect("create target manifests");
        fs::create_dir_all(target_root.join("models")).expect("create target models");
        fs::create_dir_all(target_root.join("packs")).expect("create target packs");
        fs::create_dir_all(target_root.join("runtime").join("logs")).expect("create target runtime logs");

        for file_name in ["main.py", "agent.py", "personality.md", "requirements.txt", "README.txt", "start.bat"] {
            fs::write(app_root.join(file_name), format!("test for {}", file_name)).expect("write app file");
            fs::write(target_root.join(file_name), format!("target {}", file_name)).expect("write target file");
        }

        fs::write(cache_root.join("python_runtime.zip"), "zip").expect("write runtime zip");
        fs::write(cache_root.join("python").join("python.exe"), "python").expect("write cache python exe");
        fs::write(cache_root.join("python").join("python3.dll"), "python3").expect("write cache python3");
        fs::write(cache_root.join("python").join("python312.dll"), "python312").expect("write cache python312");
        fs::write(cache_root.join("python").join("vcruntime140.dll"), "vcruntime").expect("write cache vcruntime");
        fs::write(cache_root.join("python").join("DLLs").join("_ctypes.pyd"), "ctypes").expect("write cache ctypes");
        fs::write(cache_root.join("python").join("DLLs").join("libffi-8.dll"), "libffi").expect("write cache libffi");
        fs::write(cache_root.join("python").join("Lib").join("site.py"), "site").expect("write cache site");
        fs::write(
            cache_root.join("python").join("Lib").join("site-packages").join("transformers").join("__init__.py"),
            "transformers",
        )
        .expect("write cache transformers init");

        fs::write(target_root.join(BUILDER_MARKER_FILE), "{}").expect("write marker");
        fs::write(target_root.join("python").join("python.exe"), "python").expect("write target python exe");
        fs::write(target_root.join("python").join("python3.dll"), "python3").expect("write target python3");
        fs::write(target_root.join("python").join("python312.dll"), "python312").expect("write target python312");
        fs::write(target_root.join("python").join("vcruntime140.dll"), "vcruntime").expect("write target vcruntime");
        fs::write(target_root.join("python").join("site.py"), "site").expect("write flattened site");
        fs::write(target_root.join("python").join("_ctypes.pyd"), "ctypes").expect("write flattened ctypes");

        let build_plan = BuildPlanResponse {
            summary: BuildPlanSummary {
                build_id: "test-build".to_string(),
                target_label: "USBRAIN Test".to_string(),
                target_mount_path: target_root.display().to_string(),
                filesystem: "exfat".to_string(),
                build_mode: "offline-first".to_string(),
                profile_mode: "offline".to_string(),
                privacy_mode: "local-first".to_string(),
                capacity_gb: Some(64.0),
                available_gb: Some(64.0),
                install_budget_gb: Some(52.0),
                estimated_model_slots: 0,
                selected_model_count: 0,
                pending_model_download_count: 0,
                selected_pack_count: 0,
                allow_gpu: false,
                allow_cloud_fallback: false,
                primary_language: "pt-BR".to_string(),
                languages: vec!["pt-BR".to_string()],
                warnings: vec![],
            },
            models: vec![],
            pending_model_downloads: vec![],
            host_ram_recommendations: vec![],
            packs: vec![],
            manifests: vec![],
        };

        let validation = validate_assembled_target_with_sources(&build_plan, &target_root, &app_root, &cache_root);
        assert_eq!(validation.validation_status, "failed");
        assert!(validation
            .errors
            .iter()
            .any(|error| error.contains("Runtime Python Windows incompleto")));
        assert!(validation
            .errors
            .iter()
            .any(|error| error.contains("Runtime Python achatado")));

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn candidate_local_model_filter_ignores_appledouble_entries() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-model-filter-test-{}", unique));
        fs::create_dir_all(&test_root).expect("create temp test root");

        let valid = test_root.join("Phi-3.5-mini-instruct-Q4_K_M.gguf");
        let appledouble = test_root.join("._Phi-3.5-mini-instruct-Q4_K_M.gguf");
        let wrong_extension = test_root.join("model.bin");
        fs::write(&valid, "gguf").expect("write valid gguf");
        fs::write(&appledouble, "metadata").expect("write appledouble");
        fs::write(&wrong_extension, "bin").expect("write wrong extension");

        assert!(is_candidate_local_model_path(&valid));
        assert!(!is_candidate_local_model_path(&appledouble));
        assert!(!is_candidate_local_model_path(&wrong_extension));

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn assembly_removes_stale_models_from_managed_target_before_copying_selection() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-clean-models-test-{}", unique));
        let app_root = test_root.join("app");
        let cache_root = test_root.join("cache");
        let target_root = test_root.join("target");

        fs::create_dir_all(app_root.join("tools")).expect("create app tools");
        fs::create_dir_all(app_root.join("skills")).expect("create app skills");
        fs::create_dir_all(app_root.join("rag")).expect("create app rag");
        fs::create_dir_all(app_root.join("webui").join("templates")).expect("create app webui");
        fs::create_dir_all(cache_root.join("bge-m3-model")).expect("create bge cache");
        fs::create_dir_all(cache_root.join("python")).expect("create python cache");
        fs::create_dir_all(target_root.join("models")).expect("create target models");

        fs::write(app_root.join("main.py"), "print('ok')\n").expect("write main");
        fs::write(app_root.join("agent.py"), "class USBAgent:\n    pass\n").expect("write agent");
        fs::write(app_root.join("personality.md"), "# personality").expect("write personality");
        fs::write(app_root.join("requirements.txt"), "aiohttp").expect("write requirements");
        fs::write(app_root.join("README.txt"), "USBRAIN").expect("write readme");
        fs::write(app_root.join("start.bat"), "@echo off\r\npython main.py\r\n").expect("write bat");
        fs::write(app_root.join("start.sh"), "#!/bin/sh\npython3 main.py\n").expect("write sh");
        fs::write(app_root.join("tools").join("registry.py"), "REGISTRY = {}\n").expect("write tool");
        fs::write(app_root.join("skills").join("files.py"), "def ping():\n    return 'ok'\n").expect("write skill");
        fs::write(app_root.join("rag").join("server.py"), "def run():\n    return 'ok'\n").expect("write rag");
        fs::write(app_root.join("webui").join("server.py"), "def run():\n    return 'ok'\n").expect("write webui");
        fs::write(app_root.join("webui").join("templates").join("index.html"), "<html>ok</html>")
            .expect("write webui template");
        fs::write(cache_root.join("llamafile.exe"), "llama").expect("write llamafile");
        fs::write(cache_root.join("python_runtime.zip"), "zip").expect("write runtime zip");
        fs::write(cache_root.join("python").join("python.exe"), "python").expect("write python exe");
        write_fake_bge_assets(&cache_root);

        fs::write(target_root.join(BUILDER_MARKER_FILE), "{}").expect("write builder marker");
        fs::write(target_root.join("models").join("Phi-3.5-mini-instruct-Q4_K_M.gguf"), "stale")
            .expect("write stale model");
        fs::write(target_root.join("models").join("qwen2.5-1.5b-instruct-q4_k_m.gguf"), "stale")
            .expect("write another stale model");

        let selected_model_source = test_root.join("selected-model.gguf");
        fs::write(&selected_model_source, "fresh").expect("write selected model");

        let build_plan = BuildPlanResponse {
            summary: BuildPlanSummary {
                build_id: "test-build".to_string(),
                target_label: "USBRAIN Test".to_string(),
                target_mount_path: target_root.display().to_string(),
                filesystem: "exfat".to_string(),
                build_mode: "offline-first".to_string(),
                profile_mode: "offline".to_string(),
                privacy_mode: "local-first".to_string(),
                capacity_gb: Some(64.0),
                available_gb: Some(64.0),
                install_budget_gb: Some(52.0),
                estimated_model_slots: 2,
                selected_model_count: 1,
                pending_model_download_count: 0,
                selected_pack_count: 1,
                allow_gpu: false,
                allow_cloud_fallback: false,
                primary_language: "pt-BR".to_string(),
                languages: vec!["pt-BR".to_string()],
                warnings: vec![],
            },
            models: vec![PlannedModel {
                installed_model_id: "selected-model".to_string(),
                display_name: "Selected Model".to_string(),
                role: "manual-selection".to_string(),
                disk_size_gb: 1.0,
                recommended_ram_gb: 8.0,
                runtime_family: "llama.cpp".to_string(),
                source_file_name: "selected-model.gguf".to_string(),
                source_absolute_path: selected_model_source.display().to_string(),
                relative_target_path: "models/selected-model.gguf".to_string(),
                available_locally: true,
                availability_status: "available-local".to_string(),
                availability_label: "Biblioteca local".to_string(),
            }],
            pending_model_downloads: vec![],
            host_ram_recommendations: vec![],
            packs: vec![PlannedPack {
                installed_pack_id: "pack-core-runtime".to_string(),
                name: "USBRAIN Core Runtime Pack".to_string(),
                pack_type: "behavior".to_string(),
                activation_mode: "always".to_string(),
                category: "core".to_string(),
            }],
            manifests: vec![GeneratedManifest {
                file_name: "build-manifest.json".to_string(),
                title: "Build Manifest".to_string(),
                contents: "{\n  \"ok\": true\n}".to_string(),
            }],
        };

        assemble_build_plan_with_sources(&build_plan, &target_root, &app_root, &cache_root)
            .expect("assembly should succeed");

        let models_dir = target_root.join("models");
        assert!(models_dir.join("selected-model.gguf").exists());
        assert!(!models_dir.join("Phi-3.5-mini-instruct-Q4_K_M.gguf").exists());
        assert!(!models_dir.join("qwen2.5-1.5b-instruct-q4_k_m.gguf").exists());

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn assembly_replaces_stale_python_runtime_contents_without_removing_root() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-clean-python-test-{}", unique));
        let app_root = test_root.join("app");
        let cache_root = test_root.join("cache");
        let target_root = test_root.join("target");

        fs::create_dir_all(app_root.join("tools")).expect("create app tools");
        fs::create_dir_all(app_root.join("skills")).expect("create app skills");
        fs::create_dir_all(app_root.join("rag")).expect("create app rag");
        fs::create_dir_all(app_root.join("webui").join("templates")).expect("create app webui");
        fs::create_dir_all(cache_root.join("python").join("Lib").join("site-packages").join("transformers"))
            .expect("create cache transformers");
        fs::create_dir_all(cache_root.join("python").join("DLLs")).expect("create cache DLLs");
        fs::create_dir_all(cache_root.join("bge-m3-model")).expect("create bge cache");
        fs::create_dir_all(target_root.join("python").join("Lib")).expect("create target stale python");

        fs::write(app_root.join("main.py"), "print('ok')\n").expect("write main");
        fs::write(app_root.join("agent.py"), "class USBAgent:\n    pass\n").expect("write agent");
        fs::write(app_root.join("personality.md"), "# personality").expect("write personality");
        fs::write(app_root.join("requirements.txt"), "aiohttp").expect("write requirements");
        fs::write(app_root.join("README.txt"), "USBRAIN").expect("write readme");
        fs::write(app_root.join("start.bat"), "@echo off\r\npython main.py\r\n").expect("write bat");
        fs::write(app_root.join("start.sh"), "#!/bin/sh\npython3 main.py\n").expect("write sh");
        fs::write(app_root.join("tools").join("registry.py"), "REGISTRY = {}\n").expect("write tool");
        fs::write(app_root.join("skills").join("files.py"), "def ping():\n    return 'ok'\n").expect("write skill");
        fs::write(app_root.join("rag").join("server.py"), "def run():\n    return 'ok'\n").expect("write rag");
        fs::write(app_root.join("webui").join("server.py"), "def run():\n    return 'ok'\n").expect("write webui");
        fs::write(app_root.join("webui").join("templates").join("index.html"), "<html>ok</html>")
            .expect("write webui template");

        fs::write(cache_root.join("llamafile.exe"), "llama").expect("write llamafile");
        fs::write(cache_root.join("python_runtime.zip"), "zip").expect("write runtime zip");
        fs::write(cache_root.join("python").join("python.exe"), "python").expect("write python exe");
        fs::write(cache_root.join("python").join("python3.dll"), "python3").expect("write python3");
        fs::write(cache_root.join("python").join("python312.dll"), "python312").expect("write python312");
        fs::write(cache_root.join("python").join("vcruntime140.dll"), "vcruntime").expect("write vcruntime");
        fs::write(cache_root.join("python").join("DLLs").join("_ctypes.pyd"), "ctypes").expect("write ctypes");
        fs::write(cache_root.join("python").join("DLLs").join("libffi-8.dll"), "libffi").expect("write libffi");
        fs::write(cache_root.join("python").join("Lib").join("site.py"), "site").expect("write site");
        fs::write(
            cache_root.join("python").join("Lib").join("site-packages").join("transformers").join("__init__.py"),
            "transformers",
        )
        .expect("write transformers");
        write_fake_bge_assets(&cache_root);

        fs::write(target_root.join(BUILDER_MARKER_FILE), "{}").expect("write marker");
        fs::write(target_root.join("python").join("stale-root.txt"), "stale").expect("write stale root");
        fs::write(target_root.join("python").join("Lib").join("obsolete.py"), "obsolete").expect("write stale nested");
        fs::write(target_root.join("python").join("._transient"), "ignore").expect("write ignored noise");

        let build_plan = BuildPlanResponse {
            summary: BuildPlanSummary {
                build_id: "test-build".to_string(),
                target_label: "USBRAIN Test".to_string(),
                target_mount_path: target_root.display().to_string(),
                filesystem: "exfat".to_string(),
                build_mode: "offline-first".to_string(),
                profile_mode: "offline".to_string(),
                privacy_mode: "local-first".to_string(),
                capacity_gb: Some(64.0),
                available_gb: Some(64.0),
                install_budget_gb: Some(52.0),
                estimated_model_slots: 0,
                selected_model_count: 0,
                pending_model_download_count: 0,
                selected_pack_count: 1,
                allow_gpu: false,
                allow_cloud_fallback: false,
                primary_language: "pt-BR".to_string(),
                languages: vec!["pt-BR".to_string()],
                warnings: vec![],
            },
            models: vec![],
            pending_model_downloads: vec![],
            host_ram_recommendations: vec![],
            packs: vec![PlannedPack {
                installed_pack_id: "pack-core-runtime".to_string(),
                name: "USBRAIN Core Runtime Pack".to_string(),
                pack_type: "behavior".to_string(),
                activation_mode: "always".to_string(),
                category: "core".to_string(),
            }],
            manifests: vec![GeneratedManifest {
                file_name: "build-manifest.json".to_string(),
                title: "Build Manifest".to_string(),
                contents: "{\n  \"ok\": true\n}".to_string(),
            }],
        };

        assemble_build_plan_with_sources(&build_plan, &target_root, &app_root, &cache_root)
            .expect("assembly should succeed");

        assert!(target_root.join("python").exists());
        assert!(target_root.join("python").join("Lib").join("site.py").exists());
        assert!(target_root.join("python").join("DLLs").join("_ctypes.pyd").exists());
        assert!(!target_root.join("python").join("stale-root.txt").exists());
        assert!(!target_root.join("python").join("Lib").join("obsolete.py").exists());

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn wipe_directory_contents_preserving_root_removes_dense_directory_entries() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-wipe-dense-dir-{}", unique));
        let dense_dir = test_root.join("python").join("Lib").join("site-packages").join("joblib").join("test").join("data");

        fs::create_dir_all(&dense_dir).expect("create dense directory");
        for file_name in [
            "joblib_0.9.2_pickle_py35_np19.pkl",
            "joblib_0.9.2_pickle_py35_np19.pkl_01.npy",
            "joblib_0.9.2_pickle_py35_np19.pkl_02.npy",
            "joblib_0.9.2_pickle_py35_np19.pkl_03.npy",
            "joblib_0.9.2_pickle_py35_np19.pkl_04.npy",
            "joblib_0.9.4.dev0_compressed_cache_size_pickle_py35_np19.gz",
            "joblib_0.9.4.dev0_compressed_cache_size_pickle_py35_np19.gz_01.npy.z",
            "joblib_0.9.4.dev0_compressed_cache_size_pickle_py35_np19.gz_02.npy.z",
            "joblib_0.9.4.dev0_compressed_cache_size_pickle_py35_np19.gz_03.npy.z",
        ] {
            fs::write(dense_dir.join(file_name), "fixture").expect("write dense file");
        }

        wipe_directory_contents_preserving_root(&dense_dir).expect("wipe dense directory");

        let remaining: Vec<PathBuf> = fs::read_dir(&dense_dir)
            .expect("read dense directory")
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect();
        assert!(remaining.is_empty(), "dense directory should be empty after wipe");

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn planned_models_marks_missing_catalog_selection_as_pending_download() {
        let target = CandidateTarget {
            label: "USBRAIN".to_string(),
            mount_path: "/Volumes/USBRAIN".to_string(),
            source: "macos-usb:disk10s1".to_string(),
            filesystem: "exfat".to_string(),
            capacity_gb: Some(64.0),
            available_gb: Some(52.0),
            usage_percent: Some(18.0),
            removable: true,
            recommended_model_tier: "Build completo".to_string(),
            estimated_model_slots: 2,
            warnings: vec![],
        };
        let catalog_models = vec![CatalogModelRecord {
            model_id: "catalog-only-test-model".to_string(),
            display_name: "Catalog Only Test Model".to_string(),
            family: "test".to_string(),
            parameter_scale: "7B".to_string(),
            variant: "Q4_K_M".to_string(),
            disk_size_gb: 4.2,
            ram_min_gb: 8.0,
            ram_recommended_gb: 16.0,
            host_ram_tier_gb: 16,
            languages: vec!["pt-BR".to_string()],
            domains: vec!["general".to_string()],
            capabilities: vec!["chat".to_string()],
            quality_score: 80,
            cpu_only_viable: true,
            gpu_optional: true,
            license: "apache-2.0".to_string(),
            homepage_or_source: "https://example.com/catalog-only-test-model".to_string(),
            description: "Model available only via catalog download".to_string(),
            rationale: "Used to verify pending download planning.".to_string(),
            priority: 10,
            download_locations: vec!["https://example.com/catalog-only-test-model.gguf".to_string()],
            checksums: vec!["sha256:test-catalog-model".to_string()],
            provider_or_origin: None,
            variant_id: None,
            quantization: None,
            file_format: None,
            editorial_recommendation: None,
            benchmark_summary: None,
            benchmark_metrics: vec![],
            technical_profile: None,
            local_file_name: Some("catalog-only-test-model.gguf".to_string()),
        }];

        let (models, pending_downloads, warnings) = planned_models(
            &target,
            &["pt-BR".to_string()],
            false,
            &["catalog-only-test-model".to_string()],
            &catalog_models,
        );

        assert!(models.is_empty());
        assert_eq!(pending_downloads.len(), 1);
        assert_eq!(pending_downloads[0].model_id, "catalog-only-test-model");
        assert_eq!(pending_downloads[0].availability_status, "requires-download");
        assert_eq!(pending_downloads[0].availability_label, "Precisa baixar");
        assert!(warnings
            .iter()
            .any(|warning| warning.contains("download sob demanda sera necessario")));
    }

    #[test]
    fn prepare_builder_library_copies_local_model_and_writes_pending_manifest() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-library-test-{}", unique));
        let library_root = test_root.join("library");
        let model_source = test_root.join("legacy-model.gguf");
        fs::create_dir_all(&library_root).expect("create library root");
        fs::write(&model_source, "gguf").expect("write model source");

        let build_plan = BuildPlanResponse {
            summary: BuildPlanSummary {
                build_id: "test-build".to_string(),
                target_label: "USBRAIN Test".to_string(),
                target_mount_path: "/Volumes/USBRAIN".to_string(),
                filesystem: "exfat".to_string(),
                build_mode: "offline-first".to_string(),
                profile_mode: "offline".to_string(),
                privacy_mode: "local-first".to_string(),
                capacity_gb: Some(64.0),
                available_gb: Some(52.0),
                install_budget_gb: Some(52.0),
                estimated_model_slots: 2,
                selected_model_count: 1,
                pending_model_download_count: 1,
                selected_pack_count: 1,
                allow_gpu: false,
                allow_cloud_fallback: false,
                primary_language: "pt-BR".to_string(),
                languages: vec!["pt-BR".to_string()],
                warnings: vec![],
            },
            models: vec![PlannedModel {
                installed_model_id: "legacy-model".to_string(),
                display_name: "Legacy Model".to_string(),
                role: "manual-selection".to_string(),
                disk_size_gb: 1.0,
                recommended_ram_gb: 8.0,
                runtime_family: "llama.cpp".to_string(),
                source_file_name: "legacy-model.gguf".to_string(),
                source_absolute_path: model_source.display().to_string(),
                relative_target_path: "models/legacy-model.gguf".to_string(),
                available_locally: true,
                availability_status: "available-local".to_string(),
                availability_label: "Legado em models/".to_string(),
            }],
            pending_model_downloads: vec![PendingModelDownload {
                model_id: "catalog-only-test-model".to_string(),
                display_name: "Catalog Only Test Model".to_string(),
                disk_size_gb: 4.2,
                recommended_ram_gb: 16.0,
                homepage_or_source: "https://example.com/catalog-only-test-model".to_string(),
                download_locations: vec!["https://example.com/catalog-only-test-model.gguf".to_string()],
                checksums: vec!["sha256:test-catalog-model".to_string()],
                local_file_name: "catalog-only-test-model.gguf".to_string(),
                relative_target_path: "models/catalog-only-test-model.gguf".to_string(),
                availability_status: "requires-download".to_string(),
                availability_label: "Precisa baixar".to_string(),
            }],
            host_ram_recommendations: vec![],
            packs: vec![],
            manifests: vec![],
        };

        let result =
            prepare_builder_library_with_root(&build_plan, &library_root).expect("prepare library should succeed");

        assert_eq!(result.copied_models_count, 1);
        assert_eq!(result.written_manifests_count, 1);
        assert!(library_root.join("models").join("legacy-model.gguf").exists());
        assert!(library_root
            .join("manifests")
            .join("model-downloads")
            .join("catalog-only-test-model.json")
            .exists());

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn builder_library_inventory_and_delete_cover_models_and_pending_requests() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-library-inventory-test-{}", unique));
        let library_root = test_root.join("library");
        let models_dir = library_root.join("models");
        let manifests_dir = library_root.join("manifests").join("model-downloads");
        fs::create_dir_all(&models_dir).expect("create models dir");
        fs::create_dir_all(&manifests_dir).expect("create manifests dir");
        fs::write(models_dir.join("phi-3.5-mini.gguf"), "gguf").expect("write library model");
        fs::write(
            manifests_dir.join("qwen2-5-7b-q4.json"),
            manifest_json(&json!({
                "model_id": "qwen2-5-7b-q4",
                "display_name": "Qwen2.5 7B Q4",
                "homepage_or_source": "https://example.com/qwen2-5-7b-q4"
            })),
        )
        .expect("write pending manifest");

        let inventory = builder_library_inventory_with_root(&library_root).expect("inventory should succeed");
        assert_eq!(inventory.artifact_count, 2);
        assert!(inventory
            .artifacts
            .iter()
            .any(|artifact| artifact.artifact_type == "model" && artifact.artifact_id == "phi-3.5-mini.gguf"));
        assert!(inventory
            .artifacts
            .iter()
            .any(|artifact| artifact.artifact_type == "download-request" && artifact.artifact_id == "qwen2-5-7b-q4"));

        let delete_result = delete_builder_library_artifact_with_root(
            &library_root,
            DeleteBuilderLibraryArtifactRequest {
                artifact_type: "download-request".to_string(),
                artifact_id: "qwen2-5-7b-q4".to_string(),
            },
        )
        .expect("delete should succeed");

        assert_eq!(delete_result.artifact_type, "download-request");
        assert!(!manifests_dir.join("qwen2-5-7b-q4.json").exists());

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn download_builder_library_artifact_materializes_model_and_removes_manifest() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-library-download-test-{}", unique));
        let library_root = test_root.join("library");
        let manifests_dir = library_root.join("manifests").join("model-downloads");
        let models_dir = library_root.join("models");
        fs::create_dir_all(&manifests_dir).expect("create manifests dir");
        fs::create_dir_all(&models_dir).expect("create models dir");

        let payload = b"mock-gguf-contents";
        let payload_sha256 = {
            let mut hasher = Sha256::new();
            hasher.update(payload);
            format!("{:x}", hasher.finalize())
        };

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
        let port = listener.local_addr().expect("read local addr").port();
        let payload_len = payload.len();
        let server = thread::spawn(move || {
            let (mut socket, _) = listener.accept().expect("accept download client");
            let mut request_buffer = [0u8; 1024];
            let _ = socket.read(&mut request_buffer);
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                payload_len
            );
            socket.write_all(header.as_bytes()).expect("write HTTP header");
            socket.write_all(payload).expect("write HTTP body");
        });

        fs::write(
            manifests_dir.join("qwen2-5-7b-q4.json"),
            manifest_json(&json!({
                "model_id": "qwen2-5-7b-q4",
                "display_name": "Qwen2.5 7B Q4",
                "download_locations": [format!("http://127.0.0.1:{}/qwen2.gguf", port)],
                "checksums": [format!("sha256:{}", payload_sha256)],
                "local_file_name": "Qwen2.5-7B-Instruct-Q4_K_M.gguf"
            })),
        )
        .expect("write pending manifest");

        let result = download_builder_library_artifact_with_root(
            &library_root,
            DownloadBuilderLibraryArtifactRequest {
                artifact_id: "qwen2-5-7b-q4".to_string(),
            },
        )
        .expect("download should succeed");

        assert_eq!(result.status, "downloaded");
        assert!(models_dir.join("Qwen2.5-7B-Instruct-Q4_K_M.gguf").exists());
        assert!(!manifests_dir.join("qwen2-5-7b-q4.json").exists());
        assert_eq!(
            fs::read(models_dir.join("Qwen2.5-7B-Instruct-Q4_K_M.gguf")).expect("read downloaded model"),
            payload
        );

        server.join().expect("join test server");
        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn download_catalog_record_materializes_model_directly_from_catalog_metadata() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-catalog-direct-download-test-{}", unique));
        let library_root = test_root.join("library");
        fs::create_dir_all(&library_root).expect("create library root");

        let payload = b"mock-catalog-gguf";
        let payload_sha256 = {
            let mut hasher = Sha256::new();
            hasher.update(payload);
            format!("{:x}", hasher.finalize())
        };

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
        let port = listener.local_addr().expect("read local addr").port();
        let payload_len = payload.len();
        let server = thread::spawn(move || {
            let (mut socket, _) = listener.accept().expect("accept download client");
            let mut request_buffer = [0u8; 1024];
            let _ = socket.read(&mut request_buffer);
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                payload_len
            );
            socket.write_all(header.as_bytes()).expect("write HTTP header");
            socket.write_all(payload).expect("write HTTP body");
        });

        let result = download_catalog_record_with_root(
            &library_root,
            CatalogModelRecord {
                model_id: "qwen2-5-7b-instruct-q4".to_string(),
                display_name: "Qwen2.5 7B Instruct Q4".to_string(),
                family: "Qwen2.5".to_string(),
                parameter_scale: "7B".to_string(),
                variant: "Instruct GGUF Q4".to_string(),
                disk_size_gb: 4.7,
                ram_min_gb: 10.0,
                ram_recommended_gb: 16.0,
                host_ram_tier_gb: 16,
                languages: vec!["pt-BR".to_string()],
                domains: vec!["general".to_string()],
                capabilities: vec!["offline".to_string()],
                quality_score: 88,
                cpu_only_viable: true,
                gpu_optional: true,
                license: "apache-2.0".to_string(),
                homepage_or_source: "https://example.com/qwen2-5-7b-instruct-q4".to_string(),
                description: "Modelo de teste".to_string(),
                rationale: "Validar download direto a partir do catalogo.".to_string(),
                priority: 10,
                download_locations: vec![format!("http://127.0.0.1:{}/qwen2.gguf", port)],
                checksums: vec![format!("sha256:{}", payload_sha256)],
                provider_or_origin: Some("Teste".to_string()),
                variant_id: Some("qwen2.5-7b-instruct-q4-k-m".to_string()),
                quantization: Some("Q4_K_M".to_string()),
                file_format: Some("GGUF".to_string()),
                editorial_recommendation: None,
                benchmark_summary: None,
                benchmark_metrics: vec![],
                technical_profile: None,
                local_file_name: Some("Qwen2.5-7B-Instruct-Q4_K_M.gguf".to_string()),
            },
        )
        .expect("direct catalog download should succeed");

        assert_eq!(result.status, "downloaded");
        assert!(library_root
            .join("models")
            .join("Qwen2.5-7B-Instruct-Q4_K_M.gguf")
            .exists());
        assert!(!library_root
            .join("manifests")
            .join("model-downloads")
            .join("qwen2-5-7b-instruct-q4.json")
            .exists());

        server.join().expect("join test server");
        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn preflight_current_host_runtime_binary_accepts_shell_wrapped_binary() {
        use std::os::unix::fs::PermissionsExt;

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-preflight-test-{}", unique));
        fs::create_dir_all(&test_root).expect("create temp test root");

        let binary_path = test_root.join("llamafile");
        fs::write(&binary_path, "echo wrapped llamafile help\nexit 0\n").expect("write fake shell-wrapped binary");
        let mut permissions = fs::metadata(&binary_path).expect("read fake binary metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&binary_path, permissions).expect("chmod fake binary");

        let result = preflight_current_host_runtime_binary(&test_root).expect("preflight should accept shell wrapped binary");
        assert_eq!(result, Some(binary_path.display().to_string()));

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn validate_assembled_build_reports_pass_after_assembly() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-validation-test-{}", unique));
        let app_root = test_root.join("app");
        let cache_root = test_root.join("cache");
        let target_root = test_root.join("target");

        fs::create_dir_all(app_root.join("tools")).expect("create app tools");
        fs::create_dir_all(app_root.join("skills")).expect("create app skills");
        fs::create_dir_all(app_root.join("rag")).expect("create app rag");
        fs::create_dir_all(app_root.join("webui").join("templates")).expect("create app webui");
        fs::create_dir_all(cache_root.join("bge-m3-model")).expect("create bge cache");
        fs::create_dir_all(cache_root.join("python")).expect("create python cache");
        fs::create_dir_all(&target_root).expect("create target root");
        write_fake_llamafile_binaries(&app_root);
        write_fake_llamafile_binaries(&app_root);

        for file_name in [
            "main.py",
            "agent.py",
            "personality.md",
            "requirements.txt",
            "README.txt",
            "start.bat",
        ] {
            fs::write(app_root.join(file_name), format!("test for {}", file_name)).expect("write app file");
        }
        fs::write(app_root.join("tools").join("registry.py"), "tool-registry").expect("write tool");
        fs::write(app_root.join("skills").join("files.py"), "skill-files").expect("write skill");
        fs::write(app_root.join("rag").join("server.py"), "rag-server").expect("write rag");
        fs::write(app_root.join("webui").join("server.py"), "webui-server").expect("write webui");
        fs::write(app_root.join("webui").join("templates").join("index.html"), "<html>ok</html>")
            .expect("write webui template");
        fs::write(cache_root.join("llamafile.exe"), "llama").expect("write llamafile");
        fs::write(cache_root.join("python_runtime.zip"), "zip").expect("write runtime zip");
        fs::write(cache_root.join("python").join("python.exe"), "python").expect("write python exe");
        write_fake_bge_assets(&cache_root);
        let model_source = test_root.join("model.gguf");
        fs::write(&model_source, "gguf").expect("write model");

        let build_plan = BuildPlanResponse {
            summary: BuildPlanSummary {
                build_id: "test-build".to_string(),
                target_label: "USBRAIN Test".to_string(),
                target_mount_path: target_root.display().to_string(),
                filesystem: "exfat".to_string(),
                build_mode: "offline-first".to_string(),
                profile_mode: "offline".to_string(),
                privacy_mode: "local-first".to_string(),
                capacity_gb: Some(64.0),
                available_gb: Some(64.0),
                install_budget_gb: Some(52.0),
                estimated_model_slots: 2,
                selected_model_count: 1,
                pending_model_download_count: 0,
                selected_pack_count: 1,
                allow_gpu: false,
                allow_cloud_fallback: false,
                primary_language: "pt-BR".to_string(),
                languages: vec!["pt-BR".to_string()],
                warnings: vec![],
            },
            models: vec![PlannedModel {
                installed_model_id: "test-model".to_string(),
                display_name: "Test Model".to_string(),
                role: "primary".to_string(),
                disk_size_gb: 1.0,
                recommended_ram_gb: 8.0,
                runtime_family: "llama.cpp".to_string(),
                source_file_name: "model.gguf".to_string(),
                source_absolute_path: model_source.display().to_string(),
                relative_target_path: "models/model.gguf".to_string(),
                available_locally: true,
                availability_status: "available-local".to_string(),
                availability_label: "Biblioteca local".to_string(),
            }],
            pending_model_downloads: vec![],
            host_ram_recommendations: vec![],
            packs: vec![PlannedPack {
                installed_pack_id: "pack-core-runtime".to_string(),
                name: "USBRAIN Core Runtime Pack".to_string(),
                pack_type: "behavior".to_string(),
                activation_mode: "always".to_string(),
                category: "core".to_string(),
            }],
            manifests: vec![GeneratedManifest {
                file_name: "build-manifest.json".to_string(),
                title: "Build Manifest".to_string(),
                contents: "{\n  \"ok\": true\n}".to_string(),
            }],
        };

        assemble_build_plan_with_sources(&build_plan, &target_root, &app_root, &cache_root)
            .expect("assembly should succeed");
        let validation =
            validate_assembled_target_with_sources(&build_plan, &target_root, &app_root, &cache_root);

        assert_eq!(validation.validation_status, "passed");
        assert!(validation.errors.is_empty());

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn assembly_requires_confirmation_before_cleaning_unmanaged_target() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-confirm-clean-test-{}", unique));
        let target_root = test_root.join("target");

        fs::create_dir_all(&target_root).expect("create target root");
        fs::write(target_root.join("user-file.txt"), "keep or wipe").expect("write user file");

        let result = ensure_target_is_ready_for_assembly(&target_root, false);

        assert!(result.is_err());
        assert!(
            result
                .expect_err("should require confirmation")
                .contains("Confirme a limpeza completa do pendrive")
        );

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn assembly_can_clean_unmanaged_target_after_confirmation() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-clean-after-confirm-test-{}", unique));
        let target_root = test_root.join("target");

        fs::create_dir_all(target_root.join("old-folder")).expect("create nested folder");
        fs::write(target_root.join("user-file.txt"), "old data").expect("write user file");
        fs::write(target_root.join("old-folder").join("nested.txt"), "nested data").expect("write nested file");

        ensure_target_is_ready_for_assembly(&target_root, true).expect("cleaning after confirmation should succeed");

        assert!(!target_root.join("user-file.txt").exists());
        assert!(!target_root.join("old-folder").exists());
        assert!(target_meaningful_entries(&target_root).is_empty());

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn smoke_test_reports_pass_after_valid_assembly() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-smoke-test-{}", unique));
        let app_root = test_root.join("app");
        let cache_root = test_root.join("cache");
        let target_root = test_root.join("target");

        fs::create_dir_all(app_root.join("tools")).expect("create app tools");
        fs::create_dir_all(app_root.join("skills")).expect("create app skills");
        fs::create_dir_all(app_root.join("rag")).expect("create app rag");
        fs::create_dir_all(app_root.join("webui").join("templates")).expect("create app webui");
        fs::create_dir_all(cache_root.join("bge-m3-model")).expect("create bge cache");
        fs::create_dir_all(cache_root.join("python")).expect("create python cache");
        fs::create_dir_all(&target_root).expect("create target root");
        write_fake_llamafile_binaries(&app_root);
        write_fake_llamafile_binaries(&app_root);

        fs::write(
            app_root.join("main.py"),
            r#"import argparse

def main():
    parser = argparse.ArgumentParser(description="USBRAIN test runtime")
    parser.add_argument("--help-check", action="store_true")
    parser.parse_args()

if __name__ == "__main__":
    main()
"#,
        )
        .expect("write main");
        fs::write(
            app_root.join("agent.py"),
            r#"class USBAgent:
    pass
"#,
        )
        .expect("write agent");
        fs::write(app_root.join("personality.md"), "# personality").expect("write personality");
        fs::write(app_root.join("requirements.txt"), "aiohttp").expect("write requirements");
        fs::write(app_root.join("README.txt"), "USBRAIN").expect("write readme");
        fs::write(app_root.join("start.bat"), "@echo off\r\npython main.py --help\r\n").expect("write bat");
        fs::write(app_root.join("tools").join("registry.py"), "REGISTRY = {}\n").expect("write tool");
        fs::write(app_root.join("skills").join("files.py"), "def ping():\n    return 'ok'\n").expect("write skill");
        fs::write(app_root.join("rag").join("server.py"), "def run():\n    return 'ok'\n").expect("write rag");
        fs::write(app_root.join("webui").join("server.py"), "def run():\n    return 'ok'\n").expect("write webui");
        fs::write(app_root.join("webui").join("templates").join("index.html"), "<html>ok</html>")
            .expect("write webui template");
        fs::write(cache_root.join("llamafile.exe"), "llama").expect("write llamafile");
        fs::write(cache_root.join("python_runtime.zip"), "zip").expect("write runtime zip");
        fs::write(cache_root.join("python").join("python.exe"), "python").expect("write python exe");
        write_fake_bge_assets(&cache_root);
        let model_source = test_root.join("model.gguf");
        fs::write(&model_source, "gguf").expect("write model");

        let build_plan = BuildPlanResponse {
            summary: BuildPlanSummary {
                build_id: "test-build".to_string(),
                target_label: "USBRAIN Test".to_string(),
                target_mount_path: target_root.display().to_string(),
                filesystem: "exfat".to_string(),
                build_mode: "offline-first".to_string(),
                profile_mode: "offline".to_string(),
                privacy_mode: "local-first".to_string(),
                capacity_gb: Some(64.0),
                available_gb: Some(64.0),
                install_budget_gb: Some(52.0),
                estimated_model_slots: 2,
                selected_model_count: 1,
                pending_model_download_count: 0,
                selected_pack_count: 1,
                allow_gpu: false,
                allow_cloud_fallback: false,
                primary_language: "pt-BR".to_string(),
                languages: vec!["pt-BR".to_string()],
                warnings: vec![],
            },
            models: vec![PlannedModel {
                installed_model_id: "test-model".to_string(),
                display_name: "Test Model".to_string(),
                role: "primary".to_string(),
                disk_size_gb: 1.0,
                recommended_ram_gb: 8.0,
                runtime_family: "llama.cpp".to_string(),
                source_file_name: "model.gguf".to_string(),
                source_absolute_path: model_source.display().to_string(),
                relative_target_path: "models/model.gguf".to_string(),
                available_locally: true,
                availability_status: "available-local".to_string(),
                availability_label: "Biblioteca local".to_string(),
            }],
            pending_model_downloads: vec![],
            host_ram_recommendations: vec![],
            packs: vec![PlannedPack {
                installed_pack_id: "pack-core-runtime".to_string(),
                name: "USBRAIN Core Runtime Pack".to_string(),
                pack_type: "behavior".to_string(),
                activation_mode: "always".to_string(),
                category: "core".to_string(),
            }],
            manifests: vec![
                GeneratedManifest {
                    file_name: "build-manifest.json".to_string(),
                    title: "Build Manifest".to_string(),
                    contents: "{\n  \"ok\": true\n}".to_string(),
                },
                GeneratedManifest {
                    file_name: "runtime-config.json".to_string(),
                    title: "Runtime Config".to_string(),
                    contents: "{\n  \"runtime\": true\n}".to_string(),
                },
                GeneratedManifest {
                    file_name: "installed-models.json".to_string(),
                    title: "Installed Models".to_string(),
                    contents: "{\n  \"models\": []\n}".to_string(),
                },
            ],
        };

        assemble_build_plan_with_sources(&build_plan, &target_root, &app_root, &cache_root)
            .expect("assembly should succeed");
        let smoke = run_assembled_smoke_test_with_sources(&build_plan, &target_root, &app_root, &cache_root);

        assert_ne!(smoke.host_python_command, "indisponivel");
        assert_eq!(smoke.smoke_status, "passed");
        assert!(smoke.errors.is_empty());
        assert!(smoke.checks.iter().any(|check| check.id == "main-help" && check.status == "passed"));

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn runtime_probe_reports_pass_after_valid_assembly() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-runtime-probe-test-{}", unique));
        let app_root = test_root.join("app");
        let cache_root = test_root.join("cache");
        let target_root = test_root.join("target");

        fs::create_dir_all(app_root.join("tools")).expect("create app tools");
        fs::create_dir_all(app_root.join("skills")).expect("create app skills");
        fs::create_dir_all(app_root.join("rag")).expect("create app rag");
        fs::create_dir_all(app_root.join("webui").join("templates")).expect("create app webui");
        fs::create_dir_all(cache_root.join("bge-m3-model")).expect("create bge cache");
        fs::create_dir_all(cache_root.join("python")).expect("create python cache");
        fs::create_dir_all(&target_root).expect("create target root");

        fs::write(
            app_root.join("main.py"),
            r#"import argparse
import http.server
import json
import socketserver

def main():
    parser = argparse.ArgumentParser(description="USBRAIN probe runtime")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8000)
    parser.add_argument("--bge-port", type=int, default=38422)
    parser.add_argument("--llm-port", type=int, default=38421)
    parser.add_argument("--db", default="runtime/tmp/runtime-probe.db")
    parser.add_argument("--no-llm", action="store_true")
    args = parser.parse_args()

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path == "/api/stats":
                payload = json.dumps({"status": "ok", "db": args.db}).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
            else:
                self.send_response(404)
                self.end_headers()

        def log_message(self, format, *args):
            return

    with socketserver.TCPServer((args.host, args.port), Handler) as server:
        server.serve_forever()

if __name__ == "__main__":
    main()
"#,
        )
        .expect("write main");
        fs::write(app_root.join("agent.py"), "class USBAgent:\n    pass\n").expect("write agent");
        fs::write(app_root.join("personality.md"), "# personality").expect("write personality");
        fs::write(app_root.join("requirements.txt"), "aiohttp").expect("write requirements");
        fs::write(app_root.join("README.txt"), "USBRAIN").expect("write readme");
        fs::write(app_root.join("start.bat"), "@echo off\r\npython main.py --no-llm\r\n").expect("write bat");
        fs::write(app_root.join("tools").join("registry.py"), "REGISTRY = {}\n").expect("write tool");
        fs::write(app_root.join("skills").join("files.py"), "def ping():\n    return 'ok'\n").expect("write skill");
        fs::write(app_root.join("rag").join("server.py"), "def run():\n    return 'ok'\n").expect("write rag");
        fs::write(app_root.join("webui").join("server.py"), "def run():\n    return 'ok'\n").expect("write webui");
        fs::write(app_root.join("webui").join("templates").join("index.html"), "<html>ok</html>")
            .expect("write webui template");
        fs::write(cache_root.join("llamafile.exe"), "llama").expect("write llamafile");
        fs::write(cache_root.join("python_runtime.zip"), "zip").expect("write runtime zip");
        fs::write(cache_root.join("python").join("python.exe"), "python").expect("write python exe");
        write_fake_bge_assets(&cache_root);
        let model_source = test_root.join("model.gguf");
        fs::write(&model_source, "gguf").expect("write model");

        let build_plan = BuildPlanResponse {
            summary: BuildPlanSummary {
                build_id: "test-build".to_string(),
                target_label: "USBRAIN Test".to_string(),
                target_mount_path: target_root.display().to_string(),
                filesystem: "exfat".to_string(),
                build_mode: "offline-first".to_string(),
                profile_mode: "offline".to_string(),
                privacy_mode: "local-first".to_string(),
                capacity_gb: Some(64.0),
                available_gb: Some(64.0),
                install_budget_gb: Some(52.0),
                estimated_model_slots: 2,
                selected_model_count: 1,
                pending_model_download_count: 0,
                selected_pack_count: 1,
                allow_gpu: false,
                allow_cloud_fallback: false,
                primary_language: "pt-BR".to_string(),
                languages: vec!["pt-BR".to_string()],
                warnings: vec![],
            },
            models: vec![PlannedModel {
                installed_model_id: "test-model".to_string(),
                display_name: "Test Model".to_string(),
                role: "primary".to_string(),
                disk_size_gb: 1.0,
                recommended_ram_gb: 8.0,
                runtime_family: "llama.cpp".to_string(),
                source_file_name: "model.gguf".to_string(),
                source_absolute_path: model_source.display().to_string(),
                relative_target_path: "models/model.gguf".to_string(),
                available_locally: true,
                availability_status: "available-local".to_string(),
                availability_label: "Biblioteca local".to_string(),
            }],
            pending_model_downloads: vec![],
            host_ram_recommendations: vec![],
            packs: vec![PlannedPack {
                installed_pack_id: "pack-core-runtime".to_string(),
                name: "USBRAIN Core Runtime Pack".to_string(),
                pack_type: "behavior".to_string(),
                activation_mode: "always".to_string(),
                category: "core".to_string(),
            }],
            manifests: vec![
                GeneratedManifest {
                    file_name: "build-manifest.json".to_string(),
                    title: "Build Manifest".to_string(),
                    contents: "{\n  \"ok\": true\n}".to_string(),
                },
                GeneratedManifest {
                    file_name: "runtime-config.json".to_string(),
                    title: "Runtime Config".to_string(),
                    contents: "{\n  \"runtime\": true\n}".to_string(),
                },
                GeneratedManifest {
                    file_name: "installed-models.json".to_string(),
                    title: "Installed Models".to_string(),
                    contents: "{\n  \"models\": []\n}".to_string(),
                },
            ],
        };

        assemble_build_plan_with_sources(&build_plan, &target_root, &app_root, &cache_root)
            .expect("assembly should succeed");
        let probe = run_runtime_probe_with_sources(&build_plan, &target_root, &app_root, &cache_root);

        assert_eq!(probe.probe_status, "passed");
        assert!(probe.errors.is_empty());
        assert!(probe.checks.iter().any(|check| check.id == "http-endpoint" && check.status == "passed"));

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn end_to_end_probe_reports_pass_after_valid_assembly() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-e2e-probe-test-{}", unique));
        let app_root = test_root.join("app");
        let cache_root = test_root.join("cache");
        let target_root = test_root.join("target");

        fs::create_dir_all(app_root.join("tools")).expect("create app tools");
        fs::create_dir_all(app_root.join("skills")).expect("create app skills");
        fs::create_dir_all(app_root.join("rag")).expect("create app rag");
        fs::create_dir_all(app_root.join("webui").join("templates")).expect("create app webui");
        fs::create_dir_all(cache_root.join("bge-m3-model")).expect("create bge cache");
        fs::create_dir_all(cache_root.join("python")).expect("create python cache");
        fs::create_dir_all(&target_root).expect("create target root");

        fs::write(
            app_root.join("main.py"),
            r#"import argparse
import http.server
import json
import socketserver

def main():
    parser = argparse.ArgumentParser(description="USBRAIN e2e probe runtime")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8000)
    parser.add_argument("--bge-port", type=int, default=38422)
    parser.add_argument("--llm-port", type=int, default=38421)
    parser.add_argument("--db", default="runtime/tmp/e2e-probe.db")
    parser.add_argument("--no-llm", action="store_true")
    args = parser.parse_args()

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path == "/api/stats":
                payload = json.dumps({"status": "ok", "db": args.db}).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
            else:
                self.send_response(404)
                self.end_headers()

        def do_POST(self):
            if self.path == "/api/chat":
                length = int(self.headers.get("Content-Length", "0"))
                raw = self.rfile.read(length).decode("utf-8")
                body = json.loads(raw)
                prompt = body.get("message", "")
                payload = json.dumps({"response": f"OK: {prompt}", "model": "probe-model"}).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
            else:
                self.send_response(404)
                self.end_headers()

        def log_message(self, format, *args):
            return

    with socketserver.TCPServer((args.host, args.port), Handler) as server:
        server.serve_forever()

if __name__ == "__main__":
    main()
"#,
        )
        .expect("write main");
        fs::write(app_root.join("agent.py"), "class USBAgent:\n    pass\n").expect("write agent");
        fs::write(app_root.join("personality.md"), "# personality").expect("write personality");
        fs::write(app_root.join("requirements.txt"), "aiohttp").expect("write requirements");
        fs::write(app_root.join("README.txt"), "USBRAIN").expect("write readme");
        fs::write(app_root.join("start.bat"), "@echo off\r\npython main.py --no-llm\r\n").expect("write bat");
        fs::write(app_root.join("tools").join("registry.py"), "REGISTRY = {}\n").expect("write tool");
        fs::write(app_root.join("skills").join("files.py"), "def ping():\n    return 'ok'\n").expect("write skill");
        fs::write(app_root.join("rag").join("server.py"), "def run():\n    return 'ok'\n").expect("write rag");
        fs::write(app_root.join("webui").join("server.py"), "def run():\n    return 'ok'\n").expect("write webui");
        fs::write(app_root.join("webui").join("templates").join("index.html"), "<html>ok</html>")
            .expect("write webui template");
        fs::write(cache_root.join("llamafile.exe"), "llama").expect("write llamafile");
        fs::write(cache_root.join("python_runtime.zip"), "zip").expect("write runtime zip");
        fs::write(cache_root.join("python").join("python.exe"), "python").expect("write python exe");
        write_fake_bge_assets(&cache_root);
        let model_source = test_root.join("model.gguf");
        fs::write(&model_source, "gguf").expect("write model");

        let build_plan = BuildPlanResponse {
            summary: BuildPlanSummary {
                build_id: "test-build".to_string(),
                target_label: "USBRAIN Test".to_string(),
                target_mount_path: target_root.display().to_string(),
                filesystem: "exfat".to_string(),
                build_mode: "offline-first".to_string(),
                profile_mode: "offline".to_string(),
                privacy_mode: "local-first".to_string(),
                capacity_gb: Some(64.0),
                available_gb: Some(64.0),
                install_budget_gb: Some(52.0),
                estimated_model_slots: 2,
                selected_model_count: 1,
                pending_model_download_count: 0,
                selected_pack_count: 1,
                allow_gpu: false,
                allow_cloud_fallback: false,
                primary_language: "pt-BR".to_string(),
                languages: vec!["pt-BR".to_string()],
                warnings: vec![],
            },
            models: vec![PlannedModel {
                installed_model_id: "test-model".to_string(),
                display_name: "Test Model".to_string(),
                role: "primary".to_string(),
                disk_size_gb: 1.0,
                recommended_ram_gb: 8.0,
                runtime_family: "llama.cpp".to_string(),
                source_file_name: "model.gguf".to_string(),
                source_absolute_path: model_source.display().to_string(),
                relative_target_path: "models/model.gguf".to_string(),
                available_locally: true,
                availability_status: "available-local".to_string(),
                availability_label: "Biblioteca local".to_string(),
            }],
            pending_model_downloads: vec![],
            host_ram_recommendations: vec![],
            packs: vec![PlannedPack {
                installed_pack_id: "pack-core-runtime".to_string(),
                name: "USBRAIN Core Runtime Pack".to_string(),
                pack_type: "behavior".to_string(),
                activation_mode: "always".to_string(),
                category: "core".to_string(),
            }],
            manifests: vec![
                GeneratedManifest {
                    file_name: "build-manifest.json".to_string(),
                    title: "Build Manifest".to_string(),
                    contents: "{\n  \"ok\": true\n}".to_string(),
                },
                GeneratedManifest {
                    file_name: "runtime-config.json".to_string(),
                    title: "Runtime Config".to_string(),
                    contents: "{\n  \"runtime\": true\n}".to_string(),
                },
                GeneratedManifest {
                    file_name: "installed-models.json".to_string(),
                    title: "Installed Models".to_string(),
                    contents: "{\n  \"models\": []\n}".to_string(),
                },
            ],
        };

        assemble_build_plan_with_sources(&build_plan, &target_root, &app_root, &cache_root)
            .expect("assembly should succeed");
        let probe = run_end_to_end_probe_with_sources(&build_plan, &target_root, &app_root, &cache_root);

        assert_eq!(probe.probe_status, "passed");
        assert!(probe.errors.is_empty());
        assert!(probe.response_preview.contains("OK: USBRAIN probe"));
        assert!(probe.checks.iter().any(|check| check.id == "chat-endpoint" && check.status == "passed"));

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn full_local_llm_probe_reports_pass_after_valid_assembly() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-full-llm-probe-test-{}", unique));
        let app_root = test_root.join("app");
        let cache_root = test_root.join("cache");
        let target_root = test_root.join("target");

        fs::create_dir_all(app_root.join("tools")).expect("create app tools");
        fs::create_dir_all(app_root.join("skills")).expect("create app skills");
        fs::create_dir_all(app_root.join("rag")).expect("create app rag");
        fs::create_dir_all(app_root.join("webui").join("templates")).expect("create app webui");
        fs::create_dir_all(cache_root.join("bge-m3-model")).expect("create bge cache");
        fs::create_dir_all(cache_root.join("python")).expect("create python cache");
        fs::create_dir_all(&target_root).expect("create target root");
        write_fake_llamafile_binaries(&app_root);

        fs::write(
            app_root.join("main.py"),
            r#"import argparse
import http.server
import json
import socketserver

def main():
    parser = argparse.ArgumentParser(description="USBRAIN full llm probe runtime")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8000)
    parser.add_argument("--bge-port", type=int, default=38422)
    parser.add_argument("--llm-port", type=int, default=38421)
    parser.add_argument("--db", default="runtime/tmp/full-llm-probe.db")
    parser.add_argument("--no-llm", action="store_true")
    args = parser.parse_args()

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path == "/api/stats":
                payload = json.dumps({"status": "ok", "llm_active": not args.no_llm}).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
            else:
                self.send_response(404)
                self.end_headers()

        def do_POST(self):
            if self.path == "/api/chat":
                length = int(self.headers.get("Content-Length", "0"))
                raw = self.rfile.read(length).decode("utf-8")
                body = json.loads(raw)
                prompt = body.get("message", "")
                if args.no_llm:
                    payload = json.dumps({"response": "Erro: LLM desativada", "model": "probe-model"}).encode("utf-8")
                else:
                    payload = json.dumps({"response": f"OK local llm: {prompt}", "model": "probe-model"}).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
            else:
                self.send_response(404)
                self.end_headers()

        def log_message(self, format, *args):
            return

    with socketserver.TCPServer((args.host, args.port), Handler) as server:
        server.serve_forever()

if __name__ == "__main__":
    main()
"#,
        )
        .expect("write main");
        fs::write(app_root.join("agent.py"), "class USBAgent:\n    pass\n").expect("write agent");
        fs::write(app_root.join("personality.md"), "# personality").expect("write personality");
        fs::write(app_root.join("requirements.txt"), "aiohttp").expect("write requirements");
        fs::write(app_root.join("README.txt"), "USBRAIN").expect("write readme");
        fs::write(app_root.join("start.bat"), "@echo off\r\npython main.py\r\n").expect("write bat");
        fs::write(app_root.join("tools").join("registry.py"), "REGISTRY = {}\n").expect("write tool");
        fs::write(app_root.join("skills").join("files.py"), "def ping():\n    return 'ok'\n").expect("write skill");
        fs::write(app_root.join("rag").join("server.py"), "def run():\n    return 'ok'\n").expect("write rag");
        fs::write(app_root.join("webui").join("server.py"), "def run():\n    return 'ok'\n").expect("write webui");
        fs::write(app_root.join("webui").join("templates").join("index.html"), "<html>ok</html>")
            .expect("write webui template");
        fs::write(cache_root.join("llamafile.exe"), "llama").expect("write llamafile");
        fs::write(cache_root.join("python_runtime.zip"), "zip").expect("write runtime zip");
        fs::write(cache_root.join("python").join("python.exe"), "python").expect("write python exe");
        write_fake_bge_assets(&cache_root);
        let model_source = test_root.join("model.gguf");
        fs::write(&model_source, "gguf").expect("write model");

        let build_plan = BuildPlanResponse {
            summary: BuildPlanSummary {
                build_id: "test-build".to_string(),
                target_label: "USBRAIN Test".to_string(),
                target_mount_path: target_root.display().to_string(),
                filesystem: "exfat".to_string(),
                build_mode: "offline-first".to_string(),
                profile_mode: "offline".to_string(),
                privacy_mode: "local-first".to_string(),
                capacity_gb: Some(64.0),
                available_gb: Some(64.0),
                install_budget_gb: Some(52.0),
                estimated_model_slots: 2,
                selected_model_count: 1,
                pending_model_download_count: 0,
                selected_pack_count: 1,
                allow_gpu: false,
                allow_cloud_fallback: false,
                primary_language: "pt-BR".to_string(),
                languages: vec!["pt-BR".to_string()],
                warnings: vec![],
            },
            models: vec![PlannedModel {
                installed_model_id: "test-model".to_string(),
                display_name: "Test Model".to_string(),
                role: "primary".to_string(),
                disk_size_gb: 1.0,
                recommended_ram_gb: 8.0,
                runtime_family: "llama.cpp".to_string(),
                source_file_name: "model.gguf".to_string(),
                source_absolute_path: model_source.display().to_string(),
                relative_target_path: "models/model.gguf".to_string(),
                available_locally: true,
                availability_status: "available-local".to_string(),
                availability_label: "Biblioteca local".to_string(),
            }],
            pending_model_downloads: vec![],
            host_ram_recommendations: vec![],
            packs: vec![PlannedPack {
                installed_pack_id: "pack-core-runtime".to_string(),
                name: "USBRAIN Core Runtime Pack".to_string(),
                pack_type: "behavior".to_string(),
                activation_mode: "always".to_string(),
                category: "core".to_string(),
            }],
            manifests: vec![
                GeneratedManifest {
                    file_name: "build-manifest.json".to_string(),
                    title: "Build Manifest".to_string(),
                    contents: "{\n  \"ok\": true\n}".to_string(),
                },
                GeneratedManifest {
                    file_name: "runtime-config.json".to_string(),
                    title: "Runtime Config".to_string(),
                    contents: "{\n  \"runtime\": true\n}".to_string(),
                },
                GeneratedManifest {
                    file_name: "installed-models.json".to_string(),
                    title: "Installed Models".to_string(),
                    contents: "{\n  \"models\": []\n}".to_string(),
                },
            ],
        };

        assemble_build_plan_with_sources(&build_plan, &target_root, &app_root, &cache_root)
            .expect("assembly should succeed");
        let probe = run_full_local_llm_probe_with_sources(&build_plan, &target_root, &app_root, &cache_root);

        assert_eq!(probe.probe_status, "passed");
        assert!(probe.errors.is_empty());
        assert!(probe.response_preview.contains("OK local llm"));
        assert!(probe.checks.iter().any(|check| check.id == "chat-endpoint" && check.status == "passed"));

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn readiness_report_blocks_distribution_when_local_llm_fails() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-readiness-test-{}", unique));
        let app_root = test_root.join("app");
        let cache_root = test_root.join("cache");
        let target_root = test_root.join("target");

        fs::create_dir_all(app_root.join("tools")).expect("create app tools");
        fs::create_dir_all(app_root.join("skills")).expect("create app skills");
        fs::create_dir_all(app_root.join("rag")).expect("create app rag");
        fs::create_dir_all(app_root.join("webui").join("templates")).expect("create app webui");
        fs::create_dir_all(cache_root.join("bge-m3-model")).expect("create bge cache");
        fs::create_dir_all(cache_root.join("python")).expect("create python cache");
        fs::create_dir_all(&target_root).expect("create target root");

        fs::write(
            app_root.join("main.py"),
            r#"import argparse
import http.server
import json
import socketserver

def main():
    parser = argparse.ArgumentParser(description="USBRAIN readiness runtime")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8000)
    parser.add_argument("--bge-port", type=int, default=38422)
    parser.add_argument("--llm-port", type=int, default=38421)
    parser.add_argument("--db", default="runtime/tmp/readiness.db")
    parser.add_argument("--no-llm", action="store_true")
    args = parser.parse_args()

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path == "/api/stats":
                payload = json.dumps({"status": "ok"}).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
            else:
                self.send_response(404)
                self.end_headers()

        def do_POST(self):
            if self.path == "/api/chat":
                payload = json.dumps({"response": "Erro: LLM indisponivel", "model": "probe-model"}).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
            else:
                self.send_response(404)
                self.end_headers()

        def log_message(self, format, *args):
            return

    with socketserver.TCPServer((args.host, args.port), Handler) as server:
        server.serve_forever()

if __name__ == "__main__":
    main()
"#,
        )
        .expect("write main");
        fs::write(app_root.join("agent.py"), "class USBAgent:\n    pass\n").expect("write agent");
        fs::write(app_root.join("personality.md"), "# personality").expect("write personality");
        fs::write(app_root.join("requirements.txt"), "aiohttp").expect("write requirements");
        fs::write(app_root.join("README.txt"), "USBRAIN").expect("write readme");
        fs::write(app_root.join("start.bat"), "@echo off\r\npython main.py\r\n").expect("write bat");
        fs::write(app_root.join("tools").join("registry.py"), "REGISTRY = {}\n").expect("write tool");
        fs::write(app_root.join("skills").join("files.py"), "def ping():\n    return 'ok'\n").expect("write skill");
        fs::write(app_root.join("rag").join("server.py"), "def run():\n    return 'ok'\n").expect("write rag");
        fs::write(app_root.join("webui").join("server.py"), "def run():\n    return 'ok'\n").expect("write webui");
        fs::write(app_root.join("webui").join("templates").join("index.html"), "<html>ok</html>")
            .expect("write webui template");
        fs::write(cache_root.join("llamafile.exe"), "llama").expect("write llamafile");
        fs::write(cache_root.join("python_runtime.zip"), "zip").expect("write runtime zip");
        fs::write(cache_root.join("python").join("python.exe"), "python").expect("write python exe");
        write_fake_bge_assets(&cache_root);
        let model_source = test_root.join("model.gguf");
        fs::write(&model_source, "gguf").expect("write model");

        let build_plan = BuildPlanResponse {
            summary: BuildPlanSummary {
                build_id: "test-build".to_string(),
                target_label: "USBRAIN Test".to_string(),
                target_mount_path: target_root.display().to_string(),
                filesystem: "exfat".to_string(),
                build_mode: "offline-first".to_string(),
                profile_mode: "offline".to_string(),
                privacy_mode: "local-first".to_string(),
                capacity_gb: Some(64.0),
                available_gb: Some(64.0),
                install_budget_gb: Some(52.0),
                estimated_model_slots: 2,
                selected_model_count: 1,
                pending_model_download_count: 0,
                selected_pack_count: 1,
                allow_gpu: false,
                allow_cloud_fallback: false,
                primary_language: "pt-BR".to_string(),
                languages: vec!["pt-BR".to_string()],
                warnings: vec![],
            },
            models: vec![PlannedModel {
                installed_model_id: "test-model".to_string(),
                display_name: "Test Model".to_string(),
                role: "primary".to_string(),
                disk_size_gb: 1.0,
                recommended_ram_gb: 8.0,
                runtime_family: "llama.cpp".to_string(),
                source_file_name: "model.gguf".to_string(),
                source_absolute_path: model_source.display().to_string(),
                relative_target_path: "models/model.gguf".to_string(),
                available_locally: true,
                availability_status: "available-local".to_string(),
                availability_label: "Biblioteca local".to_string(),
            }],
            pending_model_downloads: vec![],
            host_ram_recommendations: vec![],
            packs: vec![PlannedPack {
                installed_pack_id: "pack-core-runtime".to_string(),
                name: "USBRAIN Core Runtime Pack".to_string(),
                pack_type: "behavior".to_string(),
                activation_mode: "always".to_string(),
                category: "core".to_string(),
            }],
            manifests: vec![
                GeneratedManifest {
                    file_name: "build-manifest.json".to_string(),
                    title: "Build Manifest".to_string(),
                    contents: "{\n  \"ok\": true\n}".to_string(),
                },
                GeneratedManifest {
                    file_name: "runtime-config.json".to_string(),
                    title: "Runtime Config".to_string(),
                    contents: "{\n  \"runtime\": true\n}".to_string(),
                },
                GeneratedManifest {
                    file_name: "installed-models.json".to_string(),
                    title: "Installed Models".to_string(),
                    contents: "{\n  \"models\": []\n}".to_string(),
                },
            ],
        };

        assemble_build_plan_with_sources(&build_plan, &target_root, &app_root, &cache_root)
            .expect("assembly should succeed");
        let report = build_readiness_report_with_sources(&build_plan, &target_root, &app_root, &cache_root);
        let distribution = prepare_distribution_bundle_to_dir(&build_plan, &test_root.join("output"), &app_root, &cache_root)
            .expect("distribution bundle should be generated even when blocked");

        assert!(!report.distribution_ready);
        assert_eq!(report.readiness_status, "blocked");
        assert!(report
            .blocked_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("LLM local falhou")));
        assert!(!distribution.distribution_ready);
        assert_eq!(distribution.distribution_status, "blocked");
        assert!(Path::new(&distribution.readiness_report_file).exists());
        assert!(Path::new(&distribution.distribution_summary_file).exists());

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn distribution_bundle_is_ready_when_local_llm_probe_passes() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-distribution-ready-test-{}", unique));
        let app_root = test_root.join("app");
        let cache_root = test_root.join("cache");
        let target_root = test_root.join("target");
        let output_dir = test_root.join("output");

        fs::create_dir_all(app_root.join("tools")).expect("create app tools");
        fs::create_dir_all(app_root.join("skills")).expect("create app skills");
        fs::create_dir_all(app_root.join("rag")).expect("create app rag");
        fs::create_dir_all(app_root.join("webui").join("templates")).expect("create app webui");
        fs::create_dir_all(cache_root.join("bge-m3-model")).expect("create bge cache");
        fs::create_dir_all(cache_root.join("python")).expect("create python cache");
        fs::create_dir_all(&target_root).expect("create target root");
        write_fake_llamafile_binaries(&app_root);

        fs::write(
            app_root.join("main.py"),
            r#"import argparse
import http.server
import json
import socketserver

def main():
    parser = argparse.ArgumentParser(description="USBRAIN distribution ready runtime")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8000)
    parser.add_argument("--bge-port", type=int, default=38422)
    parser.add_argument("--llm-port", type=int, default=38421)
    parser.add_argument("--db", default="runtime/tmp/distribution-ready.db")
    parser.add_argument("--no-llm", action="store_true")
    args = parser.parse_args()

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path == "/api/stats":
                payload = json.dumps({"status": "ok"}).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
            else:
                self.send_response(404)
                self.end_headers()

        def do_POST(self):
            if self.path == "/api/chat":
                payload = json.dumps({"response": "OK local llm", "model": "probe-model"}).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
            else:
                self.send_response(404)
                self.end_headers()

        def log_message(self, format, *args):
            return

    with socketserver.TCPServer((args.host, args.port), Handler) as server:
        server.serve_forever()

if __name__ == "__main__":
    main()
"#,
        )
        .expect("write main");
        fs::write(app_root.join("agent.py"), "class USBAgent:\n    pass\n").expect("write agent");
        fs::write(app_root.join("personality.md"), "# personality").expect("write personality");
        fs::write(app_root.join("requirements.txt"), "aiohttp").expect("write requirements");
        fs::write(app_root.join("README.txt"), "USBRAIN").expect("write readme");
        fs::write(app_root.join("start.bat"), "@echo off\r\npython main.py\r\n").expect("write bat");
        fs::write(app_root.join("tools").join("registry.py"), "REGISTRY = {}\n").expect("write tool");
        fs::write(app_root.join("skills").join("files.py"), "def ping():\n    return 'ok'\n").expect("write skill");
        fs::write(app_root.join("rag").join("server.py"), "def run():\n    return 'ok'\n").expect("write rag");
        fs::write(app_root.join("webui").join("server.py"), "def run():\n    return 'ok'\n").expect("write webui");
        fs::write(app_root.join("webui").join("templates").join("index.html"), "<html>ok</html>")
            .expect("write webui template");
        fs::write(cache_root.join("llamafile.exe"), "llama").expect("write llamafile");
        fs::write(cache_root.join("python_runtime.zip"), "zip").expect("write runtime zip");
        fs::write(cache_root.join("python").join("python.exe"), "python").expect("write python exe");
        write_fake_bge_assets(&cache_root);
        let model_source = test_root.join("model.gguf");
        fs::write(&model_source, "gguf").expect("write model");

        let build_plan = BuildPlanResponse {
            summary: BuildPlanSummary {
                build_id: "test-build".to_string(),
                target_label: "USBRAIN Test".to_string(),
                target_mount_path: target_root.display().to_string(),
                filesystem: "exfat".to_string(),
                build_mode: "offline-first".to_string(),
                profile_mode: "offline".to_string(),
                privacy_mode: "local-first".to_string(),
                capacity_gb: Some(64.0),
                available_gb: Some(64.0),
                install_budget_gb: Some(52.0),
                estimated_model_slots: 2,
                selected_model_count: 1,
                pending_model_download_count: 0,
                selected_pack_count: 1,
                allow_gpu: false,
                allow_cloud_fallback: false,
                primary_language: "pt-BR".to_string(),
                languages: vec!["pt-BR".to_string()],
                warnings: vec![],
            },
            models: vec![PlannedModel {
                installed_model_id: "test-model".to_string(),
                display_name: "Test Model".to_string(),
                role: "primary".to_string(),
                disk_size_gb: 1.0,
                recommended_ram_gb: 8.0,
                runtime_family: "llama.cpp".to_string(),
                source_file_name: "model.gguf".to_string(),
                source_absolute_path: model_source.display().to_string(),
                relative_target_path: "models/model.gguf".to_string(),
                available_locally: true,
                availability_status: "available-local".to_string(),
                availability_label: "Biblioteca local".to_string(),
            }],
            pending_model_downloads: vec![],
            host_ram_recommendations: vec![],
            packs: vec![PlannedPack {
                installed_pack_id: "pack-core-runtime".to_string(),
                name: "USBRAIN Core Runtime Pack".to_string(),
                pack_type: "behavior".to_string(),
                activation_mode: "always".to_string(),
                category: "core".to_string(),
            }],
            manifests: vec![
                GeneratedManifest {
                    file_name: "build-manifest.json".to_string(),
                    title: "Build Manifest".to_string(),
                    contents: "{\n  \"ok\": true\n}".to_string(),
                },
                GeneratedManifest {
                    file_name: "runtime-config.json".to_string(),
                    title: "Runtime Config".to_string(),
                    contents: "{\n  \"runtime\": true\n}".to_string(),
                },
                GeneratedManifest {
                    file_name: "installed-models.json".to_string(),
                    title: "Installed Models".to_string(),
                    contents: "{\n  \"models\": []\n}".to_string(),
                },
            ],
        };

        assemble_build_plan_with_sources(&build_plan, &target_root, &app_root, &cache_root)
            .expect("assembly should succeed");
        let distribution = prepare_distribution_bundle_to_dir(&build_plan, &output_dir, &app_root, &cache_root)
            .expect("distribution bundle should succeed");

        assert!(distribution.distribution_ready);
        assert_eq!(distribution.distribution_status, "ready");
        assert!(distribution.exported_files.iter().any(|file| file.file_name == "readiness-report.json"));
        assert!(distribution.exported_files.iter().any(|file| file.file_name == "distribution-summary.json"));

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn release_package_is_blocked_when_distribution_gate_is_blocked() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-release-blocked-test-{}", unique));
        let app_root = test_root.join("app");
        let cache_root = test_root.join("cache");
        let target_root = test_root.join("target");
        let output_dir = test_root.join("output");

        fs::create_dir_all(app_root.join("tools")).expect("create app tools");
        fs::create_dir_all(app_root.join("skills")).expect("create app skills");
        fs::create_dir_all(app_root.join("rag")).expect("create app rag");
        fs::create_dir_all(app_root.join("webui").join("templates")).expect("create app webui");
        fs::create_dir_all(cache_root.join("bge-m3-model")).expect("create bge cache");
        fs::create_dir_all(cache_root.join("python")).expect("create python cache");
        fs::create_dir_all(&target_root).expect("create target root");

        fs::write(
            app_root.join("main.py"),
            r#"import argparse
import http.server
import json
import socketserver

def main():
    parser = argparse.ArgumentParser(description="USBRAIN release blocked runtime")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8000)
    parser.add_argument("--bge-port", type=int, default=38422)
    parser.add_argument("--llm-port", type=int, default=38421)
    parser.add_argument("--db", default="runtime/tmp/release-blocked.db")
    parser.add_argument("--no-llm", action="store_true")
    args = parser.parse_args()

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path == "/api/stats":
                payload = json.dumps({"status": "ok"}).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
            else:
                self.send_response(404)
                self.end_headers()

        def do_POST(self):
            if self.path == "/api/chat":
                payload = json.dumps({"response": "Erro: local llm indisponivel", "model": "probe-model"}).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
            else:
                self.send_response(404)
                self.end_headers()

        def log_message(self, format, *args):
            return

    with socketserver.TCPServer((args.host, args.port), Handler) as server:
        server.serve_forever()

if __name__ == "__main__":
    main()
"#,
        )
        .expect("write main");
        fs::write(app_root.join("agent.py"), "class USBAgent:\n    pass\n").expect("write agent");
        fs::write(app_root.join("personality.md"), "# personality").expect("write personality");
        fs::write(app_root.join("requirements.txt"), "aiohttp").expect("write requirements");
        fs::write(app_root.join("README.txt"), "USBRAIN").expect("write readme");
        fs::write(app_root.join("start.bat"), "@echo off\r\npython main.py\r\n").expect("write bat");
        fs::write(app_root.join("tools").join("registry.py"), "REGISTRY = {}\n").expect("write tool");
        fs::write(app_root.join("skills").join("files.py"), "def ping():\n    return 'ok'\n").expect("write skill");
        fs::write(app_root.join("rag").join("server.py"), "def run():\n    return 'ok'\n").expect("write rag");
        fs::write(app_root.join("webui").join("server.py"), "def run():\n    return 'ok'\n").expect("write webui");
        fs::write(cache_root.join("llamafile.exe"), "llama").expect("write llamafile");
        fs::write(cache_root.join("python_runtime.zip"), "zip").expect("write runtime zip");
        fs::write(cache_root.join("python").join("python.exe"), "python").expect("write python exe");
        write_fake_bge_assets(&cache_root);
        let model_source = test_root.join("model.gguf");
        fs::write(&model_source, "gguf").expect("write model");

        let build_plan = BuildPlanResponse {
            summary: BuildPlanSummary {
                build_id: "test-build".to_string(),
                target_label: "USBRAIN Test".to_string(),
                target_mount_path: target_root.display().to_string(),
                filesystem: "exfat".to_string(),
                build_mode: "offline-first".to_string(),
                profile_mode: "offline".to_string(),
                privacy_mode: "local-first".to_string(),
                capacity_gb: Some(64.0),
                available_gb: Some(64.0),
                install_budget_gb: Some(52.0),
                estimated_model_slots: 2,
                selected_model_count: 1,
                pending_model_download_count: 0,
                selected_pack_count: 1,
                allow_gpu: false,
                allow_cloud_fallback: false,
                primary_language: "pt-BR".to_string(),
                languages: vec!["pt-BR".to_string()],
                warnings: vec![],
            },
            models: vec![PlannedModel {
                installed_model_id: "test-model".to_string(),
                display_name: "Test Model".to_string(),
                role: "primary".to_string(),
                disk_size_gb: 1.0,
                recommended_ram_gb: 8.0,
                runtime_family: "llama.cpp".to_string(),
                source_file_name: "model.gguf".to_string(),
                source_absolute_path: model_source.display().to_string(),
                relative_target_path: "models/model.gguf".to_string(),
                available_locally: true,
                availability_status: "available-local".to_string(),
                availability_label: "Biblioteca local".to_string(),
            }],
            pending_model_downloads: vec![],
            host_ram_recommendations: vec![],
            packs: vec![PlannedPack {
                installed_pack_id: "pack-core-runtime".to_string(),
                name: "USBRAIN Core Runtime Pack".to_string(),
                pack_type: "behavior".to_string(),
                activation_mode: "always".to_string(),
                category: "core".to_string(),
            }],
            manifests: vec![
                GeneratedManifest {
                    file_name: "build-manifest.json".to_string(),
                    title: "Build Manifest".to_string(),
                    contents: "{\n  \"ok\": true\n}".to_string(),
                },
                GeneratedManifest {
                    file_name: "runtime-config.json".to_string(),
                    title: "Runtime Config".to_string(),
                    contents: "{\n  \"runtime\": true\n}".to_string(),
                },
                GeneratedManifest {
                    file_name: "installed-models.json".to_string(),
                    title: "Installed Models".to_string(),
                    contents: "{\n  \"models\": []\n}".to_string(),
                },
            ],
        };

        assemble_build_plan_with_sources(&build_plan, &target_root, &app_root, &cache_root)
            .expect("assembly should succeed");
        let release = prepare_release_package_to_dir(&build_plan, &output_dir, &app_root, &cache_root)
            .expect("release package should be generated even when blocked");

        assert!(!release.release_ready);
        assert_eq!(release.release_status, "blocked");
        assert!(Path::new(&release.release_manifest_file).exists());
        assert!(Path::new(&release.github_release_notes_file).exists());
        assert!(Path::new(&release.usbrain_release_summary_file).exists());
        assert!(Path::new(&release.checksums_file).exists());
        assert!(release
            .artifact_checksums
            .iter()
            .any(|artifact| artifact.file_name == "distribution-summary.json"));

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn release_package_is_ready_when_distribution_gate_passes() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        let test_root = std::env::temp_dir().join(format!("usbrain-builder-release-ready-test-{}", unique));
        let app_root = test_root.join("app");
        let cache_root = test_root.join("cache");
        let target_root = test_root.join("target");
        let output_dir = test_root.join("output");

        fs::create_dir_all(app_root.join("tools")).expect("create app tools");
        fs::create_dir_all(app_root.join("skills")).expect("create app skills");
        fs::create_dir_all(app_root.join("rag")).expect("create app rag");
        fs::create_dir_all(app_root.join("webui")).expect("create app webui");
        fs::create_dir_all(cache_root.join("bge-m3-model")).expect("create bge cache");
        fs::create_dir_all(cache_root.join("python")).expect("create python cache");
        fs::create_dir_all(&target_root).expect("create target root");
        write_fake_llamafile_binaries(&app_root);

        fs::write(
            app_root.join("main.py"),
            r#"import argparse
import http.server
import json
import socketserver

def main():
    parser = argparse.ArgumentParser(description="USBRAIN release ready runtime")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8000)
    parser.add_argument("--bge-port", type=int, default=38422)
    parser.add_argument("--llm-port", type=int, default=38421)
    parser.add_argument("--db", default="runtime/tmp/release-ready.db")
    parser.add_argument("--no-llm", action="store_true")
    args = parser.parse_args()

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path == "/api/stats":
                payload = json.dumps({"status": "ok"}).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
            else:
                self.send_response(404)
                self.end_headers()

        def do_POST(self):
            if self.path == "/api/chat":
                payload = json.dumps({"response": "OK local llm", "model": "probe-model"}).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)
            else:
                self.send_response(404)
                self.end_headers()

        def log_message(self, format, *args):
            return

    with socketserver.TCPServer((args.host, args.port), Handler) as server:
        server.serve_forever()

if __name__ == "__main__":
    main()
"#,
        )
        .expect("write main");
        fs::write(app_root.join("agent.py"), "class USBAgent:\n    pass\n").expect("write agent");
        fs::write(app_root.join("personality.md"), "# personality").expect("write personality");
        fs::write(app_root.join("requirements.txt"), "aiohttp").expect("write requirements");
        fs::write(app_root.join("README.txt"), "USBRAIN").expect("write readme");
        fs::write(app_root.join("start.bat"), "@echo off\r\npython main.py\r\n").expect("write bat");
        fs::write(app_root.join("tools").join("registry.py"), "REGISTRY = {}\n").expect("write tool");
        fs::write(app_root.join("skills").join("files.py"), "def ping():\n    return 'ok'\n").expect("write skill");
        fs::write(app_root.join("rag").join("server.py"), "def run():\n    return 'ok'\n").expect("write rag");
        fs::write(app_root.join("webui").join("server.py"), "def run():\n    return 'ok'\n").expect("write webui");
        fs::write(cache_root.join("llamafile.exe"), "llama").expect("write llamafile");
        fs::write(cache_root.join("python_runtime.zip"), "zip").expect("write runtime zip");
        fs::write(cache_root.join("python").join("python.exe"), "python").expect("write python exe");
        write_fake_bge_assets(&cache_root);
        let model_source = test_root.join("model.gguf");
        fs::write(&model_source, "gguf").expect("write model");

        let build_plan = BuildPlanResponse {
            summary: BuildPlanSummary {
                build_id: "test-build".to_string(),
                target_label: "USBRAIN Test".to_string(),
                target_mount_path: target_root.display().to_string(),
                filesystem: "exfat".to_string(),
                build_mode: "offline-first".to_string(),
                profile_mode: "offline".to_string(),
                privacy_mode: "local-first".to_string(),
                capacity_gb: Some(64.0),
                available_gb: Some(64.0),
                install_budget_gb: Some(52.0),
                estimated_model_slots: 2,
                selected_model_count: 1,
                pending_model_download_count: 0,
                selected_pack_count: 1,
                allow_gpu: false,
                allow_cloud_fallback: false,
                primary_language: "pt-BR".to_string(),
                languages: vec!["pt-BR".to_string(), "en".to_string(), "es".to_string()],
                warnings: vec![],
            },
            models: vec![PlannedModel {
                installed_model_id: "test-model".to_string(),
                display_name: "Test Model".to_string(),
                role: "primary".to_string(),
                disk_size_gb: 1.0,
                recommended_ram_gb: 8.0,
                runtime_family: "llama.cpp".to_string(),
                source_file_name: "model.gguf".to_string(),
                source_absolute_path: model_source.display().to_string(),
                relative_target_path: "models/model.gguf".to_string(),
                available_locally: true,
                availability_status: "available-local".to_string(),
                availability_label: "Biblioteca local".to_string(),
            }],
            pending_model_downloads: vec![],
            host_ram_recommendations: vec![],
            packs: vec![PlannedPack {
                installed_pack_id: "pack-core-runtime".to_string(),
                name: "USBRAIN Core Runtime Pack".to_string(),
                pack_type: "behavior".to_string(),
                activation_mode: "always".to_string(),
                category: "core".to_string(),
            }],
            manifests: vec![
                GeneratedManifest {
                    file_name: "build-manifest.json".to_string(),
                    title: "Build Manifest".to_string(),
                    contents: "{\n  \"ok\": true\n}".to_string(),
                },
                GeneratedManifest {
                    file_name: "runtime-config.json".to_string(),
                    title: "Runtime Config".to_string(),
                    contents: "{\n  \"runtime\": true\n}".to_string(),
                },
                GeneratedManifest {
                    file_name: "installed-models.json".to_string(),
                    title: "Installed Models".to_string(),
                    contents: "{\n  \"models\": []\n}".to_string(),
                },
            ],
        };

        assemble_build_plan_with_sources(&build_plan, &target_root, &app_root, &cache_root)
            .expect("assembly should succeed");
        let release = prepare_release_package_to_dir(&build_plan, &output_dir, &app_root, &cache_root)
            .expect("release package should succeed");

        assert!(release.release_ready);
        assert_eq!(release.release_status, "ready");
        assert!(release.recommended_release_tag.starts_with("usbrain-builder-v"));
        assert!(release.exported_files.iter().any(|file| file.file_name == "SHA256SUMS.txt"));
        assert!(release.exported_files.iter().any(|file| file.file_name == "github-release-notes.md"));
        assert!(release.exported_files.iter().any(|file| file.file_name == "usbrain-release-summary.json"));
        assert!(release.exported_files.iter().any(|file| file.file_name == "release-package.json"));

        let checksums = fs::read_to_string(&release.checksums_file).expect("read checksums");
        assert!(checksums.contains("distribution-summary.json"));
        assert!(checksums.contains("readiness-report.json"));

        let release_notes =
            fs::read_to_string(&release.github_release_notes_file).expect("read github release notes");
        assert!(release_notes.contains("Suggested tag"));
        assert!(release_notes.contains("Build pronto para publicacao open source"));

        fs::remove_dir_all(&test_root).ok();
    }

    #[test]
    fn recommendation_for_64gb_capacity_exposes_all_host_ram_tiers() {
        let (label, slots, warnings) = recommendation_from_capacity_gb(Some(64.0));

        assert_eq!(label, "Build completo com lineup para PCs de 8 GB, 16 GB, 32 GB e 64 GB");
        assert!(slots >= 4);
        assert!(!warnings.is_empty());
    }

    #[test]
    fn host_ram_matrix_for_64gb_target_covers_8_to_64gb_hosts() {
        let target = CandidateTarget {
            label: "USBRAIN".to_string(),
            mount_path: "/Volumes/USBRAIN".to_string(),
            source: "macos-usb:disk10s1".to_string(),
            filesystem: "exfat".to_string(),
            capacity_gb: Some(64.0),
            available_gb: Some(52.0),
            usage_percent: Some(18.0),
            removable: true,
            recommended_model_tier: "Build completo".to_string(),
            estimated_model_slots: 4,
            warnings: vec![],
        };

        let catalog_snapshot: ModelCatalogSnapshot = serde_json::from_str(
            &fs::read_to_string(bundled_model_catalog_path()).expect("read bundled catalog fixture"),
        )
        .expect("parse bundled catalog fixture");
        let recommendations = host_ram_recommendations(&target, &[], &catalog_snapshot.models);
        let tiers = recommendations.iter().map(|item| item.host_ram_gb).collect::<Vec<_>>();

        assert_eq!(tiers, vec![8, 16, 32, 64]);
        assert_eq!(
            recommendations[0].suggested_model.display_name,
            "Phi-3.5 Mini Instruct Q4"
        );
        assert_eq!(
            recommendations[3].suggested_model.display_name,
            "DeepSeek R1 Distill Qwen 32B Q4"
        );
    }
}
