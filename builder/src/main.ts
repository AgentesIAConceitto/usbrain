/// <reference types="vite/client" />
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { message, open } from "@tauri-apps/plugin-dialog";
import { startAccessingSecurityScopedResource, stopAccessingSecurityScopedResource } from "@tauri-apps/plugin-fs";
import "./styles.css";

const factoryQaTarget =
  typeof import.meta !== "undefined" && typeof import.meta.env.VITE_FACTORY_QA_TARGET === "string"
    ? import.meta.env.VITE_FACTORY_QA_TARGET.trim()
    : "";
const isMacOsDesktop =
  typeof navigator !== "undefined" && /Mac/i.test(navigator.userAgent);

type AppOverview = {
  appName: string;
  appVersion: string;
  workspaceRoot: string;
  schemaCount: number;
  documentCount: number;
  tauriBackendAvailable: boolean;
};

type SchemaSummary = {
  fileName: string;
  title: string;
  schemaId: string;
};

type WorkspaceSnapshot = {
  rootPath: string;
  schemaDirectory: string;
  keyDocuments: string[];
};

type CandidateTarget = {
  label: string;
  mountPath: string;
  source: string;
  filesystem: string;
  capacityGb?: number;
  availableGb?: number;
  usagePercent?: number;
  removable: boolean;
  recommendedModelTier: string;
  estimatedModelSlots: number;
  warnings: string[];
};

type BuildMode = "offline-first" | "offline-only" | "local-assisted" | "hybrid";

type PlannerDraft = {
  selectedTargetMountPath: string | null;
  buildMode: BuildMode;
  languages: string[];
  allowGpu: boolean;
  allowCloudFallback: boolean;
  creatorLabel: string;
  selectedModelIds: string[];
};

type PlannedModel = {
  installedModelId: string;
  displayName: string;
  role: string;
  diskSizeGb: number;
  recommendedRamGb: number;
  runtimeFamily: string;
  sourceFileName: string;
  sourceAbsolutePath: string;
  relativeTargetPath: string;
  availableLocally: boolean;
  availabilityStatus: string;
  availabilityLabel: string;
};

type PendingModelDownload = {
  modelId: string;
  displayName: string;
  diskSizeGb: number;
  recommendedRamGb: number;
  homepageOrSource: string;
  downloadLocations: string[];
  checksums: string[];
  localFileName: string;
  relativeTargetPath: string;
  availabilityStatus: string;
  availabilityLabel: string;
};

type SuggestedCatalogModel = {
  modelId: string;
  displayName: string;
  estimatedDiskSizeGb: number;
  targetHostRamGb: number;
  rationale: string;
  availableLocally: boolean;
  availabilityStatus: string;
  availabilityLabel: string;
};

type CatalogBenchmarkSummary = {
  sourceLabel: string;
  summary: string;
  compositeScore: number | null;
  rankLabel: string | null;
  lastVerifiedAt: string | null;
  matchOrigin: string;
  confidenceLabel: string;
  confidenceLevel: string;
};

type CatalogBenchmarkMetric = {
  label: string;
  value: number;
  unit: string | null;
  benchmarkSuite: string | null;
  higherIsBetter: boolean | null;
};

type CatalogTechnicalProfile = {
  contextWindowTokens: number | null;
  estimatedPromptTokensPerSecond: number | null;
  estimatedGenerationTokensPerSecond: number | null;
  estimatedPeakMemoryGb: number | null;
  recommendedGpuClass: string | null;
  notes: string | null;
};

type HostRamRecommendation = {
  hostRamGb: number;
  label: string;
  summary: string;
  suggestedModel: SuggestedCatalogModel;
  alternatives: SuggestedCatalogModel[];
};

type CatalogModelSummary = {
  modelId: string;
  displayName: string;
  family: string;
  parameterScale: string;
  variant: string;
  diskSizeGb: number;
  ramMinGb: number;
  ramRecommendedGb: number;
  hostRamTierGb: number;
  languages: string[];
  domains: string[];
  capabilities: string[];
  qualityScore: number;
  cpuOnlyViable: boolean;
  gpuOptional: boolean;
  license: string;
  homepageOrSource: string;
  downloadLocations: string[];
  checksums: string[];
  providerOrOrigin: string | null;
  variantId: string | null;
  quantization: string | null;
  fileFormat: string | null;
  editorialRecommendation: string | null;
  benchmarkSummary: CatalogBenchmarkSummary | null;
  benchmarkMetrics: CatalogBenchmarkMetric[];
  technicalProfile: CatalogTechnicalProfile | null;
  description: string;
  rationale: string;
  localFileName: string | null;
  availableLocally: boolean;
  availabilityStatus: string;
  availabilityLabel: string;
  fitsSelectedTarget: boolean;
};

type ModelCatalogResponse = {
  catalogVersion: string;
  lastUpdated: string;
  source: string;
  installBudgetGb?: number;
  domainOptions: string[];
  models: CatalogModelSummary[];
};

type PlannedPack = {
  installedPackId: string;
  name: string;
  packType: string;
  activationMode: string;
  category: string;
};

type GeneratedManifest = {
  fileName: string;
  title: string;
  contents: string;
};

type BuildPlanSummary = {
  buildId: string;
  targetLabel: string;
  targetMountPath: string;
  filesystem: string;
  capacityGb?: number;
  buildMode: string;
  profileMode: string;
  privacyMode: string;
  availableGb?: number;
  installBudgetGb?: number;
  estimatedModelSlots: number;
  selectedModelCount: number;
  pendingModelDownloadCount: number;
  selectedPackCount: number;
  allowGpu: boolean;
  allowCloudFallback: boolean;
  primaryLanguage: string;
  languages: string[];
  warnings: string[];
};

type BuildPlanResponse = {
  summary: BuildPlanSummary;
  models: PlannedModel[];
  pendingModelDownloads: PendingModelDownload[];
  hostRamRecommendations: HostRamRecommendation[];
  packs: PlannedPack[];
  manifests: GeneratedManifest[];
};

type ExportedFileSummary = {
  fileName: string;
  absolutePath: string;
  relativePath: string;
  sizeBytes: number;
};

type BuildPlanExportResult = {
  buildId: string;
  exportRoot: string;
  manifestsDirectory: string;
  summaryFile: string;
  exportedFiles: ExportedFileSummary[];
};

type BuilderLibraryPreparationResult = {
  buildId: string;
  libraryRoot: string;
  modelLibraryDirectory: string;
  downloadManifestDirectory: string;
  copiedModelsCount: number;
  alreadyAvailableCount: number;
  writtenManifestsCount: number;
  exportedFiles: ExportedFileSummary[];
  pendingDownloads: PendingModelDownload[];
  warnings: string[];
};

type BuilderLibraryArtifactSummary = {
  artifactId: string;
  artifactType: string;
  displayName: string;
  sourceLabel: string;
  localPath: string;
  sizeBytes: number;
  integrityStatus: string;
  downloadedAt: string | null;
  sourceUrl: string | null;
  removable: boolean;
};

type BuilderLibraryInventory = {
  libraryRoot: string;
  artifactCount: number;
  totalSizeBytes: number;
  artifacts: BuilderLibraryArtifactSummary[];
};

type DeleteBuilderLibraryArtifactRequest = {
  artifactType: string;
  artifactId: string;
};

type DeleteBuilderLibraryArtifactResult = {
  libraryRoot: string;
  artifactId: string;
  artifactType: string;
  deletedPath: string;
  recoveredSizeBytes: number;
};

type DownloadBuilderLibraryArtifactRequest = {
  artifactId: string;
};

type DownloadCatalogModelRequest = {
  modelId: string;
};

type DownloadBuilderLibraryArtifactResult = {
  status: string;
  libraryRoot: string;
  artifactId: string;
  displayName: string;
  sourceUrl: string;
  downloadedPath: string;
  removedManifestPath: string;
  sizeBytes: number;
  sha256: string;
};

type RefreshModelCatalogResult = {
  status: string;
  sourceUrl: string;
  cachePath: string;
  catalogVersion: string;
  lastUpdated: string;
  modelCount: number;
  warnings: string[];
};

type BuildDistributionGateResult = {
  buildId: string;
  exportRoot: string;
  distributionStatus: string;
  distributionReady: boolean;
  message: string;
  readinessReportFile: string;
  distributionSummaryFile: string;
  exportedFiles: ExportedFileSummary[];
  readinessReport: BuildReadinessReport;
};

type ReleaseArtifactChecksum = {
  fileName: string;
  absolutePath: string;
  relativePath: string;
  sizeBytes: number;
  sha256: string;
};

type BuildReleasePackageResult = {
  buildId: string;
  exportRoot: string;
  releaseStatus: string;
  releaseReady: boolean;
  message: string;
  recommendedReleaseTag: string;
  releaseManifestFile: string;
  githubReleaseNotesFile: string;
  usbrainReleaseSummaryFile: string;
  checksumsFile: string;
  exportedFiles: ExportedFileSummary[];
  artifactChecksums: ReleaseArtifactChecksum[];
  distributionResult: BuildDistributionGateResult;
};

type AssembledDirectorySummary = {
  relativePath: string;
  fileCount: number;
};

type BuildAssemblyResult = {
  buildId: string;
  targetRoot: string;
  markerFile: string;
  manifestDirectory: string;
  selectedModelsDirectory: string;
  exportedFiles: ExportedFileSummary[];
  assembledDirectories: AssembledDirectorySummary[];
  warnings: string[];
};

type ValidationCheck = {
  id: string;
  status: string;
  message: string;
};

type BuildValidationResult = {
  buildId: string;
  targetRoot: string;
  validationStatus: string;
  checks: ValidationCheck[];
  errors: string[];
  warnings: string[];
};

type SmokeTestCheck = {
  id: string;
  status: string;
  message: string;
  command: string | null;
};

type BuildSmokeTestResult = {
  buildId: string;
  targetRoot: string;
  smokeStatus: string;
  validationStatus: string;
  hostPythonCommand: string;
  checks: SmokeTestCheck[];
  errors: string[];
  warnings: string[];
};

type RuntimeProbeCheck = {
  id: string;
  status: string;
  message: string;
  command: string | null;
};

type RuntimeProbeResult = {
  buildId: string;
  targetRoot: string;
  probeStatus: string;
  hostPythonCommand: string;
  endpointUrl: string;
  webPort: number;
  bgePort: number;
  llmPort: number;
  durationMs: number;
  stdoutLogPath: string;
  stderrLogPath: string;
  checks: RuntimeProbeCheck[];
  errors: string[];
  warnings: string[];
};

type EndToEndProbeCheck = {
  id: string;
  status: string;
  message: string;
  command: string | null;
};

type EndToEndProbeResult = {
  buildId: string;
  targetRoot: string;
  probeStatus: string;
  hostPythonCommand: string;
  statsEndpointUrl: string;
  chatEndpointUrl: string;
  probePrompt: string;
  responsePreview: string;
  webPort: number;
  bgePort: number;
  llmPort: number;
  durationMs: number;
  stdoutLogPath: string;
  stderrLogPath: string;
  checks: EndToEndProbeCheck[];
  errors: string[];
  warnings: string[];
};

type FullLocalLlmProbeCheck = {
  id: string;
  status: string;
  message: string;
  command: string | null;
};

type FullLocalLlmProbeResult = {
  buildId: string;
  targetRoot: string;
  probeStatus: string;
  hostPythonCommand: string;
  statsEndpointUrl: string;
  chatEndpointUrl: string;
  probePrompt: string;
  responsePreview: string;
  selectedModelCount: number;
  webPort: number;
  bgePort: number;
  llmPort: number;
  durationMs: number;
  stdoutLogPath: string;
  stderrLogPath: string;
  checks: FullLocalLlmProbeCheck[];
  errors: string[];
  warnings: string[];
};

type FactoryPipelineResult = {
  buildId: string;
  targetRoot: string;
  overallStatus: "passed" | "passed-with-warnings" | "failed";
  reportPath: string;
  assemblyResult: BuildAssemblyResult | null;
  assemblyError: string | null;
  validationResult: BuildValidationResult | null;
  smokeTestResult: BuildSmokeTestResult | null;
  fullLocalLlmProbeResult: FullLocalLlmProbeResult | null;
};

type ReadinessSection = {
  id: string;
  label: string;
  status: string;
  required: boolean;
  summary: string;
  errorCount: number;
  warningCount: number;
};

type BuildReadinessReport = {
  buildId: string;
  targetRoot: string;
  readinessStatus: string;
  readinessScore: number;
  distributionReady: boolean;
  blockedReason: string | null;
  generatedAt: string;
  sections: ReadinessSection[];
  blockingIssues: string[];
  advisoryIssues: string[];
};

type PendingWipeConfirmation = {
  targetLabel: string;
  targetMountPath: string;
  message: string;
};

type AssemblyProgressState = {
  status: "idle" | "running" | "completed" | "error";
  percent: number;
  message: string;
};

type AssemblyProgressEvent = {
  phase: string;
  percent: number;
  message: string;
};

type BuilderState = {
  overview: AppOverview;
  schemas: SchemaSummary[];
  snapshot: WorkspaceSnapshot;
  targets: CandidateTarget[];
  isRefreshingTargets: boolean;
  targetRefreshError: string | null;
  modelCatalog: ModelCatalogResponse | null;
  catalogError: string | null;
  catalogRefreshResult: RefreshModelCatalogResult | null;
  isRefreshingCatalog: boolean;
  catalogRefreshError: string | null;
  manifestPreview: string;
  buildPlan: BuildPlanResponse | null;
  libraryPreparationResult: BuilderLibraryPreparationResult | null;
  isPreparingLibrary: boolean;
  libraryPreparationError: string | null;
  libraryInventory: BuilderLibraryInventory | null;
  isRefreshingLibrary: boolean;
  isDeletingLibraryArtifact: boolean;
  isDownloadingLibraryArtifact: boolean;
  libraryDownloadResult: DownloadBuilderLibraryArtifactResult | null;
  libraryDownloadError: string | null;
  libraryInventoryError: string | null;
  exportResult: BuildPlanExportResult | null;
  isExporting: boolean;
  exportError: string | null;
  assemblyResult: BuildAssemblyResult | null;
  isAssembling: boolean;
  assemblyError: string | null;
  authorizedTargetMountPaths: string[];
  isAuthorizingTargetAccess: boolean;
  targetAuthorizationError: string | null;
  pendingWipeConfirmation: PendingWipeConfirmation | null;
  assemblyProgress: AssemblyProgressState;
  validationResult: BuildValidationResult | null;
  isValidating: boolean;
  validationError: string | null;
  smokeTestResult: BuildSmokeTestResult | null;
  isRunningSmokeTest: boolean;
  smokeTestError: string | null;
  runtimeProbeResult: RuntimeProbeResult | null;
  isRunningRuntimeProbe: boolean;
  runtimeProbeError: string | null;
  endToEndProbeResult: EndToEndProbeResult | null;
  isRunningEndToEndProbe: boolean;
  endToEndProbeError: string | null;
  fullLocalLlmProbeResult: FullLocalLlmProbeResult | null;
  isRunningFullLocalLlmProbe: boolean;
  fullLocalLlmProbeError: string | null;
  readinessReport: BuildReadinessReport | null;
  isGeneratingReadinessReport: boolean;
  readinessReportError: string | null;
  distributionResult: BuildDistributionGateResult | null;
  isPreparingDistribution: boolean;
  distributionError: string | null;
  releasePackageResult: BuildReleasePackageResult | null;
  isPreparingReleasePackage: boolean;
  releasePackageError: string | null;
  mode: "tauri" | "web";
  error: string | null;
};

const defaultPlannerDraft: PlannerDraft = {
  selectedTargetMountPath: null,
  buildMode: "offline-first",
  languages: ["pt-BR", "en", "es"],
  allowGpu: false,
  allowCloudFallback: false,
  creatorLabel: "",
  selectedModelIds: [],
};

type CatalogViewDraft = {
  search: string;
  domain: string;
  language: string;
  hostRam: "all" | "8" | "16" | "32" | "64";
  benchmarkConfidence: "all" | "high" | "medium" | "guarded" | "none";
  cpuOnlyOnly: boolean;
};

type LibraryViewDraft = {
  search: string;
  filter: "all" | "download-request" | "ready" | "issues";
};

type MainView = "create" | "models" | "library" | "validation" | "advanced";
type CreateStep = "target" | "profile" | "review";
type ModelSelectionMode = "recommended" | "manual" | "library";
type ModelBulkSelectionAction = "recommended" | "visible-local" | "clear";

const defaultCatalogViewDraft: CatalogViewDraft = {
  search: "",
  domain: "all",
  language: "all",
  hostRam: "all",
  benchmarkConfidence: "all",
  cpuOnlyOnly: true,
};

const defaultLibraryViewDraft: LibraryViewDraft = {
  search: "",
  filter: "all",
};

const defaultState: BuilderState = {
  overview: {
    appName: "USBRAIN Builder",
    appVersion: "0.1.0",
    workspaceRoot: "workspace indisponivel",
    schemaCount: 0,
    documentCount: 0,
    tauriBackendAvailable: false,
  },
  schemas: [],
  snapshot: {
    rootPath: "workspace indisponivel",
    schemaDirectory: "schemas indisponivel",
    keyDocuments: [],
  },
  targets: [],
  isRefreshingTargets: false,
  targetRefreshError: null,
  modelCatalog: null,
  catalogError: null,
  catalogRefreshResult: null,
  isRefreshingCatalog: false,
  catalogRefreshError: null,
  manifestPreview: "{}",
  buildPlan: null,
  libraryPreparationResult: null,
  isPreparingLibrary: false,
  libraryPreparationError: null,
  libraryInventory: null,
  isRefreshingLibrary: false,
  isDeletingLibraryArtifact: false,
  isDownloadingLibraryArtifact: false,
  libraryDownloadResult: null,
  libraryDownloadError: null,
  libraryInventoryError: null,
  exportResult: null,
  isExporting: false,
  exportError: null,
  assemblyResult: null,
  isAssembling: false,
  assemblyError: null,
  authorizedTargetMountPaths: [],
  isAuthorizingTargetAccess: false,
  targetAuthorizationError: null,
  pendingWipeConfirmation: null,
  assemblyProgress: {
    status: "idle",
    percent: 0,
    message: "Aguardando a criacao do USBRAIN.",
  },
  validationResult: null,
  isValidating: false,
  validationError: null,
  smokeTestResult: null,
  isRunningSmokeTest: false,
  smokeTestError: null,
  runtimeProbeResult: null,
  isRunningRuntimeProbe: false,
  runtimeProbeError: null,
  endToEndProbeResult: null,
  isRunningEndToEndProbe: false,
  endToEndProbeError: null,
  fullLocalLlmProbeResult: null,
  isRunningFullLocalLlmProbe: false,
  fullLocalLlmProbeError: null,
  readinessReport: null,
  isGeneratingReadinessReport: false,
  readinessReportError: null,
  distributionResult: null,
  isPreparingDistribution: false,
  distributionError: null,
  releasePackageResult: null,
  isPreparingReleasePackage: false,
  releasePackageError: null,
  mode: "web",
  error: null,
};

let currentState: BuilderState = defaultState;
let plannerDraft: PlannerDraft = { ...defaultPlannerDraft };
let catalogViewDraft: CatalogViewDraft = { ...defaultCatalogViewDraft };
let libraryViewDraft: LibraryViewDraft = { ...defaultLibraryViewDraft };
let activeMainView: MainView = "create";
let activeCreateStep: CreateStep = "target";
let activeModelSelectionMode: ModelSelectionMode = "recommended";
let selectedManifestFileName = "build-manifest.json";
let lastResolvedBuildRequest: Record<string, unknown> | null = null;
let assemblyProgressIntervalId: number | null = null;
let assemblyProgressUnlisten: UnlistenFn | null = null;
const authorizedTargetScopedPaths = new Map<string, string>();

const appRoot = document.querySelector<HTMLDivElement>("#app");
if (!appRoot) {
  throw new Error("Falha ao encontrar o container principal do app.");
}
const app = appRoot;

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

type FocusSnapshot = {
  id: string;
  selectionStart: number | null;
  selectionEnd: number | null;
};

function captureFocusSnapshot(): FocusSnapshot | null {
  const activeElement = document.activeElement;
  if (!(activeElement instanceof HTMLElement) || !activeElement.id) {
    return null;
  }

  if (activeElement instanceof HTMLInputElement || activeElement instanceof HTMLTextAreaElement) {
    return {
      id: activeElement.id,
      selectionStart: activeElement.selectionStart,
      selectionEnd: activeElement.selectionEnd,
    };
  }

  return {
    id: activeElement.id,
    selectionStart: null,
    selectionEnd: null,
  };
}

function restoreFocusSnapshot(snapshot: FocusSnapshot | null): void {
  if (!snapshot) {
    return;
  }

  const nextElement = document.getElementById(snapshot.id);
  if (!(nextElement instanceof HTMLElement)) {
    return;
  }

  nextElement.focus();
  if (
    (nextElement instanceof HTMLInputElement || nextElement instanceof HTMLTextAreaElement) &&
    snapshot.selectionStart !== null &&
    snapshot.selectionEnd !== null
  ) {
    nextElement.setSelectionRange(snapshot.selectionStart, snapshot.selectionEnd);
  }
}

function formatUnknownError(error: unknown, fallback: string): string {
  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }

  if (typeof error === "string" && error.trim()) {
    return error;
  }

  if (typeof error === "object" && error !== null) {
    try {
      const serialized = JSON.stringify(error);
      if (serialized && serialized !== "{}") {
        return serialized;
      }
    } catch {
      // Ignore serialization failures and fall back to the generic message.
    }
  }

  return fallback;
}

function normalizeTargetPath(value: string): string {
  const trimmedValue = value.trim();
  if (!trimmedValue) {
    return "";
  }

  let normalized = trimmedValue;
  if (trimmedValue.startsWith("file://")) {
    try {
      normalized = decodeURIComponent(new URL(trimmedValue).pathname);
    } catch {
      normalized = trimmedValue.replace(/^file:\/\//, "");
    }
  }

  if (normalized.length > 1) {
    normalized = normalized.replace(/\/+$/, "");
  }
  return normalized;
}

async function ejectMountedTargetWithConfirmation(targetRoot: string): Promise<void> {
  if (!targetRoot || !isMacOsDesktop) {
    return;
  }

  currentState = {
    ...currentState,
    assemblyProgress: {
      status: "running",
      percent: 100,
      message: "Criacao validada. Ejetando o pendrive automaticamente no macOS.",
    },
  };
  render(currentState);

  try {
    await sleep(1200);
    const ejectMessage = await invoke<string>("eject_target", { targetMountPath: targetRoot });
    currentState = {
      ...currentState,
      assemblyProgress: {
        status: "completed",
        percent: 100,
        message: ejectMessage,
      },
    };
    render(currentState);
    await message(
      `${ejectMessage}\n\nClique em OK para confirmar que o pendrive ja foi ejetado e pode ser removido com seguranca.`,
      {
        title: "Pendrive ejetado",
        kind: "info",
      },
    );
  } catch (error) {
    const ejectError = formatUnknownError(error, "A criacao terminou, mas a ejeção automatica do pendrive falhou.");
    currentState = {
      ...currentState,
      assemblyError: ejectError,
      assemblyProgress: {
        status: "error",
        percent: 100,
        message: ejectError,
      },
    };
    render(currentState);
  }
}

function isMountPathAuthorized(authorizedMountPaths: string[], mountPath: string): boolean {
  const normalizedMountPath = normalizeTargetPath(mountPath);
  return authorizedMountPaths.some((authorizedPath) => normalizeTargetPath(authorizedPath) === normalizedMountPath);
}

function isTargetAccessAuthorizationRequired(state: BuilderState, selectedTarget: CandidateTarget | null): boolean {
  return (
    isMacOsDesktop &&
    state.mode === "tauri" &&
    Boolean(selectedTarget) &&
    !isMountPathAuthorized(state.authorizedTargetMountPaths, selectedTarget!.mountPath)
  );
}

async function forgetAuthorizedTargetAccess(mountPath: string): Promise<void> {
  const normalizedMountPath = normalizeTargetPath(mountPath);
  const scopedPath = authorizedTargetScopedPaths.get(normalizedMountPath);
  authorizedTargetScopedPaths.delete(normalizedMountPath);
  if (!scopedPath) {
    return;
  }

  try {
    await stopAccessingSecurityScopedResource(scopedPath);
  } catch {
    // Ignore cleanup failures. The next authorization flow can still proceed.
  }
}

async function authorizeSelectedTargetAccess(): Promise<boolean> {
  const selectedTarget = getSelectedTarget(currentState.targets, plannerDraft.selectedTargetMountPath);
  if (!selectedTarget) {
    currentState = {
      ...currentState,
      targetAuthorizationError: "Nenhum pendrive USB foi selecionado para autorizacao.",
    };
    render(currentState);
    return false;
  }

  if (!isTargetAccessAuthorizationRequired(currentState, selectedTarget)) {
    return true;
  }

  currentState = {
    ...currentState,
    isAuthorizingTargetAccess: true,
    targetAuthorizationError: null,
    assemblyError: null,
  };
  render(currentState);

  try {
    const chosenPath = await open({
      directory: true,
      multiple: false,
      recursive: true,
      canCreateDirectories: false,
      defaultPath: selectedTarget.mountPath,
      title: `Autorize o acesso ao pendrive ${selectedTarget.label}`,
      fileAccessMode: "scoped",
    });

    if (!chosenPath) {
      currentState = {
        ...currentState,
        isAuthorizingTargetAccess: false,
        targetAuthorizationError:
          "A autorizacao do pendrive foi cancelada. O Builder precisa desse acesso para gravar o USBRAIN no macOS.",
      };
      render(currentState);
      return false;
    }

    const normalizedChosenPath = normalizeTargetPath(Array.isArray(chosenPath) ? chosenPath[0] ?? "" : chosenPath);
    const authorizedTarget =
      currentState.targets.find((target) => normalizeTargetPath(target.mountPath) === normalizedChosenPath) ?? null;

    if (!authorizedTarget) {
      currentState = {
        ...currentState,
        isAuthorizingTargetAccess: false,
        targetAuthorizationError:
          "O diretorio autorizado nao corresponde a um pendrive USB detectado pelo Builder. Escolha exatamente o pendrive selecionado.",
      };
      render(currentState);
      return false;
    }

    if (normalizeTargetPath(authorizedTarget.mountPath) !== normalizeTargetPath(selectedTarget.mountPath)) {
      currentState = {
        ...currentState,
        isAuthorizingTargetAccess: false,
        targetAuthorizationError:
          `O Builder estava apontando para ${selectedTarget.mountPath}, mas a autorizacao foi concedida para ${authorizedTarget.mountPath}. Autorize o mesmo pendrive selecionado antes de criar o USBRAIN.`,
      };
      render(currentState);
      return false;
    }

    await startAccessingSecurityScopedResource(chosenPath);
    authorizedTargetScopedPaths.set(
      normalizeTargetPath(authorizedTarget.mountPath),
      Array.isArray(chosenPath) ? chosenPath[0] ?? chosenPath.toString() : chosenPath,
    );
    plannerDraft.selectedTargetMountPath = authorizedTarget.mountPath;
    currentState = {
      ...currentState,
      isAuthorizingTargetAccess: false,
      targetAuthorizationError: null,
      authorizedTargetMountPaths: Array.from(
        new Set([
          ...currentState.authorizedTargetMountPaths.map((mountPath) => normalizeTargetPath(mountPath)),
          normalizeTargetPath(authorizedTarget.mountPath),
        ]),
      ),
    };
    render(currentState);
    return true;
  } catch (error) {
    currentState = {
      ...currentState,
      isAuthorizingTargetAccess: false,
      targetAuthorizationError: formatUnknownError(
        error,
        "O macOS recusou a autorizacao do pendrive. Tente novamente e confirme o acesso ao volume no seletor nativo.",
      ),
    };
    render(currentState);
    return false;
  }
}

const assemblyProgressStages: Array<{ percent: number; message: string }> = [
  { percent: 10, message: "Preparando a criacao do USBRAIN no pendrive." },
  { percent: 24, message: "Validando o destino USB e organizando a limpeza necessaria." },
  { percent: 46, message: "Copiando runtime, ferramentas e estrutura base do USBRAIN." },
  { percent: 72, message: "Gravando modelos, manifests e runtime auxiliar no pendrive." },
  { percent: 90, message: "Finalizando a montagem e consolidando o USBRAIN." },
];

function stopAssemblyProgressTimer(): void {
  if (assemblyProgressIntervalId !== null) {
    window.clearInterval(assemblyProgressIntervalId);
    assemblyProgressIntervalId = null;
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => {
    window.setTimeout(resolve, ms);
  });
}

async function ensureAssemblyProgressListener(): Promise<void> {
  if (assemblyProgressUnlisten || currentState.mode !== "tauri") {
    return;
  }

  try {
    assemblyProgressUnlisten = await listen<AssemblyProgressEvent>("usbrain://assembly-progress", (event) => {
      currentState = {
        ...currentState,
        assemblyProgress: {
          status: event.payload.percent >= 100 ? "completed" : "running",
          percent: event.payload.percent,
          message: event.payload.message,
        },
      };
      render(currentState);
    });
  } catch {
    // Ignore listener errors in fallback/web environments.
  }
}

function startAssemblyProgress(): void {
  stopAssemblyProgressTimer();
  if (currentState.mode === "tauri") {
    return;
  }

  let stageIndex = 0;
  assemblyProgressIntervalId = window.setInterval(() => {
    if (!currentState.isAssembling) {
      stopAssemblyProgressTimer();
      return;
    }

    stageIndex = Math.min(stageIndex + 1, assemblyProgressStages.length - 1);
    const stage = assemblyProgressStages[stageIndex];
    const nextPercent =
      stageIndex === assemblyProgressStages.length - 1
        ? Math.min(Math.max(currentState.assemblyProgress.percent, stage.percent), 94)
        : stage.percent;

    currentState = {
      ...currentState,
      assemblyProgress: {
        status: "running",
        percent: nextPercent,
        message: stage.message,
      },
    };
    render(currentState);
  }, 900);
}

function formatStorage(value?: number): string {
  return typeof value === "number" ? `${value.toFixed(1)} GB` : "indisponivel";
}

function formatCatalogMetricValue(value: number, unit: string | null): string {
  const formattedValue = Number.isInteger(value) ? value.toString() : value.toFixed(1);
  return unit ? `${formattedValue} ${unit}` : formattedValue;
}

function renderCatalogPerformanceMarkup(model: CatalogModelSummary): string {
  const technicalItems: string[] = [];
  if (typeof model.technicalProfile?.contextWindowTokens === "number") {
    technicalItems.push(`Contexto ${escapeHtml(model.technicalProfile.contextWindowTokens.toLocaleString("en-US"))} tokens`);
  }
  if (typeof model.technicalProfile?.estimatedPeakMemoryGb === "number") {
    technicalItems.push(`Pico de memoria ${escapeHtml(formatStorage(model.technicalProfile.estimatedPeakMemoryGb))}`);
  }
  if (typeof model.technicalProfile?.estimatedPromptTokensPerSecond === "number") {
    technicalItems.push(
      `Prompt ${escapeHtml(formatCatalogMetricValue(model.technicalProfile.estimatedPromptTokensPerSecond, "tok/s"))}`,
    );
  }
  if (typeof model.technicalProfile?.estimatedGenerationTokensPerSecond === "number") {
    technicalItems.push(
      `Geracao ${escapeHtml(formatCatalogMetricValue(model.technicalProfile.estimatedGenerationTokensPerSecond, "tok/s"))}`,
    );
  }
  if (model.technicalProfile?.recommendedGpuClass) {
    technicalItems.push(`GPU opcional ${escapeHtml(model.technicalProfile.recommendedGpuClass)}`);
  }

  const technicalMarkup = technicalItems.length
    ? `<p class="muted">Perfil tecnico: ${technicalItems.join(" · ")}</p>`
    : "";
  const technicalNotesMarkup = model.technicalProfile?.notes
    ? `<p class="muted">${escapeHtml(model.technicalProfile.notes)}</p>`
    : "";
  const benchmarkMetricsMarkup = model.benchmarkMetrics.length
    ? `<p class="muted">Indicadores: ${escapeHtml(
        model.benchmarkMetrics
          .map((metric) =>
            `${metric.benchmarkSuite ? `${metric.benchmarkSuite} ` : ""}${metric.label} ${formatCatalogMetricValue(metric.value, metric.unit)}`,
          )
          .join(" · "),
      )}</p>`
    : "";

  return `${technicalMarkup}${technicalNotesMarkup}${benchmarkMetricsMarkup}`;
}

function benchmarkConfidenceClass(summary: CatalogBenchmarkSummary): string {
  switch (summary.confidenceLevel) {
    case "high":
      return "benchmark-confidence-high";
    case "medium":
      return "benchmark-confidence-medium";
    case "guarded":
      return "benchmark-confidence-guarded";
    default:
      return "";
  }
}

function benchmarkConfidenceRank(summary: CatalogBenchmarkSummary | null): number {
  switch (summary?.confidenceLevel) {
    case "high":
      return 3;
    case "medium":
      return 2;
    case "guarded":
      return 1;
    default:
      return 0;
  }
}

function compareCatalogModels(left: CatalogModelSummary, right: CatalogModelSummary): number {
  return (
    benchmarkConfidenceRank(right.benchmarkSummary) - benchmarkConfidenceRank(left.benchmarkSummary) ||
    Number(Boolean(right.benchmarkSummary)) - Number(Boolean(left.benchmarkSummary)) ||
    Number(Boolean(right.availableLocally)) - Number(Boolean(left.availableLocally)) ||
    right.qualityScore - left.qualityScore ||
    left.diskSizeGb - right.diskSizeGb ||
    left.displayName.localeCompare(right.displayName)
  );
}

function getSelectedTarget(targets: CandidateTarget[], mountPath: string | null): CandidateTarget | null {
  if (!targets.length) {
    return null;
  }

  return targets.find((target) => target.mountPath === mountPath) ?? targets[0];
}

function buildPreviewRequest(targets: CandidateTarget[]): Record<string, unknown> {
  return buildPreviewRequestWithOptions(targets);
}

function buildPreviewRequestWithOptions(
  targets: CandidateTarget[],
  options?: { wipeTargetIfNeeded?: boolean },
): Record<string, unknown> {
  const selectedTarget = getSelectedTarget(targets, plannerDraft.selectedTargetMountPath);

  return {
    request: {
      targetMountPath: selectedTarget?.mountPath ?? null,
      targetLabel: selectedTarget?.label ?? null,
      buildMode: plannerDraft.buildMode,
      languages: plannerDraft.languages,
      allowGpu: plannerDraft.allowGpu,
      allowCloudFallback: plannerDraft.allowCloudFallback,
      creatorLabel: plannerDraft.creatorLabel.trim() || null,
      selectedModelIds: plannerDraft.selectedModelIds,
      wipeTargetIfNeeded: options?.wipeTargetIfNeeded ?? false,
    },
  };
}

// #region debug-point B:planner-request-debug
function cloneBuildRequest(request: Record<string, unknown>): Record<string, unknown> {
  return JSON.parse(JSON.stringify(request)) as Record<string, unknown>;
}

function captureResolvedBuildRequest(targets: CandidateTarget[]): Record<string, unknown> {
  const request = buildPreviewRequest(targets);
  lastResolvedBuildRequest = cloneBuildRequest(request);
  return request;
}

function resolvedBuildRequest(
  targets: CandidateTarget[],
  options?: { wipeTargetIfNeeded?: boolean },
): Record<string, unknown> {
  const request = lastResolvedBuildRequest ? cloneBuildRequest(lastResolvedBuildRequest) : buildPreviewRequest(targets);
  if (options?.wipeTargetIfNeeded === undefined) {
    return request;
  }

  const payload = request.request;
  if (!payload || typeof payload !== "object" || Array.isArray(payload)) {
    return buildPreviewRequestWithOptions(targets, options);
  }

  return {
    ...request,
    request: {
      ...(payload as Record<string, unknown>),
      wipeTargetIfNeeded: options.wipeTargetIfNeeded,
    },
  };
}

function buildModelCatalogRequest(targets: CandidateTarget[]): Record<string, unknown> {
  const selectedTarget = getSelectedTarget(targets, plannerDraft.selectedTargetMountPath);

  return {
    query: {
      targetMountPath: selectedTarget?.mountPath ?? null,
    },
  };
}

function filteredCatalogModels(state: BuilderState): CatalogModelSummary[] {
  const models = state.modelCatalog?.models ?? [];
  const search = catalogViewDraft.search.trim().toLowerCase();

  return models
    .filter((model) => {
    if (search) {
      const haystack = [
        model.displayName,
        model.family,
        model.description,
        model.rationale,
        model.domains.join(" "),
        model.capabilities.join(" "),
      ]
        .join(" ")
        .toLowerCase();

      if (!haystack.includes(search)) {
        return false;
      }
    }

    if (catalogViewDraft.domain !== "all" && !model.domains.includes(catalogViewDraft.domain)) {
      return false;
    }

    if (catalogViewDraft.language !== "all" && !model.languages.includes(catalogViewDraft.language)) {
      return false;
    }

    if (catalogViewDraft.hostRam !== "all" && model.hostRamTierGb !== Number(catalogViewDraft.hostRam)) {
      return false;
    }

    if (catalogViewDraft.benchmarkConfidence === "none" && model.benchmarkSummary) {
      return false;
    }

    if (
      catalogViewDraft.benchmarkConfidence !== "all" &&
      catalogViewDraft.benchmarkConfidence !== "none" &&
      model.benchmarkSummary?.confidenceLevel !== catalogViewDraft.benchmarkConfidence
    ) {
      return false;
    }

    if (catalogViewDraft.cpuOnlyOnly && !model.cpuOnlyViable) {
      return false;
    }

    return true;
    })
    .sort(compareCatalogModels);
}

function buildFallbackManifestPreview(targets: CandidateTarget[]): string {
  const selectedTarget = getSelectedTarget(targets, plannerDraft.selectedTargetMountPath);
  const payload = {
    schema_version: "1.0",
    build_id: "preview-usbrain-build",
    build_timestamp: "2026-06-27T00:00:00Z",
    builder_version: currentState.overview.appVersion,
    runtime_version_min: "0.1.0",
    target_os_family: ["windows", "linux"],
    target_device_profile: {
      label: selectedTarget?.label ?? "USB target preview",
      capacity_gb: selectedTarget?.capacityGb ?? 64,
      filesystem: selectedTarget?.filesystem ?? "desconhecido",
      removable: selectedTarget?.removable ?? true,
    },
    profile_ref: "manifests/usbrain-profile.json",
    execution_policy_ref: "manifests/execution-policy.json",
    installed_models_ref: "manifests/installed-models.json",
    installed_packs_ref: "manifests/installed-packs.json",
    runtime_config_ref: "manifests/runtime-config.json",
    provider_config_ref: "manifests/provider-config.json",
    validation_report_ref: "manifests/validation-report.json",
    build_mode: plannerDraft.buildMode,
    languages: plannerDraft.languages,
    created_by: plannerDraft.creatorLabel || undefined,
    notes: `Preview local para ${selectedTarget?.mountPath ?? "workspace indisponivel"}`,
    planning_hints: {
      available_gb: selectedTarget?.availableGb ?? null,
      usage_percent: selectedTarget?.usagePercent ?? null,
      recommended_model_tier: selectedTarget?.recommendedModelTier ?? "indisponivel",
      estimated_model_slots: selectedTarget?.estimatedModelSlots ?? 0,
      allow_gpu: plannerDraft.allowGpu,
      allow_cloud_fallback: plannerDraft.allowCloudFallback,
      warnings: selectedTarget?.warnings ?? [],
    },
  };

  return JSON.stringify(payload, null, 2);
}

function buildFallbackBuildPlan(targets: CandidateTarget[]): BuildPlanResponse {
  const selectedTarget = getSelectedTarget(targets, plannerDraft.selectedTargetMountPath);
  const manifestPreview = buildFallbackManifestPreview(targets);

  return {
    summary: {
      buildId: "preview-usbrain-build",
      targetLabel: selectedTarget?.label ?? "USB target preview",
      targetMountPath: selectedTarget?.mountPath ?? "indisponivel",
      filesystem: selectedTarget?.filesystem ?? "desconhecido",
      capacityGb: selectedTarget?.capacityGb,
      buildMode: plannerDraft.buildMode,
      profileMode: plannerDraft.buildMode === "hybrid" ? "hybrid" : plannerDraft.buildMode === "local-assisted" ? "local-assisted" : "offline",
      privacyMode: plannerDraft.allowCloudFallback ? "hybrid-explicit" : "local-first",
      availableGb: selectedTarget?.availableGb,
      installBudgetGb: selectedTarget?.capacityGb ? Math.max(selectedTarget.capacityGb - 12, 0) : selectedTarget?.availableGb,
      estimatedModelSlots: selectedTarget?.estimatedModelSlots ?? 0,
      selectedModelCount: 0,
      pendingModelDownloadCount: 0,
      selectedPackCount: 0,
      allowGpu: plannerDraft.allowGpu,
      allowCloudFallback: plannerDraft.allowCloudFallback,
      primaryLanguage: plannerDraft.languages[0] ?? "pt-BR",
      languages: plannerDraft.languages,
      warnings: selectedTarget?.warnings ?? [],
    },
    models: [],
    pendingModelDownloads: [],
    hostRamRecommendations: [],
    packs: [],
    manifests: [
      {
        fileName: "build-manifest.json",
        title: "Build Manifest",
        contents: manifestPreview,
      },
    ],
  };
}

function getSelectedManifest(buildPlan: BuildPlanResponse | null): GeneratedManifest | null {
  if (!buildPlan?.manifests.length) {
    return null;
  }

  return buildPlan.manifests.find((manifest) => manifest.fileName === selectedManifestFileName) ?? buildPlan.manifests[0];
}

function formatBytes(value: number): string {
  if (value < 1024) {
    return `${value} B`;
  }

  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KB`;
  }

  return `${(value / (1024 * 1024)).toFixed(2)} MB`;
}

async function invokeSafe<T>(command: string, fallback: T, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch {
    return fallback;
  }
}

function renderMainTabButton(tab: MainView, label: string, description: string): string {
  const activeClass = activeMainView === tab ? "tab-button-active" : "";
  return `
    <button
      type="button"
      class="tab-button ${activeClass}"
      data-main-tab="${tab}"
      aria-pressed="${activeMainView === tab ? "true" : "false"}"
    >
      <strong>${label}</strong>
      <span>${description}</span>
    </button>
  `;
}

function buildPipelineStatus(state: BuilderState): { label: string; className: string } {
  if (state.assemblyError || state.validationError || state.fullLocalLlmProbeError) {
    return { label: "Atencao necessaria", className: "status bad" };
  }

  if (state.fullLocalLlmProbeResult?.probeStatus === "passed") {
    return { label: "Pendrive aprovado", className: "status good" };
  }

  if (state.assemblyProgress.status === "running") {
    return { label: "Criando USBRAIN", className: "status warn" };
  }

  if (state.validationResult?.validationStatus === "passed" || state.smokeTestResult?.smokeStatus === "passed") {
    return { label: "Validacao em progresso", className: "status warn" };
  }

  return { label: "Planejamento", className: "status warn" };
}

function catalogResultSummary(count: number, total: number): string {
  if (!total) {
    return "Nenhum modelo carregado";
  }

  if (count === total) {
    return `${count} modelos visiveis`;
  }

  return `${count} de ${total} modelos visiveis`;
}

function createStepClass(step: CreateStep): string {
  if (activeCreateStep === step) {
    return "wizard-step-active";
  }

  if (
    (step === "target" && !!plannerDraft.selectedTargetMountPath) ||
    (step === "profile" && plannerDraft.selectedModelIds.length > 0) ||
    (step === "review" && !!plannerDraft.selectedTargetMountPath)
  ) {
    return "wizard-step-done";
  }

  return "wizard-step-pending";
}

function renderModelSelectionModeButton(
  mode: ModelSelectionMode,
  label: string,
  description: string,
  badge?: string,
): string {
  const activeClass = activeModelSelectionMode === mode ? "mode-button-active" : "";
  return `
    <button
      type="button"
      class="mode-button ${activeClass}"
      data-model-selection-mode="${mode}"
      aria-pressed="${activeModelSelectionMode === mode ? "true" : "false"}"
    >
      <strong>${label}</strong>
      <span>${description}</span>
      ${badge ? `<em>${badge}</em>` : ""}
    </button>
  `;
}

function renderModelBulkActionButton(
  action: ModelBulkSelectionAction,
  label: string,
  disabled: boolean,
  variant: "secondary" | "ghost" = "secondary",
): string {
  const className = variant === "ghost" ? "primary-button ghost-button" : "primary-button secondary-button";
  return `<button type="button" class="${className}" data-bulk-selection="${action}" ${disabled ? "disabled" : ""}>${label}</button>`;
}

function renderLibraryFilterButton(filter: LibraryViewDraft["filter"], label: string, count: number): string {
  const activeClass = libraryViewDraft.filter === filter ? "mode-button-active" : "";
  return `
    <button
      type="button"
      class="mode-button ${activeClass}"
      data-library-filter="${filter}"
      aria-pressed="${libraryViewDraft.filter === filter ? "true" : "false"}"
    >
      <strong>${label}</strong>
      <em>${count}</em>
    </button>
  `;
}

function buildModeLabel(mode: BuildMode): string {
  switch (mode) {
    case "offline-first":
      return "Offline recomendado";
    case "offline-only":
      return "Offline estrito";
    case "local-assisted":
      return "Local assistido";
    case "hybrid":
      return "Hibrido";
  }
}

function buildModeSummary(mode: BuildMode): string {
  switch (mode) {
    case "offline-first":
      return "Prioriza funcionamento local com apoio opcional do host.";
    case "offline-only":
      return "Bloqueia qualquer dependencia externa e prioriza isolamento.";
    case "local-assisted":
      return "Mantem foco local, mas assume um host mais colaborativo.";
    case "hybrid":
      return "Permite uma experiencia mista quando o contexto exigir.";
  }
}

function languageLabel(language: string): string {
  switch (language) {
    case "pt-BR":
      return "Portugues";
    case "en":
      return "Ingles";
    case "es":
      return "Espanhol";
    default:
      return language;
  }
}

function render(state: BuilderState): void {
  const focusSnapshot = captureFocusSnapshot();
  const selectedTarget = getSelectedTarget(state.targets, plannerDraft.selectedTargetMountPath);
  const hasPendingModelDownloads = (state.buildPlan?.summary.pendingModelDownloadCount ?? 0) > 0;
  const targetAccessAuthorizationRequired = isTargetAccessAuthorizationRequired(state, selectedTarget);
  const selectedManifest = getSelectedManifest(state.buildPlan);
  const selectedCatalogModels = (state.modelCatalog?.models ?? []).filter((model) =>
    plannerDraft.selectedModelIds.includes(model.modelId),
  );
  const selectedModelsMarkup = selectedCatalogModels.length
    ? `
      <div class="selection-summary">
        ${selectedCatalogModels
          .map((model) => `<span class="selection-chip">${escapeHtml(model.displayName)}</span>`)
          .join("")}
      </div>
    `
    : `
      <div class="warning-box">
        <strong>Nenhuma LLM selecionada manualmente</strong>
        <p>
          O Builder ainda nao recebeu uma selecao manual de modelos para esta criacao.
          Se voce prosseguir assim, o app pode cair no lineup automatico em vez das LLMs que voce queria.
        </p>
      </div>
    `;
  const schemaCards = state.schemas.length
    ? state.schemas
        .map(
          (schema) => `
            <article class="card compact">
              <div class="eyebrow">Schema</div>
              <h3>${escapeHtml(schema.title || schema.fileName)}</h3>
              <p class="muted">${escapeHtml(schema.fileName)}</p>
              <code>${escapeHtml(schema.schemaId || "sem schema id")}</code>
            </article>
          `,
        )
        .join("")
    : `
      <article class="card compact empty-state">
        <h3>Nenhum schema carregado</h3>
        <p>O backend ainda nao respondeu ou os schemas nao foram localizados.</p>
      </article>
    `;

  const docs = state.snapshot.keyDocuments.length
    ? state.snapshot.keyDocuments.map((doc) => `<li>${escapeHtml(doc)}</li>`).join("")
    : "<li>Nenhum documento-chave encontrado.</li>";

  const targets = state.targets.length
    ? state.targets
        .map(
          (target) => `
            <article class="card compact target-card ${target.mountPath === selectedTarget?.mountPath ? "target-active" : ""}" data-target-mount="${escapeHtml(target.mountPath)}" tabindex="0" role="button" aria-pressed="${target.mountPath === selectedTarget?.mountPath ? "true" : "false"}">
              <div class="target-card-header">
                <div>
                  <div class="eyebrow">Pendrive USB</div>
                  <h3>${escapeHtml(target.label)}</h3>
                </div>
                ${target.mountPath === selectedTarget?.mountPath ? '<span class="status good">Selecionado</span>' : ""}
              </div>
              <p class="muted target-path">${escapeHtml(target.mountPath)}</p>
              <dl class="target-metrics">
                <div><dt>Total</dt><dd>${formatStorage(target.capacityGb)}</dd></div>
                <div><dt>Livre</dt><dd>${formatStorage(target.availableGb)}</dd></div>
                <div><dt>Uso</dt><dd>${typeof target.usagePercent === "number" ? `${target.usagePercent.toFixed(0)}%` : "indisponivel"}</dd></div>
              </dl>
              <div class="pill-row">
                <span class="info-pill">${escapeHtml(target.filesystem)}</span>
                <span class="info-pill">${escapeHtml(target.recommendedModelTier)}</span>
                <span class="info-pill">${target.warnings.length ? `${target.warnings.length} alerta(s)` : "Sem alertas"}</span>
              </div>
              <p class="muted">${escapeHtml(target.source)}</p>
            </article>
          `,
        )
        .join("")
    : `
      <article class="card compact empty-state">
        <h3>Nenhum pendrive USB detectado</h3>
        <p>Conecte um pendrive USB real para habilitar a criacao do USBRAIN.</p>
      </article>
    `;

  const modeLabel = state.mode === "tauri" ? "Backend Tauri ativo" : "Modo web de desenvolvimento";
  const modeClass = state.mode === "tauri" ? "status good" : "status warn";
  const createButtonDisabled =
    Boolean(state.pendingWipeConfirmation) ||
    state.isAuthorizingTargetAccess ||
    state.isAssembling ||
    state.isExporting ||
    state.isValidating ||
    state.isRunningSmokeTest ||
    state.isRunningRuntimeProbe ||
    state.isRunningEndToEndProbe ||
    state.isRunningFullLocalLlmProbe ||
    state.isGeneratingReadinessReport ||
    state.isPreparingDistribution ||
    state.isPreparingReleasePackage ||
    hasPendingModelDownloads ||
    !selectedTarget;
  const selectedTargetAuthorized =
    Boolean(selectedTarget) && isMountPathAuthorized(state.authorizedTargetMountPaths, selectedTarget!.mountPath);
  const authorizationMarkup = selectedTarget && isMacOsDesktop && state.mode === "tauri"
    ? targetAccessAuthorizationRequired
      ? `
        <div class="warning-box">
          <strong>Autorizacao nativa do macOS</strong>
          <p>
            O Builder vai pedir acesso explicito ao pendrive ${escapeHtml(selectedTarget.label)}
            antes de gravar em ${escapeHtml(selectedTarget.mountPath)}.
          </p>
          <p class="muted">Isso evita o bloqueio "Operation not permitted" ao atualizar o USBRAIN no volume removivel.</p>
          <div class="inline-actions">
            <button id="authorize-target-access" class="primary-button ghost-button" type="button" ${state.isAuthorizingTargetAccess ? "disabled" : ""}>
              ${state.isAuthorizingTargetAccess ? "Autorizando..." : "Autorizar pendrive agora"}
            </button>
          </div>
        </div>
      `
      : selectedTargetAuthorized
        ? `
          <p class="status good">Acesso nativo ao pendrive autorizado para esta sessao do Builder.</p>
        `
        : ""
    : "";
  const assemblyProgressMarkup =
    state.assemblyProgress.status !== "idle"
      ? `
        <div class="assembly-progress-panel">
          <div class="progress-header">
            <strong>${
              state.assemblyProgress.status === "completed"
                ? "USBRAIN finalizado"
                : state.assemblyProgress.status === "error"
                  ? "Falha na criacao"
                  : "Criando USBRAIN"
            }</strong>
            <span>${state.assemblyProgress.percent}%</span>
          </div>
          <div class="progress-bar" aria-hidden="true">
            <div class="progress-fill ${state.assemblyProgress.status === "running" ? "progress-fill-running" : ""}" style="width: ${state.assemblyProgress.percent}%"></div>
          </div>
          <p class="muted">${escapeHtml(state.assemblyProgress.message)}</p>
        </div>
      `
      : "";
  const wipeConfirmationMarkup = state.pendingWipeConfirmation
    ? `
      <div class="warning-box">
        <strong>Confirmacao necessaria</strong>
        <p>
          O pendrive ${escapeHtml(state.pendingWipeConfirmation.targetLabel)} em
          ${escapeHtml(state.pendingWipeConfirmation.targetMountPath)} ja contem dados.
          Confirme abaixo para apagar o conteudo atual e criar o USBRAIN.
        </p>
        <p class="muted">${escapeHtml(state.pendingWipeConfirmation.message)}</p>
        <div class="inline-actions">
          <button id="confirm-wipe-create" class="primary-button" type="button">Apagar pendrive e criar USBRAIN</button>
          <button id="cancel-wipe-create" class="primary-button ghost-button" type="button">Cancelar</button>
        </div>
      </div>
    `
    : "";
  const targetOptions = state.targets
    .map(
      (target) => `
        <option value="${escapeHtml(target.mountPath)}" ${target.mountPath === selectedTarget?.mountPath ? "selected" : ""}>
          ${escapeHtml(target.label)} (${formatStorage(target.capacityGb)} totais · ${formatStorage(target.availableGb)} livres)
        </option>
      `,
    )
    .join("");
  const targetRefreshStatusMarkup = state.targetRefreshError
    ? `<p class="status bad">${escapeHtml(state.targetRefreshError)}</p>`
    : state.isRefreshingTargets
      ? '<p class="status warn">Atualizando lista de pendrives...</p>'
      : '<p class="muted">Conectou o pendrive depois de abrir o Builder? Atualize a lista aqui sem reiniciar a aplicacao.</p>';
  const warningList =
    selectedTarget?.warnings.length
      ? selectedTarget.warnings.map((warning) => `<li>${escapeHtml(warning)}</li>`).join("")
      : "<li>Nenhum alerta relevante para este alvo.</li>";
  const catalogModels = filteredCatalogModels(state);
  const localCatalogModels = catalogModels.filter((model) => model.availableLocally);
  const recommendedModelIds = Array.from(
    new Set((state.buildPlan?.hostRamRecommendations ?? []).map((profile) => profile.suggestedModel.modelId)),
  );
  const visibleLocalModelIds = Array.from(new Set(localCatalogModels.map((model) => model.modelId)));
  const directDownloadReadyCount = (state.modelCatalog?.models ?? []).filter((model) => model.downloadLocations.length > 0).length;
  const renderCatalogCards = (models: typeof catalogModels): string =>
    models.length
      ? models
        .map(
          (model) => `
            <article class="card compact catalog-card ${plannerDraft.selectedModelIds.includes(model.modelId) ? "catalog-card-selected" : ""}" data-model-id="${escapeHtml(model.modelId)}" tabindex="0" role="checkbox" aria-checked="${plannerDraft.selectedModelIds.includes(model.modelId) ? "true" : "false"}">
              <label class="catalog-card-header">
                <input type="checkbox" class="catalog-model-toggle" data-model-id="${escapeHtml(model.modelId)}" ${plannerDraft.selectedModelIds.includes(model.modelId) ? "checked" : ""} />
                <div>
                  <div class="eyebrow">Catalogo vivo</div>
                  <h3>${escapeHtml(model.displayName)}</h3>
                </div>
              </label>
              <p class="muted">${escapeHtml(model.family)} · ${escapeHtml(model.variant)} · score ${model.qualityScore}</p>
              <dl class="target-metrics">
                <div><dt>Disco</dt><dd>${formatStorage(model.diskSizeGb)}</dd></div>
                <div><dt>RAM host</dt><dd>${model.hostRamTierGb} GB</dd></div>
                <div><dt>Status</dt><dd>${escapeHtml(model.availabilityLabel)}</dd></div>
              </dl>
              <div class="pill-row">
                <span class="info-pill">${escapeHtml(model.domains.join(" · ") || "Sem dominio")}</span>
                <span class="info-pill">${escapeHtml(model.languages.join(" · ") || "Sem idioma")}</span>
                <span class="info-pill">${model.fitsSelectedTarget ? "Cabe no alvo" : "Nao cabe no alvo"}</span>
              </div>
              <p class="catalog-card-summary">${escapeHtml(model.editorialRecommendation ?? model.description)}</p>
              ${
                model.benchmarkSummary
                  ? `<p class="muted">Benchmark: ${escapeHtml(model.benchmarkSummary.sourceLabel)} <span class="benchmark-confidence ${benchmarkConfidenceClass(model.benchmarkSummary)}">${escapeHtml(model.benchmarkSummary.confidenceLabel)}</span>${model.benchmarkSummary.rankLabel ? ` · ${escapeHtml(model.benchmarkSummary.rankLabel)}` : ""}${typeof model.benchmarkSummary.compositeScore === "number" ? ` · score ${model.benchmarkSummary.compositeScore.toFixed(1)}` : ""}</p>`
                  : `<p class="muted">Benchmark: ainda nao foi encontrado benchmark publico comparavel para esta variante/familia.</p>`
              }
              <details class="card-disclosure">
                <summary>Detalhes tecnicos</summary>
                <p>${escapeHtml(model.description)}</p>
                <p class="muted">Origem: ${escapeHtml(model.providerOrOrigin ?? "nao informado")} · Licenca: ${escapeHtml(model.license)}</p>
                <p class="muted">Variante tecnica: ${escapeHtml(model.variantId ?? model.variant)} · ${escapeHtml(model.quantization ?? "quantizacao nao informada")} · ${escapeHtml(model.fileFormat ?? "formato nao informado")}</p>
                ${renderCatalogPerformanceMarkup(model)}
                ${
                  model.benchmarkSummary
                    ? `<p>${escapeHtml(model.benchmarkSummary.summary)}</p>`
                    : ""
                }
                <p>${escapeHtml(model.rationale)}</p>
                <p class="muted">${escapeHtml(model.homepageOrSource)}</p>
              </details>
              ${
                !model.availableLocally && model.downloadLocations.length
                  ? `<button class="primary-button ghost-button catalog-download-model" type="button" data-model-id="${escapeHtml(model.modelId)}" ${state.isDownloadingLibraryArtifact || state.isRefreshingCatalog || state.isRefreshingLibrary ? "disabled" : ""}>${state.isDownloadingLibraryArtifact ? "Baixando..." : "Baixar para biblioteca"}</button>`
                  : !model.availableLocally
                    ? `<p class="muted">URL direta ainda nao resolvida neste snapshot.</p>`
                    : ""
              }
            </article>
          `,
        )
        .join("")
      : `
      <article class="card compact empty-state">
        <h3>Nenhum modelo encontrado</h3>
        <p>Ajuste a busca ou os filtros do catalogo para encontrar outra combinacao de LLMs.</p>
      </article>
    `;
  const catalogCards = renderCatalogCards(catalogModels);
  const localCatalogCards = localCatalogModels.length
    ? renderCatalogCards(localCatalogModels)
    : `
      <article class="card compact empty-state">
        <h3>Nenhum modelo local neste recorte</h3>
        <p>A biblioteca ainda nao contem modelos visiveis para os filtros atuais.</p>
      </article>
    `;
  const catalogSelectionMarkup = selectedCatalogModels.length
    ? selectedCatalogModels
        .map(
          (model) =>
            `<li>${escapeHtml(model.displayName)} · ${formatStorage(model.diskSizeGb)} · alvo ${model.hostRamTierGb} GB · ${escapeHtml(model.availabilityLabel)}</li>`,
        )
        .join("")
    : "<li>Nenhuma LLM escolhida manualmente ainda.</li>";
  const modelCards = state.buildPlan?.models.length
    ? state.buildPlan.models
        .map(
          (model) => `
            <article class="card compact">
              <div class="eyebrow">Modelo planejado</div>
              <h3>${escapeHtml(model.displayName)}</h3>
              <p class="muted">${escapeHtml(model.role)} · ${escapeHtml(model.runtimeFamily)}</p>
              <dl class="target-metrics">
                <div><dt>Disco</dt><dd>${formatStorage(model.diskSizeGb)}</dd></div>
                <div><dt>RAM alvo</dt><dd>${formatStorage(model.recommendedRamGb)}</dd></div>
                <div><dt>Arquivo</dt><dd>${escapeHtml(model.sourceFileName)}</dd></div>
                <div><dt>Origem</dt><dd>${escapeHtml(model.availabilityLabel)}</dd></div>
              </dl>
            </article>
          `,
        )
        .join("")
    : `
      <article class="card compact empty-state">
        <h3>Nenhum modelo planejado</h3>
        <p>Nenhum modelo ja resolvido na biblioteca local entrou no plano deste pendrive.</p>
      </article>
    `;
  const pendingModelDownloadCards = state.buildPlan?.pendingModelDownloads.length
    ? state.buildPlan.pendingModelDownloads
        .map(
          (model) => `
            <article class="card compact">
              <div class="eyebrow">Download pendente</div>
              <h3>${escapeHtml(model.displayName)}</h3>
              <p class="muted">${escapeHtml(model.availabilityLabel)}</p>
              <dl class="target-metrics">
                <div><dt>Disco</dt><dd>${formatStorage(model.diskSizeGb)}</dd></div>
                <div><dt>RAM alvo</dt><dd>${formatStorage(model.recommendedRamGb)}</dd></div>
                <div><dt>Arquivo</dt><dd>${escapeHtml(model.localFileName)}</dd></div>
              </dl>
              <p class="muted">${escapeHtml(model.homepageOrSource)}</p>
            </article>
          `,
        )
        .join("")
    : `
      <article class="card compact empty-state">
        <h3>Nenhum download pendente</h3>
        <p>Todas as LLMs selecionadas para este plano ja estao resolvidas na biblioteca local.</p>
      </article>
    `;
  const libraryPreparationStatusMarkup = state.libraryPreparationResult
    ? `<div class="warning-box">
        <strong>Biblioteca local preparada</strong>
        <p>
          ${state.libraryPreparationResult.copiedModelsCount} modelo(s) copiados, ${state.libraryPreparationResult.alreadyAvailableCount}
          ja disponivel(is) e ${state.libraryPreparationResult.writtenManifestsCount} manifesto(s) de aquisicao gerado(s).
        </p>
        <p class="muted">${escapeHtml(state.libraryPreparationResult.libraryRoot)}</p>
      </div>`
    : "";
  const catalogRefreshStatusMarkup = state.catalogRefreshResult
    ? `<div class="warning-box">
        <strong>Catalogo vivo atualizado</strong>
        <p>${state.catalogRefreshResult.modelCount} modelo(s) sincronizados de ${escapeHtml(state.catalogRefreshResult.sourceUrl)}</p>
        <p class="muted">${escapeHtml(state.catalogRefreshResult.catalogVersion)} · ${escapeHtml(state.catalogRefreshResult.cachePath)}</p>
      </div>`
    : "";
  const libraryDownloadStatusMarkup = state.libraryDownloadResult
    ? `<div class="warning-box">
        <strong>Download concluido</strong>
        <p>${escapeHtml(state.libraryDownloadResult.displayName)} foi materializado na biblioteca local.</p>
        <p class="muted">${escapeHtml(state.libraryDownloadResult.downloadedPath)} · sha256 ${escapeHtml(state.libraryDownloadResult.sha256)}</p>
      </div>`
    : "";
  const libraryArtifacts = state.libraryInventory?.artifacts ?? [];
  const libraryFilteredArtifacts = libraryArtifacts.filter((artifact) => {
    const search = libraryViewDraft.search.trim().toLowerCase();
    const matchesSearch =
      !search ||
      artifact.displayName.toLowerCase().includes(search) ||
      artifact.artifactId.toLowerCase().includes(search) ||
      artifact.artifactType.toLowerCase().includes(search) ||
      artifact.sourceLabel.toLowerCase().includes(search);
    if (!matchesSearch) {
      return false;
    }

    if (libraryViewDraft.filter === "all") {
      return true;
    }
    if (libraryViewDraft.filter === "download-request") {
      return artifact.artifactType === "download-request";
    }
    if (libraryViewDraft.filter === "ready") {
      return artifact.artifactType !== "download-request";
    }

    return artifact.integrityStatus.toLowerCase() !== "ok";
  });
  const libraryArtifactGroups = [
    {
      id: "download-request",
      title: "Pendentes de download",
      items: libraryFilteredArtifacts.filter((artifact) => artifact.artifactType === "download-request"),
    },
    {
      id: "ready",
      title: "Prontos para uso",
      items: libraryFilteredArtifacts.filter((artifact) => artifact.artifactType !== "download-request"),
    },
  ].filter((group) => group.items.length > 0);
  const libraryArtifactCards = libraryArtifacts.length
    ? libraryFilteredArtifacts.length
      ? libraryArtifactGroups
          .map(
            (group) => `
              <section class="library-group">
                <div class="section-header compact-section-header">
                  <div>
                    <div class="eyebrow">Biblioteca</div>
                    <h3>${group.title}</h3>
                  </div>
                </div>
                <div class="grid three-columns">
                  ${group.items
                    .map(
                      (artifact) => `
                        <article class="card compact">
                          <div class="eyebrow">${escapeHtml(artifact.sourceLabel)}</div>
                          <h3>${escapeHtml(artifact.displayName)}</h3>
                          <div class="pill-row">
                            <span class="info-pill">${escapeHtml(artifact.artifactType)}</span>
                            <span class="info-pill">${escapeHtml(artifact.integrityStatus)}</span>
                          </div>
                          <dl class="target-metrics">
                            <div><dt>Tamanho</dt><dd>${formatStorage(artifact.sizeBytes / 1024 / 1024 / 1024)}</dd></div>
                            <div><dt>ID</dt><dd>${escapeHtml(artifact.artifactId)}</dd></div>
                          </dl>
                          <p class="muted">${escapeHtml(artifact.localPath)}</p>
                          ${artifact.sourceUrl ? `<p class="muted artifact-secondary-line">${escapeHtml(artifact.sourceUrl)}</p>` : ""}
                          <div class="inline-actions library-actions">
                          ${
                            artifact.artifactType === "download-request"
                              ? `<button class="primary-button secondary-button library-download-artifact" type="button" data-artifact-id="${escapeHtml(artifact.artifactId)}" ${state.isDeletingLibraryArtifact || state.isDownloadingLibraryArtifact || state.isRefreshingLibrary ? "disabled" : ""}>${state.isDownloadingLibraryArtifact ? "Baixando..." : "Baixar agora"}</button>`
                              : ""
                          }
                          ${
                            artifact.removable
                              ? `<button class="secondary-button library-delete-artifact" type="button" data-artifact-id="${escapeHtml(artifact.artifactId)}" data-artifact-type="${escapeHtml(artifact.artifactType)}" ${state.isDeletingLibraryArtifact || state.isDownloadingLibraryArtifact ? "disabled" : ""}>Excluir da biblioteca</button>`
                              : ""
                          }
                          </div>
                        </article>
                      `,
                    )
                    .join("")}
                </div>
              </section>
            `,
          )
          .join("")
      : `
        <article class="card compact empty-state">
          <h3>Nenhum artefato neste recorte</h3>
          <p>Ajuste a busca ou o filtro da biblioteca para encontrar outros artefatos.</p>
        </article>
      `
    : `
      <article class="card compact empty-state">
        <h3>Biblioteca local vazia</h3>
        <p>Nenhum artefato foi materializado ainda na biblioteca do Builder.</p>
      </article>
    `;
  const hostRamRecommendationCards = state.buildPlan?.hostRamRecommendations.length
    ? state.buildPlan.hostRamRecommendations
        .map(
          (profile) => `
            <article class="card compact">
              <div class="eyebrow">${escapeHtml(profile.label)}</div>
              <h3>${escapeHtml(profile.suggestedModel.displayName)}</h3>
              <p class="muted">${escapeHtml(profile.summary)}</p>
              <dl class="target-metrics">
                <div><dt>Disco</dt><dd>${formatStorage(profile.suggestedModel.estimatedDiskSizeGb)}</dd></div>
                <div><dt>RAM alvo</dt><dd>${profile.hostRamGb} GB</dd></div>
                <div><dt>Status</dt><dd>${escapeHtml(profile.suggestedModel.availabilityLabel)}</dd></div>
              </dl>
              <p>${escapeHtml(profile.suggestedModel.rationale)}</p>
              ${
                profile.alternatives.length
                  ? `<p class="muted">Alternativas: ${escapeHtml(
                      profile.alternatives.map((model) => model.displayName).join(" · "),
                    )}</p>`
                  : ""
              }
              <button type="button" class="primary-button ghost-button recommended-search-button" data-recommended-search="${escapeHtml(profile.suggestedModel.displayName)}">Ver no catalogo</button>
            </article>
          `,
        )
        .join("")
    : `
      <article class="card compact empty-state">
        <h3>Nenhuma sugestao de LLM ainda</h3>
        <p>Conecte um pendrive USB valido para ver a matriz de modelos por faixa de RAM do host.</p>
      </article>
    `;
  const packCards = state.buildPlan?.packs.length
    ? state.buildPlan.packs
        .map(
          (pack) => `
            <article class="card compact">
              <div class="eyebrow">Pack planejado</div>
              <h3>${escapeHtml(pack.name)}</h3>
              <p class="muted">${escapeHtml(pack.packType)} · ${escapeHtml(pack.category)}</p>
              <code>${escapeHtml(pack.activationMode)}</code>
            </article>
          `,
        )
        .join("")
    : `
      <article class="card compact empty-state">
        <h3>Nenhum pack planejado</h3>
        <p>O backend ainda nao gerou packs derivados deste plano.</p>
      </article>
    `;
  const manifestCards = state.buildPlan?.manifests.length
    ? state.buildPlan.manifests
        .map(
          (manifest) => `
            <button
              type="button"
              class="manifest-card ${manifest.fileName === selectedManifest?.fileName ? "manifest-card-active" : ""}"
              data-manifest-file="${escapeHtml(manifest.fileName)}"
            >
              <span class="eyebrow">Manifesto</span>
              <strong>${escapeHtml(manifest.title)}</strong>
              <span class="muted">${escapeHtml(manifest.fileName)}</span>
            </button>
          `,
        )
        .join("")
    : "";
  const exportedFilesMarkup = state.exportResult?.exportedFiles.length
    ? state.exportResult.exportedFiles
        .map(
          (file) => `
            <article class="card compact">
              <div class="eyebrow">Arquivo exportado</div>
              <h3>${escapeHtml(file.fileName)}</h3>
              <p class="muted">${escapeHtml(file.relativePath)}</p>
              <code>${escapeHtml(formatBytes(file.sizeBytes))}</code>
            </article>
          `,
        )
        .join("")
    : `
      <article class="card compact empty-state">
        <h3>Nenhuma exportacao ainda</h3>
        <p>Use o botao abaixo para gravar este plano e seus manifestos em disco.</p>
      </article>
    `;
  const assembledDirectoriesMarkup = state.assemblyResult?.assembledDirectories.length
    ? state.assemblyResult.assembledDirectories
        .map(
          (directory) => `
            <article class="card compact">
              <div class="eyebrow">Diretorio montado</div>
              <h3>${escapeHtml(directory.relativePath)}</h3>
              <code>${escapeHtml(`${directory.fileCount} arquivos`)}</code>
            </article>
          `,
        )
        .join("")
    : `
      <article class="card compact empty-state">
        <h3>Nenhuma montagem ainda</h3>
        <p>Use o fluxo de montagem para copiar runtime, BGE, modelos e manifestos para o destino selecionado.</p>
      </article>
    `;
  const assemblyFilesMarkup = state.assemblyResult?.exportedFiles.length
    ? state.assemblyResult.exportedFiles
        .map(
          (file) => `
            <article class="card compact">
              <div class="eyebrow">Arquivo montado</div>
              <h3>${escapeHtml(file.fileName)}</h3>
              <p class="muted">${escapeHtml(file.relativePath)}</p>
              <code>${escapeHtml(formatBytes(file.sizeBytes))}</code>
            </article>
          `,
        )
        .join("")
    : `
      <article class="card compact empty-state">
        <h3>Nenhum arquivo montado</h3>
        <p>A montagem ainda nao gravou arquivos no destino selecionado.</p>
      </article>
    `;
  const assemblyWarningsMarkup = state.assemblyResult?.warnings.length
    ? state.assemblyResult.warnings.map((warning) => `<li>${escapeHtml(warning)}</li>`).join("")
    : "<li>Nenhum alerta retornado pela montagem.</li>";
  const validationChecksMarkup = state.validationResult?.checks.length
    ? state.validationResult.checks
        .map(
          (check) => `
            <article class="card compact">
              <div class="eyebrow">Check</div>
              <h3>${escapeHtml(check.id)}</h3>
              <p class="muted">${escapeHtml(check.status)}</p>
              <p>${escapeHtml(check.message)}</p>
            </article>
          `,
        )
        .join("")
    : `
      <article class="card compact empty-state">
        <h3>Nenhuma validacao ainda</h3>
        <p>Rode a validacao para confirmar se o USBRAIN montado ficou consistente.</p>
      </article>
    `;
  const validationErrorsMarkup = state.validationResult?.errors.length
    ? state.validationResult.errors.map((error) => `<li>${escapeHtml(error)}</li>`).join("")
    : "<li>Nenhum erro registrado pela validacao.</li>";
  const validationWarningsMarkup = state.validationResult?.warnings.length
    ? state.validationResult.warnings.map((warning) => `<li>${escapeHtml(warning)}</li>`).join("")
    : "<li>Nenhum alerta adicional na validacao.</li>";
  const validationBadge =
    state.validationResult?.validationStatus === "passed"
      ? '<span class="status good">Validacao OK</span>'
      : state.validationResult?.validationStatus === "passed-with-warnings"
        ? '<span class="status warn">Validacao com alertas</span>'
        : state.validationResult?.validationStatus === "failed"
          ? '<span class="status bad">Validacao falhou</span>'
          : '<span class="status warn">Validacao pendente</span>';
  const smokeChecksMarkup = state.smokeTestResult?.checks.length
    ? state.smokeTestResult.checks
        .map(
          (check) => `
            <article class="card compact">
              <div class="eyebrow">Smoke check</div>
              <h3>${escapeHtml(check.id)}</h3>
              <p class="muted">${escapeHtml(check.status)}</p>
              ${check.command ? `<code>${escapeHtml(check.command)}</code>` : ""}
              <p>${escapeHtml(check.message)}</p>
            </article>
          `,
        )
        .join("")
    : `
      <article class="card compact empty-state">
        <h3>Nenhum smoke test ainda</h3>
        <p>Rode o smoke test para verificar prontidao operacional minima do USBRAIN montado.</p>
      </article>
    `;
  const smokeErrorsMarkup = state.smokeTestResult?.errors.length
    ? state.smokeTestResult.errors.map((error) => `<li>${escapeHtml(error)}</li>`).join("")
    : "<li>Nenhum erro operacional registrado.</li>";
  const smokeWarningsMarkup = state.smokeTestResult?.warnings.length
    ? state.smokeTestResult.warnings.map((warning) => `<li>${escapeHtml(warning)}</li>`).join("")
    : "<li>Nenhum alerta operacional adicional.</li>";
  const smokeBadge =
    state.smokeTestResult?.smokeStatus === "passed"
      ? '<span class="status good">Smoke test OK</span>'
      : state.smokeTestResult?.smokeStatus === "passed-with-warnings"
        ? '<span class="status warn">Smoke test com alertas</span>'
        : state.smokeTestResult?.smokeStatus === "failed"
          ? '<span class="status bad">Smoke test falhou</span>'
          : '<span class="status warn">Smoke test pendente</span>';
  const runtimeProbeChecksMarkup = state.runtimeProbeResult?.checks.length
    ? state.runtimeProbeResult.checks
        .map(
          (check) => `
            <article class="card compact">
              <div class="eyebrow">Runtime probe</div>
              <h3>${escapeHtml(check.id)}</h3>
              <p class="muted">${escapeHtml(check.status)}</p>
              ${check.command ? `<code>${escapeHtml(check.command)}</code>` : ""}
              <p>${escapeHtml(check.message)}</p>
            </article>
          `,
        )
        .join("")
    : `
      <article class="card compact empty-state">
        <h3>Nenhum runtime probe ainda</h3>
        <p>Execute o probe para subir o USBRAIN montado em modo de teste e validar o endpoint HTTP.</p>
      </article>
    `;
  const runtimeProbeErrorsMarkup = state.runtimeProbeResult?.errors.length
    ? state.runtimeProbeResult.errors.map((error) => `<li>${escapeHtml(error)}</li>`).join("")
    : "<li>Nenhum erro de runtime probe registrado.</li>";
  const runtimeProbeWarningsMarkup = state.runtimeProbeResult?.warnings.length
    ? state.runtimeProbeResult.warnings.map((warning) => `<li>${escapeHtml(warning)}</li>`).join("")
    : "<li>Nenhum alerta adicional no runtime probe.</li>";
  const runtimeProbeBadge =
    state.runtimeProbeResult?.probeStatus === "passed"
      ? '<span class="status good">Runtime probe OK</span>'
      : state.runtimeProbeResult?.probeStatus === "passed-with-warnings"
        ? '<span class="status warn">Runtime probe com alertas</span>'
        : state.runtimeProbeResult?.probeStatus === "failed"
          ? '<span class="status bad">Runtime probe falhou</span>'
          : '<span class="status warn">Runtime probe pendente</span>';
  const endToEndProbeChecksMarkup = state.endToEndProbeResult?.checks.length
    ? state.endToEndProbeResult.checks
        .map(
          (check) => `
            <article class="card compact">
              <div class="eyebrow">E2E probe</div>
              <h3>${escapeHtml(check.id)}</h3>
              <p class="muted">${escapeHtml(check.status)}</p>
              ${check.command ? `<code>${escapeHtml(check.command)}</code>` : ""}
              <p>${escapeHtml(check.message)}</p>
            </article>
          `,
        )
        .join("")
    : `
      <article class="card compact empty-state">
        <h3>Nenhum probe ponta a ponta ainda</h3>
        <p>Execute o probe E2E para validar o contrato HTTP de chat no USBRAIN montado.</p>
      </article>
    `;
  const endToEndProbeErrorsMarkup = state.endToEndProbeResult?.errors.length
    ? state.endToEndProbeResult.errors.map((error) => `<li>${escapeHtml(error)}</li>`).join("")
    : "<li>Nenhum erro de probe ponta a ponta registrado.</li>";
  const endToEndProbeWarningsMarkup = state.endToEndProbeResult?.warnings.length
    ? state.endToEndProbeResult.warnings.map((warning) => `<li>${escapeHtml(warning)}</li>`).join("")
    : "<li>Nenhum alerta adicional no probe ponta a ponta.</li>";
  const endToEndProbeBadge =
    state.endToEndProbeResult?.probeStatus === "passed"
      ? '<span class="status good">Probe E2E OK</span>'
      : state.endToEndProbeResult?.probeStatus === "passed-with-warnings"
        ? '<span class="status warn">Probe E2E com alertas</span>'
        : state.endToEndProbeResult?.probeStatus === "failed"
          ? '<span class="status bad">Probe E2E falhou</span>'
          : '<span class="status warn">Probe E2E pendente</span>';
  const fullLocalLlmProbeChecksMarkup = state.fullLocalLlmProbeResult?.checks.length
    ? state.fullLocalLlmProbeResult.checks
        .map(
          (check) => `
            <article class="card compact">
              <div class="eyebrow">LLM local</div>
              <h3>${escapeHtml(check.id)}</h3>
              <p class="muted">${escapeHtml(check.status)}</p>
              ${check.command ? `<code>${escapeHtml(check.command)}</code>` : ""}
              <p>${escapeHtml(check.message)}</p>
            </article>
          `,
        )
        .join("")
    : `
      <article class="card compact empty-state">
        <h3>Nenhum probe com LLM local ainda</h3>
        <p>Execute o probe completo para validar o USBRAIN montado com modelo local ativo.</p>
      </article>
    `;
  const fullLocalLlmProbeErrorsMarkup = state.fullLocalLlmProbeResult?.errors.length
    ? state.fullLocalLlmProbeResult.errors.map((error) => `<li>${escapeHtml(error)}</li>`).join("")
    : "<li>Nenhum erro de probe completo com LLM registrado.</li>";
  const fullLocalLlmProbeWarningsMarkup = state.fullLocalLlmProbeResult?.warnings.length
    ? state.fullLocalLlmProbeResult.warnings.map((warning) => `<li>${escapeHtml(warning)}</li>`).join("")
    : "<li>Nenhum alerta adicional no probe completo com LLM.</li>";
  const fullLocalLlmProbeBadge =
    state.fullLocalLlmProbeResult?.probeStatus === "passed"
      ? '<span class="status good">LLM local OK</span>'
      : state.fullLocalLlmProbeResult?.probeStatus === "passed-with-warnings"
        ? '<span class="status warn">LLM local com alertas</span>'
        : state.fullLocalLlmProbeResult?.probeStatus === "failed"
          ? '<span class="status bad">LLM local falhou</span>'
          : '<span class="status warn">LLM local pendente</span>';
  const readinessSectionsMarkup = state.readinessReport?.sections.length
    ? state.readinessReport.sections
        .map(
          (section) => `
            <article class="card compact">
              <div class="eyebrow">${section.required ? "Gate obrigatorio" : "Gate consultivo"}</div>
              <h3>${escapeHtml(section.label)}</h3>
              <p class="muted">${escapeHtml(section.status)}</p>
              <p>${escapeHtml(section.summary)}</p>
            </article>
          `,
        )
        .join("")
    : `
      <article class="card compact empty-state">
        <h3>Nenhum relatorio de prontidao ainda</h3>
        <p>Gere o relatorio consolidado para decidir se o build pode ser distribuido.</p>
      </article>
    `;
  const readinessBlockingMarkup = state.readinessReport?.blockingIssues.length
    ? state.readinessReport.blockingIssues.map((issue) => `<li>${escapeHtml(issue)}</li>`).join("")
    : "<li>Nenhum bloqueio registrado.</li>";
  const readinessAdvisoryMarkup = state.readinessReport?.advisoryIssues.length
    ? state.readinessReport.advisoryIssues.map((issue) => `<li>${escapeHtml(issue)}</li>`).join("")
    : "<li>Nenhum alerta consultivo registrado.</li>";
  const readinessBadge =
    state.readinessReport?.readinessStatus === "ready"
      ? '<span class="status good">Pronto para distribuir</span>'
      : state.readinessReport?.readinessStatus === "ready-with-warnings"
        ? '<span class="status warn">Pronto com alertas</span>'
        : state.readinessReport?.readinessStatus === "blocked"
          ? '<span class="status bad">Distribuicao bloqueada</span>'
          : '<span class="status warn">Relatorio pendente</span>';
  const distributionFilesMarkup = state.distributionResult?.exportedFiles.length
    ? state.distributionResult.exportedFiles
        .map(
          (file) => `
            <article class="card compact">
              <div class="eyebrow">Distribuicao</div>
              <h3>${escapeHtml(file.fileName)}</h3>
              <p class="muted">${escapeHtml(file.relativePath)}</p>
              <code>${escapeHtml(formatBytes(file.sizeBytes))}</code>
            </article>
          `,
        )
        .join("")
    : `
      <article class="card compact empty-state">
        <h3>Nenhum bundle de distribuicao ainda</h3>
        <p>Prepare a distribuicao final para persistir o gate de prontidao junto do build exportado.</p>
      </article>
    `;
  const distributionBadge =
    state.distributionResult?.distributionReady
      ? '<span class="status good">Distribuicao liberada</span>'
      : state.distributionResult?.distributionStatus === "blocked"
        ? '<span class="status bad">Distribuicao bloqueada</span>'
        : '<span class="status warn">Distribuicao pendente</span>';
  const releaseFilesMarkup = state.releasePackageResult?.exportedFiles.length
    ? state.releasePackageResult.exportedFiles
        .map(
          (file) => `
            <article class="card compact">
              <div class="eyebrow">Release</div>
              <h3>${escapeHtml(file.fileName)}</h3>
              <p class="muted">${escapeHtml(file.relativePath)}</p>
              <code>${escapeHtml(formatBytes(file.sizeBytes))}</code>
            </article>
          `,
        )
        .join("")
    : `
      <article class="card compact empty-state">
        <h3>Nenhum pacote de release ainda</h3>
        <p>Prepare o release final para gerar checksums e metadados publicaveis para GitHub e usbrain.org.</p>
      </article>
    `;
  const releaseChecksumsMarkup = state.releasePackageResult?.artifactChecksums.length
    ? state.releasePackageResult.artifactChecksums
        .map(
          (artifact) => `
            <article class="card compact">
              <div class="eyebrow">Checksum</div>
              <h3>${escapeHtml(artifact.fileName)}</h3>
              <p class="muted">${escapeHtml(artifact.relativePath)}</p>
              <code>${escapeHtml(artifact.sha256)}</code>
            </article>
          `,
        )
        .join("")
    : `
      <article class="card compact empty-state">
        <h3>Nenhum checksum ainda</h3>
        <p>Os hashes SHA256 aparecerao aqui apos a preparacao do pacote de release.</p>
      </article>
    `;
  const releaseBadge =
    state.releasePackageResult?.releaseReady
      ? '<span class="status good">Release pronto</span>'
      : state.releasePackageResult?.releaseStatus === "blocked"
        ? '<span class="status bad">Release bloqueado</span>'
        : '<span class="status warn">Release pendente</span>';
  const pipelineStatus = buildPipelineStatus(state);
  const mainTabsMarkup = [
    renderMainTabButton("create", "Criar", "Pendrive e configuracao"),
    renderMainTabButton("models", "LLMs", "Catalogo e selecao"),
    renderMainTabButton("library", "Biblioteca", "Arquivos locais"),
    renderMainTabButton("validation", "Validar", "Passo a passo de QA"),
    renderMainTabButton("advanced", "Avancado", "Detalhes tecnicos"),
  ].join("");
  const catalogSummaryLabel = catalogResultSummary(catalogModels.length, state.modelCatalog?.models.length ?? 0);
  const modelSelectionModesMarkup = [
    renderModelSelectionModeButton(
      "recommended",
      "Recomendadas",
      "Comece pelas sugestoes por faixa de RAM.",
      `${state.buildPlan?.hostRamRecommendations.length ?? 0} perfis`,
    ),
    renderModelSelectionModeButton(
      "manual",
      "Manual",
      "Explore o catalogo completo com busca e filtros.",
      `${catalogModels.length} visiveis`,
    ),
    renderModelSelectionModeButton(
      "library",
      "Na biblioteca",
      "Veja primeiro o que ja esta pronto para uso offline.",
      `${localCatalogModels.length} locais`,
    ),
  ].join("");
  const modelBulkActionsMarkup = `
    <div class="inline-actions bulk-actions">
      ${
        activeModelSelectionMode === "recommended"
          ? renderModelBulkActionButton("recommended", "Usar recomendadas", !recommendedModelIds.length)
          : activeModelSelectionMode === "library"
            ? renderModelBulkActionButton("visible-local", "Selecionar locais visiveis", !visibleLocalModelIds.length)
            : renderModelBulkActionButton("visible-local", "Selecionar locais visiveis", !visibleLocalModelIds.length)
      }
      ${renderModelBulkActionButton("clear", "Limpar selecao", !plannerDraft.selectedModelIds.length, "ghost")}
    </div>
  `;
  const libraryIntegrityIssueCount =
    state.libraryInventory?.artifacts.filter((artifact) => artifact.integrityStatus.toLowerCase() !== "ok").length ?? 0;
  const libraryFilterButtonsMarkup = [
    renderLibraryFilterButton("all", "Tudo", libraryArtifacts.length),
    renderLibraryFilterButton(
      "download-request",
      "Pendentes",
      libraryArtifacts.filter((artifact) => artifact.artifactType === "download-request").length,
    ),
    renderLibraryFilterButton(
      "ready",
      "Prontos",
      libraryArtifacts.filter((artifact) => artifact.artifactType !== "download-request").length,
    ),
    renderLibraryFilterButton("issues", "Com alerta", libraryIntegrityIssueCount),
  ].join("");
  const createWizardMarkup = `
    <section class="wizard-strip">
      <button type="button" class="wizard-step-card ${createStepClass("target")}" data-create-step="target">
        <span class="wizard-step-index">1</span>
        <div>
          <strong>Escolha o pendrive</strong>
          <p>${selectedTarget ? escapeHtml(selectedTarget.label) : "Selecione o destino fisico do USBRAIN."}</p>
        </div>
      </button>
      <button type="button" class="wizard-step-card ${createStepClass("profile")}" data-create-step="profile">
        <span class="wizard-step-index">2</span>
        <div>
          <strong>Defina o perfil</strong>
          <p>${plannerDraft.selectedModelIds.length ? `${plannerDraft.selectedModelIds.length} LLMs escolhidas` : "Escolha as LLMs e o comportamento do build."}</p>
        </div>
      </button>
      <button type="button" class="wizard-step-card ${createStepClass("review")}" data-create-step="review">
        <span class="wizard-step-index">3</span>
        <div>
          <strong>Revise e grave</strong>
          <p>${selectedTarget && !hasPendingModelDownloads ? "Tudo pronto para criar no pendrive." : "Confirme downloads e revise o resumo final."}</p>
        </div>
      </button>
    </section>
  `;

  app.innerHTML = `
    <main class="layout">
      <section class="hero card hero-card">
        <div>
          <div class="eyebrow">USBRAIN Builder</div>
          <h1>Crie um USBRAIN com um fluxo mais claro</h1>
          <p>O essencial fica nas abas principais. O detalhe tecnico continua disponivel, mas fora do caminho principal.</p>
        </div>
        <div class="hero-side">
          <span class="${modeClass}">${modeLabel}</span>
          <span class="${pipelineStatus.className}">${pipelineStatus.label}</span>
          <div class="metric-grid">
            <article class="metric">
              <span>Pendrive</span>
              <strong>${escapeHtml(selectedTarget?.label ?? "Nenhum")}</strong>
            </article>
            <article class="metric">
              <span>LLMs</span>
              <strong>${plannerDraft.selectedModelIds.length}</strong>
            </article>
            <article class="metric">
              <span>Versao</span>
              <strong>${state.overview.appVersion}</strong>
            </article>
          </div>
        </div>
      </section>

      <section class="summary-strip">
        <article class="summary-card">
          <span class="eyebrow">Pendrive</span>
          <strong>${escapeHtml(selectedTarget?.label ?? "Nenhum selecionado")}</strong>
          <span>${formatStorage(selectedTarget?.availableGb)} livres</span>
        </article>
        <article class="summary-card">
          <span class="eyebrow">Build</span>
          <strong>${buildModeLabel(plannerDraft.buildMode)}</strong>
          <span>${formatStorage(state.buildPlan?.summary.installBudgetGb)} para modelos</span>
        </article>
        <article class="summary-card">
          <span class="eyebrow">LLMs</span>
          <strong>${plannerDraft.selectedModelIds.length} selecionadas</strong>
          <span>${catalogSummaryLabel}</span>
        </article>
        <article class="summary-card">
          <span class="eyebrow">Biblioteca</span>
          <strong>${state.libraryInventory?.artifactCount ?? 0} artefatos</strong>
          <span>${hasPendingModelDownloads ? `${state.buildPlan?.summary.pendingModelDownloadCount ?? 0} pendentes` : "Sem pendencias"}</span>
        </article>
        <article class="summary-card">
          <span class="eyebrow">QA</span>
          <strong>${pipelineStatus.label}</strong>
          <span>${state.buildPlan?.summary.buildId ?? "Build ainda nao criado"}</span>
        </article>
      </section>

      <section class="tab-nav" aria-label="Navegacao principal">
        ${mainTabsMarkup}
      </section>

      ${
        activeMainView === "create"
          ? `
            ${createWizardMarkup}

            ${
              activeCreateStep === "target"
                ? `
                  <section class="grid two-columns">
                    <article class="card">
                      <div class="eyebrow">Etapa 1</div>
                      <h2>Escolha o pendrive</h2>
                      <p class="muted section-intro">Clique em um dos pendrives abaixo ou use a lista para definir o destino do USBRAIN.</p>
                      ${targetRefreshStatusMarkup}
                      <label class="field">
                        <span>Pendrive USB</span>
                        <select id="target-select" ${state.targets.length ? "" : "disabled"}>
                          ${targetOptions || '<option value="">Nenhum pendrive USB detectado</option>'}
                        </select>
                      </label>
                      <div class="inline-actions create-step-actions">
                        <button type="button" id="refresh-targets" class="primary-button secondary-button" ${state.isRefreshingTargets ? "disabled" : ""}>
                          ${state.isRefreshingTargets ? "Atualizando..." : "Atualizar pendrives"}
                        </button>
                        <button type="button" class="primary-button" data-create-step="profile" ${selectedTarget ? "" : "disabled"}>Continuar para o perfil</button>
                      </div>
                    </article>

                    <article class="card">
                      <div class="eyebrow">Destino</div>
                      <h2>Pendrive selecionado</h2>
                      <p class="muted section-intro">Aqui fica o retrato rapido do alvo atual, sem ruido de debug.</p>
                      <dl class="detail-list">
                        <div><dt>Pendrive ativo</dt><dd>${escapeHtml(selectedTarget?.label ?? "Nenhum pendrive selecionado")}</dd></div>
                        <div><dt>Espaco livre</dt><dd>${formatStorage(selectedTarget?.availableGb)}</dd></div>
                        <div><dt>Capacidade total</dt><dd>${formatStorage(selectedTarget?.capacityGb)}</dd></div>
                        <div><dt>Perfis estimados</dt><dd>${state.buildPlan?.summary.estimatedModelSlots ?? selectedTarget?.estimatedModelSlots ?? 0}</dd></div>
                        <div><dt>Uso atual</dt><dd>${typeof selectedTarget?.usagePercent === "number" ? `${selectedTarget.usagePercent.toFixed(0)}% ocupado` : "indisponivel"}</dd></div>
                      </dl>
                      <ul class="doc-list">${warningList}</ul>
                    </article>
                  </section>

                  <section>
                    <div class="section-header">
                      <div>
                        <div class="eyebrow">Pendrives</div>
                        <h2>Pendrives USB detectados</h2>
                      </div>
                    </div>
                    <div class="grid three-columns">${targets}</div>
                  </section>
                `
                : activeCreateStep === "profile"
                  ? `
                    <section class="grid two-columns">
                      <article class="card">
                        <div class="eyebrow">Etapa 2</div>
                        <h2>Defina o perfil</h2>
                        <p class="muted section-intro">Ajuste apenas o que muda o comportamento do pendrive. A escolha de LLMs continua na aba propria.</p>
                        <div class="planner-form">
                          <label class="field">
                            <span>Pendrive USB</span>
                            <select id="target-select" ${state.targets.length ? "" : "disabled"}>
                              ${targetOptions || '<option value="">Nenhum pendrive USB detectado</option>'}
                            </select>
                          </label>
                          <label class="field">
                            <span>Modo do build</span>
                            <select id="build-mode-select">
                              <option value="offline-first" ${plannerDraft.buildMode === "offline-first" ? "selected" : ""}>${buildModeLabel("offline-first")}</option>
                              <option value="offline-only" ${plannerDraft.buildMode === "offline-only" ? "selected" : ""}>${buildModeLabel("offline-only")}</option>
                              <option value="local-assisted" ${plannerDraft.buildMode === "local-assisted" ? "selected" : ""}>${buildModeLabel("local-assisted")}</option>
                              <option value="hybrid" ${plannerDraft.buildMode === "hybrid" ? "selected" : ""}>${buildModeLabel("hybrid")}</option>
                            </select>
                          </label>
                          <label class="field">
                            <span>Criador</span>
                            <input id="creator-label" type="text" placeholder="Nome opcional do criador" value="${escapeHtml(plannerDraft.creatorLabel)}" />
                          </label>
                          <div class="toggle-row">
                            <label class="toggle">
                              <input id="allow-gpu" type="checkbox" ${plannerDraft.allowGpu ? "checked" : ""} />
                              <span>Permitir GPU opcional</span>
                            </label>
                            <label class="toggle">
                              <input id="allow-cloud" type="checkbox" ${plannerDraft.allowCloudFallback ? "checked" : ""} />
                              <span>Permitir fallback em nuvem</span>
                            </label>
                          </div>
                          <div class="language-grid">
                            <label class="toggle">
                              <input name="language" type="checkbox" value="pt-BR" ${plannerDraft.languages.includes("pt-BR") ? "checked" : ""} />
                              <span>${languageLabel("pt-BR")}</span>
                            </label>
                            <label class="toggle">
                              <input name="language" type="checkbox" value="en" ${plannerDraft.languages.includes("en") ? "checked" : ""} />
                              <span>${languageLabel("en")}</span>
                            </label>
                            <label class="toggle">
                              <input name="language" type="checkbox" value="es" ${plannerDraft.languages.includes("es") ? "checked" : ""} />
                              <span>${languageLabel("es")}</span>
                            </label>
                          </div>
                        </div>
                        <div class="inline-actions create-step-actions">
                          <button type="button" class="primary-button ghost-button" data-create-step="target">Voltar</button>
                          <button type="button" class="primary-button secondary-button" data-main-tab="models">Abrir aba LLMs</button>
                          <button type="button" class="primary-button" data-create-step="review" ${selectedTarget ? "" : "disabled"}>Continuar para revisao</button>
                        </div>
                      </article>

                      <article class="card">
                        <div class="eyebrow">Perfil atual</div>
                        <h2>Resumo rapido</h2>
                        <div class="focus-grid create-focus-grid">
                          <article class="focus-item">
                            <span>Pendrive</span>
                            <strong>${escapeHtml(selectedTarget?.label ?? "Nenhum selecionado")}</strong>
                          </article>
                          <article class="focus-item">
                            <span>Modo</span>
                            <strong>${buildModeLabel(plannerDraft.buildMode)}</strong>
                          </article>
                          <article class="focus-item">
                            <span>Idiomas</span>
                            <strong>${escapeHtml(plannerDraft.languages.map(languageLabel).join(" · "))}</strong>
                          </article>
                          <article class="focus-item">
                            <span>LLMs</span>
                            <strong>${plannerDraft.selectedModelIds.length || 0}</strong>
                          </article>
                        </div>
                        <div class="soft-panel">
                          <strong>${buildModeLabel(plannerDraft.buildMode)}</strong>
                          <p class="muted">${buildModeSummary(plannerDraft.buildMode)}</p>
                        </div>
                        ${selectedModelsMarkup}
                      </article>
                    </section>
                  `
                  : `
                    <section class="grid two-columns">
                      <article class="card">
                        <div class="eyebrow">Etapa 3</div>
                        <h2>Revise e crie</h2>
                        <p class="muted section-intro">
                          ${selectedTarget ? `Tudo aponta para ${escapeHtml(selectedTarget.label)}. Revise o resumo rapido e inicie a gravacao quando estiver satisfeito.` : "Conecte e selecione um pendrive USB para liberar a criacao."}
                        </p>
                        <div class="focus-grid create-focus-grid">
                          <article class="focus-item">
                            <span>Pendrive</span>
                            <strong>${escapeHtml(selectedTarget?.label ?? "Nenhum selecionado")}</strong>
                          </article>
                          <article class="focus-item">
                            <span>Modo</span>
                            <strong>${buildModeLabel(plannerDraft.buildMode)}</strong>
                          </article>
                          <article class="focus-item">
                            <span>Orcamento</span>
                            <strong>${formatStorage(state.buildPlan?.summary.installBudgetGb)}</strong>
                          </article>
                          <article class="focus-item">
                            <span>LLMs</span>
                            <strong>${plannerDraft.selectedModelIds.length || 0}</strong>
                          </article>
                        </div>
                        <div class="soft-panel">
                          <strong>${buildModeLabel(plannerDraft.buildMode)}</strong>
                          <p class="muted">${buildModeSummary(plannerDraft.buildMode)}</p>
                        </div>
                        ${selectedModelsMarkup}
                        <div class="export-toolbar create-actions">
                          <button id="create-usbrain" class="primary-button" type="button" ${createButtonDisabled ? "disabled" : ""}>
                            ${state.isAssembling ? "Criando USBRAIN..." : "Criar agora"}
                          </button>
                          <button id="export-build-plan" class="primary-button secondary-button" type="button" ${state.isExporting || state.isAssembling || state.isValidating || state.isRunningSmokeTest || state.isRunningRuntimeProbe || state.isRunningEndToEndProbe || state.isRunningFullLocalLlmProbe || state.isGeneratingReadinessReport || state.isPreparingDistribution || state.isPreparingReleasePackage ? "disabled" : ""}>
                            ${state.isExporting ? "Exportando..." : "Exportar"}
                          </button>
                          <button type="button" class="primary-button ghost-button" data-create-step="profile">Voltar</button>
                        </div>
                        ${wipeConfirmationMarkup}
                        ${authorizationMarkup}
                        ${assemblyProgressMarkup}
                        ${state.targetAuthorizationError ? `<p class="error-box">${escapeHtml(state.targetAuthorizationError)}</p>` : ""}
                        ${state.assemblyError ? `<p class="error-box">${escapeHtml(state.assemblyError)}</p>` : ""}
                        ${hasPendingModelDownloads ? `<div class="warning-box"><strong>Downloads pendentes</strong><p>As LLMs selecionadas ainda nao estao na biblioteca local. Resolva isso antes de gravar o pendrive offline.</p></div>` : ""}
                        ${state.assemblyResult ? `<p class="status good">USBRAIN montado em ${escapeHtml(state.assemblyResult.targetRoot)}</p>` : ""}
                        ${state.error ? `<p class="error-box">${escapeHtml(state.error)}</p>` : ""}
                      </article>

                      <article class="card">
                        <div class="eyebrow">Resumo final</div>
                        <h2>O que vai para o pendrive</h2>
                        <p class="muted section-intro">Uma leitura curta do que este USBRAIN vai levar para o pendrive.</p>
                        <dl class="detail-list">
                          <div><dt>Build ID</dt><dd>${escapeHtml(state.buildPlan?.summary.buildId ?? "indisponivel")}</dd></div>
                          <div><dt>Idiomas</dt><dd>${escapeHtml(plannerDraft.languages.map(languageLabel).join(" · "))}</dd></div>
                          <div><dt>GPU opcional</dt><dd>${plannerDraft.allowGpu ? "Permitida" : "Desligada"}</dd></div>
                          <div><dt>Cloud fallback</dt><dd>${plannerDraft.allowCloudFallback ? "Permitido" : "Desligado"}</dd></div>
                          <div><dt>Packs</dt><dd>${state.buildPlan?.summary.selectedPackCount ?? 0}</dd></div>
                        </dl>
                        <div class="pill-row">
                          <span class="info-pill">${escapeHtml(selectedTarget?.recommendedModelTier ?? "Perfil indisponivel")}</span>
                          <span class="info-pill">${formatStorage(state.buildPlan?.summary.installBudgetGb)}</span>
                          <span class="info-pill">${plannerDraft.selectedModelIds.length} LLMs</span>
                        </div>
                      </article>
                    </section>
                  `
            }
          `
          : ""
      }

      ${
        activeMainView === "models"
          ? `
            <section class="card">
              <div class="eyebrow">Catalogo vivo</div>
              <h2>Escolha as LLMs do pendrive</h2>
              <p class="muted">Comece por recomendadas, abra manual quando quiser explorar mais e use a biblioteca para focar no que ja esta pronto.</p>
              <section class="mode-switcher">
                ${modelSelectionModesMarkup}
              </section>
              <div class="pill-row">
                <span class="info-pill">${catalogSummaryLabel}</span>
                <span class="info-pill">${plannerDraft.selectedModelIds.length} modelos marcados</span>
                <span class="info-pill">${directDownloadReadyCount} com download direto</span>
                <span class="info-pill">${formatStorage(state.modelCatalog?.installBudgetGb)}</span>
              </div>
              ${modelBulkActionsMarkup}
              ${selectedModelsMarkup}
              ${
                activeModelSelectionMode === "manual" || activeModelSelectionMode === "library"
                  ? `
                    <div class="catalog-topbar">
                      <label class="field catalog-search-field">
                        <span>Buscar modelo ou dominio</span>
                        <input id="catalog-search" type="text" placeholder="Ex.: law, medicine, qwen, engenharia" value="${escapeHtml(catalogViewDraft.search)}" />
                      </label>
                      <button id="refresh-live-catalog" class="primary-button secondary-button" type="button" ${state.isRefreshingCatalog ? "disabled" : ""}>
                        ${state.isRefreshingCatalog ? "Atualizando catalogo..." : "Atualizar catalogo"}
                      </button>
                    </div>
                    <details class="card-disclosure-block filters-disclosure">
                      <summary>Refinar catalogo</summary>
                      <div class="catalog-toolbar">
                        <label class="field">
                          <span>Dominio</span>
                          <select id="catalog-domain">
                            <option value="all" ${catalogViewDraft.domain === "all" ? "selected" : ""}>Todos</option>
                            ${(state.modelCatalog?.domainOptions ?? [])
                              .map(
                                (domain) =>
                                  `<option value="${escapeHtml(domain)}" ${catalogViewDraft.domain === domain ? "selected" : ""}>${escapeHtml(domain)}</option>`,
                              )
                              .join("")}
                          </select>
                        </label>
                        <label class="field">
                          <span>Idioma</span>
                          <select id="catalog-language">
                            <option value="all" ${catalogViewDraft.language === "all" ? "selected" : ""}>Todos</option>
                            <option value="pt-BR" ${catalogViewDraft.language === "pt-BR" ? "selected" : ""}>pt-BR</option>
                            <option value="en" ${catalogViewDraft.language === "en" ? "selected" : ""}>en</option>
                            <option value="es" ${catalogViewDraft.language === "es" ? "selected" : ""}>es</option>
                          </select>
                        </label>
                        <label class="field">
                          <span>RAM do host</span>
                          <select id="catalog-host-ram">
                            <option value="all" ${catalogViewDraft.hostRam === "all" ? "selected" : ""}>Todas</option>
                            <option value="8" ${catalogViewDraft.hostRam === "8" ? "selected" : ""}>8 GB</option>
                            <option value="16" ${catalogViewDraft.hostRam === "16" ? "selected" : ""}>16 GB</option>
                            <option value="32" ${catalogViewDraft.hostRam === "32" ? "selected" : ""}>32 GB</option>
                            <option value="64" ${catalogViewDraft.hostRam === "64" ? "selected" : ""}>64 GB</option>
                          </select>
                        </label>
                        <label class="field">
                          <span>Benchmark</span>
                          <select id="catalog-benchmark-confidence">
                            <option value="all" ${catalogViewDraft.benchmarkConfidence === "all" ? "selected" : ""}>Todos</option>
                            <option value="high" ${catalogViewDraft.benchmarkConfidence === "high" ? "selected" : ""}>Benchmark proprio</option>
                            <option value="medium" ${catalogViewDraft.benchmarkConfidence === "medium" ? "selected" : ""}>Benchmark comparavel</option>
                            <option value="guarded" ${catalogViewDraft.benchmarkConfidence === "guarded" ? "selected" : ""}>Benchmark herdado</option>
                            <option value="none" ${catalogViewDraft.benchmarkConfidence === "none" ? "selected" : ""}>Sem benchmark</option>
                          </select>
                        </label>
                      </div>
                      <label class="toggle catalog-toggle">
                        <input id="catalog-cpu-only" type="checkbox" ${catalogViewDraft.cpuOnlyOnly ? "checked" : ""} />
                        <span>Mostrar apenas modelos compativeis com CPU-only</span>
                      </label>
                    </details>
                  `
                  : `
                    <div class="catalog-topbar">
                      <div class="soft-panel compact-soft-panel">
                        <strong>Selecao assistida</strong>
                        <p class="muted">Use as sugestoes por RAM para partir de um ponto seguro. Quando quiser, abra o catalogo manual para explorar variantes.</p>
                      </div>
                      <button id="refresh-live-catalog" class="primary-button secondary-button" type="button" ${state.isRefreshingCatalog ? "disabled" : ""}>
                        ${state.isRefreshingCatalog ? "Atualizando catalogo..." : "Atualizar catalogo"}
                      </button>
                    </div>
                  `
              }
              ${state.catalogError ? `<p class="error-box">${escapeHtml(state.catalogError)}</p>` : ""}
              ${state.catalogRefreshError ? `<p class="error-box">${escapeHtml(state.catalogRefreshError)}</p>` : ""}
              ${catalogRefreshStatusMarkup}
              ${
                activeModelSelectionMode === "recommended"
                  ? `<div class="catalog-grid">${hostRamRecommendationCards}</div>`
                  : activeModelSelectionMode === "library"
                    ? `<div class="catalog-grid">${localCatalogCards}</div>`
                    : `<div class="catalog-grid">${catalogCards}</div>`
              }
            </section>

            ${
              activeModelSelectionMode === "recommended"
                ? `
                  <section class="grid two-columns">
                    <article class="card">
                      <div class="eyebrow">Plano derivado</div>
                      <h2>Modelos ja resolvidos para este pendrive</h2>
                      <div class="grid two-columns nested-grid">${modelCards}</div>
                    </article>

                    <article class="card">
                      <div class="eyebrow">Aquisicao</div>
                      <h2>Downloads pendentes</h2>
                      <div class="export-toolbar">
                        <button id="prepare-builder-library" class="primary-button secondary-button" type="button" ${state.isPreparingLibrary || state.isAssembling || state.isExporting || state.isValidating || state.isRunningSmokeTest || state.isRunningRuntimeProbe || state.isRunningEndToEndProbe || state.isRunningFullLocalLlmProbe || state.isGeneratingReadinessReport || state.isPreparingDistribution || state.isPreparingReleasePackage || !state.buildPlan ? "disabled" : ""}>
                          ${state.isPreparingLibrary ? "Preparando biblioteca..." : "Preparar biblioteca"}
                        </button>
                      </div>
                      ${state.libraryPreparationError ? `<p class="error-box">${escapeHtml(state.libraryPreparationError)}</p>` : ""}
                      ${libraryPreparationStatusMarkup}
                      <div class="grid two-columns nested-grid">${pendingModelDownloadCards}</div>
                    </article>
                  </section>
                `
                : `
                  <section class="grid two-columns">
                    <article class="card">
                      <div class="eyebrow">Plano derivado</div>
                      <h2>Modelos ja resolvidos para este pendrive</h2>
                      <div class="grid two-columns nested-grid">${modelCards}</div>
                    </article>

                    <article class="card">
                      <div class="eyebrow">Aquisicao</div>
                      <h2>Downloads pendentes</h2>
                      <div class="export-toolbar">
                        <button id="prepare-builder-library" class="primary-button secondary-button" type="button" ${state.isPreparingLibrary || state.isAssembling || state.isExporting || state.isValidating || state.isRunningSmokeTest || state.isRunningRuntimeProbe || state.isRunningEndToEndProbe || state.isRunningFullLocalLlmProbe || state.isGeneratingReadinessReport || state.isPreparingDistribution || state.isPreparingReleasePackage || !state.buildPlan ? "disabled" : ""}>
                          ${state.isPreparingLibrary ? "Preparando biblioteca..." : "Preparar biblioteca"}
                        </button>
                      </div>
                      ${state.libraryPreparationError ? `<p class="error-box">${escapeHtml(state.libraryPreparationError)}</p>` : ""}
                      ${libraryPreparationStatusMarkup}
                      <div class="grid two-columns nested-grid">${pendingModelDownloadCards}</div>
                    </article>
                  </section>

                  <section class="grid two-columns">
                    <article class="card">
                      <div class="eyebrow">Sugestoes</div>
                      <h2>Modelos recomendados por RAM</h2>
                      <div class="grid two-columns nested-grid">${hostRamRecommendationCards}</div>
                    </article>

                    <article class="card">
                      <div class="eyebrow">Packs</div>
                      <h2>Packs planejados para o build</h2>
                      <ul class="doc-list">${catalogSelectionMarkup}</ul>
                      <div class="grid two-columns nested-grid">${packCards}</div>
                    </article>
                  </section>
                `
            }
          `
          : ""
      }

      ${
        activeMainView === "library"
          ? `
            <section class="card">
              <div class="eyebrow">Biblioteca local</div>
              <h2>Artefatos prontos para montagem offline</h2>
              <p class="muted">Aqui fica o que o Builder ja materializou localmente, com espaco, integridade e acoes de manutencao no mesmo lugar.</p>
              <section class="mode-switcher">
                ${libraryFilterButtonsMarkup}
              </section>
              <div class="export-toolbar">
                <label class="field catalog-search-field">
                  <span>Buscar artefato</span>
                  <input id="library-search" type="text" placeholder="Nome, ID, origem ou tipo" value="${escapeHtml(libraryViewDraft.search)}" />
                </label>
                <button id="refresh-builder-library" class="primary-button secondary-button" type="button" ${state.isRefreshingLibrary || state.isDeletingLibraryArtifact || state.isDownloadingLibraryArtifact ? "disabled" : ""}>
                  ${state.isRefreshingLibrary ? "Atualizando biblioteca..." : "Atualizar biblioteca"}
                </button>
              </div>
              ${state.libraryInventoryError ? `<p class="error-box">${escapeHtml(state.libraryInventoryError)}</p>` : ""}
              ${state.libraryDownloadError ? `<p class="error-box">${escapeHtml(state.libraryDownloadError)}</p>` : ""}
              ${libraryDownloadStatusMarkup}
              ${
                state.libraryInventory
                  ? `
                    <div class="focus-grid create-focus-grid">
                      <article class="focus-item">
                        <span>Artefatos</span>
                        <strong>${state.libraryInventory.artifactCount}</strong>
                      </article>
                      <article class="focus-item">
                        <span>Espaco ocupado</span>
                        <strong>${formatStorage(state.libraryInventory.totalSizeBytes / 1024 / 1024 / 1024)}</strong>
                      </article>
                      <article class="focus-item">
                        <span>Integridade</span>
                        <strong>${libraryIntegrityIssueCount ? `${libraryIntegrityIssueCount} com alerta` : "Tudo ok"}</strong>
                      </article>
                      <article class="focus-item">
                        <span>Raiz local</span>
                        <strong>${escapeHtml(state.libraryInventory.libraryRoot)}</strong>
                      </article>
                    </div>
                  `
                  : ""
              }
              ${
                state.libraryInventory
                  ? `<div class="soft-panel"><strong>Biblioteca local</strong><p class="muted">Use esta area para baixar pendencias, confirmar integridade e limpar artefatos antes de montar um novo USBRAIN.</p></div>`
                  : ""
              }
              <div class="library-results">${libraryArtifactCards}</div>
            </section>
          `
          : ""
      }

      ${
        activeMainView === "validation"
          ? `
            <section class="card">
              <div class="eyebrow">QA guiado</div>
              <h2>Validar o USBRAIN por etapas</h2>
              <p class="muted">Siga a ordem abaixo. Cada etapa confirma um nivel diferente de prontidao antes da distribuicao.</p>
              ${state.exportError ? `<p class="error-box">${escapeHtml(state.exportError)}</p>` : ""}
              ${state.assemblyError ? `<p class="error-box">${escapeHtml(state.assemblyError)}</p>` : ""}
              ${state.validationError ? `<p class="error-box">${escapeHtml(state.validationError)}</p>` : ""}
              ${state.smokeTestError ? `<p class="error-box">${escapeHtml(state.smokeTestError)}</p>` : ""}
              ${state.runtimeProbeError ? `<p class="error-box">${escapeHtml(state.runtimeProbeError)}</p>` : ""}
              ${state.endToEndProbeError ? `<p class="error-box">${escapeHtml(state.endToEndProbeError)}</p>` : ""}
              ${state.fullLocalLlmProbeError ? `<p class="error-box">${escapeHtml(state.fullLocalLlmProbeError)}</p>` : ""}
              ${state.readinessReportError ? `<p class="error-box">${escapeHtml(state.readinessReportError)}</p>` : ""}
              ${state.distributionError ? `<p class="error-box">${escapeHtml(state.distributionError)}</p>` : ""}
              ${state.releasePackageError ? `<p class="error-box">${escapeHtml(state.releasePackageError)}</p>` : ""}
            </section>

            <section class="workflow-grid">
              <article class="card workflow-card">
                <span class="workflow-step">1</span>
                <div class="eyebrow">Montagem</div>
                <h3>Criar no pendrive</h3>
                <p class="muted">Gera o USBRAIN no destino selecionado e dispara a cadeia automatica da fabrica.</p>
                ${state.assemblyResult ? `<span class="status good">Montado</span>` : '<span class="status warn">Pendente</span>'}
                <button id="assemble-build-plan" class="primary-button secondary-button" type="button" ${state.isAssembling || state.isExporting || state.isValidating || state.isRunningSmokeTest || state.isRunningRuntimeProbe || state.isRunningEndToEndProbe || state.isRunningFullLocalLlmProbe || state.isGeneratingReadinessReport || state.isPreparingDistribution || state.isPreparingReleasePackage || hasPendingModelDownloads || !selectedTarget ? "disabled" : ""}>
                  ${state.isAssembling ? "Criando..." : "Criar no pendrive"}
                </button>
              </article>

              <article class="card workflow-card">
                <span class="workflow-step">2</span>
                <div class="eyebrow">Estrutura</div>
                <h3>Validar montagem</h3>
                <p class="muted">Confere se runtime, manifestos, modelos e arquivos essenciais ficaram consistentes.</p>
                ${validationBadge}
                <button id="validate-assembly" class="primary-button ghost-button" type="button" ${state.isValidating || state.isAssembling || state.isExporting || state.isRunningSmokeTest || state.isRunningRuntimeProbe || state.isRunningEndToEndProbe || state.isRunningFullLocalLlmProbe || state.isGeneratingReadinessReport || state.isPreparingDistribution || state.isPreparingReleasePackage || !selectedTarget ? "disabled" : ""}>
                  ${state.isValidating ? "Validando..." : "Validar"}
                </button>
              </article>

              <article class="card workflow-card">
                <span class="workflow-step">3</span>
                <div class="eyebrow">Runtime minimo</div>
                <h3>Smoke test</h3>
                <p class="muted">Verifica se o USBRAIN sobe no basico antes dos probes mais caros.</p>
                ${smokeBadge}
                <button id="run-smoke-test" class="primary-button ghost-button" type="button" ${state.isRunningSmokeTest || state.isValidating || state.isAssembling || state.isExporting || state.isRunningRuntimeProbe || state.isRunningEndToEndProbe || state.isRunningFullLocalLlmProbe || state.isGeneratingReadinessReport || state.isPreparingDistribution || state.isPreparingReleasePackage || !selectedTarget ? "disabled" : ""}>
                  ${state.isRunningSmokeTest ? "Executando..." : "Executar"}
                </button>
              </article>

              <article class="card workflow-card">
                <span class="workflow-step">4</span>
                <div class="eyebrow">Endpoint</div>
                <h3>Runtime probe</h3>
                <p class="muted">Confirma o boot de teste e a exposicao do endpoint HTTP do agente.</p>
                ${runtimeProbeBadge}
                <button id="run-runtime-probe" class="primary-button ghost-button" type="button" ${state.isRunningRuntimeProbe || state.isRunningSmokeTest || state.isValidating || state.isAssembling || state.isExporting || state.isRunningEndToEndProbe || state.isRunningFullLocalLlmProbe || state.isGeneratingReadinessReport || state.isPreparingDistribution || state.isPreparingReleasePackage || !selectedTarget ? "disabled" : ""}>
                  ${state.isRunningRuntimeProbe ? "Executando..." : "Executar"}
                </button>
              </article>

              <article class="card workflow-card">
                <span class="workflow-step">5</span>
                <div class="eyebrow">Contrato</div>
                <h3>Probe E2E</h3>
                <p class="muted">Valida stats, chat e resposta ponta a ponta sem depender da leitura manual dos logs.</p>
                ${endToEndProbeBadge}
                <button id="run-e2e-probe" class="primary-button ghost-button" type="button" ${state.isRunningEndToEndProbe || state.isRunningRuntimeProbe || state.isRunningSmokeTest || state.isValidating || state.isAssembling || state.isExporting || state.isRunningFullLocalLlmProbe || state.isGeneratingReadinessReport || state.isPreparingDistribution || state.isPreparingReleasePackage || !selectedTarget ? "disabled" : ""}>
                  ${state.isRunningEndToEndProbe ? "Executando..." : "Executar"}
                </button>
              </article>

              <article class="card workflow-card">
                <span class="workflow-step">6</span>
                <div class="eyebrow">LLM local</div>
                <h3>Probe completo</h3>
                <p class="muted">Etapa final da fabrica: garante o USBRAIN operando com modelo local real.</p>
                ${fullLocalLlmProbeBadge}
                <button id="run-full-llm-probe" class="primary-button ghost-button" type="button" ${state.isRunningFullLocalLlmProbe || state.isRunningEndToEndProbe || state.isRunningRuntimeProbe || state.isRunningSmokeTest || state.isValidating || state.isAssembling || state.isExporting || state.isGeneratingReadinessReport || state.isPreparingDistribution || state.isPreparingReleasePackage || !selectedTarget ? "disabled" : ""}>
                  ${state.isRunningFullLocalLlmProbe ? "Executando..." : "Executar"}
                </button>
              </article>

              <article class="card workflow-card">
                <span class="workflow-step">7</span>
                <div class="eyebrow">Prontidao</div>
                <h3>Relatorio final</h3>
                <p class="muted">Consolida o gate de distribuicao antes de gerar bundle e release.</p>
                ${readinessBadge}
                <div class="inline-actions compact-actions">
                  <button id="generate-readiness-report" class="primary-button secondary-button" type="button" ${state.isGeneratingReadinessReport || state.isRunningFullLocalLlmProbe || state.isRunningEndToEndProbe || state.isRunningRuntimeProbe || state.isRunningSmokeTest || state.isValidating || state.isAssembling || state.isExporting || state.isPreparingDistribution || state.isPreparingReleasePackage || !selectedTarget ? "disabled" : ""}>
                    ${state.isGeneratingReadinessReport ? "Gerando..." : "Gerar relatorio"}
                  </button>
                  <button id="prepare-distribution-bundle" class="primary-button secondary-button" type="button" ${state.isPreparingDistribution || state.isGeneratingReadinessReport || state.isRunningFullLocalLlmProbe || state.isRunningEndToEndProbe || state.isRunningRuntimeProbe || state.isRunningSmokeTest || state.isValidating || state.isAssembling || state.isExporting || state.isPreparingReleasePackage || !selectedTarget ? "disabled" : ""}>
                    ${state.isPreparingDistribution ? "Preparando..." : "Preparar bundle"}
                  </button>
                  <button id="prepare-release-package" class="primary-button secondary-button" type="button" ${state.isPreparingReleasePackage || state.isPreparingDistribution || state.isGeneratingReadinessReport || state.isRunningFullLocalLlmProbe || state.isRunningEndToEndProbe || state.isRunningRuntimeProbe || state.isRunningSmokeTest || state.isValidating || state.isAssembling || state.isExporting || !selectedTarget ? "disabled" : ""}>
                    ${state.isPreparingReleasePackage ? "Preparando..." : "Gerar release"}
                  </button>
                </div>
              </article>
            </section>

            <section class="grid three-columns">
              <article class="card compact">
                <div class="eyebrow">Estrutural</div>
                <h3>Validacao</h3>
                ${validationBadge}
                <p class="muted">${state.validationResult?.checks.length ?? 0} checks · ${state.validationResult?.errors.length ?? 0} erros</p>
              </article>
              <article class="card compact">
                <div class="eyebrow">Operacional</div>
                <h3>Smoke test</h3>
                ${smokeBadge}
                <p class="muted">${state.smokeTestResult?.checks.length ?? 0} checks · ${state.smokeTestResult?.warnings.length ?? 0} alertas</p>
              </article>
              <article class="card compact">
                <div class="eyebrow">Boot</div>
                <h3>Runtime probe</h3>
                ${runtimeProbeBadge}
                <p class="muted">${state.runtimeProbeResult ? `${state.runtimeProbeResult.durationMs} ms` : "Sem execucao ainda"}</p>
              </article>
              <article class="card compact">
                <div class="eyebrow">Contrato</div>
                <h3>Probe E2E</h3>
                ${endToEndProbeBadge}
                <p class="muted">${state.endToEndProbeResult ? `${state.endToEndProbeResult.durationMs} ms` : "Sem execucao ainda"}</p>
              </article>
              <article class="card compact">
                <div class="eyebrow">LLM local</div>
                <h3>Probe completo</h3>
                ${fullLocalLlmProbeBadge}
                <p class="muted">${state.fullLocalLlmProbeResult?.selectedModelCount ?? 0} modelos validados</p>
              </article>
              <article class="card compact">
                <div class="eyebrow">Prontidao</div>
                <h3>Gate de distribuicao</h3>
                ${readinessBadge}
                <p class="muted">${state.readinessReport?.readinessScore ?? 0}/100 · ${state.readinessReport?.blockedReason ?? "Sem bloqueios registrados"}</p>
              </article>
              <article class="card compact">
                <div class="eyebrow">Bundle final</div>
                <h3>Distribuicao</h3>
                ${distributionBadge}
                <p class="muted">${state.distributionResult?.message ?? "Sem bundle final ainda"}</p>
              </article>
              <article class="card compact">
                <div class="eyebrow">Distribuicao</div>
                <h3>Release</h3>
                ${releaseBadge}
                <p class="muted">${state.releasePackageResult?.recommendedReleaseTag ?? "Sem pacote ainda"}</p>
              </article>
            </section>

            <section class="card">
              <div class="eyebrow">Detalhes</div>
              <h2>Logs, checks e relatorios</h2>
              <details class="card-disclosure-block">
                <summary>Validacao estrutural</summary>
                <div class="grid two-columns nested-grid">
                  <article class="card compact">
                    <h3>Problemas identificados</h3>
                    <h4 class="subheading">Erros</h4>
                    <ul class="doc-list">${validationErrorsMarkup}</ul>
                    <h4 class="subheading">Alertas</h4>
                    <ul class="doc-list">${validationWarningsMarkup}</ul>
                  </article>
                  <article class="card compact">
                    <h3>Checks executados</h3>
                    <div class="grid two-columns nested-grid">${validationChecksMarkup}</div>
                  </article>
                </div>
              </details>
              <details class="card-disclosure-block">
                <summary>Smoke test</summary>
                <div class="grid two-columns nested-grid">
                  <article class="card compact">
                    <h3>Problemas operacionais</h3>
                    <h4 class="subheading">Erros</h4>
                    <ul class="doc-list">${smokeErrorsMarkup}</ul>
                    <h4 class="subheading">Alertas</h4>
                    <ul class="doc-list">${smokeWarningsMarkup}</ul>
                  </article>
                  <article class="card compact">
                    <h3>Checks executados</h3>
                    <div class="grid two-columns nested-grid">${smokeChecksMarkup}</div>
                  </article>
                </div>
              </details>
              <details class="card-disclosure-block">
                <summary>Runtime probe e probe E2E</summary>
                <div class="grid two-columns nested-grid">
                  <article class="card compact">
                    <h3>Runtime probe</h3>
                    <ul class="doc-list">${runtimeProbeErrorsMarkup}</ul>
                    <ul class="doc-list">${runtimeProbeWarningsMarkup}</ul>
                    <ul class="doc-list">
                      <li>${escapeHtml(state.runtimeProbeResult?.stdoutLogPath ?? "stdout indisponivel")}</li>
                      <li>${escapeHtml(state.runtimeProbeResult?.stderrLogPath ?? "stderr indisponivel")}</li>
                    </ul>
                    <div class="grid two-columns nested-grid">${runtimeProbeChecksMarkup}</div>
                  </article>
                  <article class="card compact">
                    <h3>Probe E2E</h3>
                    <ul class="doc-list">${endToEndProbeErrorsMarkup}</ul>
                    <ul class="doc-list">${endToEndProbeWarningsMarkup}</ul>
                    <ul class="doc-list">
                      <li>${escapeHtml(state.endToEndProbeResult?.stdoutLogPath ?? "stdout indisponivel")}</li>
                      <li>${escapeHtml(state.endToEndProbeResult?.stderrLogPath ?? "stderr indisponivel")}</li>
                    </ul>
                    <div class="grid two-columns nested-grid">${endToEndProbeChecksMarkup}</div>
                  </article>
                </div>
              </details>
              <details class="card-disclosure-block">
                <summary>LLM local, prontidao e release</summary>
                <div class="grid two-columns nested-grid">
                  <article class="card compact">
                    <h3>Probe com LLM local</h3>
                    <ul class="doc-list">${fullLocalLlmProbeErrorsMarkup}</ul>
                    <ul class="doc-list">${fullLocalLlmProbeWarningsMarkup}</ul>
                    <ul class="doc-list">
                      <li>${escapeHtml(state.fullLocalLlmProbeResult?.stdoutLogPath ?? "stdout indisponivel")}</li>
                      <li>${escapeHtml(state.fullLocalLlmProbeResult?.stderrLogPath ?? "stderr indisponivel")}</li>
                    </ul>
                    <div class="grid two-columns nested-grid">${fullLocalLlmProbeChecksMarkup}</div>
                  </article>
                  <article class="card compact">
                    <h3>Prontidao e release</h3>
                    <h4 class="subheading">Bloqueios</h4>
                    <ul class="doc-list">${readinessBlockingMarkup}</ul>
                    <h4 class="subheading">Alertas</h4>
                    <ul class="doc-list">${readinessAdvisoryMarkup}</ul>
                    <div class="grid two-columns nested-grid">${readinessSectionsMarkup}</div>
                    <div class="grid two-columns nested-grid">${distributionFilesMarkup}</div>
                    <div class="grid two-columns nested-grid">${releaseFilesMarkup}</div>
                    <div class="grid two-columns nested-grid">${releaseChecksumsMarkup}</div>
                  </article>
                </div>
              </details>
            </section>
          `
          : ""
      }

      ${
        activeMainView === "advanced"
          ? `
            <section class="grid two-columns">
              <article class="card">
                <div class="eyebrow">Workspace</div>
                <h2>Base tecnica do Builder</h2>
                <p class="muted">${escapeHtml(state.overview.workspaceRoot)}</p>
                <dl class="detail-list">
                  <div><dt>Diretorio de schemas</dt><dd>${escapeHtml(state.snapshot.schemaDirectory)}</dd></div>
                  <div><dt>Backend Rust</dt><dd>${state.overview.tauriBackendAvailable ? "Disponivel" : "Nao conectado"}</dd></div>
                  <div><dt>Schemas</dt><dd>${state.overview.schemaCount}</dd></div>
                  <div><dt>Documentos-chave</dt><dd>${state.overview.documentCount}</dd></div>
                </dl>
                <ul class="doc-list">${docs}</ul>
              </article>

              <article class="card">
                <div class="eyebrow">Schemas</div>
                <h2>Contratos expostos ao Builder</h2>
                <div class="grid two-columns nested-grid">${schemaCards}</div>
              </article>
            </section>

            <section class="card">
              <div class="eyebrow">Manifestos</div>
              <h2>${escapeHtml(selectedManifest?.title ?? "Previa do build-manifest.json")}</h2>
              <div class="manifest-selector">${manifestCards}</div>
              <pre class="manifest-preview"><code>${escapeHtml(selectedManifest?.contents ?? state.manifestPreview)}</code></pre>
            </section>

            <section class="grid two-columns">
              <article class="card">
                <div class="eyebrow">Exportacao</div>
                <h2>Arquivos persistidos em disco</h2>
                <div class="grid two-columns nested-grid">${exportedFilesMarkup}</div>
              </article>

              <article class="card">
                <div class="eyebrow">Montagem</div>
                <h2>Diretorios e arquivos no destino</h2>
                <h3 class="subheading">Diretorios</h3>
                <div class="grid two-columns nested-grid">${assembledDirectoriesMarkup}</div>
                <h3 class="subheading">Arquivos</h3>
                <div class="grid two-columns nested-grid">${assemblyFilesMarkup}</div>
                <h3 class="subheading">Alertas</h3>
                <ul class="doc-list">${assemblyWarningsMarkup}</ul>
              </article>
            </section>
          `
          : ""
      }
    </main>
  `;

  bindPlannerEvents();
  restoreFocusSnapshot(focusSnapshot);
}

async function refreshBuildPlan(): Promise<void> {
  const buildPlan = await invokeSafe<BuildPlanResponse>(
    "generate_build_plan",
    buildFallbackBuildPlan(currentState.targets),
    captureResolvedBuildRequest(currentState.targets),
  );

  const selectedManifest = getSelectedManifest(buildPlan);
  currentState = {
    ...currentState,
    buildPlan,
    manifestPreview: selectedManifest?.contents ?? currentState.manifestPreview,
    exportError: null,
    assemblyError: null,
    validationError: null,
    smokeTestError: null,
    runtimeProbeError: null,
    endToEndProbeError: null,
    fullLocalLlmProbeError: null,
    readinessReportError: null,
    distributionError: null,
    releasePackageError: null,
  };
  render(currentState);
}

async function refreshModelCatalog(): Promise<void> {
  try {
    const modelCatalog = await invoke<ModelCatalogResponse>("get_model_catalog", buildModelCatalogRequest(currentState.targets));
    const selectedModelIds = new Set(modelCatalog.models.map((model) => model.modelId));
    plannerDraft.selectedModelIds = plannerDraft.selectedModelIds.filter((modelId) => selectedModelIds.has(modelId));
    currentState = {
      ...currentState,
      modelCatalog,
      catalogError: null,
    };
  } catch {
    plannerDraft.selectedModelIds = [];
    currentState = {
      ...currentState,
      modelCatalog: null,
      catalogError:
        "Nenhum catalogo real esta disponivel no momento. Sincronize uma fonte remota verificavel antes de escolher ou baixar LLMs.",
    };
  }
  render(currentState);
}

async function refreshTargets(): Promise<void> {
  currentState = {
    ...currentState,
    isRefreshingTargets: true,
    targetRefreshError: null,
  };
  render(currentState);

  const previousSelection = plannerDraft.selectedTargetMountPath;
  const refreshedTargets = await invokeSafe<CandidateTarget[]>("list_candidate_targets", currentState.targets);
  const resolvedSelection =
    refreshedTargets.find((target) => target.mountPath === previousSelection)?.mountPath ??
    refreshedTargets[0]?.mountPath ??
    null;
  plannerDraft.selectedTargetMountPath = resolvedSelection;

  currentState = {
    ...currentState,
    targets: refreshedTargets,
    isRefreshingTargets: false,
    targetRefreshError: refreshedTargets.length
      ? null
      : "Nenhum pendrive USB foi detectado agora. Conecte o dispositivo e clique novamente em atualizar.",
    targetAuthorizationError:
      refreshedTargets.find((target) => target.mountPath === resolvedSelection) ? currentState.targetAuthorizationError : null,
  };
  render(currentState);

  await Promise.all([refreshBuildPlan(), refreshModelCatalog()]);
}

async function refreshLiveCatalog(): Promise<void> {
  currentState = {
    ...currentState,
    isRefreshingCatalog: true,
    catalogRefreshError: null,
  };
  render(currentState);

  try {
    const catalogRefreshResult = await invoke<RefreshModelCatalogResult>("refresh_model_catalog", {});
    await refreshModelCatalog();
    currentState = {
      ...currentState,
      catalogRefreshResult,
      isRefreshingCatalog: false,
      catalogRefreshError: null,
    };
  } catch {
    currentState = {
      ...currentState,
      isRefreshingCatalog: false,
      catalogRefreshError:
        "O backend Tauri nao conseguiu atualizar o catalogo vivo. Configure uma fonte remota em `catalog/catalog-sources.json` ou `USBRAIN_MODEL_CATALOG_URL`.",
    };
  }

  render(currentState);
}

async function refreshBuilderLibrary(): Promise<void> {
  currentState = {
    ...currentState,
    isRefreshingLibrary: true,
    libraryInventoryError: null,
  };
  render(currentState);

  const fallbackInventory: BuilderLibraryInventory = {
    libraryRoot: "biblioteca indisponivel",
    artifactCount: 0,
    totalSizeBytes: 0,
    artifacts: [],
  };

  const libraryInventory = await invokeSafe<BuilderLibraryInventory>("list_builder_library", fallbackInventory);
  currentState = {
    ...currentState,
    libraryInventory,
    isRefreshingLibrary: false,
    libraryInventoryError:
      libraryInventory.libraryRoot === "biblioteca indisponivel"
        ? "O backend Tauri nao respondeu para listar a biblioteca local do Builder."
        : null,
  };
  render(currentState);
}

async function prepareBuilderLibrary(): Promise<void> {
  currentState = {
    ...currentState,
    isPreparingLibrary: true,
    libraryPreparationError: null,
  };
  render(currentState);

  try {
    const libraryPreparationResult = await invoke<BuilderLibraryPreparationResult>(
      "prepare_builder_library",
      resolvedBuildRequest(currentState.targets),
    );
    await Promise.all([refreshBuildPlan(), refreshModelCatalog(), refreshBuilderLibrary()]);
    currentState = {
      ...currentState,
      libraryPreparationResult,
      isPreparingLibrary: false,
      libraryPreparationError: null,
    };
  } catch {
    currentState = {
      ...currentState,
      isPreparingLibrary: false,
      libraryPreparationError:
        "O backend Tauri nao conseguiu preparar a biblioteca local do Builder neste ambiente.",
    };
  }

  render(currentState);
}

async function deleteBuilderLibraryArtifact(request: DeleteBuilderLibraryArtifactRequest): Promise<void> {
  currentState = {
    ...currentState,
    isDeletingLibraryArtifact: true,
    libraryDownloadError: null,
    libraryInventoryError: null,
  };
  render(currentState);

  try {
    await invoke<DeleteBuilderLibraryArtifactResult>("delete_builder_library_artifact", { request });
    await Promise.all([refreshBuildPlan(), refreshModelCatalog(), refreshBuilderLibrary()]);
    currentState = {
      ...currentState,
      isDeletingLibraryArtifact: false,
      libraryPreparationResult: null,
      libraryDownloadResult: null,
      libraryInventoryError: null,
    };
  } catch {
    currentState = {
      ...currentState,
      isDeletingLibraryArtifact: false,
      libraryInventoryError: "O backend Tauri nao conseguiu excluir o artefato da biblioteca local.",
    };
  }

  render(currentState);
}

async function downloadBuilderLibraryArtifact(request: DownloadBuilderLibraryArtifactRequest): Promise<void> {
  currentState = {
    ...currentState,
    isDownloadingLibraryArtifact: true,
    libraryDownloadError: null,
    libraryInventoryError: null,
  };
  render(currentState);

  try {
    const libraryDownloadResult = await invoke<DownloadBuilderLibraryArtifactResult>(
      "download_builder_library_artifact",
      { request },
    );
    await Promise.all([refreshBuildPlan(), refreshModelCatalog(), refreshBuilderLibrary()]);
    currentState = {
      ...currentState,
      isDownloadingLibraryArtifact: false,
      libraryDownloadResult,
      libraryDownloadError: null,
      libraryPreparationResult: null,
      libraryInventoryError: null,
    };
  } catch {
    currentState = {
      ...currentState,
      isDownloadingLibraryArtifact: false,
      libraryDownloadError:
        "O backend Tauri nao conseguiu baixar o artefato pendente para a biblioteca local.",
    };
  }

  render(currentState);
}

async function downloadCatalogModel(request: DownloadCatalogModelRequest): Promise<void> {
  currentState = {
    ...currentState,
    isDownloadingLibraryArtifact: true,
    libraryDownloadError: null,
    libraryInventoryError: null,
  };
  render(currentState);

  try {
    const libraryDownloadResult = await invoke<DownloadBuilderLibraryArtifactResult>("download_catalog_model", {
      request,
    });
    await Promise.all([refreshBuildPlan(), refreshModelCatalog(), refreshBuilderLibrary()]);
    currentState = {
      ...currentState,
      isDownloadingLibraryArtifact: false,
      libraryDownloadResult,
      libraryDownloadError: null,
      libraryPreparationResult: null,
      libraryInventoryError: null,
    };
  } catch {
    currentState = {
      ...currentState,
      isDownloadingLibraryArtifact: false,
      libraryDownloadError:
        "O backend Tauri nao conseguiu baixar diretamente este modelo a partir do catalogo vivo.",
    };
  }

  render(currentState);
}

async function exportBuildPlan(): Promise<void> {
  currentState = {
    ...currentState,
    isExporting: true,
    exportError: null,
  };
  render(currentState);

  const fallbackExportResult: BuildPlanExportResult = {
    buildId: currentState.buildPlan?.summary.buildId ?? "preview-usbrain-build",
    exportRoot: "builder/output/preview-usbrain-build",
    manifestsDirectory: "builder/output/preview-usbrain-build/manifests",
    summaryFile: "builder/output/preview-usbrain-build/build-plan.json",
    exportedFiles: currentState.buildPlan?.manifests.map((manifest) => ({
      fileName: manifest.fileName,
      absolutePath: `builder/output/preview-usbrain-build/manifests/${manifest.fileName}`,
      relativePath: `manifests/${manifest.fileName}`,
      sizeBytes: manifest.contents.length,
    })) ?? [],
  };

  try {
    const exportResult = await invoke<BuildPlanExportResult>("export_build_plan", resolvedBuildRequest(currentState.targets));
    currentState = {
      ...currentState,
      exportResult,
      isExporting: false,
      exportError: null,
    };
  } catch {
    currentState = {
      ...currentState,
      exportResult: fallbackExportResult,
      isExporting: false,
      exportError: "O backend Tauri nao conseguiu persistir em disco neste ambiente. O preview local foi mantido como fallback.",
    };
  }

  render(currentState);
}

async function runRuntimeProbe(): Promise<void> {
  currentState = {
    ...currentState,
    isRunningRuntimeProbe: true,
    runtimeProbeError: null,
  };
  render(currentState);

  const fallbackRuntimeProbe: RuntimeProbeResult = {
    buildId: currentState.buildPlan?.summary.buildId ?? "preview-usbrain-build",
    targetRoot: currentState.buildPlan?.summary.targetMountPath ?? "indisponivel",
    probeStatus: "passed-with-warnings",
    hostPythonCommand: "indisponivel",
    endpointUrl: "indisponivel",
    webPort: 0,
    bgePort: 0,
    llmPort: 0,
    durationMs: 0,
    stdoutLogPath: "indisponivel",
    stderrLogPath: "indisponivel",
    checks: [],
    errors: [],
    warnings: ["O backend Tauri nao respondeu; resultado local mantido como fallback."],
  };

  try {
    const runtimeProbeResult = await invoke<RuntimeProbeResult>("run_runtime_probe", resolvedBuildRequest(currentState.targets));
    currentState = {
      ...currentState,
      runtimeProbeResult,
      isRunningRuntimeProbe: false,
      runtimeProbeError: null,
    };
  } catch (error) {
    currentState = {
      ...currentState,
      runtimeProbeResult: fallbackRuntimeProbe,
      isRunningRuntimeProbe: false,
      runtimeProbeError: error instanceof Error
        ? error.message
        : "O runtime probe falhou no backend Tauri. Verifique a montagem e tente novamente.",
    };
  }

  render(currentState);
}

async function runEndToEndProbe(): Promise<void> {
  currentState = {
    ...currentState,
    isRunningEndToEndProbe: true,
    endToEndProbeError: null,
  };
  render(currentState);

  const fallbackEndToEndProbe: EndToEndProbeResult = {
    buildId: currentState.buildPlan?.summary.buildId ?? "preview-usbrain-build",
    targetRoot: currentState.buildPlan?.summary.targetMountPath ?? "indisponivel",
    probeStatus: "passed-with-warnings",
    hostPythonCommand: "indisponivel",
    statsEndpointUrl: "indisponivel",
    chatEndpointUrl: "indisponivel",
    probePrompt: "USBRAIN probe: responda com OK",
    responsePreview: "indisponivel",
    webPort: 0,
    bgePort: 0,
    llmPort: 0,
    durationMs: 0,
    stdoutLogPath: "indisponivel",
    stderrLogPath: "indisponivel",
    checks: [],
    errors: [],
    warnings: ["O backend Tauri nao respondeu; resultado local mantido como fallback."],
  };

  try {
    const endToEndProbeResult = await invoke<EndToEndProbeResult>("run_end_to_end_probe", resolvedBuildRequest(currentState.targets));
    currentState = {
      ...currentState,
      endToEndProbeResult,
      isRunningEndToEndProbe: false,
      endToEndProbeError: null,
    };
  } catch (error) {
    currentState = {
      ...currentState,
      endToEndProbeResult: fallbackEndToEndProbe,
      isRunningEndToEndProbe: false,
      endToEndProbeError: error instanceof Error
        ? error.message
        : "O probe ponta a ponta falhou no backend Tauri. Verifique a montagem e tente novamente.",
    };
  }

  render(currentState);
}

async function runFullLocalLlmProbe(): Promise<void> {
  await performFullLocalLlmProbe(false);
}

async function generateReadinessReport(): Promise<void> {
  currentState = {
    ...currentState,
    isGeneratingReadinessReport: true,
    readinessReportError: null,
  };
  render(currentState);

  const fallbackReadinessReport: BuildReadinessReport = {
    buildId: currentState.buildPlan?.summary.buildId ?? "preview-usbrain-build",
    targetRoot: currentState.buildPlan?.summary.targetMountPath ?? "indisponivel",
    readinessStatus: "ready-with-warnings",
    readinessScore: 0,
    distributionReady: false,
    blockedReason: "O backend Tauri nao respondeu para consolidar a prontidao.",
    generatedAt: "indisponivel",
    sections: [],
    blockingIssues: [],
    advisoryIssues: ["O backend Tauri nao respondeu; resultado local mantido como fallback."],
  };

  try {
    const readinessReport = await invoke<BuildReadinessReport>("generate_build_readiness_report", resolvedBuildRequest(currentState.targets));
    currentState = {
      ...currentState,
      readinessReport,
      isGeneratingReadinessReport: false,
      readinessReportError: null,
    };
  } catch (error) {
    currentState = {
      ...currentState,
      readinessReport: fallbackReadinessReport,
      isGeneratingReadinessReport: false,
      readinessReportError: error instanceof Error
        ? error.message
        : "O relatorio consolidado de prontidao falhou no backend Tauri. Verifique a montagem e tente novamente.",
    };
  }

  render(currentState);
}

async function prepareDistributionBundle(): Promise<void> {
  currentState = {
    ...currentState,
    isPreparingDistribution: true,
    distributionError: null,
  };
  render(currentState);

  const fallbackDistributionResult: BuildDistributionGateResult = {
    buildId: currentState.buildPlan?.summary.buildId ?? "preview-usbrain-build",
    exportRoot: "builder/output/preview-usbrain-build",
    distributionStatus: "blocked",
    distributionReady: false,
    message: "O backend Tauri nao respondeu para preparar a distribuicao final.",
    readinessReportFile: "builder/output/preview-usbrain-build/readiness-report.json",
    distributionSummaryFile: "builder/output/preview-usbrain-build/distribution-summary.json",
    exportedFiles: [],
    readinessReport: currentState.readinessReport ?? {
      buildId: currentState.buildPlan?.summary.buildId ?? "preview-usbrain-build",
      targetRoot: currentState.buildPlan?.summary.targetMountPath ?? "indisponivel",
      readinessStatus: "blocked",
      readinessScore: 0,
      distributionReady: false,
      blockedReason: "O backend Tauri nao respondeu para consolidar a distribuicao.",
      generatedAt: "indisponivel",
      sections: [],
      blockingIssues: [],
      advisoryIssues: ["O backend Tauri nao respondeu; resultado local mantido como fallback."],
    },
  };

  try {
    const distributionResult = await invoke<BuildDistributionGateResult>("prepare_distribution_bundle", resolvedBuildRequest(currentState.targets));
    currentState = {
      ...currentState,
      distributionResult,
      readinessReport: distributionResult.readinessReport,
      isPreparingDistribution: false,
      distributionError: distributionResult.distributionReady
        ? null
        : "Distribuicao final bloqueada pelo gate de prontidao. Revise o relatorio consolidado antes de distribuir.",
    };
  } catch (error) {
    currentState = {
      ...currentState,
      distributionResult: fallbackDistributionResult,
      readinessReport: fallbackDistributionResult.readinessReport,
      isPreparingDistribution: false,
      distributionError: error instanceof Error
        ? error.message
        : "A preparacao da distribuicao final falhou no backend Tauri. Verifique a montagem e tente novamente.",
    };
  }

  render(currentState);
}

async function prepareReleasePackage(): Promise<void> {
  currentState = {
    ...currentState,
    isPreparingReleasePackage: true,
    releasePackageError: null,
  };
  render(currentState);

  const fallbackReleasePackageResult: BuildReleasePackageResult = {
    buildId: currentState.buildPlan?.summary.buildId ?? "preview-usbrain-build",
    exportRoot: "builder/output/preview-usbrain-build",
    releaseStatus: "blocked",
    releaseReady: false,
    message: "O backend Tauri nao respondeu para preparar o pacote de release.",
    recommendedReleaseTag: "usbrain-builder-v0.1.0-preview-usbrain-build",
    releaseManifestFile: "builder/output/preview-usbrain-build/release-package.json",
    githubReleaseNotesFile: "builder/output/preview-usbrain-build/github-release-notes.md",
    usbrainReleaseSummaryFile: "builder/output/preview-usbrain-build/usbrain-release-summary.json",
    checksumsFile: "builder/output/preview-usbrain-build/SHA256SUMS.txt",
    exportedFiles: [],
    artifactChecksums: [],
    distributionResult: currentState.distributionResult ?? {
      buildId: currentState.buildPlan?.summary.buildId ?? "preview-usbrain-build",
      exportRoot: "builder/output/preview-usbrain-build",
      distributionStatus: "blocked",
      distributionReady: false,
      message: "O backend Tauri nao respondeu para preparar a distribuicao final.",
      readinessReportFile: "builder/output/preview-usbrain-build/readiness-report.json",
      distributionSummaryFile: "builder/output/preview-usbrain-build/distribution-summary.json",
      exportedFiles: [],
      readinessReport: currentState.readinessReport ?? {
        buildId: currentState.buildPlan?.summary.buildId ?? "preview-usbrain-build",
        targetRoot: currentState.buildPlan?.summary.targetMountPath ?? "indisponivel",
        readinessStatus: "blocked",
        readinessScore: 0,
        distributionReady: false,
        blockedReason: "O backend Tauri nao respondeu para consolidar o release.",
        generatedAt: "indisponivel",
        sections: [],
        blockingIssues: [],
        advisoryIssues: ["O backend Tauri nao respondeu; resultado local mantido como fallback."],
      },
    },
  };

  try {
    const releasePackageResult = await invoke<BuildReleasePackageResult>("prepare_release_package", resolvedBuildRequest(currentState.targets));
    currentState = {
      ...currentState,
      releasePackageResult,
      distributionResult: releasePackageResult.distributionResult,
      readinessReport: releasePackageResult.distributionResult.readinessReport,
      isPreparingReleasePackage: false,
      releasePackageError: releasePackageResult.releaseReady
        ? null
        : "Pacote de release gerado com bloqueios. Revise o gate de distribuicao antes de publicar no GitHub ou no usbrain.org.",
    };
  } catch (error) {
    currentState = {
      ...currentState,
      releasePackageResult: fallbackReleasePackageResult,
      distributionResult: fallbackReleasePackageResult.distributionResult,
      readinessReport: fallbackReleasePackageResult.distributionResult.readinessReport,
      isPreparingReleasePackage: false,
      releasePackageError: error instanceof Error
        ? error.message
        : "A preparacao do pacote de release falhou no backend Tauri. Verifique a distribuicao final e tente novamente.",
    };
  }

  render(currentState);
}

function isTargetWipeConfirmationError(message: string): boolean {
  return message.includes("ja contem arquivos") && message.includes("limpeza completa do pendrive");
}

async function assembleBuildPlan(wipeTargetIfNeeded = false): Promise<void> {
  if (!plannerDraft.selectedModelIds.length) {
    currentState = {
      ...currentState,
      assemblyError:
        "Nenhuma LLM foi selecionada manualmente. Marque explicitamente os modelos desejados antes de criar o pendrive, para evitar lineup automatico invisivel.",
      assemblyProgress: {
        status: "idle",
        percent: 0,
        message: "Aguardando a selecao manual das LLMs para a criacao do USBRAIN.",
      },
    };
    render(currentState);
    return;
  }

  if (!(await authorizeSelectedTargetAccess())) {
    return;
  }

  currentState = {
    ...currentState,
    isAssembling: true,
    assemblyError: null,
    pendingWipeConfirmation: null,
    assemblyProgress: {
      status: "running",
      percent: 4,
      message: "Iniciando a gravacao do USBRAIN no pendrive.",
    },
  };
  render(currentState);
  startAssemblyProgress();

  try {
    const assemblyResult = await invoke<BuildAssemblyResult>(
      "assemble_build_plan",
      resolvedBuildRequest(currentState.targets, { wipeTargetIfNeeded }),
    );
    stopAssemblyProgressTimer();
    currentState = {
      ...currentState,
      assemblyResult,
      isAssembling: false,
      assemblyError: null,
      pendingWipeConfirmation: null,
      assemblyProgress: {
        status: "running",
        percent: 92,
        message: "Montagem concluida. Validando automaticamente a estrutura do USBRAIN.",
      },
    };
    render(currentState);

    const validationResult = await performValidation(true);
    if (validationResult.validationStatus === "failed") {
      currentState = {
        ...currentState,
        assemblyError: "Criacao bloqueada: a validacao estrutural do USBRAIN falhou.",
      };
      stopAssemblyProgressTimer();
      render(currentState);
      return;
    }

    const smokeTestResult = await performSmokeTest(true);
    if (smokeTestResult.smokeStatus === "failed") {
      currentState = {
        ...currentState,
        assemblyError: "Criacao bloqueada: o smoke test do USBRAIN falhou.",
      };
      stopAssemblyProgressTimer();
      render(currentState);
      return;
    }

    const fullLocalLlmProbeResult = await performFullLocalLlmProbe(true);
    if (fullLocalLlmProbeResult.probeStatus === "failed") {
      currentState = {
        ...currentState,
        assemblyError: "Criacao bloqueada: o USBRAIN montado nao passou no probe completo com LLM local. A fabrica nao pode aprovar este pendrive.",
      };
      stopAssemblyProgressTimer();
      render(currentState);
      return;
    }

    await ejectMountedTargetWithConfirmation(assemblyResult.targetRoot || fullLocalLlmProbeResult.targetRoot);
  } catch (error) {
    const errorMessage = formatUnknownError(
      error,
      "A montagem falhou no backend Tauri. Verifique se o destino esta vazio ou previamente montado pelo Builder.",
    );

    if (!wipeTargetIfNeeded && isTargetWipeConfirmationError(errorMessage)) {
      const selectedTarget = getSelectedTarget(currentState.targets, plannerDraft.selectedTargetMountPath);
      currentState = {
        ...currentState,
        isAssembling: false,
        assemblyError: errorMessage,
        pendingWipeConfirmation: {
          targetLabel: selectedTarget?.label ?? "Pendrive selecionado",
          targetMountPath: selectedTarget?.mountPath ?? "indisponivel",
          message: errorMessage,
        },
        assemblyProgress: {
          status: "idle",
          percent: 0,
          message: "Aguardando sua confirmacao para apagar o pendrive e iniciar a criacao.",
        },
      };
      stopAssemblyProgressTimer();
      render(currentState);
      return;
    }

    if (isMacOsDesktop && errorMessage.includes("Operation not permitted")) {
      const selectedTarget = getSelectedTarget(currentState.targets, plannerDraft.selectedTargetMountPath);
      if (selectedTarget) {
        await forgetAuthorizedTargetAccess(selectedTarget.mountPath);
        currentState = {
          ...currentState,
          authorizedTargetMountPaths: currentState.authorizedTargetMountPaths.filter(
            (mountPath) => mountPath !== selectedTarget.mountPath,
          ),
          targetAuthorizationError:
            "O macOS ainda bloqueou a escrita no pendrive. Reautorize o volume e tente criar o USBRAIN novamente.",
        };
      }
    }

    currentState = {
      ...currentState,
      isAssembling: false,
      assemblyError: errorMessage,
      pendingWipeConfirmation: null,
      assemblyProgress: {
        status: "error",
        percent: Math.max(currentState.assemblyProgress.percent, 8),
        message: errorMessage,
      },
    };
  }

  stopAssemblyProgressTimer();
  render(currentState);
}

async function performValidation(updateAssemblyProgress: boolean): Promise<BuildValidationResult> {
  currentState = {
    ...currentState,
    isValidating: true,
    validationError: null,
    assemblyProgress: updateAssemblyProgress
      ? {
          status: "running",
          percent: 95,
          message: "Validando automaticamente a montagem do USBRAIN.",
        }
      : currentState.assemblyProgress,
  };
  render(currentState);

  const fallbackValidation: BuildValidationResult = {
    buildId: currentState.buildPlan?.summary.buildId ?? "preview-usbrain-build",
    targetRoot: currentState.buildPlan?.summary.targetMountPath ?? "indisponivel",
    validationStatus: "failed",
    checks: [],
    errors: ["O backend Tauri nao respondeu durante a validacao estrutural."],
    warnings: [],
  };

  try {
    const validationResult = await invoke<BuildValidationResult>("validate_assembled_build", resolvedBuildRequest(currentState.targets));
    currentState = {
      ...currentState,
      validationResult,
      isValidating: false,
      validationError: null,
      assemblyProgress: updateAssemblyProgress
        ? {
            status: validationResult.validationStatus === "failed" ? "error" : "running",
            percent: validationResult.validationStatus === "failed" ? 96 : 97,
            message:
              validationResult.validationStatus === "failed"
                ? "A validacao estrutural falhou apos a criacao do USBRAIN."
                : "Validacao estrutural concluida. Rodando smoke test automatico.",
          }
        : currentState.assemblyProgress,
    };
    render(currentState);
    return validationResult;
  } catch (error) {
    currentState = {
      ...currentState,
      validationResult: fallbackValidation,
      isValidating: false,
      validationError:
        error instanceof Error
          ? error.message
          : "A validacao falhou no backend Tauri. Verifique a montagem e tente novamente.",
      assemblyProgress: updateAssemblyProgress
        ? {
            status: "error",
            percent: Math.max(currentState.assemblyProgress.percent, 96),
            message: "A validacao automatica falhou apos a criacao do USBRAIN.",
          }
        : currentState.assemblyProgress,
    };
    render(currentState);
    return fallbackValidation;
  }
}

async function validateAssembly(): Promise<void> {
  await performValidation(false);
}

async function performSmokeTest(updateAssemblyProgress: boolean): Promise<BuildSmokeTestResult> {
  currentState = {
    ...currentState,
    isRunningSmokeTest: true,
    smokeTestError: null,
    assemblyProgress: updateAssemblyProgress
      ? {
          status: "running",
          percent: 98,
          message: "Executando smoke test automatico do USBRAIN criado.",
        }
      : currentState.assemblyProgress,
  };
  render(currentState);

  const fallbackSmokeTest: BuildSmokeTestResult = {
    buildId: currentState.buildPlan?.summary.buildId ?? "preview-usbrain-build",
    targetRoot: currentState.buildPlan?.summary.targetMountPath ?? "indisponivel",
    smokeStatus: "failed",
    validationStatus: currentState.validationResult?.validationStatus ?? "pendente",
    hostPythonCommand: "indisponivel",
    checks: [],
    errors: ["O backend Tauri nao respondeu durante o smoke test."],
    warnings: [],
  };

  try {
    const smokeTestResult = await invoke<BuildSmokeTestResult>("run_assembled_smoke_test", resolvedBuildRequest(currentState.targets));
    currentState = {
      ...currentState,
      smokeTestResult,
      isRunningSmokeTest: false,
      smokeTestError: null,
      assemblyProgress: updateAssemblyProgress
        ? {
            status: smokeTestResult.smokeStatus === "failed" ? "error" : "completed",
            percent: 100,
            message:
              smokeTestResult.smokeStatus === "passed"
                ? "USBRAIN criado, validado e testado com sucesso."
                : smokeTestResult.smokeStatus === "passed-with-warnings"
                  ? "USBRAIN criado e testado com alertas. Revise os avisos abaixo."
                  : "USBRAIN criado, mas o smoke test falhou. Revise os erros abaixo.",
          }
        : currentState.assemblyProgress,
    };
    render(currentState);
    return smokeTestResult;
  } catch (error) {
    currentState = {
      ...currentState,
      smokeTestResult: fallbackSmokeTest,
      isRunningSmokeTest: false,
      smokeTestError:
        error instanceof Error
          ? error.message
          : "O smoke test falhou no backend Tauri. Verifique a montagem e tente novamente.",
      assemblyProgress: updateAssemblyProgress
        ? {
            status: "error",
            percent: 100,
            message: "O smoke test automatico falhou apos a criacao do USBRAIN.",
          }
        : currentState.assemblyProgress,
    };
    render(currentState);
    return fallbackSmokeTest;
  }
}

async function runSmokeTest(): Promise<void> {
  await performSmokeTest(false);
}

async function performFullLocalLlmProbe(updateAssemblyProgress: boolean): Promise<FullLocalLlmProbeResult> {
  currentState = {
    ...currentState,
    isRunningFullLocalLlmProbe: true,
    fullLocalLlmProbeError: null,
    assemblyProgress: updateAssemblyProgress
      ? {
          status: "running",
          percent: 99,
          message: "Executando o probe completo com LLM local para aprovar o pendrive.",
        }
      : currentState.assemblyProgress,
  };
  render(currentState);

  const fallbackFullLocalLlmProbe: FullLocalLlmProbeResult = {
    buildId: currentState.buildPlan?.summary.buildId ?? "preview-usbrain-build",
    targetRoot: currentState.buildPlan?.summary.targetMountPath ?? "indisponivel",
    probeStatus: "failed",
    hostPythonCommand: "indisponivel",
    statsEndpointUrl: "indisponivel",
    chatEndpointUrl: "indisponivel",
    probePrompt: "Responda exatamente: OK",
    responsePreview: "indisponivel",
    selectedModelCount: currentState.buildPlan?.models.length ?? 0,
    webPort: 0,
    bgePort: 0,
    llmPort: 0,
    durationMs: 0,
    stdoutLogPath: "indisponivel",
    stderrLogPath: "indisponivel",
    checks: [],
    errors: ["O backend Tauri nao respondeu durante o probe completo com LLM local."],
    warnings: [],
  };

  try {
    const fullLocalLlmProbeResult = await invoke<FullLocalLlmProbeResult>("run_full_local_llm_probe", resolvedBuildRequest(currentState.targets));
    currentState = {
      ...currentState,
      fullLocalLlmProbeResult,
      isRunningFullLocalLlmProbe: false,
      fullLocalLlmProbeError: null,
      assemblyProgress: updateAssemblyProgress
        ? {
            status: fullLocalLlmProbeResult.probeStatus === "failed" ? "error" : "completed",
            percent: 100,
            message:
              fullLocalLlmProbeResult.probeStatus === "passed"
                ? "USBRAIN criado, validado, testado e aprovado no probe completo com LLM local."
                : fullLocalLlmProbeResult.probeStatus === "passed-with-warnings"
                  ? "USBRAIN aprovado no probe completo com alertas. Revise os avisos abaixo."
                  : "Criacao bloqueada: o probe completo com LLM local falhou.",
          }
        : currentState.assemblyProgress,
    };
    render(currentState);
    return fullLocalLlmProbeResult;
  } catch (error) {
    currentState = {
      ...currentState,
      fullLocalLlmProbeResult: fallbackFullLocalLlmProbe,
      isRunningFullLocalLlmProbe: false,
      fullLocalLlmProbeError: error instanceof Error
        ? error.message
        : "O probe completo com LLM falhou no backend Tauri. Verifique a montagem e tente novamente.",
      assemblyProgress: updateAssemblyProgress
        ? {
            status: "error",
            percent: 100,
            message: "Criacao bloqueada: o probe completo com LLM local nao pode ser concluido.",
          }
        : currentState.assemblyProgress,
    };
    render(currentState);
    return fallbackFullLocalLlmProbe;
  }
}

function bindPlannerEvents(): void {
  const targetSelect = document.querySelector<HTMLSelectElement>("#target-select");
  const buildModeSelect = document.querySelector<HTMLSelectElement>("#build-mode-select");
  const creatorLabelInput = document.querySelector<HTMLInputElement>("#creator-label");
  const allowGpuInput = document.querySelector<HTMLInputElement>("#allow-gpu");
  const allowCloudInput = document.querySelector<HTMLInputElement>("#allow-cloud");
  const catalogSearchInput = document.querySelector<HTMLInputElement>("#catalog-search");
  const catalogDomainSelect = document.querySelector<HTMLSelectElement>("#catalog-domain");
  const catalogLanguageSelect = document.querySelector<HTMLSelectElement>("#catalog-language");
  const catalogHostRamSelect = document.querySelector<HTMLSelectElement>("#catalog-host-ram");
  const catalogBenchmarkConfidenceSelect = document.querySelector<HTMLSelectElement>("#catalog-benchmark-confidence");
  const catalogCpuOnlyInput = document.querySelector<HTMLInputElement>("#catalog-cpu-only");
  const catalogModelToggles = Array.from(document.querySelectorAll<HTMLInputElement>(".catalog-model-toggle"));
  const modelSelectionModeButtons = Array.from(document.querySelectorAll<HTMLButtonElement>("[data-model-selection-mode]"));
  const recommendedSearchButtons = Array.from(document.querySelectorAll<HTMLButtonElement>("[data-recommended-search]"));
  const modelBulkActionButtons = Array.from(document.querySelectorAll<HTMLButtonElement>("[data-bulk-selection]"));
  const librarySearchInput = document.querySelector<HTMLInputElement>("#library-search");
  const libraryFilterButtons = Array.from(document.querySelectorAll<HTMLButtonElement>("[data-library-filter]"));
  const languageInputs = Array.from(document.querySelectorAll<HTMLInputElement>('input[name="language"]'));
  const mainTabButtons = Array.from(document.querySelectorAll<HTMLButtonElement>("[data-main-tab]"));
  const createStepButtons = Array.from(document.querySelectorAll<HTMLButtonElement>("[data-create-step]"));
  const targetCards = Array.from(document.querySelectorAll<HTMLElement>(".target-card"));

  mainTabButtons.forEach((button) => {
    button.addEventListener("click", () => {
      const nextTab = button.dataset.mainTab as MainView | undefined;
      if (!nextTab || nextTab === activeMainView) {
        return;
      }
      activeMainView = nextTab;
      render(currentState);
    });
  });

  createStepButtons.forEach((button) => {
    button.addEventListener("click", () => {
      const nextStep = button.dataset.createStep as CreateStep | undefined;
      if (!nextStep || nextStep === activeCreateStep) {
        return;
      }
      activeCreateStep = nextStep;
      render(currentState);
    });
  });

  modelSelectionModeButtons.forEach((button) => {
    button.addEventListener("click", () => {
      const nextMode = button.dataset.modelSelectionMode as ModelSelectionMode | undefined;
      if (!nextMode || nextMode === activeModelSelectionMode) {
        return;
      }
      activeModelSelectionMode = nextMode;
      render(currentState);
    });
  });

  recommendedSearchButtons.forEach((button) => {
    button.addEventListener("click", () => {
      const search = button.dataset.recommendedSearch?.trim();
      if (!search) {
        return;
      }
      activeModelSelectionMode = "manual";
      catalogViewDraft.search = search;
      render(currentState);
    });
  });

  modelBulkActionButtons.forEach((button) => {
    button.addEventListener("click", () => {
      const action = button.dataset.bulkSelection as ModelBulkSelectionAction | undefined;
      if (!action) {
        return;
      }

      if (action === "clear") {
        plannerDraft.selectedModelIds = [];
      } else if (action === "recommended") {
        const ids = Array.from(
          new Set((currentState.buildPlan?.hostRamRecommendations ?? []).map((profile) => profile.suggestedModel.modelId)),
        );
        plannerDraft.selectedModelIds = ids;
      } else if (action === "visible-local") {
        const ids = filteredCatalogModels(currentState)
          .filter((model) => model.availableLocally)
          .map((model) => model.modelId);
        plannerDraft.selectedModelIds = Array.from(new Set(ids));
      }

      activeCreateStep = plannerDraft.selectedModelIds.length ? "review" : activeCreateStep;
      void refreshBuildPlan();
    });
  });

  libraryFilterButtons.forEach((button) => {
    button.addEventListener("click", () => {
      const nextFilter = button.dataset.libraryFilter as LibraryViewDraft["filter"] | undefined;
      if (!nextFilter || nextFilter === libraryViewDraft.filter) {
        return;
      }
      libraryViewDraft.filter = nextFilter;
      render(currentState);
    });
  });

  targetCards.forEach((card) => {
    const selectTarget = () => {
      const mountPath = card.dataset.targetMount;
      if (!mountPath) {
        return;
      }
      plannerDraft.selectedTargetMountPath = mountPath;
      activeCreateStep = "profile";
      void Promise.all([refreshBuildPlan(), refreshModelCatalog()]);
    };

    card.addEventListener("click", selectTarget);
    card.addEventListener("keydown", (event) => {
      if (event.key !== "Enter" && event.key !== " ") {
        return;
      }
      event.preventDefault();
      selectTarget();
    });
  });

  targetSelect?.addEventListener("change", () => {
    plannerDraft.selectedTargetMountPath = targetSelect.value || null;
    if (plannerDraft.selectedTargetMountPath) {
      activeCreateStep = "profile";
    }
    void Promise.all([refreshBuildPlan(), refreshModelCatalog()]);
  });

  buildModeSelect?.addEventListener("change", () => {
    plannerDraft.buildMode = buildModeSelect.value as BuildMode;
    void refreshBuildPlan();
  });

  creatorLabelInput?.addEventListener("input", () => {
    plannerDraft.creatorLabel = creatorLabelInput.value;
    void refreshBuildPlan();
  });

  allowGpuInput?.addEventListener("change", () => {
    plannerDraft.allowGpu = allowGpuInput.checked;
    void refreshBuildPlan();
  });

  allowCloudInput?.addEventListener("change", () => {
    plannerDraft.allowCloudFallback = allowCloudInput.checked;
    void refreshBuildPlan();
  });

  catalogSearchInput?.addEventListener("input", () => {
    activeModelSelectionMode = "manual";
    catalogViewDraft.search = catalogSearchInput.value;
    render(currentState);
  });

  catalogDomainSelect?.addEventListener("change", () => {
    activeModelSelectionMode = "manual";
    catalogViewDraft.domain = catalogDomainSelect.value;
    render(currentState);
  });

  catalogLanguageSelect?.addEventListener("change", () => {
    activeModelSelectionMode = "manual";
    catalogViewDraft.language = catalogLanguageSelect.value;
    render(currentState);
  });

  catalogHostRamSelect?.addEventListener("change", () => {
    activeModelSelectionMode = "manual";
    catalogViewDraft.hostRam = catalogHostRamSelect.value as CatalogViewDraft["hostRam"];
    render(currentState);
  });

  catalogBenchmarkConfidenceSelect?.addEventListener("change", () => {
    activeModelSelectionMode = "manual";
    catalogViewDraft.benchmarkConfidence =
      catalogBenchmarkConfidenceSelect.value as CatalogViewDraft["benchmarkConfidence"];
    render(currentState);
  });

  catalogCpuOnlyInput?.addEventListener("change", () => {
    activeModelSelectionMode = "manual";
    catalogViewDraft.cpuOnlyOnly = catalogCpuOnlyInput.checked;
    render(currentState);
  });

  librarySearchInput?.addEventListener("input", () => {
    libraryViewDraft.search = librarySearchInput.value;
    render(currentState);
  });

  catalogModelToggles.forEach((input) => {
    input.addEventListener("change", () => {
      const modelId = input.dataset.modelId;
      if (!modelId) {
        return;
      }

      if (input.checked) {
        plannerDraft.selectedModelIds = Array.from(new Set([...plannerDraft.selectedModelIds, modelId]));
      } else {
        plannerDraft.selectedModelIds = plannerDraft.selectedModelIds.filter((selectedId) => selectedId !== modelId);
      }

      if (activeMainView === "models" && plannerDraft.selectedModelIds.length > 0) {
        activeCreateStep = "review";
      }

      void refreshBuildPlan();
    });
  });
  const catalogCards = Array.from(document.querySelectorAll<HTMLElement>(".catalog-card"));
  catalogCards.forEach((card) => {
    const checkbox = card.querySelector<HTMLInputElement>(".catalog-model-toggle");
    if (!checkbox) {
      return;
    }

    card.addEventListener("click", (event) => {
      const target = event.target;
      if (!(target instanceof HTMLElement)) {
        return;
      }
      if (target.closest(".catalog-card-header")) {
        return;
      }
      checkbox.click();
    });

    card.addEventListener("keydown", (event) => {
      if (event.key !== "Enter" && event.key !== " ") {
        return;
      }
      event.preventDefault();
      checkbox.click();
    });
  });

  languageInputs.forEach((input) => {
    input.addEventListener("change", () => {
      const selectedLanguages = languageInputs.filter((item) => item.checked).map((item) => item.value);
      if (!selectedLanguages.length) {
        input.checked = true;
        plannerDraft.languages = [input.value];
      } else {
        plannerDraft.languages = selectedLanguages;
      }

      void refreshBuildPlan();
    });
  });

  const manifestButtons = Array.from(document.querySelectorAll<HTMLButtonElement>("[data-manifest-file]"));
  manifestButtons.forEach((button) => {
    button.addEventListener("click", () => {
      selectedManifestFileName = button.dataset.manifestFile || "build-manifest.json";
      const manifest = getSelectedManifest(currentState.buildPlan);
      currentState = {
        ...currentState,
        manifestPreview: manifest?.contents ?? currentState.manifestPreview,
      };
      render(currentState);
    });
  });

  const exportButton = document.querySelector<HTMLButtonElement>("#export-build-plan");
  exportButton?.addEventListener("click", () => {
    void exportBuildPlan();
  });

  const refreshLiveCatalogButton = document.querySelector<HTMLButtonElement>("#refresh-live-catalog");
  refreshLiveCatalogButton?.addEventListener("click", () => {
    void refreshLiveCatalog();
  });

  const refreshBuilderLibraryButton = document.querySelector<HTMLButtonElement>("#refresh-builder-library");
  refreshBuilderLibraryButton?.addEventListener("click", () => {
    void refreshBuilderLibrary();
  });

  const refreshTargetsButton = document.querySelector<HTMLButtonElement>("#refresh-targets");
  refreshTargetsButton?.addEventListener("click", () => {
    void refreshTargets();
  });

  const deleteLibraryArtifactButtons = Array.from(
    document.querySelectorAll<HTMLButtonElement>(".library-delete-artifact"),
  );
  deleteLibraryArtifactButtons.forEach((button) => {
    button.addEventListener("click", () => {
      const artifactId = button.dataset.artifactId;
      const artifactType = button.dataset.artifactType;
      if (!artifactId || !artifactType) {
        return;
      }
      void deleteBuilderLibraryArtifact({
        artifactId,
        artifactType,
      });
    });
  });

  const downloadLibraryArtifactButtons = Array.from(
    document.querySelectorAll<HTMLButtonElement>(".library-download-artifact"),
  );
  downloadLibraryArtifactButtons.forEach((button) => {
    button.addEventListener("click", () => {
      const artifactId = button.dataset.artifactId;
      if (!artifactId) {
        return;
      }
      void downloadBuilderLibraryArtifact({
        artifactId,
      });
    });
  });

  const catalogDownloadButtons = Array.from(
    document.querySelectorAll<HTMLButtonElement>(".catalog-download-model"),
  );
  catalogDownloadButtons.forEach((button) => {
    button.addEventListener("click", () => {
      const modelId = button.dataset.modelId;
      if (!modelId) {
        return;
      }
      void downloadCatalogModel({
        modelId,
      });
    });
  });

  const prepareBuilderLibraryButton = document.querySelector<HTMLButtonElement>("#prepare-builder-library");
  prepareBuilderLibraryButton?.addEventListener("click", () => {
    void prepareBuilderLibrary();
  });

  const createUsbrainButton = document.querySelector<HTMLButtonElement>("#create-usbrain");
  createUsbrainButton?.addEventListener("click", () => {
    void assembleBuildPlan();
  });

  const authorizeTargetAccessButton = document.querySelector<HTMLButtonElement>("#authorize-target-access");
  authorizeTargetAccessButton?.addEventListener("click", () => {
    void authorizeSelectedTargetAccess();
  });

  const confirmWipeCreateButton = document.querySelector<HTMLButtonElement>("#confirm-wipe-create");
  confirmWipeCreateButton?.addEventListener("click", () => {
    void assembleBuildPlan(true);
  });

  const cancelWipeCreateButton = document.querySelector<HTMLButtonElement>("#cancel-wipe-create");
  cancelWipeCreateButton?.addEventListener("click", () => {
    currentState = {
      ...currentState,
      pendingWipeConfirmation: null,
      assemblyError: "Criacao cancelada. O pendrive nao foi apagado.",
      assemblyProgress: {
        status: "idle",
        percent: 0,
        message: "Aguardando a criacao do USBRAIN.",
      },
    };
    render(currentState);
  });

  const assembleButton = document.querySelector<HTMLButtonElement>("#assemble-build-plan");
  assembleButton?.addEventListener("click", () => {
    void assembleBuildPlan();
  });

  const validateButton = document.querySelector<HTMLButtonElement>("#validate-assembly");
  validateButton?.addEventListener("click", () => {
    void validateAssembly();
  });

  const smokeButton = document.querySelector<HTMLButtonElement>("#run-smoke-test");
  smokeButton?.addEventListener("click", () => {
    void runSmokeTest();
  });

  const runtimeProbeButton = document.querySelector<HTMLButtonElement>("#run-runtime-probe");
  runtimeProbeButton?.addEventListener("click", () => {
    void runRuntimeProbe();
  });

  const endToEndProbeButton = document.querySelector<HTMLButtonElement>("#run-e2e-probe");
  endToEndProbeButton?.addEventListener("click", () => {
    void runEndToEndProbe();
  });

  const fullLocalLlmProbeButton = document.querySelector<HTMLButtonElement>("#run-full-llm-probe");
  fullLocalLlmProbeButton?.addEventListener("click", () => {
    void runFullLocalLlmProbe();
  });

  const readinessReportButton = document.querySelector<HTMLButtonElement>("#generate-readiness-report");
  readinessReportButton?.addEventListener("click", () => {
    void generateReadinessReport();
  });

  const distributionButton = document.querySelector<HTMLButtonElement>("#prepare-distribution-bundle");
  distributionButton?.addEventListener("click", () => {
    void prepareDistributionBundle();
  });

  const releasePackageButton = document.querySelector<HTMLButtonElement>("#prepare-release-package");
  releasePackageButton?.addEventListener("click", () => {
    void prepareReleasePackage();
  });
}

async function bootstrap(): Promise<void> {
  const fallbackOverview = defaultState.overview;
  const fallbackSchemas = defaultState.schemas;
  const fallbackSnapshot = defaultState.snapshot;
  const fallbackTargets = defaultState.targets;
  const fallbackManifestPreview = defaultState.manifestPreview;

  const [overview, schemas, snapshot, targets] = await Promise.all([
    invokeSafe<AppOverview>("get_app_overview", fallbackOverview),
    invokeSafe<SchemaSummary[]>("list_schemas", fallbackSchemas),
    invokeSafe<WorkspaceSnapshot>("get_workspace_snapshot", fallbackSnapshot),
    invokeSafe<CandidateTarget[]>("list_candidate_targets", fallbackTargets),
  ]);

  plannerDraft = {
    ...plannerDraft,
    selectedTargetMountPath: getSelectedTarget(targets, plannerDraft.selectedTargetMountPath)?.mountPath ?? null,
  };

  const buildPlan = await invokeSafe<BuildPlanResponse>(
    "generate_build_plan",
    buildFallbackBuildPlan(targets),
    captureResolvedBuildRequest(targets),
  );
  const fallbackLibraryInventory: BuilderLibraryInventory = {
    libraryRoot: "biblioteca indisponivel",
    artifactCount: 0,
    totalSizeBytes: 0,
    artifacts: [],
  };
  let modelCatalog: ModelCatalogResponse | null = null;
  let catalogError: string | null = null;
  try {
    modelCatalog = await invoke<ModelCatalogResponse>("get_model_catalog", buildModelCatalogRequest(targets));
  } catch {
    catalogError =
      "Nenhum catalogo real esta disponivel no bootstrap. Atualize o catalogo vivo a partir de uma fonte remota verificavel.";
  }
  const libraryInventory = await invokeSafe<BuilderLibraryInventory>("list_builder_library", fallbackLibraryInventory);
  const manifestPreview = getSelectedManifest(buildPlan)?.contents ?? fallbackManifestPreview;

  const mode: BuilderState["mode"] = overview.tauriBackendAvailable ? "tauri" : "web";
  const error =
    mode === "web"
      ? "O frontend esta funcional, mas o backend Rust/Tauri ainda nao foi executado neste ambiente porque a toolchain Rust nao esta instalada localmente."
      : null;

  currentState = {
    overview,
    schemas,
    snapshot,
    targets,
    isRefreshingTargets: false,
    targetRefreshError: null,
    modelCatalog,
    catalogError,
    catalogRefreshResult: null,
    isRefreshingCatalog: false,
    catalogRefreshError: null,
    manifestPreview,
    buildPlan,
    libraryPreparationResult: null,
    isPreparingLibrary: false,
    libraryPreparationError: null,
    libraryInventory,
    isRefreshingLibrary: false,
    isDeletingLibraryArtifact: false,
    isDownloadingLibraryArtifact: false,
    libraryDownloadResult: null,
    libraryDownloadError: null,
    libraryInventoryError:
      libraryInventory.libraryRoot === "biblioteca indisponivel"
        ? "O backend Tauri nao respondeu para listar a biblioteca local do Builder."
        : null,
    exportResult: null,
    isExporting: false,
    exportError: null,
    assemblyResult: null,
    isAssembling: false,
    assemblyError: null,
    authorizedTargetMountPaths: [],
    isAuthorizingTargetAccess: false,
    targetAuthorizationError: null,
    pendingWipeConfirmation: null,
    assemblyProgress: {
      status: "idle",
      percent: 0,
      message: "Aguardando a criacao do USBRAIN.",
    },
    validationResult: null,
    isValidating: false,
    validationError: null,
    smokeTestResult: null,
    isRunningSmokeTest: false,
    smokeTestError: null,
    runtimeProbeResult: null,
    isRunningRuntimeProbe: false,
    runtimeProbeError: null,
    endToEndProbeResult: null,
    isRunningEndToEndProbe: false,
    endToEndProbeError: null,
    fullLocalLlmProbeResult: null,
    isRunningFullLocalLlmProbe: false,
    fullLocalLlmProbeError: null,
    readinessReport: null,
    isGeneratingReadinessReport: false,
    readinessReportError: null,
    distributionResult: null,
    isPreparingDistribution: false,
    distributionError: null,
    releasePackageResult: null,
    isPreparingReleasePackage: false,
    releasePackageError: null,
    mode,
    error,
  };
  await ensureAssemblyProgressListener();
  render(currentState);
  window.addEventListener("focus", () => {
    if (!currentState.isRefreshingTargets && !currentState.isAssembling) {
      void refreshTargets();
    }
  });

  if (mode === "tauri" && factoryQaTarget) {
    plannerDraft = {
      ...plannerDraft,
      selectedTargetMountPath: factoryQaTarget,
    };

    const refreshedBuildPlan = await invokeSafe<BuildPlanResponse>(
      "generate_build_plan",
      buildFallbackBuildPlan(targets),
      captureResolvedBuildRequest(targets),
    );
    currentState = {
      ...currentState,
      buildPlan: refreshedBuildPlan,
      manifestPreview: getSelectedManifest(refreshedBuildPlan)?.contents ?? fallbackManifestPreview,
      assemblyError: null,
    };
    render(currentState);

    const pipelineResult = await invoke<FactoryPipelineResult>(
      "run_factory_pipeline",
      resolvedBuildRequest(targets, { wipeTargetIfNeeded: true }),
    );
    currentState = {
      ...currentState,
      assemblyResult: pipelineResult.assemblyResult,
      assemblyError: pipelineResult.assemblyError,
      validationResult: pipelineResult.validationResult,
      smokeTestResult: pipelineResult.smokeTestResult,
      fullLocalLlmProbeResult: pipelineResult.fullLocalLlmProbeResult,
      assemblyProgress: {
        status: pipelineResult.overallStatus === "failed" ? "error" : "completed",
        percent: 100,
        message:
          pipelineResult.overallStatus === "failed"
            ? "Factory QA falhou. Revise o relatorio gerado pelo backend."
            : pipelineResult.overallStatus === "passed-with-warnings"
              ? "Factory QA concluiu com alertas. Revise o relatorio gerado pelo backend."
              : "Factory QA concluiu com sucesso.",
      },
    };
    render(currentState);
    console.log(`FACTORY_QA_REPORT=${pipelineResult.reportPath}`);
    console.log(`FACTORY_QA_STATUS=${pipelineResult.overallStatus}`);

    if (pipelineResult.overallStatus !== "failed") {
      await ejectMountedTargetWithConfirmation(pipelineResult.targetRoot);
    }
  }
}

void bootstrap();
