# Checklist da Primeira Release Publica

Este checklist existe para orientar a abertura publica do repositorio e da primeira release do USBRAIN Builder.

## 1. Repositorio

- [x] `README.md` atualizado
- [x] `LICENSE` definida
- [x] `CHANGELOG.md` inicial criado
- [x] `CONTRIBUTING.md` adicionado
- [x] `CODE_OF_CONDUCT.md` adicionado
- [x] `SECURITY.md` adicionado
- [x] `.github/` com templates de issue e PR
- [x] `.gitignore` raiz criado

## 2. Produto

- [x] Builder com GUI desktop funcional
- [x] build plan e manifestos gerados
- [x] montagem do USBRAIN no destino
- [x] validacao estrutural
- [x] smoke test
- [x] runtime probe
- [x] probe E2E
- [x] probe com LLM local
- [x] readiness report
- [x] distribution gate
- [x] release package com checksums

## 3. Validacao tecnica

- [x] `npm test`
- [x] `cargo test`
- [x] `npm run build`
- [x] `CI=true npm run tauri build -- --debug`
- [x] smoke test automatizado da UI do Builder
- [ ] smoke test manual da UX do Builder antes da publicacao
- [ ] revisao final dos textos publicos do repositorio

## 4. Artefatos de release

- [x] `readiness-report.json`
- [x] `distribution-summary.json`
- [x] `SHA256SUMS.txt`
- [x] `github-release-notes.md`
- [x] `usbrain-release-summary.json`
- [x] `release-package.json`

## 5. GitHub

- [ ] criar repositorio publico oficial
- [ ] configurar descricao, topics e homepage
- [ ] publicar primeira tag
- [ ] anexar bundle do Builder na release
- [ ] colar `github-release-notes.md` na descricao da release

## 6. usbrain.org

- [ ] publicar pagina inicial do ecossistema
- [ ] apontar para repositorio oficial
- [ ] apontar para a release inicial do Builder
- [ ] apontar para a documentacao publica

## 7. Pos-release

- [ ] abrir backlog publico inicial
- [ ] priorizar autoria de PACKs dentro do Builder
- [ ] iniciar camada viva de catalogo de modelos
- [ ] preparar exemplos oficiais de PACKs
