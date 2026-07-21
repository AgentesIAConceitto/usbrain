# USBRAIN — Especificacao do Catalogo Vivo de Modelos

## 1. Objetivo

Este documento define a especificacao do `Catalogo Vivo de Modelos` do `USBRAIN`.

O objetivo desta camada e permitir que o `USBRAIN Builder` acompanhe continuamente a evolucao do ecossistema de `LLMs open source` e converta essa complexidade em escolhas praticas para o usuario.

Este documento estabelece:

- o papel do catalogo dentro do produto;
- quais dados precisam existir por modelo;
- como descoberta, avaliacao e recomendacao devem funcionar;
- como o catalogo se integra ao `Builder`, ao `Hub` e ao `Runtime`;
- como o sistema pode evoluir sem depender de listas estaticas e envelhecidas.

---

## 2. Definicao

O `Catalogo Vivo de Modelos` e a camada de inteligencia que monitora, organiza, versiona, compara e recomenda modelos open source compativeis com o ecossistema `USBRAIN`.

Na pratica, ele deve funcionar como:

- base de metadados de modelos;
- camada de comparacao;
- camada de benchmark e observacoes praticas;
- motor de recomendacao contextual;
- ponte entre o mundo caotico dos modelos e a experiencia simples do Builder.

Sem essa camada, o `USBRAIN Builder` envelhece rapidamente.

Com ela, o Builder continua relevante mesmo enquanto surgem dezenas de modelos novos ao longo do tempo.

---

## 3. Papel no Ecossistema

O `Catalogo Vivo de Modelos` nao existe para responder "qual e o melhor modelo do mundo?".

Ele existe para responder perguntas praticas como:

- qual modelo cabe neste pendrive;
- qual modelo funciona melhor em `16GB RAM`;
- qual modelo e melhor para `PT-BR`;
- qual modelo e melhor para uso offline em CPU-only;
- qual combinacao multi-modelo faz mais sentido;
- qual modelo combina melhor com packs e uso semantico.

Em outras palavras:

- o catalogo nao e um ranking abstrato;
- ele e uma camada de decisao aplicada.

---

## 4. Principios do Catalogo

- `Atualizacao continua`
- `Metadados estruturados`
- `Benchmark pratico, nao so marketing`
- `Recomendacao orientada por contexto`
- `Suporte multilingue real`
- `Compatibilidade explicita com formatos do USBRAIN`
- `Cache local e uso offline parcial`
- `Separacao entre dados objetivos e observacoes comunitarias`
- `Transparencia sobre licenca e limitacoes`

---

## 5. Escopo Inicial

Na fase inicial, o catalogo deve focar em modelos:

- compativeis com `GGUF` ou formatos equivalentes suportados pelo ecossistema;
- viaveis para uso local;
- relevantes para `Windows` e `Linux`;
- adequados para `CPU-only`, com `GPU` opcional;
- uteis em `PT`, `EN` e `ES`.

### O que o catalogo nao precisa fazer de inicio

- cobrir todos os modelos do mercado;
- suportar qualquer formato exotico;
- depender de benchmark automatico em todas as maquinas;
- prometer avaliacao perfeita;
- substituir o criterio do usuario.

---

## 6. Camadas do Catalogo

O `Catalogo Vivo de Modelos` deve ser pensado como quatro camadas.

## 6.1 Discovery Layer

Responsavel por encontrar modelos novos, variantes e atualizacoes.

## 6.2 Registry Layer

Responsavel por manter metadados estruturados e versionados.

## 6.3 Evaluation Layer

Responsavel por armazenar benchmarks, observacoes e perfis praticos.

## 6.4 Recommendation Layer

Responsavel por transformar os dados anteriores em recomendacoes para o `Builder`.

---

## 7. Entidades Conceituais

## 7.1 Model Record

Representa um modelo catalogado.

### Campos conceituais principais

- `model_id`
- `family`
- `display_name`
- `provider_or_origin`
- `parameter_scale`
- `primary_format`
- `license`
- `homepage_or_source`
- `description`
- `status`

---

## 7.2 Model Variant

Representa uma variante concreta do modelo.

### Exemplos

- quantizacao diferente;
- variante instruct;
- variante chat;
- variante otimizada para determinado runtime.

### Campos conceituais

- `variant_id`
- `model_id`
- `quantization`
- `file_format`
- `disk_size_gb`
- `context_window`
- `recommended_usage`
- `download_locations`
- `checksums`

---

## 7.3 Model Capability Profile

Representa capacidades praticas do modelo.

### Campos conceituais

- `languages_supported`
- `pt_quality`
- `en_quality`
- `es_quality`
- `rag_affinity`
- `tool_use_affinity`
- `instruction_following_quality`
- `reasoning_profile`
- `hallucination_risk_notes`

---

## 7.4 Model Runtime Profile

Representa informacoes operacionais do modelo em hardware local.

### Campos conceituais

- `estimated_ram_min_gb`
- `estimated_ram_recommended_gb`
- `cpu_only_viability`
- `gpu_optional`
- `recommended_threads_range`
- `preferred_context_range`
- `storage_cost_profile`
- `startup_cost_profile`

---

## 7.5 Benchmark Record

Representa uma medicao concreta ou conjunto de observacoes praticas.

### Campos conceituais

- `benchmark_id`
- `variant_id`
- `environment_class`
- `os`
- `cpu_class`
- `gpu_class`
- `ram_class`
- `tokens_per_second`
- `time_to_first_token_ms`
- `qualitative_notes`
- `source_type`

---

## 7.6 Recommendation Profile

Representa um resultado de recomendacao contextual.

### Campos conceituais

- `profile_id`
- `target_scenario`
- `target_language`
- `hardware_profile`
- `priority_mode`
- `recommended_variants`
- `why`
- `tradeoffs`

---

## 8. Informacoes Minimas por Modelo

Cada modelo no catalogo deve expor pelo menos:

- nome;
- familia;
- parametros;
- quantizacao;
- formato;
- tamanho em disco;
- RAM estimada minima;
- RAM estimada recomendada;
- contexto;
- licenca;
- qualidade geral;
- qualidade em `PT`, `EN` e `ES`;
- afinidade com `RAG`;
- afinidade com `tools`;
- viabilidade em `CPU-only`;
- compatibilidade com o runtime USBRAIN;
- observacoes e limitacoes.

---

## 9. Descoberta de Modelos

O catalogo precisa estar sempre procurando novas opcoes relevantes.

### Tipos de descoberta

- descoberta de novos modelos;
- descoberta de novas variantes;
- descoberta de novas quantizacoes;
- atualizacao de metadados;
- atualizacao de benchmark e observacoes.

### Regras da descoberta

- priorizar modelos com valor pratico real;
- evitar poluicao com duplicacoes inutilmente confusas;
- registrar origem e confiabilidade dos metadados;
- distinguir modelo base de variante concreta.

---

## 10. Registry de Modelos

O `Registry` e a base estruturada do catalogo.

### Funcoes

- armazenar metadados oficiais e normalizados;
- organizar familias e variantes;
- manter historico de alteracoes;
- fornecer dados para o `Builder`;
- separar informacao objetiva de observacoes comunitarias.

### Regra

O `Registry` deve ser pensado como fonte canonica do ecossistema USBRAIN para decisao de modelo.

---

## 11. Evaluation Layer

Os modelos precisam de avaliacao util, nao so descricao.

### A evaluation layer deve armazenar

- benchmarks objetivos;
- observacoes de uso pratico;
- notas por idioma;
- afinidade com uso semantico;
- comportamento em CPU-only;
- custo de inicializacao;
- custo de geracao;
- estabilidade percebida.

### Importante

Nem toda avaliacao precisa vir do mesmo lugar.

O sistema pode combinar:

- benchmarks oficiais do projeto;
- observacoes internas;
- observacoes comunitarias;
- benchmarks curados no Hub.

---

## 12. Recommendation Layer

Essa camada transforma dados em decisao.

### O catalogo deve ajudar o Builder a responder

- qual modelo cabe neste pendrive;
- qual modelo atende este idioma;
- qual modelo atende este perfil de hardware;
- qual modelo atende esta prioridade:
  - velocidade
  - qualidade
  - equilibrio
  - offline maximo
  - packs + RAG

### A recomendacao deve considerar

- espaco do pendrive;
- RAM esperada da maquina hospedeira;
- CPU ou GPU;
- sistema operacional;
- idioma do usuario;
- modo offline, local assistido ou hibrido;
- packs selecionados;
- preferencia entre velocidade e qualidade.

---

## 13. Perfis de Recomendacao

O catalogo deve suportar recomendacao por perfil.

### Exemplos de perfil

- `Pendrive pequeno, foco em velocidade`
- `Pendrive medio, equilibrio geral`
- `Pendrive grande, multi-modelo`
- `PC com 16GB RAM, CPU-only`
- `PC com 64GB RAM, qualidade maior`
- `Melhor em portugues`
- `Melhor para packs semanticos`
- `Melhor para boot rapido`

### Regra

O usuario nao deve ser exposto apenas a uma lista crua de modelos.

Ele deve receber contexto e sugestao pratica.

---

## 14. Recomendacao Multi-Modelo

Como o USBRAIN suporta mais de uma LLM por pendrive, o catalogo precisa recomendar combinacoes.

### Exemplos

- `3B + 7B`
- `modelo pequeno rapido + modelo medio equilibrado`
- `modelo local rapido + fallback cloud opcional`
- `modelo otimizado para PT + modelo geral`

### Objetivo

Permitir ao Builder montar pendrives inteligentes, nao apenas pendrives com um arquivo de modelo.

---

## 15. Suporte Multilingue

O catalogo precisa ser nativamente multilingue.

### Minimo obrigatorio

- `PT`
- `EN`
- `ES`

### Esse suporte precisa existir em

- descricoes publicas;
- metadados de recomendacao;
- notas por idioma;
- explicacoes exibidas no Builder;
- comparacoes de modelos.

### Regra importante

Idioma da interface e uma coisa.

Qualidade do modelo em cada idioma e outra.

As duas informacoes precisam ser exibidas de forma separada e clara.

---

## 16. Integracao com o Builder

O `USBRAIN Builder` depende diretamente desta camada.

### O Builder deve conseguir

- carregar o catalogo local;
- sincronizar atualizacoes;
- pesquisar modelos;
- filtrar modelos por contexto;
- comparar variantes;
- montar recomendacoes;
- calcular espaco;
- montar combinacoes multi-modelo;
- exibir tradeoffs ao usuario.

### Na GUI do Builder

O usuario deve conseguir ver:

- modelos recomendados;
- por que foram recomendados;
- espaco em disco;
- RAM estimada;
- qualidade por idioma;
- afinidade com packs e RAG;
- licenca e limitacoes.

---

## 17. Integracao com o Runtime

O `Runtime` nao precisa consumir o catalogo inteiro, mas deve se beneficiar dele indiretamente.

### Como isso acontece

- o Builder usa o catalogo para escolher modelos;
- o Builder grava isso em `Build Manifest` e `Execution Policy`;
- o Runtime usa os modelos instalados e as politicas geradas.

### Em fases futuras

O runtime pode ate carregar perfis resumidos do catalogo para tuning adaptativo mais refinado.

---

## 18. Integracao com o Hub

O `Hub` e o catalogo de modelos podem se reforcar mutuamente.

### Exemplos

- Hub mostrando packs recomendados para certos modelos;
- catalogo mostrando packs que combinam bem com um modelo;
- Builder combinando sugestoes de modelo + pack;
- benchmarks comunitarios retroalimentando o catalogo.

### Ativo estrategico

O dominio `usbrain.org` faz bastante sentido aqui como base publica para:

- documentacao aberta;
- pagina institucional do projeto;
- catalogo publico de modelos;
- hub comunitario de packs;
- referencias de compatibilidade e benchmark.

---

## 19. Offline, Cache e Atualizacao

O Builder precisa funcionar mesmo sem internet.

### O catalogo deve permitir

- cache local de metadados;
- uso offline do ultimo snapshot;
- sincronizacao incremental quando online;
- indicacao de desatualizacao;
- reaproveitamento de benchmarks e recomendacoes locais.

### Objetivo

Preservar o principio `offline-first` sem sacrificar a atualizacao continua.

---

## 20. Benchmark e Evidencia Pratica

O catalogo nao deve viver apenas de teoria.

### Tipos de metrica relevantes

- `time to first token`
- `tokens per second`
- custo de boot;
- custo de memoria;
- comportamento em CPU-only;
- observacao de estabilidade;
- comportamento com RAG;
- comportamento com prompts em `PT`, `EN` e `ES`.

### Regra

Benchmark no USBRAIN deve sempre tentar responder uma pergunta pratica de produto.

Exemplo:

- esse modelo e rapido o suficiente para uso local?
- esse modelo vale o espaco extra?
- esse modelo entrega melhor experiencia em portugues?

---

## 21. Licenca, Transparencia e Risco

O catalogo deve deixar claras as implicacoes de cada modelo.

### O que precisa estar visivel

- licenca;
- origem;
- restricoes de uso;
- limitacoes conhecidas;
- risco de idioma fraco;
- qualidade ou maturidade do suporte a tools;
- notas relevantes para uso local.

---

## 22. Roadmap Funcional do Catalogo

## 22.1 Fase 1

- registry basico de modelos
- variantes e quantizacoes
- metadados essenciais
- recomendacao contextual simples
- cache local

## 22.2 Fase 2

- benchmarks internos
- afinidade com packs
- qualidade por idioma refinada
- recomendacao multi-modelo

## 22.3 Fase 3

- benchmarks comunitarios curados
- recomendacoes cruzadas com Hub
- insights por hardware profile
- ranking contextual por cenarios reais

---

## 23. Nao Objetivos Iniciais

Para manter foco, o catalogo nao precisa inicialmente:

- indexar tudo que existe no mercado;
- automatizar benchmark completo de todo modelo;
- substituir curadoria humana;
- prometer resposta universal para qualquer hardware;
- depender de servico central para o Builder funcionar.

---

## 24. Resultado Esperado

Quando esta camada estiver implementada, o USBRAIN passara a contar com:

- um catalogo vivo e util de modelos;
- recomendacao contextual real no Builder;
- melhor adaptacao ao hardware do usuario;
- menor risco de obsolescencia do produto;
- base solida para evolucao comunitaria e open source.

---

## 25. Relacao com os Documentos Anteriores

Este documento deriva de:

- [DEFINICAO_PRODUTO_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/DEFINICAO_PRODUTO_USBRAIN.md)
- [ESPECIFICACAO_CONCEITUAL_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ESPECIFICACAO_CONCEITUAL_USBRAIN.md)
- [ARQUITETURA_FUNCIONAL_USBRAIN_BUILDER.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ARQUITETURA_FUNCIONAL_USBRAIN_BUILDER.md)
- [FORMATO_PACK_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/FORMATO_PACK_USBRAIN.md)
- [ESPECIFICACAO_HUB_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ESPECIFICACAO_HUB_USBRAIN.md)

Ele prepara os proximos niveis de detalhamento:

- arquitetura do runtime;
- roadmap geral do USBRAIN;
- contratos do registry de modelos;
- estrategia publica de documentacao e catalogo em `usbrain.org`.
