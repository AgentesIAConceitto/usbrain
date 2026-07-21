# Debug Session: windows-shell-access
- **Status**: [OPEN]
- **Issue**: USBRAIN no Windows afirma possuir shell/skills operacionais, mas nao executa comandos reais como `dir` e inventa saidas simuladas.
- **Debug Server**: Pending
- **Log File**: .dbg/trae-debug-log-windows-shell-access.ndjson

## Reproduction Steps
1. Gerar um pendrive USBRAIN atualizado.
2. Iniciar o USBRAIN em um PC Windows pelo fluxo normal.
3. No chat, pedir explicitamente a execucao de um comando simples como `dir` ou `dir E:\`.
4. Observar que o agente responde com texto/simulacao em vez de executar o comando real.

## Hypotheses & Verification
| ID | Hypothesis | Likelihood | Effort | Evidence |
|----|------------|------------|--------|----------|
| A | O modelo esta respondendo em linguagem natural sem realmente entrar no fluxo de `tool-calling`, apesar de as tools estarem registradas. | High | Low | Pending |
| B | O `ShellTool` esta registrado, mas indisponivel/filtrado no runtime Windows gerado pelo pendrive. | High | Med | Pending |
| C | O `ShellTool` executa, mas o resultado falha/eh descartado, fazendo o modelo improvisar uma resposta textual. | Med | Med | Pending |
| D | O prompt/orquestracao atual ainda permite que a LLM invente execucao de comandos quando a chamada de tool nao acontece. | High | Low | Pending |
| E | O artefato gerado no pendrive nao inclui alguma parte nova do core de tools, entao o runtime real difere do source local. | Med | Med | Pending |

## Log Evidence
- Debug Server ativo em `http://127.0.0.1:7777/event`.
- Env file gerado em `.dbg/windows-shell-access.env`.
- Instrumentacao adicionada em `agent.py` para registrar:
  - decisao de roteamento para tools;
  - mensagens tratadas por comando local/capabilities;
  - respostas da LLM sem `tool_calls` mesmo com tools habilitadas;
  - execucao e retorno de cada `tool_call`.
- Instrumentacao adicionada em `tools/shell.py` para registrar:
  - comando recebido;
  - `cwd`;
  - `returncode`;
  - previews de `stdout` e `stderr`;
  - timeout ou excecao, quando houver.
- `python3 -m py_compile agent.py tools/shell.py` passou.
- Nova `.app` do Builder foi recompilada e aberta para gerar um pendrive instrumentado.
