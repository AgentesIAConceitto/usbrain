# Plano de Refatoracao Online-First do Builder

## Contexto

O USBRAIN Builder no PC deve operar em modo online-first:

- pesquisar catalogos de modelos e artefatos na web;
- reutilizar a biblioteca local da maquina quando o artefato ja existir;
- baixar sob demanda quando o artefato ainda nao existir;
- montar um pendrive final offline-first, autonomo e pronto para rodar sem internet.

Este modo online-first precisa ir alem de um snapshot manual:

- o Builder deve descobrir autonomamente modelos open source relevantes na web;
- consolidar benchmarks e observacoes praticas por variante;
- resolver o local real de download do artefato selecionado;
- baixar para a biblioteca local e permitir gestao posterior dessa biblioteca.

Antes desta refatoracao, foi preservado um backup restauravel da versao funcional atual:

- `backups/usbrain-working-macos-20260704-091522`

## Estado Atual Encontrado

### 1. A UI ja comunica o produto correto parcialmente

Em `builder/src/main.ts`:

- existe `get_model_catalog`;
- existe `selectedModelIds`;
- o catalogo mostra `Catalogo remoto` versus `Pronto em models/`;
- a selecao manual de LLMs ja faz parte do fluxo principal.

Conclusao: a UX ja aponta para um Builder com catalogo vivo e biblioteca local.

### 2. O planner ainda depende do modelo offline herdado

Em `builder/src-tauri/src/commands.rs`:

- `discover_local_models()` le apenas o diretorio `models/`;
- `planned_models()` aceita selecao manual, mas exige que o GGUF ja exista localmente;
- se o modelo nao existe em `models/`, o fluxo apenas gera warning e ignora o item.

Trecho estrutural:

```rust
let Some(local_model) = catalog_model_local_source(catalog_model, &available_local_models) else {
    warnings.push(format!(
        "{} ainda nao esta disponivel em models/ para montagem offline.",
        catalog_model.display_name
    ));
    continue;
};
```

Conclusao: a camada de aquisicao ainda nao existe; o planner so resolve artefatos ja presentes.

### 3. A montagem do pendrive ainda copia de cache local pre-populado

Ainda em `builder/src-tauri/src/commands.rs`:

- o build copia runtime Python a partir de `cache/python`;
- copia BGE-M3 de `cache/bge-m3-model`;
- tenta copiar assets opcionais como `llamafile.exe` e `python_runtime.zip` do `cache`;
- copia modelos a partir de `model.source_absolute_path`, que hoje aponta para arquivos ja presentes no workspace/local.

Conclusao: a montagem final presume um Builder previamente abastecido no disco.

### 4. O bundle do app ainda embute artefatos pesados

Em `builder/src-tauri/tauri.conf.json`:

- `../../cache/llamafile.exe`
- `../../cache/python/**/*`
- `../../cache/macos-python-standalone/...`
- `../../cache/bge-m3-model/**/*`
- `../../models/**/*`

Conclusao: o executavel continua pesado porque o bootstrap sob demanda ainda nao foi concluido.

## Acoplamentos Que Precisam Sair

### Backend

- leitura direta de `models/` como fonte primaria de disponibilidade;
- leitura direta de `cache/` como fonte primaria de BGE, Python e runtimes;
- warnings do tipo "nao esta disponivel em models/" como bloqueio implicito da selecao manual;
- manifests que sinalizam `allow_background_downloads` sem uma pipeline real de aquisicao.

### Frontend

- texto que ainda trata `Catalogo remoto` como algo dependente de espelhamento manual;
- secao "Modelos locais selecionados" ainda descrevendo apenas o estado herdado do planner.

### Bundle

- empacotamento de modelos GGUF;
- empacotamento de BGE;
- empacotamento de runtime Python grande quando ele puder ser baixado ou expandido sob demanda;
- dependencia de `cache/` como parte da distribuicao primaria do Builder.

## Arquitetura Alvo

### 1. Biblioteca local da maquina host

Criar uma raiz unica da biblioteca local do Builder, separada do pendrive final, por exemplo:

```text
<builder-library>/
  catalog/
  models/
  embeddings/
  runtimes/
  downloads/
  manifests/
```

Sugestao de categorias:

- `models/`: GGUFs selecionaveis para o pendrive;
- `embeddings/`: BGE-M3 e artefatos equivalentes;
- `runtimes/`: `llamafile`, Python e outros binarios reutilizaveis;
- `catalog/`: snapshots locais e metadados normalizados do catalogo remoto;
- `downloads/`: staging temporario para download, checksum e resume;
- `manifests/`: metadados locais de aquisicao, versao, origem e integridade.

### 1.1 Gestao da biblioteca local

A biblioteca local precisa ser tratada como recurso de produto, nao apenas como cache tecnico.

Operacoes esperadas:

- listar modelos, embeddings e runtimes baixados;
- exibir origem, variante, tamanho, data de aquisicao e estado de integridade;
- mostrar se o artefato esta em uso por algum pendrive/plano recente;
- permitir exclusao para liberar espaco local;
- recalcular espaco ocupado e espaco recuperavel.

### 1.2 Metadados minimos da biblioteca

Cada artefato local deve armazenar pelo menos:

- `artifact_id`
- `artifact_type`
- `display_name`
- `variant`
- `source_url`
- `provider_or_origin`
- `checksum`
- `size_bytes`
- `downloaded_at`
- `last_used_at`
- `integrity_status`
- `local_path`

### 2. Planejamento em duas etapas

O planner precisa deixar de fazer apenas "selecionar se existe" e passar a fazer:

1. resolver a selecao do usuario no catalogo;
2. verificar disponibilidade na biblioteca local;
3. gerar uma lista de aquisicao para o que faltar;
4. apos aquisicao, materializar o plano de montagem offline.

Em vez de retornar warning e ignorar, o planner deve classificar cada item como:

- `available-local`
- `requires-download`
- `unavailable`

### 2.1 Catalogo vivo autonomo

O catalogo precisa deixar de ser somente um snapshot empacotado e passar a ter uma pipeline online:

1. descobrir modelos e variantes em fontes confiaveis;
2. consolidar benchmarks e notas praticas;
3. ranquear por contexto de uso do USBRAIN;
4. resolver URLs de download por variante;
5. persistir snapshots normalizados e reutilizaveis localmente.

Campos minimos por variante para destravar a aquisicao real:

- `download_locations`
- `checksums`
- `provider_or_origin`
- `variant_id`
- `quantization`
- `file_format`
- `license`
- `disk_size_gb`
- `estimated_ram_recommended_gb`
- `benchmark_summary`

### 2.2 Fontes e confianca

O Builder deve trabalhar com fontes online priorizadas e rastreaveis.

Diretrizes:

- priorizar fontes oficiais ou amplamente reconhecidas;
- manter rastreabilidade de onde cada benchmark e cada URL vieram;
- separar benchmark medido pelo USBRAIN de benchmark importado de terceiros;
- nunca promover automaticamente um link sem armazenar origem e integridade esperada.

### 3. Montagem baseada na biblioteca local

Depois da aquisicao:

- o pendrive continua recebendo tudo offline;
- a origem dos arquivos deixa de ser `models/` e `cache/` hardcoded;
- a origem passa a ser a biblioteca local resolvida pelo Builder.

### 4. Builder leve para Git e Windows

O repo deve conter apenas:

- codigo-fonte;
- catalogos/snapshots leves;
- manifests;
- scripts de aquisicao;
- metadados de integridade.

O repo nao deve conter:

- GGUFs pesados;
- caches de BGE;
- runtimes gigantes pre-expandido;
- bundles locais de desenvolvimento.

## Fases de Implementacao

### Fase 1. Introduzir a biblioteca local

- criar funcoes de path para a biblioteca local do Builder;
- separar `source_app_root()` de `builder_library_root()`;
- criar metadados locais por artefato: id, versao, origem, checksum, tamanho, status.

### Fase 2. Resolver catalogo contra biblioteca

- trocar `discover_local_models()` por descoberta na biblioteca local;
- expandir `CatalogModelSummary` com estado de aquisicao;
- fazer `planned_models()` retornar itens disponiveis e itens pendentes de download.

### Fase 3. Implementar aquisicao sob demanda

- baixar GGUF selecionado quando ausente;
- baixar BGE-M3 quando ausente;
- baixar runtime necessario quando ausente;
- validar integridade antes de promover o arquivo para a biblioteca local.

### Fase 3.1 Descoberta e enriquecimento do catalogo

- modelar a entidade de variante com `download_locations` e `checksums`;
- criar pipeline de descoberta online e consolidacao local;
- anexar benchmark resumido e origem confiavel por variante;
- permitir refresh manual e refresh em background do catalogo vivo.

### Fase 3.2 Administracao da biblioteca local

- criar comandos para listar artefatos locais;
- criar comandos para excluir artefatos com seguranca;
- mostrar tamanho total e espaco recuperavel;
- impedir exclusao silenciosa de artefato em uso por operacao ativa.

### Fase 4. Rebasear a montagem do pendrive

- copiar modelos da biblioteca local;
- copiar embeddings da biblioteca local;
- copiar runtimes da biblioteca local;
- manter o pendrive final 100% offline e autonomo.

### Fase 5. Enxugar o bundle Tauri

- remover `models/**/*` dos resources;
- remover `cache/bge-m3-model/**/*`;
- reduzir o empacotamento de runtimes ao minimo necessario para o bootstrap;
- manter apenas o que for indispensavel para o proprio app iniciar.

## Primeiros Arquivos a Refatorar

1. `builder/src-tauri/src/commands.rs`
2. `builder/src/main.ts`
3. `builder/src-tauri/tauri.conf.json`

## Resultado Esperado

Ao final desta refatoracao:

- o Builder podera ser versionado no Git sem LLMs e sem caches pesados;
- um clone limpo no Windows podera gerar o `.exe` do Builder sem embutir modelos;
- o usuario do Builder podera pesquisar catalogos online, reutilizar artefatos locais, baixar o que faltar e administrar a biblioteca local da maquina;
- o pendrive final continuara sendo offline-first e autonomo, que e a premissa central do USBRAIN.
