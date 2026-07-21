# USBRAIN 0.1.0

Primeira release publica da base do ecossistema USBRAIN.

## Resumo

O `USBRAIN` entra em sua primeira release publica com uma base funcional real do `USBRAIN Builder`, construida em `Tauri + Rust`, capaz de:

- analisar destinos de build;
- gerar manifestos e build plans;
- montar o USBRAIN no destino selecionado;
- validar estrutura e runtime;
- executar smoke test, runtime probe, probe E2E e probe com LLM local;
- consolidar readiness report e distribution gate;
- preparar pacote de release com checksums e artefatos publicaveis.

## Destaques desta release

- GUI desktop real para o `USBRAIN Builder`
- pipeline funcional de criacao, validacao e distribuicao
- artefatos de release com `SHA256SUMS.txt`
- base documental publica para abertura no GitHub
- governanca inicial com contribuicao, seguranca e conduta

## Escopo atual

Esta release representa a fundacao publica do projeto, nao o ecossistema completo final.

O foco deste marco e abrir o USBRAIN como plataforma coerente e executavel, com base suficiente para evoluir:

- Builder
- Runtime
- PACKs
- Hub
- Catalogo Vivo de Modelos

## Validacao executada

- `cargo test`
- `npm run build`
- `CI=true npm run tauri build -- --debug`

## Artefatos esperados

- bundle desktop do `USBRAIN Builder`
- `readiness-report.json`
- `distribution-summary.json`
- `SHA256SUMS.txt`
- `github-release-notes.md`
- `usbrain-release-summary.json`
- `release-package.json`

## Proximos passos

- autoria de PACKs dentro do Builder
- camada viva de catalogo de modelos
- exemplos oficiais de PACKs
- integracao progressiva com `usbrain.org`

## Links recomendados

- `README.md`
- `docs/MANIFESTO_USBRAIN.md`
- `docs/ECOSSISTEMA_USBRAIN.md`
- `docs/CHECKLIST_RELEASE_PUBLICA.md`
