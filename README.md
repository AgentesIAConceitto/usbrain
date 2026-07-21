# USBRAIN

USBRAIN e uma plataforma open source para criar agentes pessoais portateis em pendrive, com prioridade para uso offline, CPU-only e distribuicao simples para pessoas nao tecnicas.

O projeto esta evoluindo em duas frentes integradas:

- `USBRAIN Runtime`: executa o agente a partir do USB, incluindo interface web, busca semantica e selecao adaptativa de modelo local.
- `USBRAIN Builder`: aplicacao desktop em `Tauri + Rust` que planeja, monta, valida e prepara a distribuicao final de um USBRAIN publicavel.

## Visao do produto

- analisa o pendrive disponivel e estima o perfil de build suportado;
- recomenda modelos locais `.gguf` conforme espaco e RAM alvo;
- suporta multiplas LLMs no mesmo USB com selecao em runtime;
- incorpora `BGE-M3`, RAG, packs especialistas e configuracao de runtime;
- opera em `pt-BR`, `en` e `es`;
- prioriza modo `offline-first`, com GPU e provedores externos como opcoes explicitas.

## Estado atual

O repositorio ja contem uma base funcional real do ecossistema:

- Builder com frontend em `Vite + TypeScript` e backend em `Rust`;
- contratos em `JSON Schema` para interoperabilidade Builder <-> Runtime;
- planejamento de build com manifestos gerados;
- montagem do USBRAIN no destino selecionado;
- validacao estrutural, smoke test, runtime probe, probe ponta a ponta e probe com LLM local;
- gate consolidado de prontidao para distribuicao;
- pacote de release com `SHA256`, resumo para GitHub e resumo para `usbrain.org`.

## Fluxo do Builder

1. Detectar um destino compativel.
2. Gerar o plano de build e os manifestos.
3. Montar o USBRAIN no destino.
4. Validar estrutura e runtime.
5. Rodar o gate de distribuicao.
6. Preparar o pacote de release publicavel.

## Estrutura principal

- `builder/`: GUI desktop do USBRAIN Builder.
- `docs/`: documentacao publica inicial para manifesto, visao do ecossistema e checklist de release.
- `schemas/`: contratos oficiais do ecossistema.
- `cache/`: artefatos locais de validacao e modelos de suporte.
- `rag/`, `webui/`, `agent.py` e scripts legados: base do runtime e da validacao inicial.
- documentacao raiz `*.md`: especificacao funcional, arquitetura, formatos e roadmap.

## Comece por aqui

- `docs/README.md`
- `docs/MANIFESTO_USBRAIN.md`
- `docs/ECOSSISTEMA_USBRAIN.md`
- `docs/CHECKLIST_RELEASE_PUBLICA.md`
- `docs/RELEASE_0_1_0_GITHUB.md`
- `docs/GITHUB_REPOSITORY_PROFILE.md`

## Desenvolvimento local

### Builder

```bash
cd builder
npm install
npm run build
CI=true npm run tauri build -- --debug
```

### Backend do Builder

```bash
cd builder/src-tauri
cargo test
```

## Publicacao e release

O Builder agora gera um pacote de release com estes artefatos:

- `readiness-report.json`
- `distribution-summary.json`
- `SHA256SUMS.txt`
- `github-release-notes.md`
- `usbrain-release-summary.json`
- `release-package.json`

Esses arquivos permitem publicar o estado do build no GitHub e reaproveitar o mesmo bundle em `usbrain.org`, sem perder a trilha de validacao.

## Open Source

- licenca: `MIT`
- foco inicial: `Windows` e `Linux`, com validacao atual tambem em `macOS`
- homepage do ecossistema: [usbrain.org](https://usbrain.org)
- fluxo de contribuicao: consulte `CONTRIBUTING.md`
- conduta da comunidade: consulte `CODE_OF_CONDUCT.md`
- reporte responsavel de seguranca: consulte `SECURITY.md`

## Licenca

Este projeto esta licenciado sob a licenca `MIT`. Consulte `LICENSE`.
