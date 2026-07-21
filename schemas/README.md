# USBRAIN Schemas

Este diretorio contem os primeiros `JSON Schemas` reais dos contratos entre o `USBRAIN Builder` e o `USBRAIN Runtime`.

## Objetivo

Esses arquivos transformam os contratos conceituais do projeto em artefatos tecnicos validaveis.

Eles existem para:

- padronizar o que o `Builder` grava;
- padronizar o que o `Runtime` le;
- reduzir ambiguidade entre implementacoes;
- preparar validacao automatica;
- servir de base para evolucao versionada dos contratos.

## Schemas Iniciais

- `build-manifest.schema.json`
  - contrato raiz do build
  - primeira leitura do runtime

- `usbrain-profile.schema.json`
  - identidade e intencao principal do USBRAIN criado

- `execution-policy.schema.json`
  - regras operacionais que limitam e orientam o runtime

- `installed-models.schema.json`
  - lista de modelos locais instalados e seus metadados minimos

- `runtime-config.schema.json`
  - defaults operacionais do runtime

- `validation-report.schema.json`
  - resumo da validacao executada pelo Builder

- `installed-packs.schema.json`
  - lista de packs instalados e ativaveis pelo runtime

- `provider-config.schema.json`
  - configuracao opcional de provedores externos e suas politicas

## Ordem Recomendada de Uso

1. validar `build-manifest.json`
2. resolver referencias para os demais contratos
3. validar `usbrain-profile.json`
4. validar `execution-policy.json`
5. validar `installed-models.json`
6. validar `installed-packs.json`, quando existir
7. validar `runtime-config.json`
8. validar `provider-config.json`, quando existir
9. usar `validation-report.json` como apoio diagnostico

## Proximos Schemas Naturais

- `pack-manifest` harmonizado com o formato oficial dos packs
- schemas concretos de exemplos de manifestos gerados pelo Builder

## Relacao com a Documentacao

Esses schemas derivam de:

- [CONTRATOS_BUILDER_RUNTIME.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/CONTRATOS_BUILDER_RUNTIME.md)
- [BACKLOG_INICIAL_BUILDER_TAURI_RUST.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/BACKLOG_INICIAL_BUILDER_TAURI_RUST.md)
- [BACKLOG_INICIAL_RUNTIME.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/BACKLOG_INICIAL_RUNTIME.md)
