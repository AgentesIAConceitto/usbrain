# USBRAIN Builder

Aplicacao desktop real do `USBRAIN Builder` em `Tauri + Rust`.

## O que ja faz

- detecta destinos candidatos para montar um USBRAIN;
- gera plano de build e manifestos contratuais;
- exporta e monta o runtime no destino selecionado;
- valida a estrutura montada;
- executa smoke test, runtime probe, probe E2E e probe com LLM local;
- consolida um readiness report;
- prepara distribuicao final e pacote de release publicavel.

## Artefatos de release

Quando o fluxo final de release e executado, o Builder gera:

- `readiness-report.json`
- `distribution-summary.json`
- `SHA256SUMS.txt`
- `github-release-notes.md`
- `usbrain-release-summary.json`
- `release-package.json`

## Estrutura

- `src/`
  - frontend do Builder
- `src-tauri/`
  - backend Rust/Tauri

## Comandos uteis

```bash
npm install
npm test
npm run build
CI=true npm run tauri build -- --debug
```

```bash
cd src-tauri
cargo test
```

## Observacao importante

Em `macOS`, o bundle `.dmg` deve ser gerado com `CI=true` para evitar timeout no `Finder` durante o empacotamento automatizado.
