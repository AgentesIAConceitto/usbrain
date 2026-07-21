# TASK

## TASK (ATIVO)

- Status: em andamento
- Objetivo: realinhar o USBRAIN Builder para a arquitetura correta, com app online-first no PC e pendrive offline-first no artefato final
- Backup preservado: `backups/usbrain-working-macos-20260704-091522`
- Fase atual: a biblioteca local ja possui preparo, fila de aquisicao e download direto tanto pela fila quanto pelo catalogo vivo quando a variante traz URL resolvida
- Requisito explicitado: o Builder deve pesquisar sozinho na web os melhores modelos open source, consolidar benchmarks e descobrir o download correto por variante antes da aquisicao
- Requisito explicitado: a biblioteca local precisa de gestao de ciclo de vida, incluindo listagem, metadados de origem/tamanho/estado e exclusao para liberar espaco no PC
- Requisito explicitado: a aplicacao e sua validacao nao devem usar dados mockados, `example.com`, snapshots artificiais ou curadoria ficticia como fonte funcional
- Entrega parcial concluida: a camada de administracao da biblioteca local ja possui inventario e exclusao segura na UI e no backend
- Entrega parcial concluida: a biblioteca local agora tambem consegue consumir a fila de aquisicao e baixar um artefato pendente diretamente para a lib local com validacao opcional de `sha256`
- Entrega parcial concluida: a infraestrutura inicial do catalogo vivo ja possui refresh remoto, cache local e configuracao de fontes em `catalog/catalog-sources.json`
- Entrega parcial concluida: o snapshot empacotado do catalogo agora ja cobre downloads diretos reais para perfis-base de 8 GB e 16 GB, reduzindo o numero de cards ainda sem URL resolvida
- Entrega parcial concluida: o catalogo agora tambem expõe perfil tecnico e indicadores de benchmark estruturados para apoiar a decisao do usuario antes do download
- Entrega parcial concluida: o runtime do catalogo deixou de cair em `bundled-snapshot` e agora exige cache sincronizado de fonte remota real, falhando explicitamente quando isso nao existe
- Entrega parcial concluida: a suite mockada de frontend saiu do caminho oficial de validacao e `npm test` agora exige fonte remota real configurada, falhando explicitamente quando ela nao existe
- Entrega parcial concluida: `refresh_model_catalog` agora aceita uma fonte real da API do Hugging Face Hub, normaliza repositorios GGUF publicos para o schema do USBRAIN, resolve download direto do arquivo escolhido e aceita token opcional via ambiente para ganho de autenticacao/rate limit
- Entrega parcial concluida: `catalog/catalog-sources.json` agora ja aponta para uma busca real do Hugging Face Hub e `npm test` valida essa fonte real com sucesso
- Entrega parcial concluida: a descoberta do Hugging Face agora tambem cruza benchmark real do `OpenEvals/leaderboard-data` via `datasets-server`, preenchendo `benchmarkSummary` e `benchmarkMetrics` quando encontra o modelo base correspondente
- Entrega parcial concluida: o casamento com benchmarks reais ficou mais robusto com aliases adicionais para `base_model`, sufixo sem organizacao e nomes sem formatos de empacotamento; alem disso, o catalogo agora prioriza modelos com benchmark real encontrado
- Entrega parcial concluida: o catalogo agora tambem incorpora uma segunda fonte publica real, `BenchLM`, para complementar cobertura de benchmark open-weight, categorias como coding/reasoning/math e contexto tecnico quando o `OpenEvals` nao cobre a familia
- Entrega parcial concluida: a UI do catalogo passou a sinalizar explicitamente quando ainda nao existe benchmark publico comparavel para uma variante/familia
- Entrega parcial concluida: o refresh do catalogo deixou de parar na primeira fonte disponivel e agora agrega multiplas fontes reais configuradas, deduplicando por `modelId` e preservando a versao mais rica de cada modelo
- Entrega parcial concluida: `catalog/catalog-sources.json` agora consulta por padrao buscas reais complementares do Hugging Face para `Instruct`, `Coder` e `VL`, ampliando a cobertura descoberta no Builder
- Entrega parcial concluida: a descoberta do Hugging Face agora ranqueia melhor os candidatos antes do fetch detalhado, penaliza repositorios derivados (`adapter`, `lora`, `merged`, `distilled`, `uncensored`) e rejeita repositorios multimodais com dependencias como `mmproj`, evitando cards que o Builder ainda nao consegue materializar corretamente
- Entrega parcial concluida: mirrors genericos equivalentes do Hugging Face agora sao agrupados por familia-base + quantizacao, escolhendo automaticamente o melhor representante do grupo e reduzindo cards duplicados entre forks/quantizadores para a mesma variante util
- Entrega parcial concluida: o agrupamento canonico agora tambem considera perfil semantico de especializacao extraido de tags/capacidades, separando melhor mirrors genericos da familia de forks especializados relevantes como `Coder`, `Reasoning`, `Tool Use`, `Legal`, `Medical` e `Multilingual`
- Entrega parcial concluida: o matching de benchmark agora tambem usa aliases semanticos derivados desse mesmo perfil de especializacao, convergindo melhor nomes como `Coder`/`Reasoning`/`Tool Use` mesmo quando a ordem dos tokens muda entre Hugging Face, OpenEvals e BenchLM
- Entrega parcial concluida: variantes especializadas ou repositorios derivados agora nao herdam benchmark do modelo-base por padrao; o Builder so materializa benchmark quando encontra match proprio suficientemente aderente, evitando comparacoes otimistas demais no catalogo
- Entrega parcial concluida: o catalogo agora expõe confianca explicita do benchmark (`Benchmark proprio`, `Benchmark comparavel`, `Benchmark herdado`) no backend e na UI, com merge priorizando o benchmark mais aderente ao card antes da fonte
- Entrega parcial concluida: a UI agora tambem usa `confidenceLevel` para ordenar o catalogo de forma mais decisoria e oferece filtro explicito por benchmark (`proprio`, `comparavel`, `herdado`, `sem benchmark`), melhorando a navegacao do usuario na selecao de LLMs
- Entrega parcial concluida: as recomendacoes automaticas por RAM/pendrive agora reutilizam a mesma comparacao rica do catalogo, priorizando benchmark mais confiavel antes de qualidade bruta e ajustando a sugestao inicial do Builder para um lineup mais aderente
- Proxima entrega: continuar reduzindo falsos negativos no matching entre modelo-base e variantes GGUF, ampliar a estrategia autonoma de descoberta com ranking mais semantico por familia e aproximar os indicadores do desempenho real no hardware do usuario
