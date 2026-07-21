# USBRAIN — Definicao Oficial do Produto

## 1. Definicao

O `USBRAIN` e uma plataforma open source para criar, executar e compartilhar agentes portateis e especializados, capazes de operar em `Windows` ou `Linux`, com suporte a `LLMs locais`, `busca semantica`, `packs especialistas`, `integracoes opcionais em nuvem` e `adaptacao automatica ao hardware disponivel`.

O produto real nao e apenas um pendrive pronto.

O produto real e composto por:

- `USBRAIN Builder`
- `USBRAIN Runtime`
- `USBRAIN Packs`
- `USBRAIN Hub`
- `USBRAIN Model Intelligence Layer`

O pendrive gerado e o artefato final criado por essa plataforma.

---

## 2. Tese Central

Modelos open source pequenos e medios tornam-se muito mais uteis quando combinados com:

- recuperacao semantica local;
- conhecimento especializado empacotado;
- selecao automatica de modelo conforme hardware;
- modo offline como padrao;
- fallback opcional para provedores externos;
- uma comunidade ativa compartilhando configuracoes e packs.

Em vez de depender de uma unica LLM enorme, o `USBRAIN` permite combinar:

- uma ou mais LLMs locais;
- uma base vetorial semantica;
- comportamento de agente;
- ferramentas;
- packs especialistas;
- configuracoes hibridas.

---

## 3. Missao

Permitir que qualquer pessoa crie o seu proprio agente portatil, privado, adaptavel e especializado.

---

## 4. Visao

Tornar-se a principal plataforma open source para criacao de inteligencias pessoais e especialistas portateis, executaveis em hardware diverso, com forte autonomia do usuario sobre:

- seus dados;
- seus modelos;
- seus packs;
- suas integracoes;
- sua experiencia de uso.

---

## 5. Principios

- `Offline por padrao`
- `Privacidade em primeiro lugar`
- `Open source desde a origem`
- `Adaptacao ao hardware real`
- `Portabilidade pratica`
- `Multilingue nativo`
- `Evolucao continua do catalogo de modelos`
- `Especializacao via packs`
- `Hibrido por opcao, nao por dependencia`
- `Contribuicao comunitaria como parte do produto`

---

## 6. Escopo do Produto

O `USBRAIN` deve permitir que um usuario:

1. execute uma aplicacao em `Windows` ou `Linux`;
2. conecte um pendrive;
3. descubra quanto espaco e performance esse pendrive oferece;
4. escolha uma ou mais LLMs compativeis com o espaco disponivel;
5. configure comportamento, idioma, packs e integracoes;
6. gere um pendrive `USBRAIN` pronto para uso;
7. plugue esse pendrive em outra maquina;
8. tenha o runtime escolhendo automaticamente a melhor configuracao para o hardware encontrado.

---

## 7. Estrutura Conceitual

### 7.1 USBRAIN Builder

Aplicacao responsavel por criar o pendrive.

Responsabilidades:

- detectar pendrive conectado;
- medir capacidade disponivel;
- validar sistema de arquivos e requisitos;
- sugerir combinacoes de modelos;
- baixar ou reutilizar modelos e runtimes em cache;
- instalar o runtime do USBRAIN;
- instalar packs selecionados;
- configurar idioma, persona e provedores externos;
- validar o pendrive gerado.

### 7.2 USBRAIN Runtime

Sistema que roda dentro do pendrive.

Responsabilidades:

- detectar o hardware da maquina hospedeira;
- escolher a melhor LLM local disponivel;
- definir configuracao de execucao por CPU, RAM e GPU;
- iniciar agente, motor semantico e interface;
- operar offline quando necessario;
- permitir fallback para nuvem quando autorizado pelo usuario.

### 7.3 USBRAIN Packs

Pacotes instalaveis que adicionam conhecimento, comportamento, ferramentas ou combinacoes desses elementos.

### 7.4 USBRAIN Hub

Espaco comunitario para descoberta, publicacao e distribuicao de packs, presets e referencias de modelos.

### 7.5 Model Intelligence Layer

Camada responsavel por acompanhar a evolucao das LLMs open source e manter um catalogo vivo, atualizado e util para recomendacao.

---

## 8. Builder: Capacidades Obrigatorias

O `USBRAIN Builder` deve:

- rodar pelo menos em `Windows` e `Linux`;
- suportar interface no minimo em `Portugues`, `Ingles` e `Espanhol`;
- identificar o pendrive disponivel;
- mostrar espaco livre, espaco total e estimativa de uso;
- sugerir quais modelos cabem;
- permitir instalar mais de uma LLM no mesmo pendrive;
- permitir modos de construcao como:
  - `offline CPU-only`
  - `GPU opcional`
  - `hibrido local + nuvem`
- permitir configuracao inicial do agente;
- permitir escolha de packs;
- permitir inserir chaves de provedores externos de forma opcional e segura;
- gerar um runtime que selecione automaticamente o melhor modelo local para cada maquina.

---

## 9. Runtime: Capacidades Obrigatorias

O runtime do `USBRAIN` deve:

- iniciar de forma simples para o usuario final;
- detectar RAM, CPU e GPU disponiveis;
- escolher entre modelos locais instalados no pendrive;
- usar CPU-only como modo padrao garantido;
- permitir uso de GPU quando habilitado e compativel;
- permitir fallback para provedores externos quando configurado;
- operar com busca semantica local;
- carregar configuracoes e packs instalados pelo Builder;
- manter o agente funcional mesmo sem internet.

---

## 10. Modo Multi-Modelo

Um mesmo pendrive deve poder conter mais de uma LLM.

Exemplos:

- `3B` para maquinas com pouca RAM;
- `7B` para maquinas mais fortes;
- um modelo pequeno focado em velocidade;
- um modelo maior focado em qualidade;
- um modelo local + uma configuracao de fallback em nuvem.

O runtime deve decidir automaticamente qual modelo usar com base em:

- RAM disponivel;
- CPU disponivel;
- GPU disponivel;
- preferencia do usuario;
- perfil de uso configurado;
- politicas do pendrive criado.

---

## 11. Modo Offline, Local e Hibrido

O USBRAIN deve suportar tres modos principais:

### 11.1 Offline

- sem internet;
- LLM local;
- busca semantica local;
- packs locais;
- privacidade maxima.

### 11.2 Local Assistido

- LLM local como principal;
- uso opcional de web;
- enriquecimento via ferramentas locais.

### 11.3 Hibrido

- LLM local como base;
- fallback ou expansao com provedores externos;
- uso controlado de APIs como `OpenAI`, `Claude`, `Gemini`, `DeepSeek` e outras.

O modo hibrido e opcional. O valor do USBRAIN precisa existir mesmo sem ele.

---

## 12. Packs Especialistas

Os `Packs Especialistas` sao uma das consequencias centrais do projeto.

Eles existem para permitir que uma LLM base seja especializada por dominio sem exigir que o modelo base carregue todo o conhecimento no proprio treinamento.

Um pack pode incluir:

- documentos;
- base de conhecimento curada;
- indice vetorial;
- instrucoes de comportamento;
- ferramentas recomendadas;
- limites e avisos de seguranca;
- metadados de idioma;
- versao e compatibilidade.

### 12.1 Tipos de Packs

- `Knowledge Pack`
- `Behavior Pack`
- `Tool Pack`
- `Hybrid Pack`

### 12.2 Exemplos de Packs

- direito;
- medicina alternativa;
- animes;
- estrategia de `Mobile Legends`;
- manutencao de computadores;
- culinaria;
- negocios;
- idiomas;
- concursos;
- educacao tecnica;
- comunidades de nicho.

### 12.3 Papel dos Packs

Os packs nao sao extras cosmeticos.

Eles sao parte central da proposta de valor do USBRAIN, porque transformam a plataforma em um ecossistema de inteligencias especializadas portateis.

---

## 13. USBRAIN Hub

O `USBRAIN Hub` deve ser pensado como o espaco comunitario do projeto.

Funcoes desejadas:

- catalogo de packs;
- publicacao de packs pela comunidade;
- descoberta por categoria;
- compatibilidade por versao;
- licenca e autoria;
- avaliacao comunitaria;
- tags por idioma;
- tamanho em disco;
- dependencia de ferramentas ou modelos;
- destaque para packs bem curados.

O hub pode futuramente tambem incluir:

- presets de Builder;
- perfis de hardware;
- recomendacoes de modelos;
- packs oficiais e packs comunitarios.

---

## 14. Catalogo Vivo de Modelos

O `USBRAIN Builder` precisa conter uma camada viva de inteligencia de modelos.

Sem isso, o produto envelhece rapidamente.

### 14.1 Objetivo

Monitorar e organizar continuamente o ecossistema de modelos open source para apoiar a criacao de USBRAINs atualizados e bem ajustados ao hardware do usuario.

### 14.2 O que essa camada precisa fazer

- descobrir modelos novos;
- atualizar metadados de modelos existentes;
- acompanhar variantes e quantizacoes;
- armazenar benchmarks e caracteristicas praticas;
- recomendar modelos adequados por caso de uso;
- permitir ao usuario escolher com seguranca.

### 14.3 Informacoes minimas por modelo

- nome;
- familia;
- parametros;
- quantizacao;
- tamanho em disco;
- RAM estimada;
- qualidade geral;
- desempenho em CPU;
- desempenho em GPU;
- afinidade com RAG;
- afinidade com uso offline;
- qualidade em `PT`, `EN` e `ES`;
- licenca;
- contexto;
- observacoes e limitacoes.

### 14.4 Recomendacao orientada por contexto

O Builder nao deve sugerir o "melhor modelo do mundo".

Ele deve sugerir:

- o melhor modelo para aquele pendrive;
- o melhor modelo para aquele hardware;
- o melhor modelo para aquela prioridade;
- o melhor modelo para aquele idioma;
- o melhor modelo para uso local, offline ou hibrido.

---

## 15. Multilingue Nativo

O USBRAIN deve oferecer suporte nativo a:

- `Portugues`
- `Ingles`
- `Espanhol`

Esse suporte precisa existir em tres niveis:

### 15.1 Interface

- Builder
- Runtime
- Web UI
- mensagens de boot
- recomendacoes de modelos
- mensagens de erro

### 15.2 Conteudo e descoberta

- descricoes de modelos;
- descricoes de packs;
- categorias do hub;
- documentacao publica.

### 15.3 Experiencia do agente

- prompts e instrucoes;
- comportamento do runtime;
- metadados de idioma dos packs;
- recomendacoes adequadas ao idioma do usuario.

---

## 16. Open Source no GitHub

O projeto deve ser publicado no `GitHub` como open source apos sua conclusao.

Essa decisao afeta diretamente a definicao do produto.

O USBRAIN deve nascer preparado para:

- contribuicoes de codigo;
- contribuicoes de packs;
- contribuicoes de benchmarks e catalogo de modelos;
- contribuicoes de traducao e localizacao;
- fork por comunidades e nichos.

### 16.1 Implicacoes praticas

- arquitetura modular;
- formatos abertos para packs;
- manifestos versionados;
- politicas claras de compatibilidade;
- licencas explicitas;
- documentacao forte;
- onboarding para contribuidores;
- seguranca para tratamento de chaves e segredos.

---

## 17. Usuarios do Produto

O USBRAIN deve servir pelo menos a estes perfis:

### 17.1 Criador de USBRAIN

Pessoa que usa o Builder para criar um pendrive personalizado.

### 17.2 Usuario Final

Pessoa que usa o pendrive pronto em diferentes maquinas.

### 17.3 Curador de Packs

Pessoa ou grupo que organiza conhecimento especializado em packs.

### 17.4 Contribuidor de Modelos e Benchmarks

Pessoa que ajuda a manter o catalogo vivo de modelos atualizado.

### 17.5 Comunidade de Nicho

Grupo que constroi variantes especializadas do USBRAIN para seu proprio dominio.

---

## 18. Diferencial Estrategico

O diferencial do USBRAIN nao esta apenas em "rodar uma LLM em um pendrive".

O diferencial esta em combinar:

- portabilidade;
- adaptacao ao hardware;
- multiplos modelos;
- conhecimento semantico local;
- packs especialistas;
- opcao de integracao com nuvem;
- comunidade open source.

Isso transforma o USBRAIN em uma plataforma de criacao e distribuicao de inteligencias portateis, pessoais e especializadas.

---

## 19. Nao Objetivos Iniciais

Para manter foco, o USBRAIN nao precisa inicialmente:

- competir com os maiores provedores em tamanho bruto de modelo;
- depender de GPU para ser util;
- depender de servicos online para existir;
- centralizar tudo em infraestrutura proprietaria;
- exigir expertise tecnica alta do usuario final.

---

## 20. Definicao Curta de Referencia

`USBRAIN e uma plataforma open source para criar agentes portateis e especializados, combinando LLMs locais, busca semantica, packs instalaveis, um catalogo vivo de modelos e suporte nativo a Portugues, Ingles e Espanhol, com execucao adaptativa em Windows e Linux.`

---

## 21. Fase Atual do Projeto

O projeto atual validou o fundamento tecnico:

- LLM local;
- BGE-M3;
- Web UI;
- agente;
- runtime portatil;
- adaptacao ao hardware.

A proxima fase do USBRAIN nao e mais provar que a base tecnica funciona.

A proxima fase e transformar essa base em uma plataforma open source completa de criacao, execucao e distribuicao de inteligencias portateis especializadas.
