# USBRAIN — Contratos entre Builder e Runtime

## 1. Objetivo

Este documento define os contratos formais entre o `USBRAIN Builder` e o `USBRAIN Runtime`.

O objetivo e garantir que:

- o `Builder` grave artefatos previsiveis;
- o `Runtime` consiga interpretar esses artefatos sem suposicoes opacas;
- a evolucao de ambos aconteca com baixo acoplamento;
- o ecossistema possa crescer sem depender de comportamento informal.

Este documento responde:

- quais arquivos e manifestos o Builder deve gerar;
- quais dados o Runtime deve conseguir consumir;
- quais campos sao obrigatorios, opcionais e evolutivos;
- como compatibilidade e versionamento devem funcionar;
- como modelos, packs, politicas e credenciais entram no contrato.

---

## 2. Principio Central

O `Runtime` nao deve adivinhar o que o `Builder` quis fazer.

O `Builder` nao deve depender de detalhes internos do `Runtime` que nao estejam explicitados.

A relacao entre os dois precisa existir por `contratos versionados`.

Esses contratos devem ser:

- claros;
- auditaveis;
- legiveis;
- evolutiveis;
- tolerantes a extensao.

---

## 3. Artefatos Contratuais Principais

Os contratos entre Builder e Runtime se organizam em torno destes artefatos:

- `Build Manifest`
- `USBRAIN Profile`
- `Execution Policy`
- `Installed Models Descriptor`
- `Installed Packs Descriptor`
- `Runtime Config`
- `Provider Config`
- `Validation Report`

Na fase inicial, nem todos precisam ser arquivos separados.

Mas conceitualmente eles devem existir.

---

## 4. Regras Gerais de Contrato

### 4.1 Regras de formato

- preferir formatos abertos e legiveis, como `JSON` ou `YAML`;
- evitar binarios opacos para dados de configuracao;
- manter nomes de campos estaveis;
- permitir extensao sem quebrar leitura antiga;
- sempre registrar `schema_version`.

### 4.2 Regras de compatibilidade

- o Runtime deve validar versao do contrato antes de continuar;
- o Builder deve declarar qual versao de contrato esta gravando;
- campos desconhecidos devem ser ignorados quando seguro;
- ausencia de campos obrigatorios deve bloquear boot ou acionar erro claro.

### 4.3 Regras de responsabilidade

- o Builder decide e grava;
- o Runtime le, valida e executa;
- o Runtime pode complementar estado observado, mas nao deve reescrever a intencao do Builder sem motivo explicito.

---

## 5. Estrutura Contratual Minima no Pendrive

Uma estrutura conceitual minima recomendada:

```text
/usbrain/
  manifests/
    build-manifest.json
    usbrain-profile.json
    execution-policy.json
    installed-models.json
    installed-packs.json
    runtime-config.json
    provider-config.json
    validation-report.json
  runtime/
  models/
  packs/
  semantic/
  logs/
```

### Regra

Os caminhos exatos podem evoluir, mas o `Runtime` deve saber localizar os contratos em lugar previsivel.

---

## 6. Build Manifest

O `Build Manifest` e o contrato raiz.

Ele representa o resultado global do build executado pelo Builder.

### Responsabilidade

- identificar o build;
- identificar a versao do schema;
- apontar para os demais contratos;
- resumir o que foi instalado;
- servir como ponto inicial de leitura do Runtime.

### Campos conceituais minimos

- `schema_version`
- `build_id`
- `build_timestamp`
- `builder_version`
- `target_os_family`
- `target_device_profile`
- `profile_ref`
- `execution_policy_ref`
- `installed_models_ref`
- `installed_packs_ref`
- `runtime_config_ref`
- `provider_config_ref`
- `validation_report_ref`
- `build_mode`
- `languages`

### Campos opcionais uteis

- `catalog_snapshot_id`
- `hub_snapshot_id`
- `notes`
- `created_by`

### Exemplo conceitual

```json
{
  "schema_version": "1.0",
  "build_id": "usb-2026-06-24-001",
  "build_timestamp": "2026-06-24T19:20:00Z",
  "builder_version": "0.1.0",
  "target_os_family": ["windows", "linux"],
  "target_device_profile": {
    "label": "USB 64GB",
    "capacity_gb": 57.8
  },
  "profile_ref": "manifests/usbrain-profile.json",
  "execution_policy_ref": "manifests/execution-policy.json",
  "installed_models_ref": "manifests/installed-models.json",
  "installed_packs_ref": "manifests/installed-packs.json",
  "runtime_config_ref": "manifests/runtime-config.json",
  "provider_config_ref": "manifests/provider-config.json",
  "validation_report_ref": "manifests/validation-report.json",
  "build_mode": "offline-first",
  "languages": ["pt-BR", "en", "es"]
}
```

---

## 7. USBRAIN Profile

O `USBRAIN Profile` representa a identidade e a intencao principal do USBRAIN criado.

### Responsabilidade

- descrever o perfil escolhido pelo usuario;
- registrar idioma principal;
- registrar modo desejado de uso;
- carregar preferencias mais conceituais do que operacionais.

### Campos conceituais minimos

- `profile_id`
- `name`
- `primary_language`
- `supported_languages`
- `mode`
- `privacy_mode`
- `gpu_preference`
- `personality_profile`
- `agent_profile`

### Campos opcionais uteis

- `description`
- `default_pack_collection`
- `owner_label`

### Exemplos de `mode`

- `offline`
- `local-assisted`
- `hybrid`

### Exemplos de `privacy_mode`

- `strict-local`
- `local-first`
- `hybrid-explicit`

---

## 8. Execution Policy

A `Execution Policy` representa as regras operacionais que o Runtime deve respeitar.

### Responsabilidade

- limitar ou permitir comportamentos;
- definir preferencias de adaptacao;
- orientar escolha de modelo e fallback;
- controlar uso de cloud e GPU.

### Campos conceituais minimos

- `policy_id`
- `allow_cloud_fallback`
- `allow_gpu`
- `prefer_cpu_only`
- `priority_mode`
- `fallback_strategy`
- `max_active_model_count`
- `context_strategy`
- `pack_activation_strategy`

### Campos opcionais uteis

- `max_memory_pressure_level`
- `prefer_low_latency`
- `allow_background_downloads`

### Exemplos de `priority_mode`

- `speed`
- `quality`
- `balanced`
- `offline-max`
- `rag-oriented`

### Exemplos de `fallback_strategy`

- `none`
- `local-only`
- `local-then-cloud`
- `user-confirmed-cloud`

### Exemplos de `context_strategy`

- `adaptive`
- `conservative`
- `max-quality`

### Exemplos de `pack_activation_strategy`

- `all-compatible`
- `language-first`
- `on-demand`
- `profile-priority`

---

## 9. Installed Models Descriptor

O `Installed Models Descriptor` registra os modelos efetivamente instalados no pendrive.

### Responsabilidade

- permitir que o Runtime saiba o que esta disponivel;
- descrever metadados minimos de cada modelo;
- indicar modelo preferencial e possiveis fallbacks.

### Estrutura conceitual

- lista de `Installed Model`
- um campo opcional de `default_model_id`

### Campos conceituais por modelo

- `installed_model_id`
- `catalog_model_id`
- `variant_id`
- `display_name`
- `relative_path`
- `disk_size_gb`
- `format`
- `quantization`
- `role`
- `languages`
- `recommended_ram_min_gb`
- `recommended_ram_gb`
- `cpu_only_viable`
- `gpu_viable`
- `runtime_family`

### Exemplos de `role`

- `primary`
- `fallback`
- `fast-small`
- `quality-large`

### Regras

- o `relative_path` deve ser relativo a uma raiz previsivel do USBRAIN;
- o Runtime nao deve depender de heuristica de nome de arquivo;
- `catalog_model_id` deve existir sempre que o modelo vier do catalogo;
- modelos locais manuais podem usar identificador proprio, mas ainda devem respeitar a estrutura.

---

## 10. Installed Packs Descriptor

O `Installed Packs Descriptor` registra os packs instalados no build.

### Responsabilidade

- permitir que o Runtime identifique packs disponiveis;
- apontar manifestos dos packs;
- indicar estrategias de ativacao.

### Campos conceituais por pack

- `installed_pack_id`
- `pack_id`
- `name`
- `version`
- `pack_type`
- `relative_manifest_path`
- `relative_root_path`
- `languages`
- `category`
- `activation_mode`
- `dependencies`
- `sensitivity_level`

### Exemplos de `activation_mode`

- `always`
- `language-matched`
- `on-demand`
- `policy-controlled`

### Regra

O Runtime deve ler os manifestos dos packs instalados a partir deste descriptor e nao por varredura cega do filesystem.

---

## 11. Runtime Config

O `Runtime Config` representa configuracoes operacionais preparadas pelo Builder para facilitar o boot do Runtime.

### Responsabilidade

- fornecer defaults iniciais;
- apontar componentes base;
- declarar portas, hosts e caminhos esperados;
- desacoplar a logica do Runtime da topologia fisica do pendrive.

### Campos conceituais minimos

- `runtime_version`
- `default_ui_port`
- `default_api_port`
- `default_host`
- `runtime_root`
- `models_root`
- `packs_root`
- `semantic_root`
- `logs_root`
- `agent_entrypoint`
- `semantic_backend`
- `llm_runtime_family`

### Campos opcionais uteis

- `preferred_log_level`
- `boot_timeout_profile`
- `port_strategy`

### Regra

`Runtime Config` nao substitui politica nem manifestos.

Ele apenas descreve o ambiente operacional esperado.

---

## 12. Provider Config

O `Provider Config` trata provedores externos opcionais.

### Responsabilidade

- indicar que provedores foram configurados;
- registrar politica de uso externo;
- separar configuracao de disponibilidade real;
- evitar que o Runtime tente usar nuvem sem base contratual.

### Campos conceituais minimos

- `providers_enabled`
- `fallback_policy`
- `credential_mode`
- `providers`

### Campos conceituais por provider

- `provider_id`
- `enabled`
- `credential_ref`
- `model_preferences`
- `usage_policy`

### Regras importantes

- o contrato pode apontar para credencial sem armazenar a credencial em texto puro;
- `Provider Config` nao deve expor segredos abertamente;
- em modo `strict-local`, esse arquivo pode existir com tudo desativado.

---

## 13. Validation Report

O `Validation Report` e o resumo final da validacao feita pelo Builder.

### Responsabilidade

- indicar se o build terminou consistente;
- registrar avisos e falhas nao bloqueantes;
- facilitar diagnostico inicial.

### Campos conceituais minimos

- `validation_status`
- `validated_at`
- `builder_version`
- `checks_performed`
- `warnings`
- `errors`

### Exemplos de `validation_status`

- `passed`
- `passed-with-warnings`
- `failed`

### Regra

O Runtime nao deve confiar cegamente no `Validation Report`, mas pode usa-lo como sinal diagnostico.

---

## 14. Contrato dos Manifestos de Pack

Cada `Pack Manifest` ja e definido por [FORMATO_PACK_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/FORMATO_PACK_USBRAIN.md).

### Regra contratual

O `Builder` deve instalar o manifesto do pack sem alteracao semantica indevida.

O `Runtime` deve consumir o manifesto do pack como fonte oficial daquele pack.

### O que o Builder pode acrescentar

- metadados de instalacao;
- caminho relativo da raiz instalada;
- informacao de ativacao local.

### O que nao deve fazer

- reescrever arbitrariamente identidade do pack;
- quebrar compatibilidade do manifesto original;
- misturar campos internos do runtime dentro do manifesto publico do pack.

---

## 15. Contrato de Compatibilidade

Compatibilidade precisa ser tratada como dado contratual.

### O Builder deve declarar

- versao do schema de contratos;
- versao minima de runtime esperada;
- versao do Builder que gerou o pendrive.

### O Runtime deve validar

- schema compativel;
- campos obrigatorios presentes;
- versao de runtime suficiente;
- existencia dos arquivos referenciados.

### Resultado da validacao

- continuar boot;
- continuar com degradacao controlada;
- ou falhar com erro claro.

---

## 16. Versionamento dos Contratos

Cada contrato deve conter `schema_version`.

### Regra recomendada

- usar versionamento semantico simplificado;
- `1.x` para evolucoes compativeis;
- `2.0` para ruptura de contrato.

### Comportamento esperado

- `Runtime 1.2` deve ler `schema 1.0` e `1.1` quando possivel;
- `Runtime 1.x` nao deve fingir que entende `schema 2.0`;
- `Builder` deve registrar claramente a versao que gravou.

---

## 17. Regras de Caminho e Referencia

Para evitar fragilidade entre sistemas:

- referencias devem ser relativas a raiz do USBRAIN quando possivel;
- evitar caminhos absolutos do sistema hospedeiro;
- evitar dependencia de letra de drive fixa ou ponto de montagem fixo;
- o Runtime deve resolver caminhos a partir da raiz detectada.

### Objetivo

Permitir que o mesmo pendrive funcione em `Windows` e `Linux`.

---

## 18. Regras de Seguranca de Dados

### O que nao deve ficar em texto puro sem necessidade

- segredos de provedores externos;
- tokens temporarios;
- dados pessoais desnecessarios.

### O que pode ficar em manifesto

- ids;
- preferencias;
- referencias seguras;
- politicas;
- flags de ativacao;
- caminhos relativos.

### Regra

Manifestos contratuais devem ser legiveis e auditaveis, mas nao devem vazar segredos.

---

## 19. Contratos Minimos para o Builder MVP

Para o `Builder MVP`, o conjunto minimo recomendado e:

- `build-manifest.json`
- `usbrain-profile.json`
- `execution-policy.json`
- `installed-models.json`
- `runtime-config.json`
- `validation-report.json`

### Opcional no MVP, mas desejavel logo depois

- `installed-packs.json`
- `provider-config.json`

---

## 20. Contratos Minimos para o Runtime Alpha

Para o `Runtime Alpha`, o Runtime deve consumir pelo menos:

- `Build Manifest`
- `USBRAIN Profile`
- `Execution Policy`
- `Installed Models Descriptor`
- `Installed Packs Descriptor`, quando existir
- `Runtime Config`

### Resultado esperado

O runtime consegue:

- identificar o build;
- saber que modelos existem;
- saber que packs existem;
- aplicar politica;
- inicializar ambiente coerente.

---

## 21. Fluxo Contratual de Build e Boot

### No Builder

1. usuario escolhe perfil
2. Builder escolhe modelos e packs
3. Builder monta `BuildPlan`
4. Builder grava runtime e assets
5. Builder gera manifestos
6. Builder valida e gera `Validation Report`

### No Runtime

1. runtime detecta raiz do USBRAIN
2. runtime le `Build Manifest`
3. runtime segue referencias para os demais contratos
4. runtime valida compatibilidade
5. runtime detecta hardware
6. runtime aplica `Execution Policy`
7. runtime escolhe modelo
8. runtime ativa packs
9. runtime sobe agente e UI

---

## 22. Exemplo de Leitura em Runtime

### Sequencia recomendada

- ler `build-manifest.json`
- resolver `runtime-config.json`
- ler `usbrain-profile.json`
- ler `execution-policy.json`
- ler `installed-models.json`
- ler `installed-packs.json`, se existir
- validar referencias e arquivos
- continuar boot

### Regra

O `Build Manifest` deve ser sempre a porta de entrada principal.

---

## 23. Erros Contratuais que Devem Ser Tratados

### Exemplos

- `schema_version` desconhecida
- referencia para arquivo inexistente
- manifesto de modelo vazio
- pack declarado mas nao instalado
- runtime-config invalido
- politica inconsistente com dados instalados

### Comportamento esperado

- erro claro no log;
- mensagem compreensivel na interface, quando aplicavel;
- degradacao controlada ou falha explicita, nunca comportamento misterioso.

---

## 24. Evolucao Futura dos Contratos

Esses contratos devem permitir crescimento para:

- multi-modelo mais sofisticado;
- ativacao dinamica de packs;
- snapshots de catalogo de modelos;
- perfis resumidos de benchmark;
- politicas por dominio;
- telemetria local opcional;
- sincronizacao com `Hub`.

### Regra de evolucao

Adicionar sem quebrar sempre que possivel.

Romper apenas quando houver ganho estrutural claro.

---

## 25. Resultado Esperado

Quando esses contratos estiverem implementados:

- o Builder podera evoluir sem acoplar detalhes internos do Runtime;
- o Runtime podera inicializar com previsibilidade;
- o debug ficara muito mais facil;
- o USBRAIN ganhara base tecnica seria para crescer como plataforma.

---

## 26. Relacao com os Documentos Anteriores

Este documento deriva de:

- [ARQUITETURA_FUNCIONAL_USBRAIN_BUILDER.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ARQUITETURA_FUNCIONAL_USBRAIN_BUILDER.md)
- [ARQUITETURA_RUNTIME_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ARQUITETURA_RUNTIME_USBRAIN.md)
- [FORMATO_PACK_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/FORMATO_PACK_USBRAIN.md)
- [BACKLOG_INICIAL_BUILDER_TAURI_RUST.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/BACKLOG_INICIAL_BUILDER_TAURI_RUST.md)

Ele prepara os proximos passos de implementacao:

- backlog inicial do runtime;
- milestones tecnicos do Builder;
- estruturas reais de `JSON Schema`;
- implementacao concreta do shell `Tauri + Rust`.
