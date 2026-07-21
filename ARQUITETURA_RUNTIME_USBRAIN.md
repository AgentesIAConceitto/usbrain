# USBRAIN — Arquitetura do Runtime

## 1. Objetivo

Este documento define a arquitetura do `USBRAIN Runtime`.

O objetivo do runtime e transformar o conteudo gravado pelo `USBRAIN Builder` em uma experiencia executavel, adaptativa e realmente util em `Windows` ou `Linux`.

Este documento estabelece:

- o papel do runtime dentro do produto;
- os modulos internos do runtime;
- como acontece o boot do sistema;
- como o hardware e detectado;
- como o modelo local e escolhido;
- como `PACKs`, `RAG`, `Web UI` e integracoes externas entram em funcionamento;
- como o runtime deve equilibrar desempenho, privacidade e robustez.

---

## 2. Definicao

O `USBRAIN Runtime` e o sistema embarcado dentro do pendrive que executa a inteligencia portatil no computador hospedeiro.

Na pratica, o runtime e a camada que:

- detecta o ambiente onde foi iniciado;
- interpreta o `Build Manifest`;
- seleciona o melhor modelo local disponivel;
- ativa os packs instalados;
- sobe agente, busca semantica e interface;
- aplica politicas de execucao;
- oferece operacao offline, local assistida ou hibrida.

O runtime nao e apenas um launcher.

Ele e o orquestrador operacional do USBRAIN.

---

## 3. Papel do Runtime no Ecossistema

O `Builder` cria.

O `Hub` distribui.

O `Catalogo de Modelos` recomenda.

O `Runtime` executa.

Isso significa que o runtime e a camada onde toda a proposta do USBRAIN precisa se materializar de forma confiavel.

Se o runtime falha:

- a adaptacao ao hardware falha;
- a experiencia do usuario final falha;
- o valor do pendrive cai drasticamente.

Por isso, a arquitetura do runtime deve priorizar:

- robustez;
- previsibilidade;
- adaptacao;
- degradacao graciosa;
- experiencia de uso.

---

## 4. Principios Arquiteturais do Runtime

- `Offline por padrao`
- `Adaptacao ao hardware real`
- `Selecao automatica de modelo`
- `Degradacao graciosa`
- `Preferencia por usabilidade sobre saturacao cega`
- `Carregamento seletivo de recursos`
- `Compatibilidade com multi-modelo`
- `Compatibilidade com packs e RAG`
- `Fallback externo apenas por politica`
- `Observabilidade e logs`

---

## 5. Responsabilidades do Runtime

O runtime deve ser responsavel por:

- iniciar o sistema no computador hospedeiro;
- detectar CPU, RAM, GPU e sistema operacional;
- ler `Build Manifest`, `USBRAIN Profile` e `Execution Policy`;
- escolher o melhor modelo local;
- aplicar tuning de execucao;
- ativar `PACKs` elegiveis;
- subir a camada semantica;
- subir o agente;
- expor `Web UI` e API local;
- rotear para nuvem apenas quando permitido.

---

## 6. Visao Geral da Arquitetura

O runtime pode ser entendido como um pipeline operacional.

```text
Boot Manager
   ->
Hardware Detector
   ->
Manifest Loader
   ->
Runtime Policy Engine
   ->
Model Orchestrator
   ->
Pack Loader + Semantic Engine
   ->
Agent Core
   ->
UI/API Server
   ->
Provider Router (opcional)
```

### Interpretacao

- o boot prepara o ambiente;
- o hardware detector entende a maquina real;
- o manifest loader entende o que foi instalado;
- o policy engine decide os limites;
- o model orchestrator escolhe o modelo;
- o pack loader ativa especializacoes;
- o semantic engine sobe a camada vetorial;
- o agent core executa a inteligencia;
- o UI/API server entrega a experiencia;
- o provider router entra apenas quando permitido.

---

## 7. Modulos Principais do Runtime

## 7.1 Boot Manager

Responsavel por iniciar o runtime.

### Responsabilidades

- localizar a estrutura do USBRAIN no pendrive;
- preparar ambiente de execucao;
- validar presenca de arquivos obrigatorios;
- resolver portas locais;
- iniciar a sequencia de boot;
- registrar eventos de inicializacao.

### Objetivo

Garantir boot previsivel e observavel.

---

## 7.2 Hardware Detector

Responsavel por entender a maquina hospedeira.

### Responsabilidades

- detectar sistema operacional;
- detectar RAM total e disponivel;
- detectar CPU fisica e logica;
- detectar GPU disponivel;
- detectar limites praticos do ambiente;
- construir `Hardware Profile` em tempo de execucao.

### Saida

- `Observed Hardware Profile`

### Importancia

Esse modulo e a base da adaptacao automatica.

---

## 7.3 Manifest Loader

Responsavel por carregar a configuracao criada pelo Builder.

### Responsabilidades

- ler `Build Manifest`;
- ler `USBRAIN Profile`;
- ler `Execution Policy`;
- identificar modelos instalados;
- identificar packs instalados;
- identificar credenciais e recursos opcionais;
- verificar integridade minima da instalacao.

---

## 7.4 Runtime Policy Engine

Responsavel por transformar politicas em decisoes operacionais.

### Responsabilidades

- aplicar modo offline/local/hibrido;
- aplicar preferencia por velocidade ou qualidade;
- aplicar restricoes de nuvem;
- aplicar regras de uso de GPU;
- aplicar limites por hardware;
- orientar a selecao de modelo e contexto.

### Exemplos de regra

- nunca usar nuvem;
- usar CPU-only por padrao;
- usar modelo pequeno em hardware fraco;
- usar GPU somente se explicitamente permitido;
- limitar contexto em cenarios de baixa RAM.

---

## 7.5 Model Orchestrator

Responsavel por escolher e preparar o modelo local.

### Responsabilidades

- ler lista de `Installed Models`;
- comparar modelos com o hardware observado;
- aplicar politica do runtime;
- escolher modelo primario;
- escolher fallback local;
- definir tuning inicial;
- preparar execucao por CPU ou GPU.

### Entradas

- `Observed Hardware Profile`
- `Installed Models`
- `Execution Policy`

### Saidas

- `Selected Model`
- `Selected Runtime Tuning`
- `Fallback Plan`

---

## 7.6 Pack Loader

Responsavel por ativar packs instalados.

### Responsabilidades

- ler manifestos dos packs;
- validar compatibilidade em tempo de boot;
- ativar packs elegiveis;
- montar assets de conhecimento;
- registrar comportamento e politicas adicionais;
- informar o agente sobre os packs ativos.

### Regra importante

Nem todo pack precisa estar ativo o tempo todo.

O runtime pode ativar packs:

- por idioma;
- por perfil;
- por dominio;
- por demanda;
- por prioridade configurada.

---

## 7.7 Semantic Engine

Responsavel pela camada semantica local.

### Responsabilidades

- carregar o mecanismo de embedding/index ja preparado;
- abrir os indices vetoriais necessarios;
- realizar busca semantica;
- entregar contexto recuperado ao agente;
- trabalhar com conteudo dos packs.

### Objetivo

Ser o motor de especializacao pratica do USBRAIN.

---

## 7.8 Agent Core

Responsavel por operar a inteligencia conversacional e ferramental.

### Responsabilidades

- manter estado conversacional;
- integrar modelo local;
- integrar RAG;
- integrar packs ativos;
- integrar ferramentas locais;
- produzir respostas no idioma adequado;
- aplicar comportamento configurado.

### Importante

O `Agent Core` e a camada que faz o usuario perceber o USBRAIN como agente e nao apenas como runner de modelo.

---

## 7.9 UI/API Server

Responsavel por expor a interface do runtime.

### Responsabilidades

- servir a `Web UI`;
- expor API local quando necessario;
- exibir estado do sistema;
- expor logs e erros quando apropriado;
- permitir acesso local simples para o usuario final.

### Resultado esperado

A experiencia deve ser simples, clara e pronta para uso.

---

## 7.10 Provider Router

Responsavel por integrar provedores externos quando permitido.

### Responsabilidades

- ler credenciais disponiveis;
- aplicar politica de fallback;
- rotear apenas quando autorizado;
- manter transparencia sobre uso externo;
- preservar prioridade do modo local.

### Regra central

O provider router e opcional.

O runtime precisa continuar valioso mesmo sem ele.

---

## 8. Fluxo de Boot do Runtime

O boot precisa ser previsivel e observavel.

## 8.1 Sequencia recomendada

1. iniciar `Boot Manager`
2. localizar estrutura do USBRAIN
3. validar manifesto basico
4. detectar hardware
5. carregar politica e perfil
6. escolher modelo
7. ativar packs
8. subir semantic engine
9. subir agent core
10. subir Web UI e API local

### Resultado

O usuario final deve perceber o sistema como inicializacao coerente e nao como serie de etapas opacas.

---

## 9. Fluxo de Escolha de Modelo

O runtime precisa escolher modelo sem exigir decisao manual a cada boot.

## 9.1 Entradas para a decisao

- modelos instalados;
- RAM observada;
- CPU observada;
- GPU observada;
- sistema operacional;
- politica do build;
- prioridade entre velocidade e qualidade.

## 9.2 Estrategia de decisao

1. descartar modelos incompatíveis;
2. ranquear elegiveis;
3. escolher modelo primario;
4. preparar fallback local;
5. definir tuning inicial.

## 9.3 Exemplos

- se houver pouca RAM, usar modelo `3B`;
- se houver hardware forte, usar `7B`;
- se GPU estiver permitida e compativel, ativar perfil de GPU;
- se politica for offline estrita, nunca usar cloud fallback.

---

## 10. Tuning de Execucao

O runtime precisa aplicar tuning com foco em usabilidade e nao apenas em ocupacao maxima de recurso.

### O tuning deve considerar

- numero de threads;
- batch size;
- contexto;
- estrategia CPU-only ou GPU;
- limite de memoria;
- custo de inicializacao;
- fluidez do sistema hospedeiro.

### Regra importante

Saturar a maquina nao e objetivo.

O objetivo e obter a melhor experiencia pratica possivel no hardware disponivel.

---

## 11. Modos Operacionais do Runtime

## 11.1 Offline

### Caracteristicas

- sem dependencia de internet;
- LLM local;
- packs locais;
- busca semantica local;
- privacidade maxima.

## 11.2 Local Assistido

### Caracteristicas

- LLM local continua primaria;
- ferramentas locais podem enriquecer a experiencia;
- conectividade pode ser usada quando autorizada.

## 11.3 Hibrido

### Caracteristicas

- LLM local continua como base;
- provedores externos podem complementar;
- fallback acontece por politica explicita;
- o usuario precisa entender quando isso ocorre.

---

## 12. Relacao entre Runtime e Packs

O runtime precisa tratar packs como recursos ativos de especializacao.

### O que isso significa

- o runtime precisa saber quais packs existem;
- quais packs sao elegiveis;
- quais packs devem ser ativados;
- como eles afetam comportamento, conhecimento e ferramentas.

### Tipos de efeito no runtime

- ampliar contexto semantico;
- alterar comportamento do agente;
- declarar ferramentas e workflows;
- impor avisos e politicas;
- alterar relevancia de determinados domínios.

---

## 13. Relacao entre Runtime e Semantic Layer

O valor dos packs depende muito da camada semantica.

### O runtime deve conseguir

- identificar quais indices carregar;
- evitar carregar tudo desnecessariamente;
- trabalhar com busca por idioma;
- combinar resultados de packs diferentes;
- entregar contexto util ao agente.

### Regra

A busca semantica deve ser operacionalmente util e nao apenas conceitualmente presente.

---

## 14. Estado e Sessao

O runtime precisa gerenciar estado.

### Tipos de estado

- estado de boot;
- estado de modelo ativo;
- packs ativos;
- estado da semantic layer;
- estado do agent core;
- estado da UI/API;
- estado do provider router.

### Sessao conversacional

O runtime tambem precisa manter:

- historico;
- contexto recuperado;
- configuracao de idioma;
- metadados de ferramentas e packs ativos.

---

## 15. Observabilidade e Logs

O runtime precisa ter boa observabilidade.

### Logs devem registrar

- inicio e fim do boot;
- hardware detectado;
- modelo selecionado;
- tuning aplicado;
- packs ativados;
- latencia de resposta;
- erros de runtime;
- uso de fallback externo.

### Objetivo

Permitir debug real, especialmente em cenarios portateis e multiplataforma.

---

## 16. Seguranca e Privacidade

O runtime precisa preservar a promessa do USBRAIN.

### Regras centrais

- local por padrao;
- nuvem apenas por configuracao explicita;
- transparencia quando dados saem da maquina;
- separacao entre dados locais e credenciais;
- respeito a politicas do build;
- visibilidade sobre packs sensiveis.

---

## 17. Multilingue no Runtime

O runtime precisa operar de forma nativa em:

- `PT`
- `EN`
- `ES`

### Isso deve afetar

- interface;
- mensagens de boot;
- mensagens de erro;
- comportamento do agente;
- selecao e ativacao de packs por idioma;
- uso de modelo com melhor capacidade linguistica disponivel.

---

## 18. Degradacao Graciosa

O runtime nao pode assumir hardware ideal.

### Se o hardware for limitado, ele deve

- escolher modelo menor;
- reduzir contexto;
- ajustar tuning;
- limitar cargas desnecessarias;
- preservar usabilidade;
- continuar funcional, mesmo que com capacidade reduzida.

### Objetivo

Evitar que o USBRAIN simplesmente "quebre" em hardware fraco.

---

## 19. Integracao com o Builder

O runtime depende daquilo que o `Builder` gravou.

### Contratos principais

- `Build Manifest`
- `USBRAIN Profile`
- `Execution Policy`
- `Installed Models`
- `Pack Manifests`

### Regra

O runtime nao deve depender de suposicoes opacas.

Ele deve operar sobre contratos claros.

---

## 20. Integracao com o Catalogo de Modelos

O runtime nao precisa carregar o catalogo completo, mas se beneficia da logica definida por ele.

### Como isso acontece

- o Builder escolhe modelos com base no catalogo;
- o runtime recebe modelos ja instalados e politicas coerentes;
- em fases futuras, perfis resumidos do catalogo podem ser embutidos para tuning adaptativo.

---

## 21. Integracao com o Hub

O runtime nao precisa depender do hub para executar.

### Regra central

O `Hub` alimenta o ecossistema antes do build.

O `Runtime` executa de forma independente depois do build.

### Beneficio indireto

Melhores packs no hub levam a melhores USBRAINs em runtime.

---

## 22. Requisitos Nao Funcionais do Runtime

O runtime deve ser desenhado para:

- robustez;
- previsibilidade;
- portabilidade;
- logs claros;
- uso local confiavel;
- comportamento auditavel;
- integracao modular;
- operacao util sem internet.

---

## 23. Roadmap Funcional do Runtime

## 23.1 Fase 1

- boot manager
- hardware detector
- model orchestrator basico
- agent core
- web ui local
- semantic layer funcional

## 23.2 Fase 2

- multi-modelo refinado
- pack loader mais inteligente
- tuning adaptativo melhorado
- provider router controlado

## 23.3 Fase 3

- ativacao dinamica de packs
- perfis resumidos de catalogo embutidos
- observabilidade mais rica
- fallback mais sofisticado

---

## 24. Nao Objetivos Iniciais

Para manter foco, o runtime nao precisa inicialmente:

- treinar modelos;
- depender de GPU para existir;
- depender de internet para ser util;
- depender do hub em tempo real;
- suportar toda classe de hardware exotico;
- executar moderacao central obrigatoria.

---

## 25. Resultado Esperado

Quando esta arquitetura estiver implementada, o `USBRAIN Runtime` sera:

- o orquestrador real da inteligencia portatil;
- adaptativo ao hardware;
- compativel com multi-modelo;
- integrado a packs e RAG;
- util em modo offline;
- preparado para operar em modo hibrido quando permitido.

---

## 26. Relacao com os Documentos Anteriores

Este documento deriva de:

- [DEFINICAO_PRODUTO_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/DEFINICAO_PRODUTO_USBRAIN.md)
- [ESPECIFICACAO_CONCEITUAL_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ESPECIFICACAO_CONCEITUAL_USBRAIN.md)
- [ARQUITETURA_FUNCIONAL_USBRAIN_BUILDER.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ARQUITETURA_FUNCIONAL_USBRAIN_BUILDER.md)
- [FORMATO_PACK_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/FORMATO_PACK_USBRAIN.md)
- [ESPECIFICACAO_HUB_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ESPECIFICACAO_HUB_USBRAIN.md)
- [ESPECIFICACAO_CATALOGO_MODELOS.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ESPECIFICACAO_CATALOGO_MODELOS.md)

Ele prepara os proximos documentos:

- roadmap geral do USBRAIN;
- contratos entre Builder e Runtime;
- estrategia de implementacao incremental;
- definicao de milestones tecnicos da plataforma.
