# [OPEN] Debug Session: llm-latency-timeout

## Symptoms
- Respostas da LLM demoram mais de 2 minutos.
- Perguntas mais complexas terminam em timeout.
- Algumas respostas exibem uma linha extra no formato `Nota: ...`, que nao deveria aparecer na UI.

## Initial Hypotheses
1. O `llamafile` esta subindo com configuracao de CPU/contexto que preserva estabilidade, mas degrada muito a latencia.
2. O timeout do cliente HTTP do `agent.py` e insuficiente para respostas mais longas na velocidade atual do modelo.
3. O prompt/persona/contexto RAG esta induzindo a LLM a emitir pos-texto do tipo `Nota: ...`.
4. O contexto dinamico calculado ainda esta acima do ponto ideal para throughput em CPU.
5. A UI/agent nao esta filtrando pos-processamento textual indesejado quando a LLM retorna observacoes extras.

## Evidence Collected So Far
- O `llamafile_server.log` mostrou falha anterior de memoria por `kv cache`, corrigida com limitacao dinamica de contexto.
- O sistema agora sobe parcialmente e responde algumas perguntas, mas com alta latencia e timeout em prompts mais complexos.
- `llamafile_server.log` confirma execucao apenas em CPU (`no usable GPU found`), o que explica throughput baixo.
- `llamafile.log` mostra `ctx_size=4096`, readiness da LLM em ~20.6s e timeout apenas na pergunta mais longa, com `message_count=6`.
- O `agent.py` usava `timeout=120`, `max_tokens=4096` e mantinha historico de 20 mensagens, configuracao agressiva para CPU + pendrive.
- As imagens em `TMP` mostram que o texto `Note:`/`Nota:` vem da propria resposta do modelo, nao da UI.
- A rodada mais recente mostrou que `Boa noite` foi enviado sem ferramentas (`has_tools=false`) e mesmo assim estourou o timeout simples de `120s`, logo o problema nao era mais o custo dos schemas de tools nesse caso.
- O `llamafile.log` mostrou limpeza de processos antigos apenas na inicializacao seguinte, confirmando que houve casos em que subprocessos sobreviveram ao fechamento anterior do terminal no Windows.
- O usuario rejeitou corretamente a estrategia de resposta local/sintetica como solucao estrutural, pois conversas reais continuam podendo ser longas e usar ferramentas/web.
- O keepalive confirmou que a LLM permaneceu viva durante a conversa longa; a degradacao observada foi de throughput, nao de crash, inclusive em perguntas sem ferramentas.
- A medicao mais recente mostrou latencias de aproximadamente 9m50s e 25m24s para perguntas simples sem tools, o que fortalece a suspeita de tuning ruim de CPU e/ou page faults sobre o modelo no pendrive.

## Next Steps
- Aplicar ajuste minimo no `agent.py` para reduzir carga por chamada e evitar timeout desnecessario.
- Filtrar linhas finais `Note:`/`Nota:` antes de exibir na UI.
- Validar a correcao no projeto e no pendrive, depois solicitar nova rodada no Windows.

## Applied Fixes
- Timeout da chamada da LLM aumentado de `300s` para `600s` para acomodar respostas complexas em CPU.
- Tool-calling agora so e habilitado para pedidos que sugerem acao/diagnostico real; saudacoes e perguntas casuais nao carregam mais os 9 schemas de ferramentas.
- Perguntas sem ferramentas passam a usar timeout menor (`120s`) e teto de resposta menor (`384 tokens`) para evitar esperas desnecessarias.
- Saudacoes triviais agora usam resposta local imediata, sem depender da LLM.
- Perguntas simples sem ferramentas ficaram ainda mais enxutas, com teto de `128` tokens.
- Subprocessos do BGE e da LLM agora sao associados a um Windows Job Object com `kill-on-close`, para reduzir o risco de memoria/processos residuais apos fechar o terminal.
- A chamada `POST /v1/chat/completions` agora solicita `stream=true` e agrega os chunks SSE em memoria, inclusive reconstruindo `tool_calls`, para permitir respostas longas reais sem depender de receber o JSON inteiro so no final.
- O atalho de smalltalk e a instrucao extra de resposta curta foram removidos, para o sistema voltar a usar a LLM real tambem nas mensagens simples.
- O bootstrap da LLM agora calcula tuning de runtime baseado em hardware: `threads` de geracao, `threads-batch`, `batch-size` e decisao de usar `--no-mmap` quando houver RAM suficiente para evitar dependencia excessiva do pendrive durante a inferencia.
