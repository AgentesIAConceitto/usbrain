import fs from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, "..", "..");
const catalogSourcesPath = path.join(repoRoot, "catalog", "catalog-sources.json");

function fail(message) {
  console.error(`[real-catalog-validation] ${message}`);
  process.exit(1);
}

function isForbiddenSource(url) {
  return /example\.com|localhost|127\.0\.0\.1|mock|dummy|bundled-snapshot/i.test(url);
}

function isHuggingFaceModelsApi(url) {
  return /^https:\/\/huggingface\.co\/api\/models/i.test(url);
}

async function readConfiguredSources() {
  const contents = await fs.readFile(catalogSourcesPath, "utf8");
  const parsed = JSON.parse(contents);
  const fileSources = Array.isArray(parsed.sources)
    ? parsed.sources
        .map((entry) => String(entry?.snapshot_url ?? "").trim())
        .filter(Boolean)
    : [];
  const envSource = String(process.env.USBRAIN_MODEL_CATALOG_URL ?? "").trim();

  return [...new Set([envSource, ...fileSources].filter(Boolean))];
}

async function validateRemoteSnapshot(url) {
  if (!/^https:\/\//i.test(url)) {
    fail(`A fonte ${url} nao usa HTTPS.`);
  }
  if (isForbiddenSource(url)) {
    fail(`A fonte ${url} usa um endereco proibido para validacao oficial.`);
  }

  const headers = {
    Accept: "application/json",
    "User-Agent": "USBRAIN-real-catalog-validation/1.0",
  };
  const token = String(
    process.env.USBRAIN_HUGGINGFACE_API_KEY ??
      process.env.HUGGINGFACE_API_KEY ??
      process.env.HUGGINGFACE_HUB_TOKEN ??
      process.env.HF_TOKEN ??
      "",
  ).trim();
  if (token && isHuggingFaceModelsApi(url)) {
    headers.Authorization = `Bearer ${token}`;
  }

  const response = await fetch(url, {
    headers: {
      ...headers,
    },
  });

  if (!response.ok) {
    fail(`A fonte ${url} respondeu com HTTP ${response.status}.`);
  }

  const text = await response.text();
  if (!text.trim()) {
    fail(`A fonte ${url} respondeu vazia.`);
  }

  let snapshot;
  try {
    snapshot = JSON.parse(text);
  } catch (error) {
    fail(`A fonte ${url} nao retornou JSON valido: ${error}`);
  }

  if (/example\.com|mock|dummy/i.test(text)) {
    fail(`A fonte ${url} ainda contem indicios de dados artificiais.`);
  }

  if (Array.isArray(snapshot)) {
    const usableModels = snapshot.filter(
      (item) =>
        (typeof item?.id === "string" || typeof item?.modelId === "string") &&
        item?.private !== true &&
        item?.gated !== true,
    );
    if (!usableModels.length) {
      fail(`A fonte ${url} nao trouxe resultados reais utilizaveis do Hub.`);
    }
    console.log(`[real-catalog-validation] OK ${url} -> ${usableModels.length} repositorios reais do Hub`);
    return;
  }

  if (String(snapshot?.source ?? "").trim() === "bundled-snapshot") {
    fail(`A fonte ${url} ainda aponta para bundled-snapshot, o que viola a regra de dados reais.`);
  }

  if (!Array.isArray(snapshot?.models) || snapshot.models.length === 0) {
    fail(`A fonte ${url} nao trouxe modelos reais utilizaveis.`);
  }

  console.log(
    `[real-catalog-validation] OK ${url} -> ${snapshot.models.length} modelos, versao ${snapshot.catalogVersion ?? "sem-versao"}`,
  );
}

async function main() {
  const sources = await readConfiguredSources();
  if (!sources.length) {
    fail(
      "Nenhuma fonte real de catalogo foi configurada. Defina `USBRAIN_MODEL_CATALOG_URL` ou preencha `catalog/catalog-sources.json` com uma URL HTTPS verificavel.",
    );
  }

  for (const source of sources) {
    await validateRemoteSnapshot(source);
  }
}

await main();
