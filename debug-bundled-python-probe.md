# Debug Session: bundled-python-probe

Status: [OPEN]
Started: 2026-07-02

## Symptom

O `USBRAIN Builder.app` continua reprovando a criação porque o `fullLocalLlmProbe` falha dentro do App Sandbox no macOS.

## Expected

O probe completo deve usar um Python executável compatível com o sandbox do `.app` e aprovar o pendrive funcional.

## Hypotheses

1. O runtime `python-build-standalone` extraído não contém um executável relocável pronto para uso via bundle do Tauri.
2. O `tauri.conf.json` atual não empacota a árvore do Python macOS em `Resources`, então o `.app` continua sem um Python embutido utilizável.
3. O backend em `commands.rs` detecta apenas Python do host e ainda não resolve corretamente um Python empacotado a partir do diretório de recursos do app.
4. Mesmo com o binário presente no bundle, o probe falha por dependências adjacentes ausentes ou por caminho de `PYTHONHOME`/`sys.prefix` incorreto.
5. A UI continua exibindo a mesma falha por consumir um relatório antigo de cache, mesmo após a correção estrutural do probe.

## Evidence Plan

- Inspecionar a estrutura real do runtime extraído e localizar `python3`, libs e layout necessário.
- Verificar como os recursos já são empacotados em `tauri.conf.json`.
- Mapear em `commands.rs` onde o Python é detectado e injetado no probe.
- Só então adicionar instrumentação mínima para comprovar o caminho do Python usado pelo `.app`.

## Evidence Collected

- O standalone extraído é válido: `python/bin/python3.10 --version` retorna `Python 3.10.20`.
- O binário é relocável com `@executable_path/../lib`, portanto a árvore `bin/` + `lib/` precisa ser preservada no bundle.
- O primeiro empacotamento em `tauri.conf.json` achatou a árvore em `Resources/app-cache/python-macos/`, o que quebrou o layout esperado.
- O mapeamento foi corrigido para empacotar `bin/`, `include/`, `lib/` e `share/` separadamente.
- O `.app` rebuildado agora contém `Contents/Resources/app-cache/python-macos/bin/python3.10`, e esse executável roda com sucesso de dentro do bundle.
- O bundle foi re-assinado ad hoc com `Entitlements.plist`, e `codesign --verify --deep --strict` voltou a passar.

## Current Status

- Hipótese 1: falsificada. O runtime standalone é executável e relocável.
- Hipótese 2: confirmada e corrigida. O problema estava no empacotamento achatado do Tauri.
- Hipótese 3: confirmada e corrigida. O backend agora prioriza o Python embutido antes do host.
- Hipótese 4: ainda pendente de confirmação pela execução real da UI sandboxada.
- Hipótese 5: ainda pendente de confirmação, porque a UI precisa ser reproduzida novamente com o bundle novo.

## Exit Criteria

- O `.app` encontra um Python embutido válido.
- O `fullLocalLlmProbe` registra que usou o runtime embutido.
- A execução sandboxada deixa de falhar por `hostPythonCommand = indisponivel`.
