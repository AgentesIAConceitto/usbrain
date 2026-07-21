# Debug Session: llm-400-error

Status: OPEN

## Symptom
- USBRAIN abre no navegador e o chat inicializa.
- Ao enviar mensagem no chat, o backend da LLM agora retorna erro 400.

## Scope
- Ambiente alvo: pendrive USBRAIN executado no Windows 11.
- Evidências esperadas: screenshots em `tmp/` do pendrive e logs do boot/chat.

## Initial Hypotheses
1. O payload enviado para `/v1/chat/completions` esta invalido para a versao atual do `llamafile`.
2. O campo `model` enviado pelo agente nao e aceito pelo backend local.
3. Alguma mensagem no historico ou `tool_calls` esta em formato incompatível com a API OpenAI-compatible exposta.
4. O backend local aceita o header `Authorization`, mas rejeita outro campo do body como `max_tokens`, `temperature` ou `tools`.
5. O erro 400 vem da segunda chamada apos tool calls, e nao da primeira chamada simples do chat.

## Evidence Log
- Screenshot do navegador confirma erro na primeira chamada de chat:
  - `Client error '400 Bad Request' for url 'http://127.0.0.1:8080/v1/chat/completions'`
- `bootstrap_python.log` continua saudavel:
  - import do `main.py` OK
  - hardware detectado OK
  - smoke tests de `ctypes`, `numpy`, `torch` e `sentence_transformers` OK
- `llamafile.log` e `bge_boot.log` continuam mostrando boot correto:
  - BGE pronto
  - porta TCP da LLM pronta
  - Web UI preparada
- Reproducao local parcial no Mac:
  - payload minimo com `model`, `messages`, `temperature`, `max_tokens` e `Authorization: Bearer no-key` retorna `200`
  - payload minimo com um schema simples de `tools` tambem retorna `200`

## Hypothesis Status
- H2 parcialmente enfraquecida: `model='Phi-3.5 Mini'` sozinho nao causa `400` na reproducao local.
- H4 parcialmente enfraquecida: `temperature`, `max_tokens` e um schema simples de `tools` tambem nao causam `400` isoladamente.
- H1 continua forte: o problema deve estar no payload real completo, nao no header.
- H3 continua forte: uma parte especifica do payload completo pode estar incompatível (RAG extra, combinacao de mensagens ou um schema real do conjunto de ferramentas).
- H5 continua em aberto: ainda falta confirmar se o `400` vem da primeira ou da segunda chamada quando houver tool calls.

## New Evidence
- O `llamafile.log` do Windows confirmou:
  - `message_count = 2`
  - `has_tools = true`
  - `tool_count = 9`
  - `status_code = 400`
  - `response_text = "The HTTP client sent a request that this server could not understand."`
- Isso enfraquece a suspeita de RAG ou segunda chamada apos tool calls.
- O diferencial mais forte da primeira chamada com falha e o envio do bloco `tools` com os 9 schemas reais.

## Additional Evidence
- O fallback sem `tools` executou no Windows e ainda assim recebeu `400`.
- O `llamafile.log` mostrou claramente a segunda tentativa:
  - `stage = chat-first-call-retry-no-tools`
  - `has_tools = false`
  - `tool_count = 0`
  - `message_count = 2`
  - `status_code = 400`
- Reproducao controlada no Mac:
  - com `system_prompt` real do agente, `urllib` retorna `200`
  - com `system_prompt` real do agente, `httpx` tambem retorna `200`
- Isso enfraquece:
  - schema de tools como causa unica
  - tamanho do system prompt
  - formato do body como causa geral
- O foco passa a ser compatibilidade especifica entre o `llamafile.exe` do Windows e o transporte HTTP usado pelo agente.

## Applied Mitigation
- Foi implementado fallback minimo no `agent.py`:
  - se a primeira chamada retornar `400` e houver `tools` no body
  - o agente tenta novamente a mesma requisicao sem `tools` e sem `tool_choice`
- Objetivo:
  - preservar chat basico em backends OpenAI-compatible mais estritos
  - manter logs suficientes para continuar a depuracao do suporte a tool-calling depois

## Updated Fix
- O caminho HTTP da LLM foi trocado de `httpx` para `urllib.request` apenas para `/v1/chat/completions`.
- Motivacao:
  - reduzir variabilidade de serializacao/transporte
  - usar o cliente HTTP mais simples e compativel possivel com o backend local

## Next Step
- Proximo teste no Windows ja tera instrumentacao no `agent.py` para registrar no proprio `llamafile.log`:
  - resumo do payload enviado
  - quantidade de mensagens
  - quantidade e nomes das ferramentas
  - `status_code`
  - `response_text` bruto devolvido pela LLM no erro HTTP
