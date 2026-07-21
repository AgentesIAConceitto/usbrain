[OPEN] Debug Session: usbrain-same-error

## Sintoma
- Usuario reporta "Mesmo erro" apos rebuild do `.app` com correcao de `webui/templates/index.html`.
- Esperado: o Builder deve concluir a criacao do USBRAIN sem reprovar o artefato final.
- Observado anteriormente: havia defeito real de empacotamento da Web UI, agora corrigido no bundle; ainda resta descobrir por que a UI continua acusando erro.

## Hipoteses
1. O usuario ainda executou uma instancia antiga do `.app`, e o pendrive foi gerado por um bundle anterior sem a correcao.
2. O bundle novo contem o template correto, mas a montagem no pendrive continua vindo de uma origem diferente de `source_app_root()` durante a execucao real.
3. O erro atual ja nao e da Web UI e sim de outro gate da fabrica, como validacao estrutural, smoke test ou `fullLocalLlmProbe`.
4. O Builder corrigido falha em persistir ou exibir o motivo real do erro, entao a mensagem observada pelo usuario esconde uma causa diferente no pipeline.
5. O volume `/Volumes/USBRAIN` esta sendo reaproveitado com residuos de execucoes anteriores, contaminando a validacao final.

## Evidencias a Coletar
- Caminho de origem efetivo usado pela montagem.
- Estagio exato do pipeline que falha.
- Relatorio persistido do full probe e/ou factory QA.
- Estrutura real gerada no pendrive apos a tentativa atual.
- Identidade da build/app em execucao durante o teste.

## Evidencias Confirmadas
- A montagem real da build empacotada usa `source_app_root()` do bundle e grava `webui/templates/index.html` corretamente em `/Volumes/USBRAIN`.
- No fluxo real da UI, `validate_assembled_build` concluiu com `passed-with-warnings`.
- No fluxo real da UI, `run_assembled_smoke_test` concluiu com `passed-with-warnings`.
- No fluxo real da UI, `run_full_local_llm_probe` falhou com exatamente `1` erro e `1` warning.
- A execucao direta de `/Volumes/USBRAIN/llamafile --help` no macOS falha com `OSError(8, 'Exec format error')`.
- A execucao via `/bin/sh /Volumes/USBRAIN/llamafile --help` retorna `0`.

## Causa Raiz Atual
- O preflight do `fullLocalLlmProbe` tentava executar `llamafile` diretamente para validar o binario do host.
- No macOS, esse binario exige o mesmo fallback via `/bin/sh` que ja existia no `main.py`.
- Resultado: o Builder bloqueava o pendrive no gate final mesmo com runtime potencialmente funcional.

## Evidencias Adicionais
- Na execucao real do `.app` sandboxado, `validate_assembled_build` e `run_assembled_smoke_test` passam com warnings toleraveis.
- Na execucao real do `.app` sandboxado, `run_full_local_llm_probe` continua falhando antes de subir o runtime por falta de Python hospedeiro detectavel.
- O relatorio persistido em `~/Library/Containers/org.usbrain.builder/Data/Library/Caches/org.usbrain.builder/full-local-llm-probe-last.json` mostra:
  - `probeStatus = failed`
  - `hostPythonCommand = indisponivel`
  - `stderr` de `/usr/bin/python3 --version`: `xcrun: error: cannot be used within an App Sandbox.`
  - `stderr` de `/bin/sh -lc 'exec /usr/bin/python3 \"$@\"' ...`: `/bin/sh: /etc/profile: Operation not permitted` e novamente `xcrun: error: cannot be used within an App Sandbox.`

## Causa Raiz Atualizada
- O problema remanescente nao e mais de logica do pendrive nem do probe.
- O `.app` sandboxado do macOS nao consegue usar o Python do host para executar `main.py` durante o `fullLocalLlmProbe`.
- Sem um Python executavel embutido no bundle macOS, o gate de probe completo dentro do Builder nao consegue ser executado de forma confiavel no sandbox.

## Regras desta sessao
- Antes de evidencias de runtime, nao alterar logica de negocio.
- Primeira alteracao no codigo existente deve ser apenas instrumentacao.
- Limpeza dos artefatos de debug apenas apos confirmacao do usuario.
