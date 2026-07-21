#!/bin/bash
# USBRAIN - Portable AI Agent (Linux/macOS)
set -e

DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"

echo "============================================"
echo "   🧠 USBRAIN - Portable AI Agent"
echo "============================================"
echo ""

# ============================================================
# PASSO 0: Baixar llamafile pro SO se nao existir
# ============================================================
UNAME_S=$(uname)
if [ "$UNAME_S" = "Darwin" ] && [ ! -f "llamafile" ]; then
    echo "[0/5] Baixando llamafile para macOS..."
    curl -L -o llamafile "https://huggingface.co/mozilla-ai/llamafile_0.10/resolve/main/llamafile-0.10.3" 2>/dev/null
    chmod +x llamafile
elif [ "$UNAME_S" = "Linux" ] && [ ! -f "llamafile-linux" ]; then
    echo "[0/5] Baixando llamafile para Linux..."
    curl -L -o llamafile-linux "https://github.com/mozilla-ai/llamafile/releases/download/0.10.3/llamafile-0.10.3" 2>/dev/null
    chmod +x llamafile-linux
fi

# ============================================================
# PASSO 1: Verificar Python
# ============================================================
echo "[1/5] Verificando Python..."

if command -v python3 &>/dev/null; then
    PY=python3
elif command -v python &>/dev/null; then
    PY=python
else
    echo "❌ Python nao encontrado. Instale Python 3.11+"
    echo "  macOS: brew install python"
    echo "  Linux: sudo apt install python3 python3-pip"
    exit 1
fi
echo "Python: $($PY --version 2>&1)"
echo ""

# ============================================================
# PASSO 2: Instalar dependencias
# ============================================================
echo "[2/5] Verificando dependencias..."

$PY -c "import aiohttp" 2>/dev/null || {
    echo "Instalando dependencias..."
    $PY -m pip install -r requirements.txt --quiet 2>/dev/null || {
        echo "⚠️ Falha ao instalar dependencias. Tente manualmente:"
        echo "  pip install -r requirements.txt"
    }
}
echo "OK"
echo ""

# ============================================================
# PASSO 3: Detectar hardware
# ============================================================
echo "[3/5] Detectando hardware..."

if [[ "$(uname)" == "Darwin" ]]; then
    RAM_BYTES=$(sysctl -n hw.memsize 2>/dev/null)
else
    RAM_BYTES=$(grep MemTotal /proc/meminfo 2>/dev/null | awk '{print $2 * 1024}')
fi
RAM_GB=$((RAM_BYTES / 1073741824))
echo "RAM: ${RAM_GB}GB"

if [ "$RAM_GB" -ge 16 ]; then
    echo "Modelo preferido: Phi-4 14B"
else
    echo "Modelo preferido: Phi-3.5 Mini"
fi
echo ""

# ============================================================
# PASSO 4: Verificar llamafile
# ============================================================
echo "[4/5] Verificando llamafile..."

# Escolher llamafile do SO
if [[ "$(uname)" == "Darwin" ]]; then
    LLAMAFILE="./llamafile"
elif [[ "$(uname)" == "Linux" ]]; then
    LLAMAFILE="./llamafile-linux"
else
    LLAMAFILE=""
fi

if [ ! -f "$LLAMAFILE" ]; then
    echo "  > llamafile nao encontrado. Baixe de https://huggingface.co/mozilla-ai/llamafile_0.10"
fi
echo ""

# ============================================================
# PASSO 5: Iniciar interface web
# ============================================================
echo ""
echo "============================================"
echo "  [5/5] Iniciando interface web..."
echo "============================================"
echo ""

# Abrir navegador
python3 -c "import webbrowser; webbrowser.open('http://127.0.0.1:8000')" 2>/dev/null || true

# Iniciar UI
if [ -f "$LLAMAFILE" ]; then
    $PY main.py
else
    $PY main.py --no-llm
fi
