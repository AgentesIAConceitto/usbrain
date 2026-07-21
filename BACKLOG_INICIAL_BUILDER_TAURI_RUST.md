# USBRAIN Builder — Backlog Inicial em Tauri + Rust

## 1. Objetivo

Este documento transforma a arquitetura e o roadmap do `USBRAIN Builder` em um backlog inicial executavel.

O objetivo e sair da camada conceitual e entrar na camada de implementacao pratica.

Este backlog responde:

- quais epicos devem ser implementados primeiro;
- quais historias de usuario compoem o `Builder MVP` e o `Builder Alpha`;
- qual e a ordem recomendada de execucao;
- quais criterios de pronto devem ser usados;
- como quebrar o trabalho em incrementos realistas.

---

## 2. Escopo Deste Backlog

Este backlog cobre principalmente:

- `Builder MVP`
- `Builder Alpha`

Ou seja:

- base do app desktop;
- fluxo minimo de criacao do USBRAIN;
- primeiras capacidades de catalogo e recomendacao;
- instalacao inicial de packs existentes;
- preparacao para evolucao futura.

Este documento ainda nao detalha o backlog completo de:

- `Hub`
- `Runtime` inteiro
- `Pack Authoring`
- `Catalogo Vivo completo`

Mas prepara o terreno para essas frentes.

---

## 3. Premissas

Este backlog assume:

- `Tauri + Rust` como stack oficial do Builder;
- `GUI desktop` como experiencia obrigatoria;
- suporte inicial a `Windows` e `Linux`;
- idiomas iniciais `PT`, `EN` e `ES`;
- integracao com o runtime por contratos claros;
- foco inicial em criar USBRAINs funcionais antes de sofisticar o ecossistema.

---

## 4. Definicao de Pronto

Uma historia so deve ser considerada pronta quando:

- comportamento principal foi implementado;
- foi validada no contexto do fluxo esperado;
- erros principais foram tratados;
- impacto visual ou de UX foi revisado quando aplicavel;
- nao ha diagnosticos triviais abertos no arquivo alterado;
- a integracao com o restante do Builder nao ficou quebrada.

### Regra pratica

No USBRAIN, "pronto" nao deve significar apenas "compila".

Deve significar "vira um passo real da experiencia do produto".

---

## 5. Priorizacao

Este backlog usa quatro niveis:

- `P0` critico para o `Builder MVP`
- `P1` importante para fechar o `Builder MVP` com qualidade
- `P2` necessario para o `Builder Alpha`
- `P3` desejavel, mas nao bloqueante

---

## 6. Epicos do Builder MVP

## 6.1 Epico A — Fundacao do Aplicativo

### Objetivo

Criar a base executavel do `USBRAIN Builder` em `Tauri + Rust`.

### Prioridade

- `P0`

### Historias

#### A1. Criar shell inicial do app desktop

- Como usuario, quero abrir o Builder como aplicacao desktop real para nao depender de terminal.

##### Prioridade

- `P0`

##### Criterios de pronto

- app abre em `Windows` e `Linux`
- janela principal funciona
- estrutura inicial `Tauri + Rust` esta operacional

#### A2. Definir estrutura interna do projeto

- Como mantenedor, quero uma base organizada por camadas para que o projeto possa crescer sem virar caos.

##### Prioridade

- `P0`

##### Criterios de pronto

- modulos principais separados
- padrao inicial de comandos e eventos definido
- convencoes minimas de organizacao documentadas

#### A3. Implementar estado global inicial da aplicacao

- Como Builder, quero um estado claro de sessao para refletir corretamente a etapa atual do workflow.

##### Prioridade

- `P0`

##### Criterios de pronto

- estados basicos definidos
- UI consegue reagir ao estado
- transicoes principais de workflow existem

---

## 6.2 Epico B — Device Analyzer

### Objetivo

Detectar pendrives e analisar se podem ser usados como destino de build.

### Prioridade

- `P0`

### Historias

#### B1. Listar dispositivos removiveis

- Como usuario, quero ver os pendrives disponiveis para escolher o destino correto do USBRAIN.

##### Prioridade

- `P0`

##### Criterios de pronto

- dispositivos removiveis sao listados
- nome, capacidade e identificador aparecem
- estados vazios e erros sao tratados

#### B2. Exibir espaco total e livre

- Como usuario, quero ver capacidade e espaco livre do pendrive para entender o que cabe nele.

##### Prioridade

- `P0`

##### Criterios de pronto

- UI mostra capacidade total
- UI mostra espaco livre
- dados ficam atualizados apos rescan

#### B3. Detectar incompatibilidades basicas

- Como usuario, quero ser alertado se o pendrive estiver inadequado ou exigir formatacao.

##### Prioridade

- `P1`

##### Criterios de pronto

- sistema de arquivos e verificado
- alertas minimos aparecem
- situacoes obvias de incompatibilidade sao explicadas

---

## 6.3 Epico C — Fluxo Base da GUI

### Objetivo

Construir as telas basicas do Builder MVP.

### Prioridade

- `P0`

### Historias

#### C1. Criar tela inicial

- Como usuario, quero uma tela inicial clara para entender o que o Builder faz e iniciar o processo.

##### Prioridade

- `P0`

##### Criterios de pronto

- tela inicial renderiza corretamente
- lista de dispositivos aparece nela ou flui naturalmente a partir dela
- CTA principal esta claro

#### C2. Criar tela de configuracao base

- Como usuario, quero definir o perfil inicial do meu USBRAIN antes da montagem.

##### Prioridade

- `P0`

##### Criterios de pronto

- nome do USBRAIN
- idioma principal
- modo offline, local assistido ou hibrido
- progresso visual do fluxo

#### C3. Criar tela de revisao

- Como usuario, quero revisar minhas escolhas antes da gravacao no pendrive.

##### Prioridade

- `P0`

##### Criterios de pronto

- resumo das escolhas aparece
- alertas principais aparecem
- usuario pode voltar e editar

#### C4. Criar tela de execucao

- Como usuario, quero acompanhar a criacao do USBRAIN com progresso claro.

##### Prioridade

- `P0`

##### Criterios de pronto

- etapas do build aparecem
- progresso e erros sao exibidos
- estado final fica compreensivel

---

## 6.4 Epico D — Build Planner

### Objetivo

Transformar as escolhas do usuario em um plano consistente antes de qualquer gravacao.

### Prioridade

- `P0`

### Historias

#### D1. Criar entidade `BuildPlan`

- Como sistema, quero consolidar tudo em um plano de build para reduzir comportamento opaco.

##### Prioridade

- `P0`

##### Criterios de pronto

- estrutura de plano existe
- contem dispositivo, configuracao, modelos e packs
- pode ser serializada ou inspecionada

#### D2. Calcular espaco estimado do build

- Como usuario, quero saber quanto espaco sera usado antes de gravar no pendrive.

##### Prioridade

- `P0`

##### Criterios de pronto

- calculo de espaco funciona
- margem de seguranca e considerada
- bloqueio ocorre se nao couber

#### D3. Validar conflitos basicos do plano

- Como sistema, quero detectar problemas antes da escrita para evitar builds quebrados.

##### Prioridade

- `P1`

##### Criterios de pronto

- conflitos obvios sao detectados
- erros sao exibidos ao usuario
- plano invalido nao segue para execucao

---

## 6.5 Epico E — Runtime Assembler

### Objetivo

Montar o runtime base do USBRAIN no pendrive.

### Prioridade

- `P0`

### Historias

#### E1. Copiar runtime base para o pendrive

- Como usuario, quero que o Builder instale a base do USBRAIN automaticamente.

##### Prioridade

- `P0`

##### Criterios de pronto

- estrutura base e copiada
- erros de arquivo sao tratados
- progresso e reportado

#### E2. Instalar configuracao inicial do agente

- Como sistema, quero gravar configuracoes basicas para que o USBRAIN tenha comportamento inicial consistente.

##### Prioridade

- `P1`

##### Criterios de pronto

- configuracao e gravada
- valores escolhidos pelo usuario sao refletidos

#### E3. Gerar manifestos iniciais

- Como runtime futuro, quero encontrar contratos claros no pendrive para iniciar corretamente.

##### Prioridade

- `P0`

##### Criterios de pronto

- `Build Manifest` inicial existe
- configuracao do runtime esta representada
- formato e consistente com a arquitetura

---

## 6.6 Epico F — Instalacao de Modelo Local

### Objetivo

Permitir que o Builder instale pelo menos uma LLM local no USBRAIN.

### Prioridade

- `P0`

### Historias

#### F1. Permitir selecionar um modelo local inicial

- Como usuario, quero escolher ao menos um modelo para criar um USBRAIN funcional.

##### Prioridade

- `P0`

##### Criterios de pronto

- lista inicial de modelos suportados existe
- usuario escolhe um modelo
- escolha entra no plano de build

#### F2. Copiar ou baixar o modelo selecionado

- Como sistema, quero instalar o modelo escolhido no pendrive de forma confiavel.

##### Prioridade

- `P0`

##### Criterios de pronto

- modelo vai para local correto
- checksum ou validacao basica existe
- progresso e erros sao reportados

#### F3. Registrar o modelo nos manifestos

- Como runtime futuro, quero saber quais modelos foram instalados.

##### Prioridade

- `P0`

##### Criterios de pronto

- modelo instalado aparece nos metadados
- identificador e caminho ficam coerentes

---

## 6.7 Epico G — Validation Engine

### Objetivo

Garantir que o pendrive gerado esteja minimamente consistente.

### Prioridade

- `P0`

### Historias

#### G1. Validar presenca de arquivos obrigatorios

- Como sistema, quero verificar a estrutura final do USBRAIN antes de concluir.

##### Prioridade

- `P0`

##### Criterios de pronto

- arquivos obrigatorios sao verificados
- falhas geram mensagem compreensivel

#### G2. Validar manifestos principais

- Como sistema, quero confirmar que os contratos gravados sao consistentes.

##### Prioridade

- `P1`

##### Criterios de pronto

- manifestos sao lidos de volta
- inconsistencias sao sinalizadas

#### G3. Emitir relatorio final de sucesso ou falha

- Como usuario, quero saber claramente se meu USBRAIN foi criado com sucesso.

##### Prioridade

- `P0`

##### Criterios de pronto

- resultado final e exibido
- sucesso e falha sao claros
- logs exportaveis ou consultaveis existem em nivel basico

---

## 6.8 Epico H — Internacionalizacao Inicial

### Objetivo

Garantir que o Builder nao nasca preso a um unico idioma.

### Prioridade

- `P1`

### Historias

#### H1. Criar infraestrutura i18n

- Como produto, quero uma camada de localizacao desde cedo para suportar `PT`, `EN` e `ES`.

##### Prioridade

- `P1`

##### Criterios de pronto

- sistema de traducoes existe
- textos principais nao ficam hardcoded fora da estrategia

#### H2. Localizar telas principais do MVP

- Como usuario, quero usar o Builder no meu idioma.

##### Prioridade

- `P1`

##### Criterios de pronto

- telas principais possuem textos localizaveis
- troca de idioma funciona pelo menos no fluxo principal

---

## 7. Epicos do Builder Alpha

## 7.1 Epico I — Catalogo Inicial de Modelos

### Objetivo

Sair da lista fixa e entrar no modelo de catalogo.

### Prioridade

- `P2`

### Historias

#### I1. Implementar `Model Catalog Client` local

- Como sistema, quero carregar um catalogo de modelos estruturado para poder evoluir recomendacao.

#### I2. Exibir metadados basicos do modelo

- Como usuario, quero ver tamanho, idioma e requisitos estimados do modelo.

#### I3. Preparar sincronizacao futura de catalogo

- Como produto, quero nao ficar preso a lista estatica.

---

## 7.2 Epico J — Recommendation Engine Inicial

### Objetivo

Ajudar o usuario a escolher melhor.

### Prioridade

- `P2`

### Historias

#### J1. Recomendar modelo por espaco do pendrive

- Como usuario, quero saber quais modelos cabem no meu pendrive.

#### J2. Recomendar modelo por idioma

- Como usuario, quero ver quais modelos fazem mais sentido para `PT`, `EN` ou `ES`.

#### J3. Exibir tradeoffs entre velocidade e qualidade

- Como usuario, quero entender a diferenca entre modelos antes de escolher.

---

## 7.3 Epico K — Suporte Inicial a Multi-Modelo

### Objetivo

Permitir que um mesmo pendrive leve mais de uma LLM.

### Prioridade

- `P2`

### Historias

#### K1. Permitir selecionar mais de um modelo

- Como usuario, quero montar um USBRAIN com perfis diferentes de uso.

#### K2. Calcular espaco multi-modelo

- Como sistema, quero recalcular o build quando o usuario escolher mais de um modelo.

#### K3. Registrar multiplos modelos nos manifestos

- Como runtime futuro, quero saber qual conjunto de modelos esta instalado.

---

## 7.4 Epico L — Instalacao de Packs Existentes

### Objetivo

Permitir que o Builder ja use packs publicados ou locais, mesmo antes da autoria de packs.

### Prioridade

- `P2`

### Historias

#### L1. Ler `Pack Manifest`

- Como sistema, quero reconhecer um pack compativel.

#### L2. Exibir packs disponiveis ao usuario

- Como usuario, quero selecionar packs relevantes ao meu USBRAIN.

#### L3. Instalar assets do pack no pendrive

- Como sistema, quero preparar o runtime para usar os packs instalados.

---

## 7.5 Epico M — Segredos e Provedores Externos

### Objetivo

Permitir configuracao opcional de provedores externos sem trair a proposta local.

### Prioridade

- `P2`

### Historias

#### M1. Criar interface de provedores

- Como usuario, quero configurar provedores externos opcionais.

#### M2. Armazenar credenciais com seguranca basica

- Como sistema, quero proteger dados sensiveis.

#### M3. Gravar politica de fallback

- Como runtime futuro, quero respeitar a estrategia escolhida pelo usuario.

---

## 8. Ordem Recomendada de Implementacao

Para manter foco, a ordem recomendada e:

1. `Epico A — Fundacao do Aplicativo`
2. `Epico B — Device Analyzer`
3. `Epico C — Fluxo Base da GUI`
4. `Epico D — Build Planner`
5. `Epico E — Runtime Assembler`
6. `Epico F — Instalacao de Modelo Local`
7. `Epico G — Validation Engine`
8. `Epico H — Internacionalizacao Inicial`
9. `Epico I — Catalogo Inicial de Modelos`
10. `Epico J — Recommendation Engine Inicial`
11. `Epico K — Suporte Inicial a Multi-Modelo`
12. `Epico L — Instalacao de Packs Existentes`
13. `Epico M — Segredos e Provedores Externos`

---

## 9. Proposta de Sprints Iniciais

## 9.1 Sprint 1 — Base Executavel

### Foco

- `Epico A`
- parte de `Epico C`

### Resultado esperado

- app abre
- estrutura base existe
- tela inicial minima pronta

---

## 9.2 Sprint 2 — Dispositivo e Fluxo Basico

### Foco

- `Epico B`
- restante principal de `Epico C`

### Resultado esperado

- pendrive pode ser detectado
- usuario consegue iniciar fluxo de criacao

---

## 9.3 Sprint 3 — Planejamento e Montagem

### Foco

- `Epico D`
- `Epico E`

### Resultado esperado

- Builder gera plano de build
- runtime base e gravado no pendrive

---

## 9.4 Sprint 4 — Modelo e Validacao

### Foco

- `Epico F`
- `Epico G`
- `Epico H`

### Resultado esperado

- Builder cria um USBRAIN minimamente funcional
- validacao final existe
- i18n principal esta de pe

---

## 9.5 Sprint 5 — Catalogo e Recomendacao Inicial

### Foco

- `Epico I`
- `Epico J`

### Resultado esperado

- catalogo inicial funcional
- primeiras recomendacoes na GUI

---

## 9.6 Sprint 6 — Alpha Inicial

### Foco

- `Epico K`
- `Epico L`
- parte de `Epico M`

### Resultado esperado

- multi-modelo inicial
- packs existentes instalaveis
- politica externa opcional preparada

---

## 10. Dependencias Tecnicas Importantes

### Dependencias de produto

- sem `BuildPlan`, o fluxo de gravacao fica opaco;
- sem `Validation Engine`, o Builder entrega risco demais;
- sem `Manifestos`, o runtime nao tem contrato confiavel;
- sem i18n inicial, o produto ja nasce desalinhado com a visao.

### Dependencias de arquitetura

- `UI` depende de comandos e eventos claros;
- `Application Layer` depende de entidades e DTOs bem definidos;
- `Runtime Assembler` depende dos contratos Builder <-> Runtime;
- catalogo e recomendacao dependem de estrutura inicial de metadados.

---

## 11. Itens Deliberadamente Fora Deste Backlog Inicial

Para manter foco, estes temas nao entram como prioridade do backlog inicial:

- criacao completa de `PACKs`
- publicacao no `Hub`
- benchmark comunitario
- runtime adaptativo completo
- governanca open source
- site publico completo em `usbrain.org`

Esses temas entram em backlogs seguintes.

---

## 12. Sinal de Sucesso do Backlog Inicial

Este backlog tera cumprido seu papel quando:

- o `USBRAIN Builder` existir como app desktop real;
- o usuario conseguir criar um USBRAIN funcional pela GUI;
- o projeto tiver base suficiente para sair do MVP e entrar em Alpha;
- o time puder trabalhar em cima de epicos e historias concretas, e nao apenas em intencoes gerais.

---

## 13. Relacao com os Documentos Anteriores

Este backlog deriva de:

- [ARQUITETURA_FUNCIONAL_USBRAIN_BUILDER.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ARQUITETURA_FUNCIONAL_USBRAIN_BUILDER.md)
- [ROADMAP_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ROADMAP_USBRAIN.md)
- [ARQUITETURA_RUNTIME_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ARQUITETURA_RUNTIME_USBRAIN.md)
- [FORMATO_PACK_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/FORMATO_PACK_USBRAIN.md)
- [ESPECIFICACAO_CATALOGO_MODELOS.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ESPECIFICACAO_CATALOGO_MODELOS.md)

Ele prepara os proximos documentos de execucao:

- `CONTRATOS_BUILDER_RUNTIME.md`
- backlog inicial do runtime
- milestones tecnicos por sprint
- plano de implementacao do shell `Tauri + Rust`
