# USBRAIN — Roadmap Geral do Projeto

## 1. Objetivo

Este documento organiza a evolucao do `USBRAIN` em fases, milestones e prioridades de entrega.

O objetivo do roadmap e transformar a visao do produto em uma sequencia de execucao realista, incremental e coerente com a ambicao do projeto.

Este documento responde:

- o que ja foi validado;
- o que ainda precisa ser construido;
- em que ordem faz sentido construir;
- quais entregas formam o `MVP`, `Alpha`, `Beta` e `Release Open Source`;
- quais dependencias existem entre `Builder`, `Runtime`, `Packs`, `Hub` e `Catalogo de Modelos`.

---

## 2. Leitura Estrategica

O `USBRAIN` nao deve ser tratado como um unico bloco de software.

Ele e um ecossistema composto por:

- `Builder`
- `Runtime`
- `PACKs`
- `Hub`
- `Catalogo Vivo de Modelos`

O roadmap precisa respeitar essa realidade.

Por isso, a implementacao deve acontecer em camadas:

- primeiro a base executavel;
- depois a adaptacao real ao hardware;
- depois a especializacao por packs;
- depois a camada comunitaria;
- depois a sofisticacao do ecossistema.

---

## 3. O Que Ja Foi Validado

Antes deste roadmap, o projeto ja validou algo muito importante:

- a ideia central funciona;
- um pendrive funcional com `LLM local + BGE + Web UI + agente` e viavel;
- o runtime ja foi exercitado em ambiente real;
- o projeto ja saiu do campo puramente teorico.

Isso significa que o roadmap nao parte do zero conceitual.

Ele parte de uma validacao tecnica inicial.

---

## 4. Macrofases do Projeto

O projeto pode ser dividido em seis macrofases:

## 4.1 Fase 0 — Validacao Tecnica Inicial

### Objetivo

Provar que um USBRAIN funcional podia existir.

### Status

- concluida

### Resultado

- runtime portatil viavel
- LLM local funcional
- RAG funcional
- interface utilizavel
- experimento real validado

---

## 4.2 Fase 1 — Fundacao do Produto

### Objetivo

Transformar o experimento em produto com definicao, arquitetura e direcao.

### Status

- em consolidacao documental

### Resultado esperado

- visao oficial do produto
- definicao conceitual
- arquitetura do Builder
- arquitetura do Runtime
- formato de packs
- especificacao do Hub
- especificacao do catalogo de modelos

---

## 4.3 Fase 2 — Builder MVP

### Objetivo

Criar a primeira versao funcional do `USBRAIN Builder` como aplicacao desktop real.

### Resultado esperado

- aplicacao `Tauri + Rust`
- GUI inicial
- criacao de USBRAIN funcional
- instalacao de runtime base
- instalacao de pelo menos um modelo local
- suporte inicial a packs preexistentes

---

## 4.4 Fase 3 — Runtime Adaptativo

### Objetivo

Consolidar o runtime como orquestrador real, adaptativo ao hardware.

### Resultado esperado

- selecao automatica de modelo
- tuning por hardware
- ativacao controlada de packs
- operacao offline robusta
- modos offline e hibrido bem definidos

---

## 4.5 Fase 4 — Ecossistema de Packs e Catalogo Vivo

### Objetivo

Transformar o USBRAIN em plataforma de especializacao e nao apenas em builder de pendrive.

### Resultado esperado

- criacao de packs dentro do Builder
- publicacao basica no Hub
- catalogo vivo de modelos funcional
- recomendacoes contextualizadas

---

## 4.6 Fase 5 — Release Open Source Publico

### Objetivo

Publicar o projeto de forma solida e sustentavel para comunidade.

### Resultado esperado

- repositorio publico no `GitHub`
- documentacao forte
- instaladores publicos
- governanca inicial
- diretrizes para contribuicao
- base publica em `usbrain.org`

---

## 5. Fase 1 — Fundacao do Produto

Essa fase organiza intelectualmente o projeto.

### Entregas principais

- definicao oficial do produto
- especificacao conceitual
- arquitetura funcional do Builder
- formato oficial de `PACKs`
- especificacao do `Hub`
- especificacao do `Catalogo Vivo de Modelos`
- arquitetura do `Runtime`
- roadmap geral

### Objetivo pratico

Evitar que a implementacao futura aconteca sem direcao clara.

### Criterio de conclusao

- o projeto tem documentos suficientes para orientar a execucao real

---

## 6. Fase 2 — Builder MVP

Essa e a primeira fase de produto executavel com foco no criador do USBRAIN.

## 6.1 Objetivo

Entregar um `USBRAIN Builder` minimo, mas real.

## 6.2 Caracteristicas do MVP

- aplicacao desktop em `Tauri + Rust`
- suporte inicial a `Windows` e `Linux`
- GUI simples, limpa e funcional
- deteccao de pendrives
- escolha de configuracao base
- escolha de pelo menos uma LLM
- montagem do pendrive
- validacao final da instalacao

## 6.3 Escopo minimo do Builder MVP

- `Shell Tauri`
- `Device Analyzer`
- `Build Planner`
- `Runtime Assembler`
- `Validation Engine`
- cache local basico
- suporte i18n inicial `PT`, `EN`, `ES`

## 6.4 O que ainda nao precisa entrar com profundidade

- criacao de packs
- hub publico completo
- recomendacao sofisticada multi-modelo
- benchmark comunitario
- pack authoring avancado

## 6.5 Criterio de conclusao do MVP

- qualquer usuario consegue criar um USBRAIN funcional a partir da GUI do Builder

---

## 7. Fase 3 — Builder Alpha

Depois do MVP, o Builder precisa ficar mais inteligente.

## 7.1 Objetivo

Sair de uma criacao funcional para uma criacao contextualizada.

## 7.2 Entregas principais

- `Model Catalog Client`
- `Recommendation Engine` inicial
- tela de comparacao de modelos
- suporte a multiplos modelos no mesmo pendrive
- instalacao de packs existentes
- politica de uso local, offline ou hibrido
- configuracao de provedores externos

## 7.3 Resultado esperado

O Builder deixa de apenas montar um pendrive e passa a ajudar o usuario a tomar boas decisoes.

## 7.4 Criterio de conclusao

- o usuario recebe recomendacoes simples, mas uteis, ao montar seu USBRAIN

---

## 8. Fase 4 — Runtime Alpha

O runtime precisa deixar de ser apenas funcional e passar a ser adaptativo.

## 8.1 Objetivo

Consolidar a inteligencia operacional do USBRAIN no computador hospedeiro.

## 8.2 Entregas principais

- `Boot Manager` confiavel
- `Hardware Detector`
- `Manifest Loader`
- `Runtime Policy Engine`
- `Model Orchestrator`
- `Semantic Engine` consolidado
- `Agent Core` integrado a packs
- `UI/API Server` estavel

## 8.3 Capacidades esperadas

- escolha automatica de modelo
- ajuste por RAM e CPU
- ativacao de packs relevantes
- experiencia local previsivel
- logs melhores

## 8.4 Criterio de conclusao

- o mesmo USBRAIN se adapta de forma clara a hardwares diferentes

---

## 9. Fase 5 — Packs Beta

Depois que Builder e Runtime estiverem solidos, a especializacao precisa virar fluxo real.

## 9.1 Objetivo

Tornar `PACKs` um recurso nativo e produtivo do ecossistema.

## 9.2 Entregas principais

- modo `Criar PACK` dentro do Builder
- pipeline de ingestao
- normalizacao e chunking
- indexacao semantica
- geracao de manifesto
- validacao e empacotamento
- instalacao de packs no build

## 9.3 Resultado esperado

O projeto passa a permitir que a comunidade crie inteligencia especializada, nao apenas use modelos genericos.

## 9.4 Criterio de conclusao

- um colaborador consegue criar e instalar um pack funcional sem depender de ferramentas improvisadas

---

## 10. Fase 6 — Hub Beta

Com packs reais, o ecossistema precisa de um ponto de encontro.

## 10.1 Objetivo

Entregar o primeiro `USBRAIN Hub` funcional.

## 10.2 Entregas principais

- catalogo publico de packs
- descoberta por idioma e categoria
- leitura de manifesto e versoes
- publicacao basica
- status `official`, `community`, `experimental`
- colecoes curadas iniciais

## 10.3 Integracao esperada

- o Builder consegue consultar o Hub
- o usuario instala packs direto da GUI
- a comunidade passa a ter um local de publicacao

## 10.4 Criterio de conclusao

- o fluxo `criar pack -> publicar -> descobrir no Builder -> instalar` funciona de ponta a ponta

---

## 11. Fase 7 — Catalogo Vivo de Modelos Beta

O builder so sera realmente longevo quando souber acompanhar o mundo dos modelos.

## 11.1 Objetivo

Consolidar a camada viva de descoberta, avaliacao e recomendacao de modelos.

## 11.2 Entregas principais

- registry estruturado de modelos
- variantes e quantizacoes
- metadados por idioma
- viabilidade `CPU-only`
- perfis de benchmark
- recomendacao por contexto
- recomendacao multi-modelo

## 11.3 Resultado esperado

O Builder passa a ser um curador inteligente de modelos e nao um instalador com lista fixa.

## 11.4 Criterio de conclusao

- o usuario consegue escolher modelos com base em recomendacoes contextualizadas e atualizadas

---

## 12. Fase 8 — Beta Integrado do Ecossistema

Essa fase junta as grandes pecas.

## 12.1 Objetivo

Integrar `Builder`, `Runtime`, `PACKs`, `Hub` e `Catalogo de Modelos` em uma experiencia coerente.

## 12.2 O que precisa funcionar junto

- Builder usa o catalogo de modelos
- Builder usa o Hub
- Builder monta USBRAIN com multiplos modelos e packs
- Runtime executa e adapta ao hardware
- Packs influenciam conhecimento e comportamento
- politicas de cloud e privacidade sao respeitadas

## 12.3 Resultado esperado

O USBRAIN deixa de ser conjunto de subsistemas e vira plataforma integrada.

---

## 13. Fase 9 — Release Open Source

Essa fase e a publicacao madura do projeto para o mundo.

## 13.1 Objetivo

Abrir o projeto no `GitHub` com base suficiente para comunidade real.

## 13.2 Entregas principais

- repositorio publico organizado
- licenca definida
- README forte
- documentacao inicial consolidada
- guia de contribuicao
- guia para criacao de packs
- artefatos de distribuicao do Builder
- exemplos oficiais de packs
- pagina publica no `usbrain.org`

## 13.3 Objetivo estrategico

Fazer o projeto nascer publicamente como plataforma e nao como experimento confuso.

## 13.4 Criterio de conclusao

- um usuario externo consegue entender o projeto, baixar o Builder, criar um USBRAIN e contribuir com o ecossistema

---

## 14. Milestones Recomendados

Para tornar o roadmap mais operacional, estas sao as milestones recomendadas.

## 14.1 Milestone A — Base do Builder

- shell `Tauri + Rust`
- deteccao de pendrive
- montagem basica
- validacao final

## 14.2 Milestone B — USBRAIN Criavel

- Builder gera um pendrive funcional
- Runtime base sobe com sucesso
- GUI minima pronta

## 14.3 Milestone C — Builder Inteligente

- recomendacao inicial de modelos
- suporte multi-modelo
- packs existentes instalaveis

## 14.4 Milestone D — Runtime Adaptativo

- hardware detection
- selecao de modelo
- tuning adaptativo

## 14.5 Milestone E — Pack Authoring

- criacao de packs no Builder
- validacao de pack
- instalacao de pack

## 14.6 Milestone F — Hub Inicial

- descoberta e publicacao de packs
- integracao com Builder

## 14.7 Milestone G — Catalogo Vivo

- registry de modelos
- recomendacao contextual
- benchmark pratico

## 14.8 Milestone H — Release Publico

- GitHub
- usbrain.org
- governanca
- docs publicas

---

## 15. Ordem de Prioridade

Se precisarmos escolher com rigor, a ordem recomendada e:

1. `Builder MVP`
2. `Runtime adaptativo minimo`
3. `Catalogo inicial de modelos`
4. `Instalacao de packs existentes`
5. `Criacao de packs no Builder`
6. `Hub inicial`
7. `Catalogo vivo com benchmark e recomendacao contextual`
8. `Release open source publico`

### Motivo

Sem Builder e Runtime, o restante nao tem corpo.

Sem packs, o ecossistema perde grande parte do diferencial.

Sem catalogo vivo, o produto envelhece.

Sem Hub, a comunidade cresce mais devagar.

---

## 16. Dependencias Entre Frentes

Algumas frentes so fazem sentido depois de outras.

### Dependencias centrais

- `Runtime` depende do `Builder` gravar contratos claros
- `Criacao de PACKs` depende do formato oficial de packs
- `Hub` depende de packs publicaveis e versionados
- `Catalogo Vivo de Modelos` depende de registry estruturado
- `Release publico` depende de experiencia minima coerente

---

## 17. Riscos do Projeto

O roadmap tambem precisa reconhecer riscos.

### Riscos principais

- tentar construir tudo ao mesmo tempo;
- deixar o Builder bonito mas pouco funcional;
- deixar o Runtime funcional mas pouco adaptativo;
- subestimar a complexidade da curadoria de modelos;
- subestimar a experiencia de criacao de packs;
- abrir o projeto cedo demais sem governanca minima.

### Resposta estrategica

- construir em fases;
- manter contratos claros;
- validar fluxos inteiros antes de expandir escopo;
- documentar bem desde cedo.

---

## 18. Indicadores de Progresso

Cada fase precisa ser acompanhada por sinais claros de progresso.

### Exemplos de indicadores

- Builder gera pendrive funcional sem ajuda externa
- Runtime escolhe modelo coerente automaticamente
- instalacao de packs ocorre sem erro manual
- recomendacao de modelos aparece na GUI
- pack criado no Builder pode ser instalado
- Builder consome Hub e catalogo sem friccao
- usuario externo consegue usar o sistema lendo apenas a documentacao

---

## 19. Visao de Longo Prazo

Se o roadmap for executado corretamente, o USBRAIN pode evoluir para:

- plataforma open source de criacao de inteligencias portateis;
- ecossistema de packs especialistas;
- curadoria viva de modelos open source;
- comunidade ativa em torno de conhecimentos empacotados;
- referencia pratica de IA local, adaptativa e portatil.

---

## 20. Resultado Esperado do Roadmap

Ao final deste roadmap, o projeto deve ter saido de:

- uma validacao tecnica com pendrive funcional

para:

- uma plataforma open source coerente, documentada, distribuivel e preparada para comunidade.

---

## 21. Relacao com os Documentos Anteriores

Este roadmap deriva de:

- [DEFINICAO_PRODUTO_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/DEFINICAO_PRODUTO_USBRAIN.md)
- [ESPECIFICACAO_CONCEITUAL_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ESPECIFICACAO_CONCEITUAL_USBRAIN.md)
- [ARQUITETURA_FUNCIONAL_USBRAIN_BUILDER.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ARQUITETURA_FUNCIONAL_USBRAIN_BUILDER.md)
- [FORMATO_PACK_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/FORMATO_PACK_USBRAIN.md)
- [ESPECIFICACAO_HUB_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ESPECIFICACAO_HUB_USBRAIN.md)
- [ESPECIFICACAO_CATALOGO_MODELOS.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ESPECIFICACAO_CATALOGO_MODELOS.md)
- [ARQUITETURA_RUNTIME_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ARQUITETURA_RUNTIME_USBRAIN.md)

Ele prepara os proximos documentos praticos, como:

- contratos Builder <-> Runtime
- milestones tecnicos detalhados
- backlog inicial do Builder em `Tauri + Rust`
- estrategia de abertura publica em `GitHub` e `usbrain.org`
