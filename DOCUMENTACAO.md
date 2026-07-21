# 🧠 USBRAIN — Documentação Completa

> **Assistente de IA Portátil e Offline-First**
> Roda direto de um pendrive USB. Zero instalação. Plugou, usou.

---

## 📋 Índice

1. [Visão Geral](#-visão-geral)
2. [Origem e Motivação](#-origem-e-motivação)
3. [Localização no Disco](#-localização-no-disco)
4. [Arquitetura do Sistema](#-arquitetura-do-sistema)
5. [Estrutura de Arquivos](#-estrutura-de-arquivos)
6. [Componentes Detalhados](#-componentes-detalhados)
7. [Personalidade](#-personalidade)
8. [Modelos de IA](#-modelos-de-ia)
9. [Como Usar](#-como-usar)
10. [API REST](#-api-rest)
11. [Build e Deploy](#-build-e-deploy)
12. [Pendrive Físico — Status](#-pendrive-físico--status)
13. [Problemas Conhecidos](#-problemas-conhecidos)
14. [Dependências](#-dependências)

---

## 🎯 Visão Geral

**USBRAIN** é um agente de IA portátil que roda **totalmente offline** a partir de um pendrive USB. Combina:

- **LLM local** via llamafile (~21 tok/s em CPU M1)
- **RAG** com embeddings BGE-M3 (vetores em SQLite)
- **Web UI** com chat, upload de documentos e pesquisa na web
- **Ferramentas** (shell, arquivos, rede, diagnósticos Windows)
- **Personalidade configurável** sem editar código
- **Offline-first**: só baixa dependências no build; o pendrive final é autossuficiente
- **Detecção automática de hardware** — escolhe o melhor modelo baseado na RAM disponível

---

## 🧬 Origem e Motivação

**Projeto iniciado em 19-20/06/2026.**

A motivação surgiu da necessidade de um assistente de IA que funcionasse **sem GPU, sem internet, sem instalação** — plugou num PC Windows qualquer e já era produtivo.

### Cronologia do Desenvolvimento

| Data | Evento |
|------|--------|
| 19/06 (tarde) | Pesquisa de concorrentes: USBRAIN nome livre no GitHub. Análise do **Nanobot (HKUDS)** — 21k+ stars, ~4k linhas core, código limpo, base do agente. Scaffold inicial |
| 19/06 (noite) | Agent.py, tools, skills, RAG, WebUI, llamafile. Teste com Qwen3-8B rodando a **21 tok/s em CPU M1** |
| 20/06 (manhã) | **USBRAIN v2 — OFFLINE-FIRST**: `build_pendrive.py` com Python embed 3.12, wheels pré-baixados. Pendrive SanDisk 8GB formatado exFAT |
| 20/06 (tarde) | Primeiro build físico: 5.9GB/8GB usados (~2GB livres). Ejetado e testado no Windows |
| 21/06 | Documentação gerada. BGE-M3 ajustado com auto-index a cada 5 min |

### Decisões de Arquitetura

| Decisão | Motivo |
|---------|--------|
| **Nanobot como base** | Código pequeno (~4k linhas), limpo, fácil de modificar. 21k stars |
| **llamafile em vez de ollama** | Binary único, sem instalação, portátil |
| **Qwen3-8B em vez de Phi-4** | Qwen3-8B Q4_K_M (4.7GB) cabe em 8GB com margem |
| **Python embed (Windows)** | Zero dependências no Windows — Python já incluso |
| **exFAT em vez de FAT32** | Suporte a arquivos >4GB (modelos GGUF) |
| **BGE-M3 local** | RAG sem internet. Embeddings locais |

---

## 📁 Localização no Disco

```
~/Documents/IA/openclaw/clawd_workspace/projects/usbrain/
```

### Atalhos

| Recurso | Caminho |
|---------|---------|
| Projeto raiz | `projects/usbrain/` |
| Código fonte | `projects/usbrain/agent.py`, `main.py` |
| Interface web | `projects/usbrain/webui/` |
| RAG/BGE | `projects/usbrain/rag/` |
| Ferramentas | `projects/usbrain/tools/` |
| Skills | `projects/usbrain/skills/` |
| Modelos | `projects/usbrain/models/` |
| Build pendrive | `projects/usbrain/build_pendrive.py` |
| Personalidade | `projects/usbrain/personality.md` |

---

## 🏗️ Arquitetura do Sistema

```
┌─────────────────────────────────────────────────────────┐
│                     USBRAIN (Pendrive)                   │
│                                                         │
│  ┌─────────────┐    ┌──────────────┐    ┌───────────┐  │
│  │  Web UI     │    │  USBAgent    │    │  Tools    │  │
│  │  (aiohttp)  │◄──►│  (Core)     │◄──►│  + Skills │  │
│  │  :8000      │    │              │    │           │  │
│  └─────────────┘    └──────┬───────┘    └───────────┘  │
│                            │                            │
│              ┌─────────────┼─────────────┐              │
│              ▼             ▼             ▼              │
│  ┌──────────────┐  ┌────────────┐  ┌──────────────┐    │
│  │  LLM Server  │  │  BGE RAG   │  │  SQLite      │    │
│  │  (llamafile) │  │  :9090     │  │  (vetor.db)  │    │
│  │  :8080       │  │            │  │              │    │
│  └──────────────┘  └────────────┘  └──────────────┘    │
│                                                         │
│  ┌──────────────────────────────────────────────────┐   │
│  │  personality.md (config do agente em Markdown)   │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Fluxo de uma mensagem

1. Usuário envia mensagem pelo **Web UI** (`POST /api/chat`)
2. `WebUIServer` repassa para `USBAgent.chat()`
3. Verifica comando local (`/help`, `/stats`) → responde direto
4. Consulta **RAG** (`_query_rag`) via BGE server (`:9090`)
5. Monta prompt com personalidade + histórico + contexto RAG
6. Envia pro **LLM** (`llamafile :8080/v1/chat/completions`)
7. Se o LLM pedir tool calls → executa tools e retorna resultado
8. Resposta em Markdown pro usuário

---

## 📂 Estrutura de Arquivos

```
usbrain/
├── main.py                    # Entry point (orquestra servidores)
├── agent.py                   # Core do agente (529 linhas)
├── build_pendrive.py          # Build do pendrive (316 linhas)
├── build_pendrive_main_part.py# Parte principal do build
├── personality.md             # Personalidade editável (nome: Jarvis)
├── requirements.txt           # Dependências Python
├── README.md                  # Leia-me
├── README.txt                 # Leia-me para o pendrive
├── vetor.db                   # Banco vetorial SQLite
├── vetor.db-shm / vetor.db-wal
│
├── start.bat                  # Inicializador Windows (95 linhas)
├── start.sh                   # Inicializador Linux/macOS (135 linhas)
│
├── llamafile                  # Binary llama.cpp (macOS)
├── llamafile-linux            # Binary llama.cpp (Linux)
├── cache/                     # Cache de build
│   ├── llamafile.exe          # Binary llama.cpp (Windows)
│   ├── python-3.12.9-embed-amd64.zip
│   ├── python/                # Python 3.12 embed (Windows)
│   └── wheels/                # Wheels pré-baixados (21 wheels)
│
├── models/
│   └── Qwen_Qwen3-8B-Q4_K_M.gguf  # 4.7GB
│
├── tools/                     # Ferramentas (adaptado do Nanobot)
│   ├── __init__.py
│   ├── base.py                # Classe abstrata Tool
│   ├── registry.py            # ToolRegistry
│   ├── filesystem.py          # ReadFileTool, WriteFileTool
│   ├── shell.py               # ShellTool
│   └── web.py                 # WebSearchTool (Brave API)
│
├── skills/                    # Habilidades especializadas
│   ├── files.py               # FileSearchTool, FileInfoTool
│   ├── network.py             # NetworkDiagnoseTool (ping, traceroute, DNS, port scan)
│   ├── system.py              # SystemInfoTool (SO, CPU, RAM, disco, processos)
│   └── winfix.py              # WindowsFixTool (DISM, SFC, chkdsk, eventos)
│
├── rag/                       # RAG Local
│   ├── __init__.py
│   ├── server.py              # BGE HTTP server (embed + search + store + add_url + add_search)
│   └── bge-small-model/       # BAAI/bge-small-en-v1.5 (~33MB, 384d)
│
└── webui/                     # Interface web
    ├── server.py              # Servidor aiohttp (13 endpoints)
    ├── templates/
    │   └── index.html         # Template HTML (tema escuro, modo USBRAIN)
    └── static/
```

---

## 🔧 Componentes Detalhados

### main.py — Entry Point

Orquestra inicialização de todos os serviços:

1. **Detecta hardware** (RAM) para recomendar modelo
2. **Inicia BGE server** (embedding + vector store, porta 9090)
3. **Inicia LLM** (llamafile, porta 8080)
4. **Inicia Web UI** (porta 8000, fallback automático até +99)
5. **Registra tools e skills** no agente

```bash
python3 main.py --model-path models/Qwen_Qwen3-8B-Q4_K_M.gguf
```

**Flags CLI:**
- `--host` / `--port` — Web UI
- `--bge-port` — BGE server
- `--llm-port` — LLM server
- `--model` — Nome do modelo
- `--model-path` — Caminho do GGUF
- `--no-llm` — Apenas BGE + Web UI
- `--db` — Caminho do vetor.db

### agent.py — USBAgent (529 linhas)

| Método | Função |
|--------|--------|
| `chat()` | Pipeline: RAG → LLM → tools → resposta |
| `_load_personality()` | Parseia personality.md → system prompt |
| `_query_rag()` | Embedding via BGE:9090 → search SQLite |
| `_handle_tool_calls()` | Executa tools (shell, filesystem, web) |
| `_handle_local_commands()` | `/help`, `/reset`, `/stats` (sem LLM) |
| `add_document()` | Chunk + embed + store no vetor.db |
| `detect_hardware()` | RAM → escolha de modelo |
| `reload_personality()` | Recarrega sem reiniciar |

### tools/ — Ferramentas (Base: Nanobot)

| Tool | Descrição | Parâmetros |
|------|-----------|------------|
| `ShellTool` | Executa comandos shell no PC hospedeiro | `command`, `timeout?` |
| `ReadFileTool` | Lê qualquer arquivo | `path` |
| `WriteFileTool` | Cria/escreve arquivos | `path`, `content` |
| `WebSearchTool` | Pesquisa via Brave API | `query`, `max_results?` |

### skills/ — Habilidades Especializadas

| Skill | Descrição |
|-------|-----------|
| `NetworkDiagnoseTool` | Ping, traceroute, DNS lookup, port scan |
| `SystemInfoTool` | SO, CPU, RAM, disco, processos |
| `FileSearchTool` | Busca por padrão/glob |
| `FileInfoTool` | Metadados de arquivos |
| `WindowsFixTool` | DISM, SFC, chkdsk, eventos do Windows |

### rag/ — RAG Local

**Servidor HTTP** (BGE-Small, 384d, ~33MB) com SQLite como vector store.

| Endpoint | Método | Descrição |
|----------|--------|-----------|
| `/embed` | POST | Embedding vector do texto |
| `/store` | POST | Armazena chunk + embedding |
| `/search` | POST | Busca top-k por similaridade de cosseno |
| `/stats` | GET | Contagem de chunks |
| `/add_url` | POST | Baixa URL, chunkifica e indexa |
| `/add_search` | POST | Pesquisa web, chunkifica e indexa |

**Storage:** SQLite + WAL, busca força bruta por similaridade de cosseno.

### webui/ — Interface Web

**Servidor:** aiohttp, porta 8000
**Template:** HTML escuro, responsivo, com badges de hardware

**Endpoints:**

| Rota | Método | Descrição |
|------|--------|-----------|
| `/` | GET | Página principal |
| `/api/chat` | POST | Mensagem pro agente |
| `/api/upload` | POST | Upload TXT/MD/PDF |
| `/api/add_url` | POST | Adicionar URL à base |
| `/api/search_web` | GET | Pesquisar web e indexar |
| `/api/stats` | GET | Status do sistema |
| `/api/reset` | GET | Resetar conversa |
| `/api/personality` | GET/POST | Ler/alterar personalidade |
| `/api/skills` | GET | Listar tools |

---

## 🎭 Personalidade

Configurada em **`personality.md`**, editável a qualquer momento — sem precisar reiniciar o servidor.

### Configuração Atual

```yaml
Agent Name: Jarvis
Tone: Friendly & Casual + Sarcastic & Witty (multi-select)
Empathy: 4/5 (alto)
Verbosity: 3/5 (médio)
Humor: 4/5 (frequente)
Tech Depth: 4/5 (alto)
```

### Problema Conhecido: Nome do Agente

O **Qwen3-8B** tem um viés de treinamento forte — ele tende a se chamar de "USBRAIN" ou "Qwen" mesmo com o system prompt configurado como "Jarvis". O nome aparece corretamente na UI (header, badges), mas o LLM pode ignorar a renomeação. Soluções testadas:

- ✅ Forçar `You MUST identify as {agent_name}` no prompt — **parcial**
- ✅ Nome visível na UI (header + badges) — **funciona**
- ❌ Substituir system prompt por completo — **Qwen3 ignora**

### Seções Configuráveis

| Campo | Valores | Atual |
|-------|---------|-------|
| `Agent Name` | Texto livre | `Jarvis` |
| `Tone` | Professional / Friendly / Sarcastic / Technical | 2 selecionados |
| `Empathy Level` | 1 (Low) a 5 (High) | 4 |
| `Verbosity` | 1 (Concise) a 5 (Detailed) | 3 |
| `Humor` | 1 (None) a 5 (Frequent) | 4 |
| `Tech Depth` | 1 (Basic) a 5 (Expert) | 4 |
| `Custom Instruction` | Texto livre | Companheiro digital, natural, emojis |
| `Brave API Key` | Key | `BSAyaH63j2Mns6noio_RrQXjmiSUYD-` |

---

## 🤖 Modelos de IA

### Recomendação por RAM

| RAM | Modelo Recomendado | Tamanho |
|-----|-------------------|---------|
| ≥32GB | Qwen3-32B Q4 | ~18GB |
| ≥16GB | Phi-4-14B Q4 | ~8GB |
| <16GB | Qwen3-8B Q4 | ~4.7GB |

### Modelo Instalado

**Qwen_Qwen3-8B-Q4_K_M.gguf** — 4.7GB em `models/`

### Performance

- **21 tokens/segundo** em CPU-only (Mac M1)
- **Sem GPU** — compatível com qualquer PC
- **llamafile 0.10.3** como runtime (binary único)

### Download

Coloque arquivos `.gguf` em `models/`. O `build_pendrive.py` detecta automaticamente.

---

## ▶️ Como Usar

### Desenvolvimento (Mac)

```bash
cd ~/Documents/IA/openclaw/clawd_workspace/projects/usbrain
pip install -r requirements.txt

# Com LLM
python3 main.py --model-path models/Qwen_Qwen3-8B-Q4_K_M.gguf

# Só RAG + Web UI (sem LLM)
python3 main.py --no-llm
```

### Pendrive Windows

1. Conecte o pendrive USBRAIN
2. Execute `START.BAT`
3. Navegador abre em `http://127.0.0.1:8000`
4. **Tudo offline. Zero instalação.**

O `start.bat` faz automaticamente:
- Detecta RAM (WMIC) → recomenda modelo
- Verifica Python embed em `python/`
- Inicia llamafile com o modelo GGUF
- Busca porta livre (8000→8099)
- Abre o navegador

### Pendrive macOS/Linux

```bash
cd /Volumes/USBRAIN
chmod +x start.sh
./start.sh
```

O `start.sh` baixa o llamafile automaticamente se não existir (primeira execução).

### Comandos do Chat

| Comando | Função |
|---------|--------|
| `/help` ou `skills` | Lista skills disponíveis |
| `/reset` ou `limpar` | Nova conversa |
| `/stats` ou `status` | Status do sistema |
| `/tools` | Lista ferramentas registradas |

---

## 🌐 API REST

### BGE Server (porta 9090)

```bash
# Embedding
curl -X POST http://127.0.0.1:9090/embed \
  -H "Content-Type: application/json" \
  -d '{"text": "O que é USBRAIN?"}'

# Busca semântica
curl -X POST http://127.0.0.1:9090/search \
  -H "Content-Type: application/json" \
  -d '{"embedding": [...], "top_k": 5}'

# Indexar URL externa
curl -X POST http://127.0.0.1:9090/add_url \
  -H "Content-Type: application/json" \
  -d '{"url": "https://exemplo.com/artigo"}'

# Pesquisar web e indexar automaticamente
curl -X POST http://127.0.0.1:9090/add_search \
  -H "Content-Type: application/json" \
  -d '{"query": "como instalar python no windows"}'

# Stats
curl http://127.0.0.1:9090/stats
```

### Web UI (porta 8000)

```bash
# Status do sistema
curl http://127.0.0.1:8000/api/stats

# Listar skills
curl http://127.0.0.1:8000/api/skills

# Ler personalidade
curl http://127.0.0.1:8000/api/personality
```

---

## 🏗️ Build e Deploy

### build_pendrive.py — Construtor Offline v2

Script de 316 linhas que cria um pendrive Windows **100% autossuficiente**.

#### Flags

| Flag | Descrição |
|------|-----------|
| `--download` | Apenas prepara dependências (cache) |
| `--format` | Formata como exFAT (nome: USBRAIN) |
| `--with-model PATH` | Copia modelo GGUF |

#### Pipeline

```
1. Formata pendrive como exFAT (nome: "USBRAIN")
2. Copia Python 3.12 embed (baixado de python.org)
3. Instala wheels no site-packages do embed (Jinja2, Werkzeug, aiohttp, httpx)
4. Copia llamafile.exe (305MB)
5. Copia modelo GGUF (4.7GB)
6. Copia agent.py, main.py, tools, skills, rag, webui
7. Cria diretórios knowledge/ e models/
8. Remove metadados Apple Double (._*)
```

#### Exemplos

```bash
# Apenas preparar cache
python3 build_pendrive.py --download

# Formatando + copiando
python3 build_pendrive.py /Volumes/NO_NAME --format

# Com modelo incluso
python3 build_pendrive.py /Volumes/USB --with-model models/Qwen_Qwen3-8B-Q4_K_M.gguf

# Modo interativo
python3 build_pendrive.py
```

---

## 💾 Pendrive Físico — Status

### Hardware

| Item | Info |
|------|------|
| Modelo | SanDisk |
| Capacidade | 8 GB |
| Formato | exFAT |
| Nome do volume | `USBRAIN` |

### Ocupação

| Status | Tam. no pendrive | Tam. no projeto (Mac) |
|--------|-----------------|----------------------|
| v1 (20/06) | 5.9 GB / 8 GB (74%) | — |
| v2 (20/06) | 7.5 GB / 8 GB (94% c/ 1.3GB Qwen) | — |
| Atual | — | 5.7 GB no disco local |

**Obs:** O pendrive físico de 8GB está formatado como exFAT com nome "USBRAIN". O último build teve problemas de I/O com exFAT no macOS, e os dados foram copiados em etapas. Pode ser necessário rebuild.

---

## 🧪 Testes Realizados

### Teste do LLM (Mac M1)
- ✅ Qwen3-8B Q4_K_M carregado via llamafile
- ✅ 21 tok/s em CPU-only
- ✅ Resposta: "Olá! Tudo bem? Sou o USBRAIN, um assistente de IA portátil..."
- ❌ System prompt ignorado parcialmente (nome "USBRAIN" vs "Jarvis")

### Teste do Pendrive
- ✅ Formatado como exFAT
- ✅ Build completado (5.1GB total)
- ✅ Ejetado com segurança
- 🔲 **Pendente:** teste no Windows (pendente desde 20/06)

### Check do Build
- ✅ 1301 arquivos
- ✅ 80 módulos Python site-packages
- ✅ Modelo 4.7GB
- ✅ Todos os checks passaram

---

## ⚠️ Problemas Conhecidos

| # | Problema | Status | Solução |
|---|----------|--------|---------|
| 1 | **Qwen3-8B ignora renomeação** | Aberto | Nome só na UI; LLM continua se chamando "USBRAIN" |
| 2 | **exFAT no macOS lento** | Aberto | I/O lento para milhares de arquivos pequenos |
| 3 | **Windows não testado** | Pendente | Pendrive ejetado em 20/06, aguardando teste |
| 4 | **BGE sem sentence_transformers** | Mitigado | Fallback hash-based se não disponível |
| 5 | **space em disco** | OK | 8GB é justo; modelo maior não cabe (Qwen3-14B ~9GB) |

---

## 📦 Dependências

### Runtime (requirements.txt)

```
aiohttp
httpx
sentence-transformers  # opcional (fallback sem ele)
numpy
```

### Build (Mac)

- Python 3.12+ via Homebrew
- `pip` para download de wheels

### Pendrive Windows

- **Zero dependências externas**
- Python 3.12 embed incluso em `python/`
- 21 wheels pré-instalados (aiohttp, httpx, certifi, jinja2, werkzeug, etc.)
- llamafile.exe incluso (305MB)

---

## 🔗 Links Úteis

- **Nanobot (base do agente):** https://github.com/HKUDS/nanobot
- **llamafile:** https://github.com/mozilla-ai/llamafile
- **Qwen3 (Alibaba):** https://huggingface.co/Qwen
- **BGE-M3 (BAAI):** https://huggingface.co/BAAI/bge-m3
- **Brave Search API:** https://api.search.brave.com

---

## 👨‍💻 Manutenção

### Iniciar no Mac (desenvolvimento)

```bash
cd ~/Documents/IA/openclaw/clawd_workspace/projects/usbrain
python3 main.py --no-llm                # só RAG + UI
python3 main.py --model-path models/Qwen_Qwen3-8B-Q4_K_M.gguf  # completo
```

### Resetar banco vetorial

```bash
rm -f vetor.db vetor.db-shm vetor.db-wal
```

### Recarregar personalidade

Edite `personality.md` com qualquer editor. Recarregue via:

```bash
# API
curl -X POST http://127.0.0.1:8000/api/personality \
  -H "Content-Type: application/json" \
  -d '{"tone": "sarcastic"}'
```

### Rebuild do pendrive

```bash
python3 build_pendrive.py --download    # atualiza cache
python3 build_pendrive.py /Volumes/USBRAIN --format --with-model models/Qwen_Qwen3-8B-Q4_K_M.gguf
```

---

*Documentação gerada em 21/06/2026. Baseada em análise semântica BGE-M3 de 83.277 mensagens do chat.*
