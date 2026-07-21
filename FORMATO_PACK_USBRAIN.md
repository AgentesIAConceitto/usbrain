# USBRAIN — Formato Oficial de PACKs

## 1. Objetivo

Este documento define o formato oficial dos `PACKs` do `USBRAIN`.

O objetivo e estabelecer:

- o que um `PACK` representa dentro do ecossistema;
- como um `PACK` deve ser estruturado;
- como um `PACK` e criado dentro do `USBRAIN Builder`;
- como um `PACK` e instalado pelo Builder;
- como um `PACK` e carregado pelo Runtime;
- como um `PACK` pode ser publicado no `USBRAIN Hub`.

Este documento assume a decisao arquitetural de que a criacao de `PACKs` sera uma capacidade nativa do `USBRAIN Builder`, e nao um produto separado na fase inicial.

---

## 2. Definicao

Um `PACK` e um pacote instalavel, versionado e distribuivel que adiciona especializacao ao `USBRAIN`.

Essa especializacao pode envolver:

- conhecimento;
- comportamento;
- ferramentas;
- politicas;
- ou uma combinacao desses elementos.

O `PACK` existe para desacoplar:

- a capacidade geral de raciocinio da `LLM base`;
- do conhecimento e comportamento especializado de um dominio.

Em termos praticos, o `PACK` transforma uma `LLM local generica` em uma experiencia muito mais util para nichos, comunidades e casos de uso concretos.

---

## 3. Papel dos PACKs no USBRAIN

Os `PACKs` nao sao extras opcionais de baixo impacto.

Eles sao uma das bases da proposta de valor do `USBRAIN`, porque permitem:

- especializacao por dominio sem depender de modelos enormes;
- distribuicao de inteligencia especializada pela comunidade;
- instalacao seletiva de conhecimento;
- adaptacao do pendrive a casos de uso diferentes;
- crescimento organico do ecossistema via `Hub`.

Sem `PACKs`, o USBRAIN seria apenas uma plataforma de empacotamento de modelos.

Com `PACKs`, ele se torna uma plataforma de inteligencias portateis especializadas.

---

## 4. Principios do Formato

- `Formato aberto`
- `Manifesto obrigatorio`
- `Versionamento explicito`
- `Compatibilidade declarada`
- `Instalacao previsivel`
- `Carregamento seletivo pelo Runtime`
- `Internacionalizacao considerada desde a origem`
- `Seguranca e auditabilidade`
- `Compatibilidade com busca semantica local`

---

## 5. Tipos de PACK

O formato oficial deve suportar pelo menos quatro tipos principais.

## 5.1 Knowledge Pack

Foco principal:

- documentos;
- corpus textual;
- chunks;
- embeddings;
- indice semantico;
- referencias de dominio.

Uso tipico:

- direito;
- animes;
- estrategia de jogos;
- manutencao tecnica;
- educacao.

---

## 5.2 Behavior Pack

Foco principal:

- estilo de resposta;
- persona;
- metodo de raciocinio;
- limites e restricoes;
- regras de citacao;
- regras de seguranca.

Uso tipico:

- tutor tecnico;
- consultor juridico com avisos;
- assistente didatico;
- agente de suporte.

---

## 5.3 Tool Pack

Foco principal:

- metadados sobre ferramentas;
- workflows especificos;
- integracoes recomendadas;
- parametros de uso;
- roteiros de acao.

Uso tipico:

- automacao de tarefas;
- investigacao tecnica;
- pesquisa orientada;
- conectores de dominio.

---

## 5.4 Hybrid Pack

Foco principal:

- combinacao de conhecimento, comportamento e ferramentas.

Esse tende a ser o tipo mais importante na pratica, porque representa melhor especialistas reais.

Exemplo:

- um `Pack Juridico` com base documental, comportamento conservador e regras de citacao;
- um `Pack Mobile Legends` com dados de jogo, estilo direto e recomendações taticas;
- um `Pack de Medicina Alternativa` com conteudo curado, avisos e politicas de seguranca.

---

## 6. Criacao de PACKs no Builder

Na fase inicial do produto, a criacao de `PACKs` deve acontecer dentro do proprio `USBRAIN Builder`.

### Modos principais do Builder

- `Criar USBRAIN`
- `Criar PACK`
- `Explorar PACKs`

### Justificativa

- reduz fragmentacao de produto;
- reaproveita a mesma base de GUI e infraestrutura;
- facilita onboarding da comunidade;
- mantem a experiencia coesa;
- permite evoluir para um app separado no futuro, se necessario.

### Decisao de produto

Nao ha necessidade inicial de um executavel separado para autoria de packs.

Se no futuro o ecossistema crescer muito, essa capacidade pode evoluir para um produto derivado, como um eventual `USBRAIN Pack Studio`.

---

## 7. Pipeline de Criacao de PACK

O fluxo de autoria de `PACKs` no Builder deve seguir um pipeline claro.

## 7.1 Definicao do Pack

O autor informa:

- nome;
- id unico;
- descricao;
- tipo de pack;
- categoria;
- idioma ou idiomas;
- autor;
- licenca;
- nivel de sensibilidade;
- versao inicial.

---

## 7.2 Ingestao de Conteudo

O autor adiciona as fontes do pack.

Exemplos:

- markdown;
- txt;
- json;
- html exportado;
- pdf;
- conjuntos de notas;
- datasets curados;
- assets locais;
- referencias opcionais espelhadas.

---

## 7.3 Normalizacao

O Builder precisa preparar o conteudo para uso.

Etapas esperadas:

- limpeza de texto;
- extracao de metadados;
- remocao de ruido;
- normalizacao de encoding;
- separacao por idioma;
- organizacao por categoria ou topico.

---

## 7.4 Chunking

O conteudo textual deve ser quebrado em unidades semanticas adequadas para busca vetorial.

Configuracoes relevantes:

- tamanho de chunk;
- overlap;
- estrategia por tipo de documento;
- preservacao de titulos e secoes;
- referencia de origem.

---

## 7.5 Embedding e Indexacao

O Builder gera a estrutura necessaria para busca semantica local.

Saidas esperadas:

- chunks normalizados;
- embeddings;
- indice vetorial;
- mapeamento para origem documental.

---

## 7.6 Comportamento e Politicas

O autor pode definir elementos nao documentais do pack.

Exemplos:

- instrucoes de comportamento;
- avisos obrigatorios;
- metodo de resposta;
- regras de citacao;
- limites de atuacao;
- restricoes para temas sensiveis.

---

## 7.7 Geracao do Manifesto

O Builder gera o `Pack Manifest` com todos os metadados obrigatorios e opcionais.

---

## 7.8 Validacao

O Builder valida:

- consistencia do manifesto;
- integridade dos assets;
- estrutura obrigatoria;
- compatibilidade com o runtime;
- compatibilidade de idioma;
- dependencias declaradas.

---

## 7.9 Empacotamento

O Builder gera o artefato final do pack.

Esse artefato pode ser distribuido:

- como diretorio padronizado;
- ou como arquivo empacotado, como `*.usbrain-pack`.

---

## 8. Estrutura Conceitual do PACK

Um `PACK` deve ser entendido como um conjunto de camadas.

### Camada 1: Identidade

- id;
- nome;
- versao;
- autor;
- licenca;
- categoria;
- idiomas.

### Camada 2: Conhecimento

- documentos;
- chunks;
- embeddings;
- indice vetorial;
- referencias.

### Camada 3: Comportamento

- instrucoes;
- tom;
- limites;
- avisos;
- regras de dominio.

### Camada 4: Ferramental

- ferramentas recomendadas;
- dependencias;
- fluxos de uso;
- parametros.

### Camada 5: Politicas

- sensibilidade;
- privacidade;
- requisitos de citacao;
- restricoes de uso;
- notas de seguranca.

---

## 9. Estrutura de Arquivos Recomendada

Um pack deve ter uma estrutura previsivel.

```text
pack-root/
  manifest.json
  checksums.json
  license/
  knowledge/
    sources/
    normalized/
    chunks/
    embeddings/
    index/
  behavior/
    instructions.md
    policies.json
  tools/
    tools.json
    workflows.json
  assets/
  docs/
    README.md
```

### Observacoes

- nem todos os diretorios sao obrigatorios em todos os tipos de pack;
- `manifest.json` e obrigatorio;
- `docs/README.md` e altamente recomendado;
- `checksums.json` e recomendado para distribuicao e validacao.

---

## 10. Manifesto do PACK

Todo `PACK` deve possuir um manifesto obrigatorio.

### Campos minimos obrigatorios

- `id`
- `name`
- `version`
- `pack_type`
- `category`
- `author`
- `license`
- `languages`
- `runtime_compatibility`
- `description`

### Campos altamente recomendados

- `summary`
- `tags`
- `sensitivity_level`
- `source_attribution`
- `builder_version`
- `created_at`
- `updated_at`
- `dependencies`
- `recommended_models`
- `rag_profile`
- `tool_requirements`
- `policy_hints`

### Exemplo conceitual

```json
{
  "id": "br.legal.pack.civil-basic",
  "name": "Direito Civil Basico",
  "version": "1.0.0",
  "pack_type": "hybrid",
  "category": "legal",
  "author": "Comunidade USBRAIN Brasil",
  "license": "CC-BY-SA-4.0",
  "languages": ["pt-BR"],
  "runtime_compatibility": ">=1.0.0",
  "description": "Pack introdutorio para consultas e estudo de direito civil com base semantica e instrucoes conservadoras.",
  "sensitivity_level": "high",
  "dependencies": [],
  "recommended_models": ["llama-3.2-3b", "phi-3.5-mini"],
  "rag_profile": "dense-semantic",
  "policy_hints": {
    "require_citations": true,
    "show_safety_notice": true
  }
}
```

---

## 11. Assets do PACK

Os assets variam conforme o tipo de pack.

### Possiveis classes de asset

- `source documents`
- `normalized text`
- `semantic chunks`
- `embedding files`
- `vector index`
- `behavior instructions`
- `policy definitions`
- `tool metadata`
- `auxiliary media`

### Regra

O runtime nao precisa carregar todos os assets do pack ao mesmo tempo.

O manifesto deve permitir carregamento seletivo e previsivel.

---

## 12. Compatibilidade

Todo pack deve declarar explicitamente sua compatibilidade.

### Tipos de compatibilidade

- compatibilidade com versao de runtime;
- compatibilidade com idioma;
- compatibilidade com tipos de modelo;
- compatibilidade com ferramentas;
- compatibilidade com politicas do sistema.

### Exemplos

- pack que exige modelo com boa qualidade em `PT-BR`;
- pack que exige suporte a citacoes;
- pack que funciona apenas em modo offline;
- pack que requer determinada ferramenta;
- pack que exige determinada versao minima do Runtime.

---

## 13. Dependencias

Um pack pode declarar dependencias.

### Tipos de dependencia

- outro pack;
- modelo recomendado;
- modelo minimo;
- ferramenta;
- politica de runtime;
- recurso opcional de nuvem.

### Regra

Dependencia declarada deve ser visivel no Builder antes da instalacao.

O usuario precisa entender:

- o que e obrigatorio;
- o que e recomendado;
- o que e opcional.

---

## 14. Sensibilidade e Seguranca

Alguns packs pertencem a dominios sensiveis.

Exemplos:

- direito;
- saude;
- medicina alternativa;
- financas;
- seguranca;
- assuntos regulados.

### Para esses casos, o manifesto deve suportar

- classificacao de sensibilidade;
- avisos obrigatorios;
- requisitos de citacao;
- restricoes de uso;
- observacoes sobre limites do pack.

### Objetivo

Nao censurar a comunidade, mas tornar explicita a natureza do pack e aumentar transparencia para o usuario.

---

## 15. Instalacao de PACKs no Builder

O `USBRAIN Builder` deve conseguir instalar packs no momento da criacao do pendrive.

### Passos de instalacao

1. ler o manifesto;
2. validar compatibilidade;
3. validar integridade;
4. copiar ou baixar assets;
5. registrar o pack no `Build Manifest`;
6. preparar o runtime para reconhecer o pack.

### Resultado

O pack passa a fazer parte do USBRAIN criado e fica disponivel para o runtime na maquina hospedeira.

---

## 16. Carregamento de PACKs no Runtime

O `Runtime` deve carregar packs de forma seletiva e controlada.

### Responsabilidades do Runtime

- ler os packs instalados a partir do `Build Manifest`;
- validar compatibilidade em tempo de inicializacao;
- montar contexto semantico;
- aplicar comportamento adicional;
- registrar ferramentas e politicas do pack;
- expor o pack ao agente apenas quando apropriado.

### Importante

Nem todo pack deve ser ativado de forma identica em qualquer contexto.

O runtime pode usar:

- ativacao por perfil;
- ativacao por idioma;
- ativacao por dominio;
- ativacao por demanda;
- combinacao de packs.

---

## 17. Publicacao no USBRAIN Hub

O `USBRAIN Hub` deve funcionar como o espaco de descoberta e distribuicao de packs.

### Um pack publicado no Hub deve expor

- manifesto;
- versao;
- autoria;
- licenca;
- categoria;
- idiomas;
- tamanho em disco;
- dependencia de modelos ou ferramentas;
- compatibilidade com runtime;
- avaliacao comunitaria;
- nivel de sensibilidade.

### O Hub deve permitir

- packs oficiais;
- packs comunitarios;
- busca por categoria;
- busca por idioma;
- busca por hardware ou modelos sugeridos;
- historico de versoes;
- comparacao entre packs.

---

## 18. Versionamento

Todo pack deve ser versionado.

### Recomendacao

Usar versionamento semantico:

- `MAJOR`
- `MINOR`
- `PATCH`

### Interpretacao

- `MAJOR`: mudanca incompatível;
- `MINOR`: adicao compativel;
- `PATCH`: correcao ou ajuste sem quebra.

### Objetivo

Garantir que o Builder e o Runtime possam tomar decisoes previsiveis de compatibilidade.

---

## 19. Validacao de PACKs

O Builder deve oferecer validacao formal de packs.

### Tipos de validacao

- validacao estrutural;
- validacao de manifesto;
- validacao de checksums;
- validacao de compatibilidade;
- validacao de dependencias;
- validacao de idioma;
- validacao de integridade do indice semantico.

### Saidas da validacao

- `valid`
- `valid with warnings`
- `invalid`

---

## 20. Fluxos Principais do Ecossistema de PACKs

## 20.1 Criacao de Pack

1. Autor abre o `USBRAIN Builder`.
2. Escolhe `Criar PACK`.
3. Define metadados.
4. Adiciona conteudo.
5. Configura comportamento e politicas.
6. Gera embeddings e indice.
7. Valida.
8. Empacota.
9. Exporta ou publica.

---

## 20.2 Instalacao de Pack

1. Usuario abre o `USBRAIN Builder`.
2. Explora `PACKs`.
3. Seleciona um ou mais packs.
4. O Builder verifica compatibilidade.
5. O pack e instalado no build do pendrive.

---

## 20.3 Uso de Pack

1. O `Runtime` detecta packs instalados.
2. O `Pack Loader` ativa os elegiveis.
3. O agente passa a usar:
   - busca semantica;
   - instrucoes adicionais;
   - ferramentas e politicas do pack.

---

## 20.4 Publicacao de Pack

1. Autor gera pack validado.
2. Submete ao `USBRAIN Hub`.
3. O Hub registra metadados e versao.
4. A comunidade passa a descobrir e instalar o pack.

---

## 21. Relacao com o Builder

O formato dos packs precisa se encaixar diretamente na arquitetura do `USBRAIN Builder` definida em [ARQUITETURA_FUNCIONAL_USBRAIN_BUILDER.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ARQUITETURA_FUNCIONAL_USBRAIN_BUILDER.md).

Modulos mais diretamente envolvidos:

- `Pack Registry Client`
- `Pack Installer`
- `Validation Engine`
- `Localization Layer`
- `Runtime Assembler`

### Implicacao pratica

O Builder nao apenas instala packs.

Ele tambem deve ser capaz de:

- criar;
- validar;
- exportar;
- empacotar;
- e futuramente publicar packs.

---

## 22. Relacao com o Runtime

O formato do pack tambem precisa se encaixar na especificacao conceitual do Runtime e do `Pack Loader` descrita em [ESPECIFICACAO_CONCEITUAL_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ESPECIFICACAO_CONCEITUAL_USBRAIN.md#L105-L188).

### Regra central

O pack nao deve ser um blob opaco.

Ele precisa ser inteligivel o bastante para o runtime conseguir:

- saber o que carregar;
- quando carregar;
- com que riscos;
- em que idioma;
- com que dependencias.

---

## 23. Nao Objetivos Iniciais

Para manter foco, o formato inicial de packs nao precisa incluir:

- treinamento de modelos;
- fine-tuning local de LLM;
- dependencias obrigatorias de cloud;
- marketplace fechado;
- aprovacao central obrigatoria para toda publicacao comunitaria.

---

## 24. Resultado Esperado

Quando esse formato estiver implementado, o USBRAIN passara a suportar:

- criacao de inteligencias especializadas pela comunidade;
- instalacao previsivel de packs;
- interoperabilidade entre Builder, Runtime e Hub;
- crescimento do ecossistema sem acoplamento caotico.

---

## 25. Relacao com os Documentos Anteriores

Este documento deriva de:

- [DEFINICAO_PRODUTO_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/DEFINICAO_PRODUTO_USBRAIN.md)
- [ESPECIFICACAO_CONCEITUAL_USBRAIN.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ESPECIFICACAO_CONCEITUAL_USBRAIN.md)
- [ARQUITETURA_FUNCIONAL_USBRAIN_BUILDER.md](file:///Users/fabianoheidrichcosta/Documents/IA/openclaw/clawd_workspace/projects/usbrain/ARQUITETURA_FUNCIONAL_USBRAIN_BUILDER.md)

Ele prepara os proximos niveis de detalhamento:

- contratos tecnicos do manifesto;
- formato binario ou empacotado `*.usbrain-pack`;
- especificacao do `USBRAIN Hub`;
- fluxos de autoria de packs na GUI do Builder.
