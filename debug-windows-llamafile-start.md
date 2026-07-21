# Debug Session: windows-llamafile-start

- **Status**: [OPEN]
- **Issue**: `start.bat` conclui a preparacao no Windows 11, mas `main.py` encerra com erro generico ao iniciar o agente.
- **Debug Server**: http://127.0.0.1:7777/event
- **Log File**: `.dbg/trae-debug-log-windows-llamafile-start.ndjson`

## Reproduction Steps
1. Conectar o pendrive em um Windows 11.
2. Executar `start.bat`.
3. Aguardar a extracao do runtime Python, deteccao de RAM e escolha da porta.
4. Observar a falha final com a mensagem `Falha ao iniciar o agente`.

## Hypotheses & Verification
| ID | Hypothesis | Likelihood | Effort | Evidence |
|----|------------|------------|--------|----------|
| A | `main.py` cai na inicializacao da Web UI porque `webui/static` nao existe no pendrive e `aiohttp` falha em `add_static`. | Low | Low | Rejected |
| B | `llamafile.exe` demora muito para ficar operacional no pendrive USB 2.0, mas o boot antigo interpretava "processo vivo apos 5s" como se fosse readiness. | High | Medium | Confirmed |
| C | O servidor BGE tambem pode levar muito mais tempo para ficar operacional, e o boot antigo interpretava "processo vivo apos 2s" como se fosse readiness. | High | Medium | Confirmed |
| D | O modelo GGUF ou os argumentos do `llamafile` estao incorretos no pendrive. | Medium | Low | Partial |
| E | O erro apresentado ao usuario e majoritariamente um problema de observabilidade e semantica do boot, nao um diagnostico real de readiness do `llamafile`. | High | Low | Confirmed |

## Log Evidence
- `/Volumes/USBRAIN/llamafile.log` nao existe no pendrive retornado do Windows.
- `start.bat` sempre orienta verificar `llamafile.log`, mas `main.py` atual nao grava esse arquivo em nenhum ponto.
- `main.py` inicia `llamafile` e `rag/server.py` com `stdout=subprocess.PIPE` e `stderr=subprocess.PIPE`, mas nao consome nem persiste essas saidas.
- O modelo `/Volumes/USBRAIN/models/Phi-3.5-mini-instruct-Q4_K_M.gguf` existe no volume.
- O runtime Python foi extraido no Windows: `python/` existe no volume e `python_runtime.zip` nao esta mais presente, consistente com o fluxo esperado do `start.bat`.
- O BGE-M3 offline existe em `/Volumes/USBRAIN/rag/bge-m3-model/`.
- `webui/server.py` registra `app.router.add_static("/static", path=str(Path(__file__).parent / "static"), name="static")`.
- O diretorio `webui/static/` existe tanto no projeto quanto no pendrive, mas esta vazio; a hipotese A foi rejeitada.
- `main.py` foi instrumentado no projeto e sincronizado no pendrive para gerar `llamafile.log` com eventos do boot, excecoes e estado inicial dos subprocessos.
- A instrumentacao foi validada localmente: uma execucao com `--no-llm` gerou entradas de boot, spawn do BGE e preflight da Web UI em `llamafile.log`.
- A versao antiga do pendrive fazia apenas `time.sleep(2)` apos o spawn do BGE e `time.sleep(5)` apos o spawn do `llamafile`, seguida de `_log_process_state(...)`.
- Esses logs antigos provavam apenas que os subprocessos nao tinham morrido nos primeiros segundos; nao provavam que `BGE` ou `LLM` estavam realmente prontos para uso.
- O `main.py` local foi corrigido para aguardar readiness real antes de subir a Web UI:
  - `BGE`: espera ativa por `GET /stats` com timeout de 600s.
  - `LLM`: espera ativa pela porta TCP com timeout de 900s.
  - o boot passa a abortar explicitamente se um backend nao ficar pronto a tempo.

## Verification Conclusion
- **Pre-fix**: nao havia log confiavel do boot; depois disso, os primeiros logs ainda foram interpretados de forma errada porque "processo vivo" foi tratado como "backend pronto".
- **Current state**: o `main.py` local agora separa corretamente "spawn bem-sucedido" de "readiness real" e aguarda os backends antes de subir a UI.
- **Next step**: sincronizar essa versao no pendrive, limpar logs antigos e rodar novo teste no Windows para medir readiness real do `BGE-M3` e do `llamafile` em USB 2.0.
