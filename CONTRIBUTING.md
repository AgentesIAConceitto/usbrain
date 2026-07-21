# Contributing to USBRAIN

Obrigado por considerar contribuir com o USBRAIN.

Este projeto esta sendo construido como uma plataforma open source para criacao de agentes pessoais portateis, com foco em qualidade real de produto, interoperabilidade e experiencia acessivel para usuarios nao tecnicos.

## Principios

- priorize entregas reais, nao simulacoes que degradem a funcionalidade principal;
- preserve a proposta `offline-first` e `CPU-only` como base do produto;
- mantenha a GUI do Builder simples, moderna e adequada para usuarios nao tecnicos;
- respeite os contratos entre Builder e Runtime, especialmente os schemas em `schemas/`;
- prefira mudancas pequenas, testaveis e com motivacao clara.

## Antes de contribuir

1. Leia `README.md` para entender o estado atual do projeto.
2. Revise os documentos de arquitetura e contratos na raiz do repositorio.
3. Verifique se ja existe issue aberta para o problema, proposta ou pack relacionado.
4. Se a mudanca for grande, abra uma issue antes do pull request.

## Tipos de contribuicao bem-vindos

- correcoes de bugs no Builder, Runtime e fluxo de montagem;
- melhorias na UX da GUI desktop;
- refinamentos de validacao, probes, gates e release package;
- novos schemas, contratos ou documentacao tecnica;
- melhorias em suporte multiplataforma;
- propostas para PACKs especialistas e catalogo de modelos.

## Como desenvolver localmente

### Builder

```bash
cd builder
npm install
npm run build
CI=true npm run tauri build -- --debug
```

### Testes do backend

```bash
cd builder/src-tauri
cargo test
```

## Diretrizes para pull requests

- descreva o problema e o motivo da mudanca;
- explique impacto em Builder, Runtime, schemas ou release;
- inclua validacao executada localmente;
- atualize documentacao quando a mudanca alterar UX, fluxo ou contratos;
- evite misturar refatoracao ampla com mudanca funcional sem necessidade.

## Convencoes tecnicas

- `builder/` contem a GUI desktop em `Tauri + Rust + TypeScript`;
- `schemas/` contem contratos oficiais do ecossistema;
- arquivos de release devem continuar coerentes com o gate de distribuicao;
- artefatos gerados localmente nao devem ser commitados.

## Comunidade

Ao participar deste projeto, voce concorda em seguir `CODE_OF_CONDUCT.md`.
