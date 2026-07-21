# Debug Session: windows-mount-error

- **Status**: [OPEN]
- **Issue**: o pendrive monta no Windows, inicia o boot do USBRAIN e volta a falhar no mesmo erro recorrente do runtime.
- **Scope**: diagnosticar a causa com evidencias do volume e dos logs, sem alterar a logica de negocio nesta fase inicial.

## Reproduction Steps
1. Gravar o pendrive no macOS.
2. Ejetar com `diskutil eject`.
3. Conectar o pendrive no Windows.
4. Observar o volume montar e o boot do USBRAIN iniciar.
5. Ver o mesmo erro recorrente durante a inicializacao.
6. Reconectar o pendrive no macOS para coleta de evidencias.

## Hypotheses
| ID | Hypothesis | Likelihood | Effort | Evidence |
|----|------------|------------|--------|----------|
| A | O volume `exFAT` ficou inconsistente apos a gravacao e o Windows detecta erro estrutural no filesystem. | High | Medium | Rejected |
| B | Metadados ou arquivos auxiliares gerados no macOS estao interferindo na leitura do volume pelo Windows. | Medium | Low | Partial |
| C | A copia final deixou arquivos grandes ou diretorios em estado parcial, e o Windows interpreta isso como falha de montagem ou volume corrompido. | High | Medium | Partial |
| D | O problema nao esta no conteudo do projeto, mas no procedimento de flush/ejetar apos escrita intensa no pendrive. | Medium | Medium | Partial |

## Log Evidence
- `bootstrap_python.log` mostra que o Windows montou o pendrive como `I:\` e executou `start.bat`, portanto houve montagem bem-sucedida nesta rodada.
- `bootstrap_python.log` registra o boot ate `Iniciando BGE server na porta 9090...` e encerra com `BGE nao ficou pronto a tempo.`
- `llamafile.log` mostra que o processo do `BGE` morreu antes da readiness com `returncode 3221225477`.
- `llamafile.log` tambem registra `Windows fatal exception: access violation` durante o import de `numpy` em `I:\python\Lib\site-packages\numpy\core\_internal.py`.
- `bge_boot.log` para em `rag.server: importing numpy`, consistente com a falha no runtime Python do `BGE`.
- `diskutil verifyVolume /Volumes/USBRAIN` retornou `The volume USBRAIN appears to be OK` com `exit code 0`.
- `/sbin/fsck_exfat -n /dev/disk10s1` tambem retornou volume OK sem erros estruturais.
- O volume contem metadados do macOS, principalmente `.Spotlight-V100` e `.fseventsd`, mas nao apareceu evidencia de que isso tenha impedido a montagem no ciclo atual.
- `bootstrap_python.log` com a nova instrumentacao mostra `smoke_ctypes_start` seguido de `Windows fatal exception: access violation`, com `smoke_ctypes_exit=-1073741819`.
- O mesmo `bootstrap_python.log` mostra `smoke_numpy_exit=-1073741819` e `smoke_sentence_transformers_exit=-1073741819`, indicando que as falhas superiores sao consequencia do mesmo problema de base no runtime.
- A evidencia nova rebaixa a hipotese de falha especifica do `numpy`: o primeiro crash reproduzivel ocorre em `ctypes`, antes do boot do `BGE`.
- Com o runtime baseado em `nuget`, `bootstrap_python.log` passou em `ctypes`, `numpy` e `torch`, o que confirma que o bloqueio anterior do `python-embed` foi superado.
- A nova falha observada ocorre ao importar `sentence_transformers`, com `FileNotFoundError` para `I:\python\Lib\site-packages\sklearn\.libs\vcomp140.dll`.
- Inspecao local mostrou que o wheel `scikit_learn-1.5.2` contem `sklearn/.libs/msvcp140.dll` e `sklearn/.libs/vcomp140.dll`, mas o builder os removia/ignorava por tratar qualquer caminho iniciado com `.` como lixo oculto.
- Na rodada mais recente, o Windows nem chegou ao `bootstrap_python.log`: `erro_terminal.txt` mostra `Falha ao extrair o runtime Python`.
- Inspecao do pendrive no macOS mostra que o runtime local em `python/` estava incompleto para a validacao do `start.bat`: `transformers/__init__.py` estava ausente no volume, embora exista normalmente no cache local.
- Comparacao local:
  - cache: `cache/python/Lib/site-packages/transformers` com 1751 arquivos
  - pendrive: `python/Lib/site-packages/transformers` com apenas 305 arquivos
- Conclusao: o sintoma "piorou" porque o `start.bat` rejeitou o runtime incompleto no proprio pendrive e tentou cair para `python_runtime.zip`; a extracao fallback falhou antes de o Python principal subir.

## Instrumentation
- `start.bat` recebeu smoke tests adicionais antes do `main.py` para registrar no `bootstrap_python.log` os imports criticos do runtime Windows:
  - `ctypes`
  - `numpy`
  - `torch`
  - `sentence_transformers`
- Objetivo: separar se a queda acontece no `main.py`/`rag.server.py` ou se o runtime ja falha em imports isolados antes do boot completo.

## Remediation
- O builder foi alterado para abandonar o `python-embed` e usar o pacote oficial `nuget` do Python 3.12 para Windows como base do runtime portatil.
- O `start.bat` foi ajustado para validar o novo layout do runtime:
  - `python3.dll`
  - `DLLs\_ctypes.pyd`
  - `DLLs\libffi-8.dll`
  - `Lib\site.py`
- Os smoke tests no `start.bat` tambem foram simplificados para comandos inline, removendo a dependencia do label `run_smoke_test`.
- Validacao local no macOS apos a gravacao confirma no volume `USBRAIN`:
  - `python/DLLs/_ctypes.pyd`
  - `python/DLLs/libffi-8.dll`
  - `python/Lib/site.py`
  - `python_runtime.zip`
- O builder foi corrigido para preservar diretorios validos com ponto, como `sklearn/.libs`, removendo apenas lixo real do macOS (`._*`, `.DS_Store`, `.Spotlight-V100`, `.Trashes`, `.fseventsd`, `__MACOSX`).
- O runtime foi reconstruido e o pendrive regravado; validacao local agora confirma no volume:
  - `python/Lib/site-packages/sklearn/.libs/vcomp140.dll`
  - `python/Lib/site-packages/sklearn/.libs/msvcp140.dll`
- Ainda resta uma pendencia de integridade: validar que diretorios grandes como `transformers/` foram copiados integralmente para o pendrive antes da proxima ejecao.

## Verification Conclusion
- O usuario esclareceu que nao houve erro de montagem; houve montagem normal seguida do mesmo erro recorrente durante o boot.
- O que as evidencias sustentam e: o pendrive montou, o bootstrap iniciou, e a aplicacao caiu novamente no `BGE` durante o import de `numpy`.
- O estado do `exFAT` esta consistente no macOS neste momento, entao a falha principal registrada continua sendo de runtime, nao de filesystem.
- Com a nova instrumentacao, a causa mais baixa observavel no Windows deixou de ser `numpy` e passou a ser o import de `ctypes`.
- Isso aponta para problema no runtime Python embarcado para Windows, provavelmente em `_ctypes.pyd`, `libffi-8.dll` ou na estrategia de usar o `python-embed` com essa pilha de dependencias.
