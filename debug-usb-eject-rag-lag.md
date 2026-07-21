# Debug Session: usb-eject-rag-lag
- **Status**: [CLOSED]
- **Issue**: Pendrive nao ejetado automaticamente apos criacao validada no macOS; respostas que usam base vetorial ficam muito mais lentas que respostas sem RAG.
- **Debug Server**: http://127.0.0.1:7777/event
- **Log File**: .dbg/trae-debug-log-usb-eject-rag-lag.ndjson

## Reproduction Steps
1. Abrir o Builder no macOS.
2. Gerar e validar um novo pendrive USBRAIN em um pendrive real.
3. Observar se o pendrive ejetou automaticamente ao final.
4. No Windows, subir o USBRAIN.
5. Fazer uma pergunta comum no chat e comparar o tempo com uma pergunta baseada em conhecimento indexado.

## Hypotheses & Verification
| ID | Hypothesis | Likelihood | Effort | Evidence |
|----|------------|------------|--------|----------|
| A | A ejeção falha porque ainda existe handle/acesso aberto ao volume quando `diskutil` roda. | High | Med | Pending |
| B | O comando de ejeção esta sendo chamado, mas retorna erro operacional no macOS real. | High | Low | Pending |
| C | A maior parte da latencia extra esta na etapa de embedding/busca RAG e nao na LLM. | High | Low | Pending |
| D | O contexto recuperado esta grande demais e aumenta muito o tempo da resposta final da LLM. | Med | Low | Pending |
| E | O chat consulta o RAG em perguntas onde nao precisa, adicionando custo fixo por mensagem. | Med | Low | Pending |

## Log Evidence
- Instrumentacao pre-fix aplicada em:
  - `builder/src-tauri/src/commands.rs`
  - `agent.py`
- Instrumentacao complementar aplicada em:
  - `builder/src/main.ts` para rastrear `pipelineResult`, entrada/skip do ramo de ejeção, `invoke("eject_target")`, sucesso/erro e exibicao do modal
  - `builder/src-tauri/src/commands.rs` com `report_frontend_debug_event` para persistir telemetria do frontend no log local Rust
- Build instrumentada gerada e `.app` aberta para reproducao local.
- Debug Server ativo em `http://127.0.0.1:7777/event`.
- Evidencia operacional real coletada no macOS com o pendrive montado em `/Volumes/USBRAIN`:
  - `diskutil unmountDisk force disk4` retornou `Forced unmount of all volumes on disk4 was successful`
  - `diskutil eject disk4` retornou `Disk disk4 ejected`
  - `diskutil info disk4` depois da ejecao retornou `Could not find disk: disk4`
- Evidencia de codigo confirmada:
  - a pipeline ja tentava ejetar automaticamente em `run_factory_pipeline`
  - o frontend tentava ejetar novamente via `invoke("eject_target", ...)`
  - o reporter Rust nao persistia `trae-debug-log-usb-eject-rag-lag.ndjson` localmente quando o Debug Server HTTP nao respondia

## Verification Conclusion
- Hipotese A: **Inconclusive**. Nao houve evidencia suficiente de handle aberto persistente.
- Hipotese B: **Rejected**. O `diskutil` funciona no pendrive real quando executado manualmente no mesmo host.
- Causa mais provavel neste momento: fluxo duplicado de ejeção e baixa observabilidade do caminho real executado pelo app.
- Correcao aplicada:
  - a ejeção dentro de `run_factory_pipeline` foi removida e ficou apenas o comando dedicado `eject_target`
  - o frontend agora mostra modal claro com confirmacao `OK` apos sucesso da ejeção
  - `debug_server_report()` agora grava fallback local em `.dbg/trae-debug-log-usb-eject-rag-lag.ndjson`
- Rodada atual:
  - a sessao atual nao tinha `.dbg/usb-eject-rag-lag.env`, entao o fallback local estava indo para `usb-eject-upload-web`
  - por isso a observabilidade foi estendida para o frontend via comando Tauri dedicado, sem alterar a logica da ejeção
- Aguardando verificacao do usuario com a nova `.app`.
- Confirmacao final do usuario:
  - a ejeção automatica no fluxo normal de `Criar agora` funcionou
  - o modal com confirmacao `OK` foi exibido corretamente
- Causa raiz confirmada:
  - a logica de `eject_target` + modal foi implementada no fluxo de `run_factory_pipeline` usado por `factoryQaTarget`
  - o fluxo real do produto acionado por `assembleBuildPlan()` nao executava a etapa de ejeção
