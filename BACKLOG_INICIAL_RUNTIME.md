# USBRAIN Runtime — Backlog Inicial

## 1. Objetivo

Este documento transforma a arquitetura do `USBRAIN Runtime` e os contratos com o `Builder` em um backlog inicial executavel.

O objetivo e sair da definicao arquitetural do runtime e entrar em um plano concreto de implementacao.

Este backlog responde:

- quais epicos formam o `Runtime MVP` e o `Runtime Alpha`;
- quais historias precisam ser entregues primeiro;
- como priorizar boot, leitura de contratos, selecao de modelo e packs;
- quais criterios de pronto devem ser usados;
- como quebrar a implementacao em fases realistas.

---

## 2. Escopo Deste Backlog

Este backlog cobre principalmente:

- `Runtime MVP`
- `Runtime Alpha`

Ou seja:

- boot previsivel;
- leitura dos contratos gravados pelo Builder;
- escolha inicial de modelo;
- execucao local basica;
- ativacao inicial de packs;
- observabilidade minima;
- preparacao para adaptacao futura mais sofisticada.

Este backlog ainda nao detalha completamente:

- runtime hibrido avancado;
- fallback cloud refinado;
- ativacao dinamica sofisticada de packs;
- observabilidade rica;
- multi-modelo avancado;
- benchmark adaptativo em runtime.

---

## 3. Premissas

Este backlog assume:

- o `Builder` grava contratos conforme [CONTRATOS_BUILDER_RUNTIME.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/CONTRATOS_BUILDER_RUNTIME.md);
- o runtime deve funcionar em `Windows` e `Linux`;
- `offline CPU-only` e a base prioritária;
- `GPU` e `cloud` sao opcionais e controlados por politica;
- o runtime precisa continuar valioso mesmo sem Hub nem internet;
- o usuario final deve perceber o USBRAIN como produto funcional, nao como boot tecnico improvisado.

---

## 4. Definicao de Pronto

Uma historia do runtime so deve ser considerada pronta quando:

- comportamento principal foi implementado;
- o fluxo de boot ou operacao correspondente funciona de ponta a ponta;
- erros esperados sao tratados com clareza;
- logs minimos existem para debug;
- o contrato lido ou produzido ficou consistente;
- nao ha diagnosticos triviais abertos nos arquivos alterados.

### Regra pratica

No runtime, "pronto" nao significa apenas "sobe".

Significa "sobe, decide corretamente e falha de forma compreensivel quando necessario".

---

## 5. Priorizacao

Este backlog usa quatro niveis:

- `P0` critico para o `Runtime MVP`
- `P1` importante para fechar o `Runtime MVP` com qualidade
- `P2` necessario para o `Runtime Alpha`
- `P3` desejavel, mas nao bloqueante

---

## 6. Epicos do Runtime MVP

## 6.1 Epico A — Fundacao do Runtime

### Objetivo

Criar a base executavel do runtime como sistema embarcado previsivel.

### Prioridade

- `P0`

### Historias

#### A1. Definir estrutura interna do runtime

- Como mantenedor, quero uma base modular do runtime para que boot, politica, modelos e packs nao fiquem acoplados de forma confusa.

##### Prioridade

- `P0`

##### Criterios de pronto

- estrutura modular inicial existe
- responsabilidades basicas estao separadas
- fluxo de boot principal pode ser organizado por modulos

#### A2. Implementar entrypoint unificado do runtime

- Como sistema, quero um ponto de entrada previsivel para iniciar o USBRAIN no host.

##### Prioridade

- `P0`

##### Criterios de pronto

- entrypoint principal existe
- consegue localizar a raiz do USBRAIN
- inicia a sequencia de boot com logs minimos

#### A3. Definir estados basicos de boot

- Como sistema, quero estados claros de inicializacao para evitar comportamento opaco.

##### Prioridade

- `P1`

##### Criterios de pronto

- estados de boot existem
- transicoes principais ficam registradas
- falhas sabem em que etapa aconteceram

---

## 6.2 Epico B — Boot Manager

### Objetivo

Executar o boot do USBRAIN de forma previsivel e observavel.

### Prioridade

- `P0`

### Historias

#### B1. Detectar a raiz do USBRAIN no host

- Como runtime, quero localizar a estrutura do USBRAIN para resolver caminhos e contratos corretamente.

##### Prioridade

- `P0`

##### Criterios de pronto

- raiz do USBRAIN e encontrada
- o runtime nao depende de caminho absoluto fixo
- erros de localizacao sao tratados

#### B2. Validar estrutura contratual minima

- Como runtime, quero saber se os manifestos minimos existem antes de continuar o boot.

##### Prioridade

- `P0`

##### Criterios de pronto

- arquivos contratuais minimos sao verificados
- ausencia de arquivos obrigatorios gera erro claro

#### B3. Resolver portas e ambiente local

- Como runtime, quero preparar o ambiente local para subir UI, API e componentes auxiliares.

##### Prioridade

- `P1`

##### Criterios de pronto

- estrategia de portas funciona
- conflitos obvios sao tratados
- configuracao basica do ambiente e registrada

---

## 6.3 Epico C — Manifest Loader

### Objetivo

Ler os contratos gravados pelo Builder sem heuristica fragil.

### Prioridade

- `P0`

### Historias

#### C1. Ler `Build Manifest`

- Como runtime, quero iniciar toda a leitura pelo `Build Manifest`.

##### Prioridade

- `P0`

##### Criterios de pronto

- `build-manifest.json` e lido com sucesso
- `schema_version` e validada
- referencias para outros contratos sao resolvidas

#### C2. Ler `USBRAIN Profile`

- Como runtime, quero entender a identidade e o modo desejado do USBRAIN.

##### Prioridade

- `P0`

##### Criterios de pronto

- perfil e carregado
- idioma principal e modo ficam disponiveis ao runtime

#### C3. Ler `Execution Policy`

- Como runtime, quero aplicar restricoes e preferencias definidas pelo Builder.

##### Prioridade

- `P0`

##### Criterios de pronto

- politica e carregada
- flags essenciais ficam acessiveis aos demais modulos

#### C4. Ler `Installed Models Descriptor`

- Como runtime, quero saber exatamente quais modelos existem.

##### Prioridade

- `P0`

##### Criterios de pronto

- lista de modelos instalados e carregada
- caminhos relativos sao resolvidos corretamente

#### C5. Ler `Installed Packs Descriptor`

- Como runtime, quero saber exatamente quais packs existem, quando houver packs instalados.

##### Prioridade

- `P1`

##### Criterios de pronto

- packs sao carregados quando descriptor existir
- ausencia do arquivo nao quebra o boot do MVP

#### C6. Ler `Runtime Config`

- Como runtime, quero obter defaults operacionais para componentes e caminhos.

##### Prioridade

- `P0`

##### Criterios de pronto

- configuracao operacional e carregada
- portas, roots e entrypoints ficam disponiveis

---

## 6.4 Epico D — Hardware Detector

### Objetivo

Detectar o hardware real da maquina hospedeira.

### Prioridade

- `P0`

### Historias

#### D1. Detectar sistema operacional

- Como runtime, quero saber em qual sistema estou rodando para aplicar estrategia correta.

##### Prioridade

- `P0`

##### Criterios de pronto

- `Windows` e `Linux` sao diferenciados corretamente

#### D2. Detectar RAM total e disponivel

- Como runtime, quero adaptar o uso de modelo e contexto a memoria real.

##### Prioridade

- `P0`

##### Criterios de pronto

- RAM total e disponivel sao detectadas
- dados ficam acessiveis ao orquestrador de modelos

#### D3. Detectar CPU fisica e logica

- Como runtime, quero ajustar tuning de execucao com base na CPU real.

##### Prioridade

- `P0`

##### Criterios de pronto

- informacoes basicas de CPU existem
- dados sao entregues ao runtime como `Observed Hardware Profile`

#### D4. Detectar GPU disponivel

- Como runtime, quero saber se GPU existe e se pode ser considerada pela politica.

##### Prioridade

- `P1`

##### Criterios de pronto

- presenca de GPU e detectada em nivel basico
- ausencia de GPU nao quebra o fluxo

---

## 6.5 Epico E — Runtime Policy Engine

### Objetivo

Transformar contratos e hardware observado em regras operacionais de execucao.

### Prioridade

- `P0`

### Historias

#### E1. Aplicar politica offline/local/hibrida

- Como runtime, quero respeitar o modo de execucao escolhido no Builder.

##### Prioridade

- `P0`

##### Criterios de pronto

- modo atual fica claro internamente
- componentes proibidos nao sao ativados

#### E2. Aplicar preferencia CPU-only e GPU

- Como runtime, quero respeitar politicas de uso de CPU e GPU.

##### Prioridade

- `P0`

##### Criterios de pronto

- preferencia por CPU-only e aplicada
- uso de GPU so ocorre se permitido

#### E3. Aplicar prioridade entre velocidade e qualidade

- Como runtime, quero adaptar a estrategia de execucao conforme a prioridade escolhida.

##### Prioridade

- `P1`

##### Criterios de pronto

- politica afeta tuning ou selecao de modelo
- impacto fica visivel nos logs

---

## 6.6 Epico F — Model Orchestrator

### Objetivo

Escolher o melhor modelo disponivel com base no hardware e na politica.

### Prioridade

- `P0`

### Historias

#### F1. Filtrar modelos incompatíveis

- Como runtime, quero descartar modelos que nao fazem sentido para o hardware atual.

##### Prioridade

- `P0`

##### Criterios de pronto

- modelos claramente inviaveis sao removidos da selecao

#### F2. Escolher modelo primario

- Como runtime, quero selecionar um modelo principal sem exigir decisao manual do usuario a cada boot.

##### Prioridade

- `P0`

##### Criterios de pronto

- modelo primario e escolhido
- decisao fica registrada em log

#### F3. Preparar fallback local

- Como runtime, quero ter uma estrategia local de fallback quando houver mais de um modelo instalado.

##### Prioridade

- `P1`

##### Criterios de pronto

- fallback local basico fica definido quando aplicavel

#### F4. Gerar tuning inicial

- Como runtime, quero produzir configuracao inicial de execucao para o modelo selecionado.

##### Prioridade

- `P0`

##### Criterios de pronto

- tuning inicial existe
- contexto, threads e estrategia basica ficam disponiveis

---

## 6.7 Epico G — Pack Loader

### Objetivo

Reconhecer e ativar packs instalados de forma controlada.

### Prioridade

- `P1`

### Historias

#### G1. Ler manifestos de packs instalados

- Como runtime, quero interpretar corretamente os packs instalados pelo Builder.

##### Prioridade

- `P1`

##### Criterios de pronto

- manifestos dos packs sao lidos a partir do descriptor instalado

#### G2. Validar compatibilidade basica de packs

- Como runtime, quero ignorar ou bloquear packs claramente incompatíveis sem quebrar o boot inteiro.

##### Prioridade

- `P1`

##### Criterios de pronto

- incompatibilidades obvias sao detectadas
- comportamento fica registrado

#### G3. Definir packs ativos no boot

- Como runtime, quero ativar apenas os packs elegiveis conforme politica e idioma.

##### Prioridade

- `P1`

##### Criterios de pronto

- lista de packs ativos e produzida
- criterio de ativacao fica logado

---

## 6.8 Epico H — Semantic Engine Basico

### Objetivo

Subir a camada semantica minima para que packs de conhecimento tenham valor pratico.

### Prioridade

- `P1`

### Historias

#### H1. Resolver roots e indices semanticos

- Como runtime, quero localizar assets semanticos instalados pelo Builder.

##### Prioridade

- `P1`

##### Criterios de pronto

- caminhos semanticos sao resolvidos
- ausencia de indice e tratada com clareza

#### H2. Inicializar mecanismo semantico local

- Como runtime, quero disponibilizar busca semantica local ao agente.

##### Prioridade

- `P1`

##### Criterios de pronto

- camada semantica inicializa ou falha de forma controlada

---

## 6.9 Epico I — Agent Core Basico

### Objetivo

Conectar modelo local, camada semantica e experiencia conversacional.

### Prioridade

- `P0`

### Historias

#### I1. Integrar modelo selecionado ao agente

- Como runtime, quero expor ao agente o modelo que foi escolhido no boot.

##### Prioridade

- `P0`

##### Criterios de pronto

- agent core recebe modelo selecionado
- fluxo basico de resposta local existe

#### I2. Integrar contexto semantico ao agente

- Como runtime, quero que o agente possa consumir contexto recuperado quando houver layer semantica ativa.

##### Prioridade

- `P1`

##### Criterios de pronto

- contexto recuperado chega ao agente
- ausencia de contexto nao quebra a resposta

#### I3. Respeitar idioma principal

- Como runtime, quero que o agente opere no idioma principal do USBRAIN quando possivel.

##### Prioridade

- `P1`

##### Criterios de pronto

- idioma principal influencia comportamento inicial

---

## 6.10 Epico J — UI/API Server Basico

### Objetivo

Expor a interface local do USBRAIN para uso real.

### Prioridade

- `P0`

### Historias

#### J1. Subir Web UI local

- Como usuario, quero acessar o USBRAIN por interface local simples.

##### Prioridade

- `P0`

##### Criterios de pronto

- UI local sobe
- runtime expõe url local

#### J2. Expor API local minima

- Como sistema, quero expor uma API local basica quando necessario para o agente e a interface.

##### Prioridade

- `P1`

##### Criterios de pronto

- endpoint local minimo funciona
- erros e indisponibilidades ficam claros

---

## 6.11 Epico K — Logs e Observabilidade Minima

### Objetivo

Permitir diagnostico basico do runtime em ambiente real.

### Prioridade

- `P0`

### Historias

#### K1. Registrar etapas do boot

- Como mantenedor, quero saber exatamente em que etapa o runtime esta ou falhou.

##### Prioridade

- `P0`

##### Criterios de pronto

- etapas de boot sao registradas

#### K2. Registrar hardware e modelo selecionado

- Como mantenedor, quero saber o que foi detectado e qual modelo foi escolhido.

##### Prioridade

- `P0`

##### Criterios de pronto

- dados aparecem no log de forma legivel

#### K3. Registrar erros contratuais e operacionais

- Como mantenedor, quero diagnosticar problemas de contrato, boot e execucao com evidencias reais.

##### Prioridade

- `P1`

##### Criterios de pronto

- erros importantes ficam registrados com contexto suficiente

---

## 7. Epicos do Runtime Alpha

## 7.1 Epico L — Multi-Modelo Inicial

### Objetivo

Sair de um runtime de modelo unico para um runtime que lide melhor com multiplas opcoes locais.

### Prioridade

- `P2`

### Historias

#### L1. Escolher entre mais de um modelo instalado

- Como runtime, quero selecionar o melhor modelo entre varias opcoes locais.

#### L2. Definir fallback local mais inteligente

- Como runtime, quero trocar de estrategia quando o modelo preferido nao fizer sentido para o hardware observado.

#### L3. Tornar a decisao de multi-modelo auditavel

- Como mantenedor, quero entender por que um modelo foi escolhido e outro nao.

---

## 7.2 Epico M — Tuning Adaptativo Inicial

### Objetivo

Refinar a experiencia de execucao conforme hardware e politica.

### Prioridade

- `P2`

### Historias

#### M1. Ajustar contexto dinamicamente

- Como runtime, quero controlar contexto de acordo com memoria e perfil.

#### M2. Ajustar threads e batch de forma adaptativa

- Como runtime, quero escolher um tuning inicial melhor do que um valor estatico.

#### M3. Registrar tuning aplicado

- Como mantenedor, quero ver tuning de execucao com clareza nos logs.

---

## 7.3 Epico N — Packs Mais Inteligentes

### Objetivo

Melhorar a ativacao e a relevancia dos packs em runtime.

### Prioridade

- `P2`

### Historias

#### N1. Ativar pack por idioma

- Como runtime, quero priorizar packs alinhados ao idioma do usuario.

#### N2. Ativar pack por perfil ou dominio

- Como runtime, quero refletir o perfil do USBRAIN na ativacao inicial dos packs.

#### N3. Registrar raciocinio de ativacao

- Como mantenedor, quero entender por que um pack foi ativado ou ignorado.

---

## 7.4 Epico O — Provider Router Inicial

### Objetivo

Preparar o runtime para integracoes externas controladas por politica.

### Prioridade

- `P2`

### Historias

#### O1. Ler `Provider Config`

- Como runtime, quero saber se algum provider externo foi habilitado.

#### O2. Respeitar politica de fallback externo

- Como runtime, quero impedir uso indevido de nuvem quando a politica nao permite.

#### O3. Expor quando o uso externo esta ativo

- Como usuario e mantenedor, quero transparência quando houver possibilidade de saida de dados.

---

## 8. Ordem Recomendada de Implementacao

Para manter foco, a ordem recomendada e:

1. `Epico A — Fundacao do Runtime`
2. `Epico B — Boot Manager`
3. `Epico C — Manifest Loader`
4. `Epico D — Hardware Detector`
5. `Epico E — Runtime Policy Engine`
6. `Epico F — Model Orchestrator`
7. `Epico I — Agent Core Basico`
8. `Epico J — UI/API Server Basico`
9. `Epico K — Logs e Observabilidade Minima`
10. `Epico G — Pack Loader`
11. `Epico H — Semantic Engine Basico`
12. `Epico L — Multi-Modelo Inicial`
13. `Epico M — Tuning Adaptativo Inicial`
14. `Epico N — Packs Mais Inteligentes`
15. `Epico O — Provider Router Inicial`

---

## 9. Proposta de Fases Iniciais

## 9.1 Fase 1 — Boot Contratual

### Foco

- `Epico A`
- `Epico B`
- `Epico C`

### Resultado esperado

- runtime sobe
- encontra a raiz do USBRAIN
- le contratos principais

---

## 9.2 Fase 2 — Adaptacao Basica ao Host

### Foco

- `Epico D`
- `Epico E`
- parte de `Epico F`

### Resultado esperado

- runtime detecta hardware
- aplica politica
- escolhe modelo primario

---

## 9.3 Fase 3 — Execucao Local Minima

### Foco

- restante de `Epico F`
- `Epico I`
- `Epico J`
- parte de `Epico K`

### Resultado esperado

- runtime responde localmente
- UI sobe
- logs basicos existem

---

## 9.4 Fase 4 — Packs e Camada Semantica

### Foco

- `Epico G`
- `Epico H`
- restante de `Epico K`

### Resultado esperado

- runtime reconhece packs
- semantic layer minima funciona
- comportamento de ativacao fica observavel

---

## 9.5 Fase 5 — Alpha do Runtime

### Foco

- `Epico L`
- `Epico M`
- `Epico N`
- `Epico O`

### Resultado esperado

- runtime entra em modo alpha adaptativo
- multi-modelo inicial existe
- tuning melhora
- provider router inicial fica pronto

---

## 10. Dependencias Tecnicas Importantes

### Dependencias de contrato

- sem `Build Manifest`, o runtime nao deve iniciar de forma normal;
- sem `Installed Models Descriptor`, o runtime nao tem base para orquestracao de modelo;
- sem `Execution Policy`, o runtime fica sem limites e preferências claras;
- sem `Runtime Config`, o ambiente operacional fica fragil.

### Dependencias de arquitetura

- `Model Orchestrator` depende de `Hardware Detector` e `Manifest Loader`;
- `Pack Loader` depende de `Installed Packs Descriptor`;
- `Semantic Engine` depende de assets preparados pelo Builder;
- `Agent Core` depende do modelo escolhido e dos modulos anteriores.

---

## 11. Itens Deliberadamente Fora Deste Backlog Inicial

Para manter foco, estes temas nao entram como prioridade do backlog inicial:

- fallback cloud sofisticado;
- benchmark dinamico em tempo real;
- ativacao semantica super refinada de packs;
- telemetria local detalhada;
- multi-modelo avancado com troca dinamica durante sessao;
- integracao em tempo real com o Hub.

Esses temas entram em backlogs seguintes.

---

## 12. Sinal de Sucesso do Backlog Inicial

Este backlog tera cumprido seu papel quando:

- o runtime conseguir subir a partir dos contratos do Builder;
- detectar o hardware com consistencia;
- escolher um modelo coerente;
- expor a interface local;
- ativar packs basicos quando existirem;
- gerar logs suficientes para diagnostico real;
- servir de base para um Runtime Alpha de verdade.

---

## 13. Relacao com os Documentos Anteriores

Este backlog deriva de:

- [ARQUITETURA_RUNTIME_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ARQUITETURA_RUNTIME_USBRAIN.md)
- [CONTRATOS_BUILDER_RUNTIME.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/CONTRATOS_BUILDER_RUNTIME.md)
- [ROADMAP_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ROADMAP_USBRAIN.md)
- [BACKLOG_INICIAL_BUILDER_TAURI_RUST.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/BACKLOG_INICIAL_BUILDER_TAURI_RUST.md)

Ele prepara os proximos documentos e passos:

- milestones tecnicos do runtime;
- backlog de implementacao do `Model Orchestrator`;
- backlog de implementacao de `Pack Loader` e `Semantic Engine`;
- inicio da implementacao real do Builder e do Runtime em paralelo controlado.
