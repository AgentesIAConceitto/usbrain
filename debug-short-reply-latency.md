# Debug Session: short-reply-latency

- Status: OPEN
- Symptom: respostas muito curtas da LLM ainda levam varios minutos no Windows, inclusive saudacoes simples.
- Expected: respostas curtas deveriam voltar em poucos segundos ou, no pior caso, dezenas de segundos.

## Hypotheses
1. O gargalo dominante esta no prefill/prompt processing, nao na quantidade de tokens de saida.
2. O historico e o system prompt estao grandes o suficiente para tornar cada pergunta curta cara demais.
3. O `llamafile.exe` no Windows esta usando configuracao de batch/threads/contexto que favorece estabilidade, mas degrada brutalmente o throughput de prefill.
4. O acesso ao modelo e/ou ao KV cache ainda esta sofrendo com I/O/paginacao mesmo apos os ajustes recentes.
5. A UI so exibe a resposta no final e esconde progresso intermediario, inflando a percepcao de travamento.

## Evidence So Far
- Uma resposta de `18 palavras` ainda levou cerca de `10 minutos`, segundo o relato do usuario.
- Rodadas anteriores ja mostraram latencias de ~`9m50s` e ~`25m24s` para perguntas sem tools.
- O keepalive da LLM indicou processo vivo durante a espera, sugerindo lentidao real e nao crash.

## Next Step
- Correlacionar a latencia observada com o payload real enviado, o tamanho do historico e o comportamento do backend entre o inicio do request e o primeiro token.
