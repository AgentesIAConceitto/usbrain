# Debug Session: windows-llamafile-boot

Status: OPEN

## Sintoma
- No Windows, o USBRAIN inicia o batch, detecta Python, RAM e porta.
- O navegador abre quase imediatamente.
- Em seguida, o processo falha com `Falha ao iniciar o agente`.
- O usuario suspeita que a abertura precoce do navegador mascara o fato de que BGE e LLM ainda nao estavam prontos.

## Hipoteses Iniciais
1. `main.py` aborta antes de iniciar o `llamafile`, por erro de import/runtime Python.
2. `llamafile.exe` e encontrado, mas a linha de comando ou o caminho do modelo esta invalido no Windows.
3. O servidor BGE falha antes do `llamafile` e derruba o processo principal.
4. O `llamafile` sobe e morre logo no bootstrap por argumento/configuracao incompatível no Windows.
5. O `start.bat` abre o navegador cedo demais e induz a leitura errada da sequencia de boot.

## Evidencia Coletada
- `bootstrap_python.log` do ultimo teste contem apenas o cabecalho do batch e a linha de comando de `main.py`.
- `llamafile.log` nao foi criado no ultimo teste.
- `python_extract.log` nao foi criado no ultimo teste.
- Isso indica que o processo principal saiu cedo demais, antes do primeiro `_debug_event()` do `main.py`.
- Nova rodada com `faulthandler` confirmou `Windows fatal exception: access violation` dentro de `ctypes` ao executar `detect_hardware()`.
- O crash acontece em `main.py`, antes do bootstrap do BGE e antes de qualquer tentativa de subir o `llamafile`.

## Hipotese Confirmada
- A hipotese 1 foi confirmada: o processo principal aborta cedo demais, e a causa imediata e a deteccao de RAM via `ctypes` no Windows.

## Correcao Aplicada
- `main.py` deixou de usar `ctypes` para RAM no Windows e passou a preferir a variavel `USBRAIN_RAM_MB`, com fallback por `wmic`.
- `start.bat` agora exporta `USBRAIN_RAM_MB` apos detectar a RAM.

## Proximos Passos
1. Sincronizar `main.py` e `start.bat` corrigidos no pendrive.
2. Rodar novo teste no Windows.
3. Comparar o novo `bootstrap_python.log` com o pre-fix.
4. Se o processo passar de `detect_hardware()`, seguir para a proxima falha real de boot.
