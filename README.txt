USBRAIN v1.0 - Portable AI Agent
====================================

COMO USAR:
  1. Conecte o pendrive em qualquer PC
  2. Windows: execute START.BAT
  3. Linux/Mac: execute start.sh

O que acontece:
  - Detecta automaticamente a RAM do PC
  - Escolhe o melhor modelo de IA disponivel
  - Abre uma interface web no navegador
  - Pronto! Faca perguntas, diagnostique o PC, crie arquivos

REQUISITOS:
  - PC com 8GB+ de RAM (16GB+ recomendado)
  - 8GB de espaco para validacao inicial
  - Windows: Python ja vai embarcado no pendrive; se faltar, ha fallback offline
  - Linux/Mac: Python 3.11+ necessario

FUNCIONALIDADES:
  - Diagnostico de rede (ping, DNS, gateway)
  - Informacoes do sistema (CPU, RAM, disco)
  - Busca de arquivos por nome ou conteudo
  - Comandos de reparo do Windows (SFC, DISM)
  - Criar, editar e executar arquivos
  - Busca na web com Brave Search
  - Base de conhecimento propria (upload de docs)
  - Personalidade customizavel (tom, humor, nome)

COMANDOS NO CHAT:
  /help      - Lista todas as skills
  /skills    - Lista todas as skills
  /stats     - Status do sistema
  /reset     - Limpa conversa

OBTER CHAVE BRAVE SEARCH (opcional):
  1. Acesse https://brave.com/search/api/
  2. Crie conta gratuita (2000 consultas/mes)
  3. Cole a chave em Personalidade > Brave API Key

MODELOS SUPORTADOS:
  Coloque arquivos .gguf em models/
  - Phi-3.5 Mini Q4_K_M   (~2.4GB, validacao em 8GB)
  - Llama 3.2 3B Q4_K_M  (~2.0GB, alternativa leve)
  - Qwen2.5 3B Q4_K_M    (~2.1GB, alternativa tecnica)
  - Phi-4 14B Q4         (~9GB, 16GB+ RAM)

O USBRAIN escolhe automaticamente o melhor .gguf disponivel
para a RAM da maquina. Para o pendrive de 8GB, a prioridade
e usar Phi-3.5 Mini com BGE-M3.

Baixe modelos em: https://huggingface.co/bartowski

---
USBRAIN - Feito com ❤️
