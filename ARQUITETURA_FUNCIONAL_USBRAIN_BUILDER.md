# USBRAIN Builder — Arquitetura Funcional

## 1. Objetivo

Este documento define a arquitetura funcional do `USBRAIN Builder` assumindo a decisao tecnologica:

- `GUI desktop obrigatoria`
- `Tauri + Rust`
- suporte inicial a `Windows` e `Linux`
- interface nativa em `Portugues`, `Ingles` e `Espanhol`

O foco deste documento e responder:

- como o Builder deve ser organizado internamente;
- como a interface conversa com o nucleo da aplicacao;
- quais modulos formam o Builder;
- quais fluxos operacionais sao essenciais;
- como o produto pode crescer sem perder robustez.

Este documento nao descreve ainda detalhes de codigo ou estrutura final de pastas, mas define o comportamento arquitetural esperado.

---

## 2. Decisao Tecnologica

O `USBRAIN Builder` sera uma aplicacao desktop baseada em:

- `Tauri` para shell desktop e ponte entre interface e backend
- `Rust` para o nucleo funcional da aplicacao
- frontend web para a camada visual da GUI

### Motivos da escolha

- necessidade de `executavel real`, e nao ferramenta centrada em terminal;
- necessidade de GUI simples, bonita e profissional;
- necessidade de robustez e previsibilidade de longo prazo;
- necessidade de pouca dependencia externa em runtime;
- necessidade de boa distribuicao em `Windows` e `Linux`;
- compatibilidade com o objetivo de publicacao open source no `GitHub`.

---

## 3. Principios Arquiteturais do Builder

- `GUI first`
- `Core robusto em Rust`
- `Separacao entre interface e logica`
- `Operacoes longas desacopladas da interface`
- `Manifestos e formatos abertos`
- `Internacionalizacao nativa`
- `Planejamento antes da gravacao`
- `Validacao antes, durante e depois`
- `Offline-first com atualizacao online opcional`

---

## 4. Visao Geral da Arquitetura

O Builder sera dividido em tres camadas principais:

- `Presentation Layer`
- `Application Layer`
- `Core Domain and Infrastructure Layer`

### Representacao simplificada

```text
┌────────────────────────────────────────────────────────────┐
│                  USBRAIN Builder UI (Tauri)               │
│  telas, formularios, recomendacoes, progresso, i18n       │
└───────────────────────┬────────────────────────────────────┘
                        │ comandos / eventos
                        v
┌────────────────────────────────────────────────────────────┐
│                 Application Services Layer                │
│  orchestration, workflow, state, validation, jobs         │
└───────────────────────┬────────────────────────────────────┘
                        │ usa
                        v
┌────────────────────────────────────────────────────────────┐
│              Core Domain and Infrastructure               │
│  device scan, model catalog, packs, build, manifests,     │
│  downloads, filesystem, secrets, validation               │
└────────────────────────────────────────────────────────────┘
```

---

## 5. Camadas do Sistema

## 5.1 Presentation Layer

Responsavel por toda a experiencia do usuario.

### Responsabilidades

- renderizar a interface;
- exibir pendrives detectados;
- exibir modelos recomendados;
- exibir packs e categorias;
- coletar escolhas do usuario;
- mostrar progresso de downloads e montagem;
- mostrar validacoes, alertas e erros;
- trocar idioma da interface;
- refletir o estado do workflow sem bloquear a GUI.

### O que nao deve fazer

- nao deve conter regra de negocio central;
- nao deve manipular arquivos diretamente;
- nao deve decidir qual modelo cabe no pendrive;
- nao deve conter logica de install planner;
- nao deve conhecer detalhes internos de manifestos e runtime.

### Resultado esperado

Uma GUI simples, bonita e objetiva, com sensacao de produto pronto e nao de ferramenta tecnica improvisada.

---

## 5.2 Application Services Layer

Camada de orquestracao entre a interface e o core.

### Responsabilidades

- receber comandos da interface;
- disparar workflows;
- organizar estado de sessao;
- coordenar chamadas aos modulos de dominio;
- controlar tarefas longas;
- publicar eventos de progresso para a interface;
- transformar erros tecnicos em mensagens adequadas ao usuario.

### Tipos de responsabilidade

- `workflow orchestration`
- `session state`
- `job scheduling`
- `progress reporting`
- `error mapping`
- `localization handoff`

### Exemplos de services

- `DeviceDiscoveryService`
- `BuildPlanningService`
- `ModelRecommendationService`
- `PackSelectionService`
- `BuildExecutionService`
- `ValidationService`
- `CatalogUpdateService`
- `SettingsService`

---

## 5.3 Core Domain and Infrastructure Layer

Camada onde vive a logica central do produto.

### Responsabilidades

- detectar dispositivos;
- calcular capacidade e compatibilidade;
- manipular manifestos;
- consultar catalogo de modelos;
- selecionar e validar packs;
- montar o plano de build;
- executar downloads;
- copiar runtime e assets;
- validar checksums;
- gravar configuracoes;
- preparar o pendrive final.

### Caracteristica central

Essa camada deve ser o coracao robusto do Builder em `Rust`.

Ela deve funcionar de forma previsivel e testavel, com pouca dependencia de detalhes da interface.

---

## 6. Modulos Funcionais do Builder

## 6.1 Device Analyzer

Modulo responsavel por detectar e analisar pendrives e dispositivos de destino.

### Responsabilidades

- listar dispositivos removiveis;
- coletar nome, capacidade e espaco livre;
- identificar sistema de arquivos;
- detectar riscos de incompatibilidade;
- estimar se o dispositivo pode ser usado;
- informar necessidade de formatacao.

### Saidas

- lista de `BuildTargetDevice`;
- alertas de compatibilidade;
- recomendacoes de preparacao.

---

## 6.2 Build Planner

Modulo que transforma as escolhas do usuario em um plano executavel.

### Responsabilidades

- consolidar modelos escolhidos;
- consolidar packs escolhidos;
- considerar idioma e politicas;
- calcular espaco total necessario;
- calcular margem de seguranca;
- detectar conflitos;
- gerar um plano final de build.

### Saida

- `BuildPlan`

O `BuildPlan` deve ser aprovado antes de qualquer escrita no pendrive.

---

## 6.3 Model Catalog Client

Cliente da camada de inteligencia de modelos.

### Responsabilidades

- carregar catalogo local de modelos;
- sincronizar com fontes remotas quando permitido;
- consultar benchmark e metadados;
- normalizar informacoes para recomendacao;
- operar em modo cache quando offline.

### Entradas

- politica de atualizacao;
- idioma;
- conectividade;
- filtros do usuario.

### Saidas

- lista de `ModelRecord`;
- estado do catalogo;
- metadados de atualizacao.

---

## 6.4 Recommendation Engine

Motor de recomendacao do Builder.

### Responsabilidades

- sugerir modelos que cabem no pendrive;
- sugerir combinacoes multi-modelo;
- priorizar velocidade ou qualidade;
- filtrar por idioma;
- considerar CPU-only, GPU e hibrido;
- considerar packs e afinidade com RAG.

### Exemplos de recomendacao

- melhor modelo para `PT` em pendrive pequeno;
- melhor combinacao `3B + 7B`;
- melhor modelo offline CPU-only;
- melhor modelo para uso com packs juridicos;
- melhor modelo equilibrado para `16GB RAM`.

### Saidas

- `RecommendedModelSet`
- explicacao de recomendacao
- alertas de custo ou risco

---

## 6.5 Pack Registry Client

Modulo responsavel por descoberta e consulta de packs.

### Responsabilidades

- listar packs disponiveis;
- ler manifestos;
- filtrar por idioma e categoria;
- validar compatibilidade com runtime;
- validar dependencias entre packs;
- diferenciar packs oficiais e comunitarios.

### Saidas

- lista de `PackManifest`;
- dependencias;
- alertas de compatibilidade.

---

## 6.6 Pack Installer

Modulo responsavel por instalar os packs selecionados no pendrive.

### Responsabilidades

- baixar ou copiar assets do pack;
- validar integridade;
- instalar manifestos;
- instalar conteudo semantico;
- registrar o pack no `Build Manifest`;
- preparar assets para o runtime.

---

## 6.7 Runtime Assembler

Modulo responsavel por montar o runtime USBRAIN dentro do pendrive.

### Responsabilidades

- copiar runtime base;
- instalar configuracoes do agente;
- instalar modelos selecionados;
- instalar indexes e assets semanticos;
- gerar manifestos;
- preparar estrutura final do USBRAIN.

### Importancia

Esse modulo e o equivalente conceitual do atual processo de montagem do pendrive, mas elevado ao nivel do produto final.

---

## 6.8 Validation Engine

Modulo responsavel por validar o artefato gerado.

### Responsabilidades

- verificar presenca de arquivos obrigatorios;
- validar checksums;
- verificar espaco residual;
- verificar consistencia dos manifestos;
- verificar compatibilidade declarada de modelos e packs;
- emitir relatorio final de sucesso ou falha.

### Tipos de validacao

- `pre-build validation`
- `in-build validation`
- `post-build validation`

---

## 6.9 Download Manager

Modulo responsavel por downloads de catalogos, modelos, packs e assets.

### Responsabilidades

- baixar arquivos grandes;
- retomar downloads;
- validar checksum;
- manter cache local;
- gerenciar paralelismo com seguranca;
- reportar progresso para a interface.

---

## 6.10 Secrets Manager

Modulo responsavel por credenciais opcionais do usuario.

### Responsabilidades

- receber chaves de provedores externos;
- validar formato basico;
- armazenar de forma segura;
- decidir se a credencial fica no Builder, no pendrive ou em ambos;
- respeitar politica de privacidade.

### Regra importante

As credenciais nao devem ser tratadas como parte trivial do build. Precisam de modelo explicito de seguranca.

---

## 6.11 Localization Layer

Camada responsavel por internacionalizacao e localizacao.

### Idiomas minimos

- `PT`
- `EN`
- `ES`

### Responsabilidades

- textos da GUI;
- mensagens de erro;
- descricoes de modelos;
- descricoes de packs;
- resumo do build;
- onboarding;
- textos de ajuda.

---

## 6.12 Settings and Profile Manager

Modulo responsavel por preferencias persistentes do Builder.

### Responsabilidades

- idioma padrao;
- diretorios de cache;
- politica de atualizacao de catalogo;
- preferencia de privacidade;
- perfil de build padrao;
- historico de builds recentes.

---

## 7. Estrutura de Comunicacao UI <-> Core

No `Tauri`, a comunicacao entre interface e backend deve seguir um padrao previsivel.

### Principio

A interface envia `commands` e o backend responde com:

- resultados imediatos;
- erros estruturados;
- eventos de progresso;
- estado atualizado do workflow.

### Tipos de interacao

- `request/response`
- `event stream`
- `job progress updates`
- `state snapshot`

### Exemplos de comandos

- `scan_devices`
- `load_model_catalog`
- `get_recommendations`
- `select_build_target`
- `select_models`
- `select_packs`
- `configure_profile`
- `start_build`
- `cancel_build`
- `validate_build`

### Exemplos de eventos

- `device_scan_completed`
- `catalog_updated`
- `download_progress`
- `build_stage_changed`
- `validation_warning`
- `build_completed`
- `build_failed`

---

## 8. Estado da Aplicacao

O Builder precisa de um modelo de estado claro.

### Estado de sessao do usuario

- dispositivo selecionado;
- modelos selecionados;
- packs selecionados;
- idioma escolhido;
- politicas escolhidas;
- credenciais configuradas;
- plano de build;
- estado atual do build.

### Estado de workflow

- `idle`
- `scanning`
- `planning`
- `ready`
- `building`
- `validating`
- `completed`
- `failed`
- `cancelled`

### Regra

A GUI deve sempre refletir esse estado sem depender de inferencias locais confusas.

---

## 9. Fluxo Funcional da Interface

## 9.1 Tela Inicial

### Objetivo

Apresentar o Builder e detectar dispositivos.

### Elementos

- pendrives encontrados;
- espaco total e livre;
- status de compatibilidade;
- botao para iniciar criacao.

---

## 9.2 Tela de Configuracao Base

### Objetivo

Definir o perfil principal do USBRAIN.

### Elementos

- nome do USBRAIN;
- idioma principal;
- modo `offline`, `local assistido` ou `hibrido`;
- politica de privacidade;
- uso de GPU opcional;
- configuracao inicial do agente.

---

## 9.3 Tela de Modelos

### Objetivo

Escolher uma ou mais LLMs.

### Elementos

- modelos recomendados;
- filtros por idioma, tamanho e objetivo;
- comparacao entre modelos;
- visualizacao de espaco ocupado;
- sugestoes automáticas.

---

## 9.4 Tela de Packs

### Objetivo

Selecionar packs especialistas.

### Elementos

- categorias;
- busca;
- idioma;
- compatibilidade;
- tamanho em disco;
- relevancia para o perfil atual.

---

## 9.5 Tela de Provedores

### Objetivo

Configurar integracoes externas opcionais.

### Elementos

- lista de provedores suportados;
- formulario de credenciais;
- politica de fallback;
- explicacao de privacidade.

---

## 9.6 Tela de Revisao

### Objetivo

Apresentar o `BuildPlan` final antes da execucao.

### Elementos

- dispositivo selecionado;
- modelos escolhidos;
- packs escolhidos;
- espaco total estimado;
- riscos e alertas;
- politicas ativas;
- botao `Criar USBRAIN`.

---

## 9.7 Tela de Execucao

### Objetivo

Mostrar o build em andamento.

### Elementos

- etapa atual;
- progresso geral;
- progresso por arquivo ou lote;
- validacoes em andamento;
- falhas recuperaveis;
- opcao de cancelamento seguro.

---

## 9.8 Tela Final

### Objetivo

Confirmar sucesso ou falha do build.

### Elementos

- resultado final;
- resumo do que foi instalado;
- validacoes executadas;
- instrucoes de uso;
- logs exportaveis;
- opcao de abrir pasta do pendrive.

---

## 10. Estrutura do Core em Rust

O core em `Rust` deve ser organizado por dominios funcionais e nao por detalhes aleatorios de implementacao.

### Estrutura conceitual recomendada

- `app`
  - comandos Tauri
  - estado da aplicacao
  - orchestrators

- `domain`
  - entidades
  - manifestos
  - politicas
  - recomendacoes

- `services`
  - planner
  - assembler
  - validator
  - installer

- `infrastructure`
  - filesystem
  - downloads
  - storage local
  - os integration
  - secure storage
  - catalog fetch

- `interfaces`
  - mapeamento entre UI e core
  - DTOs e payloads

---

## 11. Jobs Assincronos e Responsividade

Como o Builder vai lidar com downloads e copia de arquivos grandes, ele precisa de um sistema claro de jobs.

### Tipos de jobs

- scan de dispositivos;
- sincronizacao de catalogo;
- download de modelo;
- download de pack;
- montagem do runtime;
- validacao final.

### Regras

- jobs nao podem travar a GUI;
- jobs devem emitir progresso estruturado;
- jobs devem ser cancelaveis quando seguro;
- jobs devem registrar estado e erro de forma consistente;
- jobs devem ser reiniciaveis quando aplicavel.

---

## 12. Cache e Persistencia Local

O Builder precisa de uma camada local de persistencia para evitar trabalho desnecessario.

### O que deve ser persistido

- catalogo de modelos em cache;
- packs em cache;
- modelos ja baixados;
- preferencias do usuario;
- historico de builds;
- checksums conhecidos;
- estado de downloads interrompidos.

### Objetivo

Permitir:

- uso mais rapido;
- menos repeticao de download;
- uso parcial offline;
- experiencia mais profissional.

---

## 13. Seguranca e Confianca

O Builder precisa inspirar confianca.

### Requisitos funcionais ligados a seguranca

- validar origem de assets;
- validar checksums;
- tornar claro o que sera gravado no pendrive;
- diferenciar conteudo oficial e comunitario;
- explicitar quando dados saem da maquina;
- proteger credenciais externas;
- separar configuracao do Builder e do pendrive.

---

## 14. Integracao com o Runtime

O Builder e o Runtime devem ser acoplados por contratos estaveis, nao por acoplamento informal.

### Contratos conceituais

- `Build Manifest`
- `Runtime Config`
- `Pack Manifest`
- `Model Install Descriptor`
- `Execution Policy`

### Objetivo

Permitir evolucao independente de:

- Builder
- Runtime
- Packs
- Hub

desde que os contratos sejam respeitados.

---

## 15. Modo Offline e Atualizacao Online

O Builder deve ser `offline-first`, mas nao `offline-only`.

### Offline-first significa

- ele continua funcional com catalogos e assets em cache;
- pode montar um USBRAIN sem internet, se os recursos ja estiverem disponiveis;
- nao depende da nuvem para existir.

### Atualizacao online opcional significa

- atualizar catalogo de modelos;
- atualizar catalogo de packs;
- baixar novas versoes do runtime;
- sincronizar benchmarks e observacoes.

---

## 16. Suporte Multilingue na Arquitetura

O suporte a `PT`, `EN` e `ES` nao deve ser um detalhe tardio.

### Deve existir desde cedo em

- textos da interface;
- mensagens de workflow;
- descricoes de modelos;
- descricoes de packs;
- erros e alertas;
- onboarding.

### Regra arquitetural

Nenhum texto relevante ao usuario deve nascer hardcoded sem estrategia de localizacao.

---

## 17. Publicacao e Distribuicao

O Builder deve ser preparado para distribuicao como aplicacao desktop.

### Artefatos esperados

- binario para `Windows`
- binario para `Linux`
- instaladores ou pacotes adequados a cada SO

### Objetivo

Que o usuario final perceba o produto como:

- aplicativo real;
- confiavel;
- simples de instalar;
- simples de usar.

---

## 18. Prioridades de Implementacao do Builder

Sequencia funcional recomendada:

1. `Shell Tauri + Rust funcionando`
2. `Device Analyzer`
3. `Build Planner`
4. `Runtime Assembler`
5. `Validation Engine`
6. `Model Catalog Client`
7. `Recommendation Engine`
8. `Pack Registry Client`
9. `Pack Installer`
10. `Localization Layer`
11. `Secrets Manager`
12. `Integracao com provedores externos`

---

## 19. Resultado Esperado

Quando esta arquitetura estiver implementada, o `USBRAIN Builder` sera:

- uma aplicacao desktop real;
- com GUI simples, bonita e profissional;
- robusta o suficiente para distribuicao ampla;
- capaz de criar USBRAINs personalizados;
- preparada para evoluir junto com modelos, packs e comunidade.

---

## 20. Relacao com os Documentos Anteriores

Este documento deriva de:

- [DEFINICAO_PRODUTO_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/DEFINICAO_PRODUTO_USBRAIN.md)
- [ESPECIFICACAO_CONCEITUAL_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ESPECIFICACAO_CONCEITUAL_USBRAIN.md)

Ele prepara o terreno para os proximos documentos:

- roadmap por fases;
- formato oficial de packs;
- especificacao do catalogo vivo de modelos;
- arquitetura do runtime;
- estrategia de implementacao em `Tauri + Rust`.
