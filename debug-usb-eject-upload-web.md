# Debug Session: usb-eject-upload-web

Status: [OPEN]

## Sintomas
- O Builder no macOS conclui a criacao, mas nao ejeta o pendrive ao final.
- O upload de arquivo `.md` para a base permanente retorna `500 Internal Server Error`.
- O fluxo de pesquisa/aprendizado web nao mostra sucesso claro nem evidencia do conteudo absorvido.

## Hipoteses Falsificaveis
1. O `auto-eject` nao executa porque o pipeline termina em estado diferente do esperado ou retorna antes de chamar `macos_eject_target()`.
2. O `auto-eject` executa, mas `diskutil` resolve o identificador errado do disco/particao e falha silenciosamente na integracao com a UI.
3. O upload `.md` quebra dentro de `agent.add_document()` ou nos endpoints `/embed` e `/store` do RAG, gerando `500` real no backend.
4. O endpoint web de aprendizado conclui parcialmente, mas a UI nao exibe o resultado porque o payload retornado nao contem feedback suficiente.
5. Quando o usuario informa um site, o backend ainda trata a entrada como busca geral e nao como URL/crawl direcionado, limitando a coleta ao Brave.

## Plano
- Instrumentar os pontos de auto-eject, upload e aprendizado web.
- Reproduzir com evidencias de runtime.
- Confirmar ou rejeitar as hipoteses.
- Aplicar a menor correcao possivel e validar novamente.

## Evidencias Coletadas
- Upload local `.md` com instrumentacao:
  - `webui/server.py:handle_upload:entry`
  - `agent.py:add_document:embed-response`
  - `rag/server.py:handle_store:entry`
  - `agent.py:add_document:store-response`
  - Resultado: fluxo completo terminou em `200`, sem falha nos endpoints `/embed` e `/store`.
- Pesquisa por URL antes da correcao:
  - `rag/server.py:handle_add_search:error`
  - Evidencia: `HTTPStatusError 403 Forbidden` ao buscar `https://pt.wikipedia.org/wiki/Guerra_dos_Farrapos`.
  - A UI recebia `500` do RAG e devolvia feedback pouco acionavel ao usuario.
- Pesquisa por URL depois da correcao:
  - `rag/server.py:handle_add_search:direct-url`
  - `webui/server.py:handle_search_web:rag-response`
  - Evidencia: `68 chunks` adicionados em `4 pagina(s)` do dominio `pt.wikipedia.org`.
- Ejeção automatica:
  - `diskutil info -plist /Volumes/USBRAIN` confirmou `ParentWholeDisk=disk4`, `Ejectable=true` e alvo USB valido.
  - Ainda falta reproducao pos-instrumentacao do fluxo real do Builder para confirmar a falha e validar o retry de ejeção.
- Evidencias adicionais no pendrive real:
  - `llamafile.log` mostrou:
    - `agent.py:add_document:entry` com `content_chars=6267`
    - `agent.py:add_document:chunks` com `chunk_count=2`
    - `agent.py:add_document:error` com `ReadTimeout('')`
  - Conclusao: o upload do `.md` falhou antes de gravar qualquer chunk porque o primeiro `/embed` excedeu o timeout no Windows CPU-only.
  - `vetor.db` do pendrive mostrou que a pesquisa por `conceitto.ind.br` indexou corretamente paginas da Conceitto Industrial (`https://conceitto.ind.br/`, `.../ecossistema`, `.../solucoes`, `.../tecnologia`), mas o chat ainda exibiu a lista bruta do Brave, causando confusao.
  - Validacao local pos-fix:
    - upload `.md` de `7290` bytes retornou `200` com `9/9 chunks`;
    - `GET /api/search_web?q=conceitto.ind.br` retornou mensagem limpa de site indexado, com 4 fontes efetivamente absorvidas.
  - Evidencia do bug de resposta alucinada:
    - `agent._should_enable_tools("me fale sobre a Conceitto Labs") => False`
    - antes da correcao, o RAG so era consultado quando `enable_tools` era verdadeiro;
    - reproducao minima pos-fix mostrou:
      - `tools? False`
      - `rag_called? True`
    - conclusao: os chunks eram indexados corretamente, mas perguntas normais de chat nao consultavam a base vetorial.

## Hipoteses
- H1: parcialmente confirmada. A rotina de auto-eject precisava ser endurecida contra volume ocupado.
- H2: rejeitada para resolucao do alvo. O metadata do volume atual mostra identificador correto (`disk4`).
- H3: nao confirmada no ambiente local, mas o fluxo foi blindado para impedir `500` por excecao escapando antes do `try`.
- H4: confirmada. O usuario recebia feedback insuficiente no fluxo de pesquisa/aprendizado.
- H5: rejeitada para o caso de URL direta. O backend ja entrava no modo de URL/crawl; o problema real era o `403` no fetch.

## Correcoes Aplicadas
- `agent.py`: grava documento com `encoding="utf-8"` e envolve toda a rotina de `add_document` no bloco protegido.
- `agent.py`: reduziu o chunking de upload para blocos menores com sobreposicao e aumentou o timeout do cliente HTTP de embedding.
- `agent.py`: desacoplou a consulta ao RAG da heuristica de tool-calling; perguntas normais agora consultam a base vetorial mesmo sem palavras-chave de acao.
- `agent.py`: adicionou filtro minimo de similaridade e logs de busca vetorial.
- `rag/server.py`: usa cabecalhos de navegador no fetch, converte bloqueios diretos em mensagem clara e preserva indexacao quando o site permite raspagem.
- `rag/server.py`: passou a devolver a lista real de fontes indexadas.
- `webui/templates/index.html`: pesquisa, URL e upload agora tambem escrevem a resposta no chat, nao apenas em toast.
- `webui/server.py`: deixou de ecoar o retorno bruto do Brave para dominios/sites explicitos e passou a responder com o resumo real da indexacao.
- `builder/src-tauri/src/commands.rs`: auto-eject no macOS ganhou `sync`, `retry`, `unmountDisk force`, `unmount force` por mount point e fallback extra de `eject`.
