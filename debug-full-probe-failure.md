# [OPEN] Debug Session: full-probe-failure

## Contexto
- Sintoma: a criacao do USBRAIN chega a 100%, mas a fabrica bloqueia a aprovacao com falha no probe completo com LLM local.
- Ambiente: macOS, Builder empacotado `.app`, pendrive `/Volumes/USBRAIN`.
- Estado atual: montagem e validacao estrutural ja avancaram; falha restante esta no probe completo.

## Hipoteses
1. O probe falha ao iniciar o runtime local do pendrive por caminho/binario incorreto.
2. O probe falha porque nao ha modelo GGUF valido disponivel para carga local.
3. O runtime inicia, mas o endpoint local nao sobe dentro do timeout esperado.
4. O probe falha por divergencia entre manifests/runtime e nao por ausencia fisica de arquivos.
5. O app empacotado ainda resolve algum caminho auxiliar fora do bundle durante o probe.

## Plano
1. Instrumentar o caminho do `run_full_local_llm_probe`.
2. Reproduzir a falha com a build atual.
3. Coletar evidencia do ponto exato de quebra.
4. Corrigir minimamente com base na evidencia.
5. Validar novamente.
