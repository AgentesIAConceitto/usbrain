#!/usr/bin/env python3
"""
USBRAIN Pendrive Builder v2 — OFFLINE-FIRST

Cria pendrive completa e autocontida para Windows.
Nada de download na hora de usar. Tudo ja vai incluso.

Uso:
  python3 build_pendrive.py                # modo interativo
  python3 build_pendrive.py /Volumes/USB   # destino direto
  python3 build_pendrive.py --download     # so prepara dependencias
  python3 build_pendrive.py --format       # forca formatacao antes

Flags:
  --format         Formata o pendrive como exFAT antes de copiar
  --with-model M   Caminho pro modelo GGUF
  --download       Baixa runtime Python Windows + dependencias (sem copiar)
  --validate-only  Valida integridade do pendrive sem copiar
"""

from __future__ import annotations

import os
import platform
import re
import shutil
import subprocess
import sys
import urllib.request
import zipfile
from contextlib import suppress
from pathlib import Path
from typing import Optional

from huggingface_hub import hf_hub_download

from main import choose_model_file, get_model_label

# Config
# O pacote nuget do Python para Windows e mais completo/estavel que o embed
# para rodar ctypes, numpy e a cadeia do BGE-M3 em modo portatil.
PYTHON_NUGET_URL = "https://www.nuget.org/api/v2/package/python/3.12.9"
LLAMAFILE_URL = "https://github.com/mozilla-ai/llamafile/releases/download/0.10.3/llamafile-0.10.3"

# Runtime necessario para a interface web, RAG local e upload de PDF.
# O build final do pendrive inclui a cadeia completa do BGE-M3 para RAG semantico.
PACKAGES = [
    "aiohttp>=3.9",
    "httpx>=0.28",
    "numpy==1.26.4",
    "pypdf>=5.0",
    "Jinja2==3.1.6",
    "markupsafe==3.0.2",
    "Werkzeug==3.1.3",
    "sentence-transformers==3.0.1",
    "transformers==4.44.2",
    "huggingface-hub==0.24.6",
    "tokenizers==0.19.1",
    "safetensors==0.4.4",
    "torch==2.4.1",
    "scipy==1.14.1",
    "scikit-learn==1.5.2",
]

BASE_SRC = Path(__file__).parent.resolve()
CACHE = BASE_SRC / "cache"
BGE_M3_CACHE = CACHE / "bge-m3-model"
BGE_M3_REPO_ID = "BAAI/bge-m3"
BGE_M3_REQUIRED_FILES = [
    "1_Pooling/config.json",
    "README.md",
    "colbert_linear.pt",
    "config.json",
    "config_sentence_transformers.json",
    "modules.json",
    "pytorch_model.bin",
    "sentence_bert_config.json",
    "sentencepiece.bpe.model",
    "sparse_linear.pt",
    "special_tokens_map.json",
    "tokenizer.json",
    "tokenizer_config.json",
]

PYTHON_RUNTIME_REQUIRED_FILES = [
    "python.exe",
    "python3.dll",
    "python312.dll",
    "vcruntime140.dll",
    "DLLs/_ctypes.pyd",
    "DLLs/libffi-8.dll",
    "Lib/site.py",
    "Lib/site-packages/aiohttp/__init__.py",
    "Lib/site-packages/httpx/__init__.py",
    "Lib/site-packages/numpy/__init__.py",
    "Lib/site-packages/pypdf/__init__.py",
    "Lib/site-packages/sentence_transformers/__init__.py",
    "Lib/site-packages/transformers/__init__.py",
    "Lib/site-packages/torch/__init__.py",
    "Lib/site-packages/sklearn/__init__.py",
    "Lib/site-packages/sklearn/.libs/vcomp140.dll",
    "Lib/site-packages/sklearn/.libs/msvcp140.dll",
]

PYTHON_RUNTIME_CRITICAL_DIRS = [
    "Lib/site-packages/transformers",
    "Lib/site-packages/sentence_transformers",
    "Lib/site-packages/torch",
    "Lib/site-packages/sklearn",
    "Lib/site-packages/sklearn/.libs",
    "Lib/site-packages/numpy",
]

ROOT_REQUIRED_FILES = [
    "main.py",
    "start.bat",
    "agent.py",
    "requirements.txt",
    "README.txt",
    "python_runtime.zip",
]


def download(url: str, dest: Path, desc: str = "") -> None:
    if dest.exists() and dest.stat().st_size > 1000:
        print(f"  OK {desc or dest.name} ja existe, pulando")
        return
    print(f"  Baixando {desc or url.split('/')[-1]}...")
    urllib.request.urlretrieve(url, dest)
    size = dest.stat().st_size / (1024**2)
    print(f"     {size:.1f}MB baixado")


def extract_zip_subdir(zip_path: Path, subdir: str, dest_dir: Path) -> None:
    """Extrai apenas um subdiretorio de um zip para o destino."""
    prefix = subdir.rstrip("/") + "/"
    with zipfile.ZipFile(zip_path, "r") as zf:
        for member in zf.infolist():
            if member.is_dir() or not member.filename.startswith(prefix):
                continue
            relative = member.filename[len(prefix):]
            if not relative:
                continue
            target_path = dest_dir / relative
            target_path.parent.mkdir(parents=True, exist_ok=True)
            with zf.open(member, "r") as src, open(target_path, "wb") as dst:
                shutil.copyfileobj(src, dst)


def is_noise_part(part: str) -> bool:
    return part in {".DS_Store", ".Spotlight-V100", ".Trashes", ".fseventsd", "__MACOSX"} or part.startswith("._")


def is_noise_relative_path(rel: Path) -> bool:
    return any(is_noise_part(part) for part in rel.parts)


def count_files_clean(path: Path) -> int:
    if not path.exists():
        return 0
    return sum(
        1
        for item in path.rglob("*")
        if item.is_file() and not is_noise_relative_path(item.relative_to(path))
    )


def list_missing_files(base_dir: Path, required_files: list[str]) -> list[str]:
    missing = []
    for rel in required_files:
        if not (base_dir / rel).exists():
            missing.append(rel)
    return missing


def compare_directory_counts(source_dir: Path, target_dir: Path, rel_paths: list[str]) -> list[str]:
    issues: list[str] = []
    for rel in rel_paths:
        src = source_dir / rel
        dst = target_dir / rel
        if not src.exists():
            issues.append(f"Origem ausente para comparacao: {src}")
            continue
        if not dst.exists():
            issues.append(f"Destino ausente: {dst}")
            continue
        src_count = count_files_clean(src)
        dst_count = count_files_clean(dst)
        if src_count != dst_count:
            issues.append(
                f"Contagem divergente em {rel}: cache={src_count} destino={dst_count}"
            )
    return issues


def validate_pendrive_integrity(cache_dir: Path, target: Path, model_path: Optional[Path] = None) -> list[str]:
    """Valida integridade minima do pendrive antes de considerar o build pronto."""
    issues: list[str] = []

    root_missing = list_missing_files(target, ROOT_REQUIRED_FILES)
    issues.extend(f"Arquivo raiz ausente: {rel}" for rel in root_missing)

    cache_python = cache_dir / "python"
    target_python = target / "python"
    python_missing = list_missing_files(target_python, PYTHON_RUNTIME_REQUIRED_FILES)
    issues.extend(f"Runtime Python incompleto: {rel}" for rel in python_missing)

    cache_bge = cache_dir / "bge-m3-model"
    target_bge = target / "rag" / "bge-m3-model"
    bge_missing = list_missing_files(target_bge, BGE_M3_REQUIRED_FILES)
    issues.extend(f"BGE-M3 incompleto: {rel}" for rel in bge_missing)

    issues.extend(compare_directory_counts(cache_python, target_python, PYTHON_RUNTIME_CRITICAL_DIRS))
    cache_bge_count = count_files_clean(cache_bge)
    target_bge_count = count_files_clean(target_bge)
    if cache_bge_count != target_bge_count:
        issues.append(
            f"Contagem divergente em rag/bge-m3-model: cache={cache_bge_count} destino={target_bge_count}"
        )

    cache_zip = cache_dir / "python_runtime.zip"
    target_zip = target / "python_runtime.zip"
    if cache_zip.exists() and target_zip.exists() and cache_zip.stat().st_size != target_zip.stat().st_size:
        issues.append(
            f"python_runtime.zip divergente: cache={cache_zip.stat().st_size} destino={target_zip.stat().st_size}"
        )

    if model_path and model_path.exists():
        target_model = target / "models" / model_path.name
        if not target_model.exists():
            issues.append(f"Modelo ausente no pendrive: models/{model_path.name}")
        elif target_model.stat().st_size != model_path.stat().st_size:
            issues.append(
                f"Modelo divergente: {model_path.name} origem={model_path.stat().st_size} destino={target_model.stat().st_size}"
            )

    return issues


def print_validation_report(issues: list[str]) -> None:
    if not issues:
        print("\nValidacao pre-ejecao: OK")
        return
    print("\nValidacao pre-ejecao: FALHOU")
    for issue in issues:
        print(f"  - {issue}")


def build_python_portable(target_dir: Path) -> None:
    """Cria diretorio python/ com runtime Windows completo + dependencias."""
    print("\n[1/4] Baixando runtime Python Windows...")
    nuget_pkg = CACHE / "python-3.12.9-nuget.nupkg"
    download(PYTHON_NUGET_URL, nuget_pkg, "Python 3.12.9 nuget")

    python_dir = target_dir / "python"
    if python_dir.exists():
        shutil.rmtree(python_dir)
    python_dir.mkdir(parents=True, exist_ok=True)

    extract_zip_subdir(nuget_pkg, "tools", python_dir)

    site_pkgs = python_dir / "Lib" / "site-packages"
    site_pkgs.mkdir(parents=True, exist_ok=True)

    print("\n[2/4] Baixando dependencias Python...")
    wheels_dir = CACHE / "wheels"
    if wheels_dir.exists():
        shutil.rmtree(wheels_dir)
    wheels_dir.mkdir(parents=True, exist_ok=True)

    # Baixa wheels via pip (batch)
    r = subprocess.run(
        [sys.executable, "-m", "pip", "download",
         "--only-binary=:all:", "--platform", "win_amd64",
         "--python-version", "3.12",
         "-d", str(wheels_dir)] + PACKAGES,
        capture_output=True, timeout=180
    )
    if r.returncode != 0:
        stderr = r.stderr.decode("utf-8", errors="replace")
        print(f"  Aviso: pip download parcial: {stderr[-300:]}")

    for whl in sorted(wheels_dir.glob("*.whl")):
        print(f"       {whl.name}")

    # Instala wheels no site-packages
    print("\n     Instalando no runtime Windows...")
    for whl in sorted(wheels_dir.glob("*.whl")):
        with zipfile.ZipFile(whl, 'r') as zf:
            zf.extractall(str(site_pkgs))
        print(f"       OK {whl.name}")

    package_python_runtime(target_dir)
    print(f"\n  Python portatil pronto ({len(list(python_dir.rglob('*')))} arquivos)")


def package_python_runtime(target_dir: Path) -> Path:
    """Cria um zip do runtime Python para uso como fallback no Windows."""
    python_dir = target_dir / "python"
    runtime_zip = target_dir / "python_runtime.zip"
    if runtime_zip.exists():
        runtime_zip.unlink()

    print("\n     Empacotando python_runtime.zip...")
    with zipfile.ZipFile(runtime_zip, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=6) as zf:
        for item in python_dir.rglob("*"):
            if item.is_file():
                arcname = Path("python") / item.relative_to(python_dir)
                zf.write(item, arcname=str(arcname))

    size_gb = runtime_zip.stat().st_size / (1024**3)
    print(f"     OK python_runtime.zip ({size_gb:.2f}GiB)")
    return runtime_zip


def download_llamafile(target_dir: Path) -> None:
    """Baixa llamafile.exe."""
    print("\n[3/4] Baixando llamafile.exe...")
    dest = target_dir / "llamafile.exe"
    download(LLAMAFILE_URL, dest, "llamafile.exe")


def download_bge_m3_model(target_dir: Path) -> None:
    """Baixa os artefatos necessarios do BGE-M3 para uso offline."""
    print("\n[4/4] Baixando BGE-M3...")
    target_dir.mkdir(parents=True, exist_ok=True)

    for relative_path in BGE_M3_REQUIRED_FILES:
        dest = target_dir / relative_path
        if dest.exists() and dest.stat().st_size > 1000:
            print(f"  OK {relative_path} ja existe, pulando")
            continue
        dest.parent.mkdir(parents=True, exist_ok=True)
        print(f"  Baixando {relative_path}...")
        hf_hub_download(
            repo_id=BGE_M3_REPO_ID,
            filename=relative_path,
            local_dir=str(target_dir),
            local_dir_use_symlinks=False,
        )

    # O modulo Normalize do sentence-transformers nao precisa de arquivos,
    # mas a estrutura de diretorios ajuda o loader local a ficar previsivel.
    (target_dir / "2_Normalize").mkdir(parents=True, exist_ok=True)
    prune_hidden_entries(target_dir)

    total_size = sum(p.stat().st_size for p in target_dir.rglob("*") if p.is_file()) / (1024**3)
    print(f"  BGE-M3 offline pronto ({total_size:.2f}GiB)")


def find_model() -> Optional[Path]:
    """Busca o melhor GGUF local para a validacao em pendrive de 8GB."""
    models_dir = BASE_SRC / "models"
    if not models_dir.exists():
        return None
    # O builder prioriza o perfil de validacao em 8GB.
    return choose_model_file(models_dir, ram_gb=8.0, preferred_model="phi-3.5-mini")


def clean_apple_metadata(path: Path) -> None:
    """Remove ._ arquivos (Apple Double) que quebram no Windows."""
    count = 0
    for f in path.rglob("._*"):
        try:
            f.unlink()
            count += 1
        except Exception:
            pass
    if count:
        print(f"  {count} arquivos ._ removidos")


def prune_hidden_entries(path: Path) -> None:
    """Remove apenas lixo oculto do macOS/HuggingFace sem apagar runtime valido."""
    removed = 0
    for item in sorted(path.rglob("*"), key=lambda p: len(p.parts), reverse=True):
        rel = item.relative_to(path)
        should_remove = is_noise_relative_path(rel)
        if should_remove:
            try:
                if item.is_dir():
                    shutil.rmtree(item)
                else:
                    item.unlink()
                removed += 1
            except Exception:
                pass
    if removed:
        print(f"  {removed} entradas ocultas removidas")


def copy_tree_files(source: Path, target: Path, *, skip_hidden: bool = True) -> int:
    """Copia uma arvore arquivo a arquivo, evitando apenas lixo oculto do macOS."""
    if platform.system() == "Darwin":
        target.mkdir(parents=True, exist_ok=True)
        cmd = ["rsync", "-a", "--delete"]
        if skip_hidden:
            cmd.extend([
                "--exclude=.DS_Store",
                "--exclude=.Spotlight-V100",
                "--exclude=.Trashes",
                "--exclude=.fseventsd",
                "--exclude=__MACOSX",
                "--exclude=._*",
            ])
        cmd.extend([f"{source}/", str(target)])
        subprocess.run(cmd, check=True)
        return sum(
            1
            for item in target.rglob("*")
            if item.is_file()
            and (
                not skip_hidden
                or not is_noise_relative_path(item.relative_to(target))
            )
        )

    files = 0
    for item in source.rglob("*"):
        if not item.is_file():
            continue
        rel = item.relative_to(source)
        if skip_hidden and is_noise_relative_path(rel):
            continue
        dest = target / rel
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy(item, dest, follow_symlinks=False)
        files += 1
    return files


def copy_to_pendrive(app_source_dir: Path, cache_dir: Path, target: Path, model_path: Optional[Path] = None) -> None:
    """Copia tudo pro pendrive sem metadata macOS."""
    print("\nCopiando para o pendrive...")

    dirs = [
        "tools", "skills", "rag", "webui",
        "webui/templates", "webui/static", "knowledge", "models"
    ]
    for d in dirs:
        (target / d).mkdir(parents=True, exist_ok=True)

    # Arquivos raiz
    root_files = [
        "main.py", "agent.py", "personality.md",
        "requirements.txt", "README.txt", "start.bat"
    ]
    for name in root_files:
        f = app_source_dir / name
        if f.exists():
            shutil.copy(f, target / name, follow_symlinks=False)

    # Python runtime completo + zip de fallback
    py_dir = cache_dir / "python"
    py_zip = cache_dir / "python_runtime.zip"
    if not py_zip.exists() and py_dir.exists():
        package_python_runtime(cache_dir)
    shutil.rmtree(target / "python", ignore_errors=True)
    if py_dir.exists():
        prune_hidden_entries(py_dir)
        files = copy_tree_files(py_dir, target / "python", skip_hidden=False)
        size_gb = sum(f.stat().st_size for f in py_dir.rglob("*") if f.is_file()) / (1024**3)
        print(f"  python/ ({files} arquivos, {size_gb:.2f}GiB)")
    if py_zip.exists():
        shutil.copy(py_zip, target / "python_runtime.zip", follow_symlinks=False)
        print(f"  python_runtime.zip ({py_zip.stat().st_size/1024**3:.2f}GiB)")

    # llamafile.exe
    llf = cache_dir / "llamafile.exe"
    if llf.exists():
        shutil.copy(llf, target / "llamafile.exe", follow_symlinks=False)
        print(f"  llamafile.exe ({llf.stat().st_size/1024**2:.0f}MB)")

    # skills/, tools/, rag/, webui/, knowledge/
    for subdir in ["skills", "tools", "rag", "webui", "knowledge"]:
        s = app_source_dir / subdir
        if s.exists():
            files = 0
            for item in s.rglob("*"):
                if item.is_file():
                    rel = item.relative_to(s)
                    if any(part.startswith(".") for part in rel.parts):
                        continue
                    if subdir == "rag" and rel.parts and rel.parts[0] == "bge-small-model":
                        continue
                    dest = target / subdir / rel
                    dest.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy(item, dest, follow_symlinks=False)
                    files += 1
            print(f"  {subdir}/ ({files} arquivos)")

    bge_src = cache_dir / "bge-m3-model"
    if bge_src.exists():
        prune_hidden_entries(bge_src)
        shutil.rmtree(target / "rag" / "bge-m3-model", ignore_errors=True)
        (target / "rag" / "bge-m3-model" / "2_Normalize").mkdir(parents=True, exist_ok=True)
        files = copy_tree_files(bge_src, target / "rag" / "bge-m3-model")
        size_gb = sum(f.stat().st_size for f in bge_src.rglob("*") if f.is_file()) / (1024**3)
        print(f"  rag/bge-m3-model/ ({files} arquivos, {size_gb:.2f}GiB)")

    # Modelo
    if model_path and model_path.exists():
        model_dest = target / "models" / model_path.name
        size_gb = model_path.stat().st_size / (1024**3)
        print(f"\n  Copiando modelo ({size_gb:.1f}GB)...")
        shutil.copy(model_path, model_dest, follow_symlinks=False)
        print("  Modelo copiado")

    # Limpeza
    print("\n  Limpando metadados...")
    clean_apple_metadata(target)

    issues = validate_pendrive_integrity(cache_dir, target, model_path)
    print_validation_report(issues)
    if issues:
        print("\nPendrive NAO liberado para ejeção.")
        raise SystemExit(1)

    total_gb = sum(f.stat().st_size for f in target.rglob("*") if f.is_file()) / (1024**3)
    print(f"\n{'='*40}")
    print(f"USBRAIN pronto em {target}")
    print(f"Tamanho: {total_gb:.1f}GB")
    print("\nInstrucoes:")
    print("  1. Ejete com seguranca")
    print("  2. Conecte no Windows")
    print("  3. Execute START.BAT")
    print("\nOFFLINE-FIRST: sem downloads na primeira execucao!")
    print()


def format_pendrive(target: Path) -> None:
    """Formata como exFAT com nome USBRAIN."""
    print(f"\nFormatando {target} como exFAT (nome: USBRAIN)...")
    system = platform.system()
    if system == "Darwin":
        target_str = str(target)
        if target_str.startswith("/dev/"):
            device_id = target_str.replace("/dev/", "")
        elif target_str.startswith("disk"):
            device_id = target_str
        else:
            r = subprocess.run(
                ["diskutil", "info", target_str],
                capture_output=True, text=True, timeout=10
            )
            m = re.search(r"Device Identifier:\s*(\S+)", r.stdout)
            if not m:
                print("Nao foi possivel identificar o volume")
                return
            device_id = m.group(1)

        whole_disk = re.sub(r"s\d+$", "", device_id)
        subprocess.run(
            ["diskutil", "eraseDisk", "ExFAT", "USBRAIN", "MBRFormat", whole_disk],
            timeout=120,
            check=True,
        )
        print("Formatado!")
    elif system == "Windows":
        subprocess.run(f"format {target} /FS:exFAT /Q /Y /V:USBRAIN", shell=True, timeout=120)
    else:
        subprocess.run(f"mkfs.exfat -n USBRAIN {target}", shell=True, timeout=30)


def main():
    CACHE.mkdir(parents=True, exist_ok=True)

    # Download only
    if "--download" in sys.argv:
        build_python_portable(CACHE)
        download_llamafile(CACHE)
        download_bge_m3_model(BGE_M3_CACHE)
        print("\nPreparacao concluida! Tudo em cache/")
        print("Agora execute: python3 build_pendrive.py")
        return

    if "--validate-only" in sys.argv:
        if len(sys.argv) < 2 or sys.argv[1].startswith("--"):
            print("Uso: python3 build_pendrive.py /Volumes/NOME --validate-only")
            sys.exit(1)
        target = Path(sys.argv[1])
        model_path = find_model()
        issues = validate_pendrive_integrity(CACHE, target, model_path)
        print_validation_report(issues)
        sys.exit(1 if issues else 0)

    # Format
    if "--format" in sys.argv:
        if len(sys.argv) < 2:
            print("Uso: python3 build_pendrive.py /Volumes/NOME --format")
            sys.exit(1)
        format_pendrive(Path(sys.argv[1]))
        return

    # Get target
    if len(sys.argv) > 1 and not sys.argv[1].startswith("--"):
        target = Path(sys.argv[1])
    else:
        volumes = Path("/Volumes")
        if volumes.exists():
            mounts = sorted(volumes.iterdir())
            print("USBRAIN Pendrive Builder v2")
            print("=" * 40)
            print()
            print("Volumes:")
            for i, d in enumerate(mounts):
                if d.is_dir() and d.name != "Macintosh HD":
                    try:
                        free = shutil.disk_usage(d).free / (1024**3)
                        print(f"  [{i+1}] {d.name:30s} {free:.0f}GB livres")
                    except Exception:
                        print(f"  [{i+1}] {d.name}")
            choice = input("\nQual volume? [1]: ") or "1"
            target = mounts[int(choice) - 1]
        else:
            target = Path(input("Caminho do pendrive: "))

    # Model
    model_path = None
    if "--with-model" in sys.argv:
        idx = sys.argv.index("--with-model")
        mp = sys.argv[idx + 1] if len(sys.argv) > idx + 1 else None
        if mp:
            model_path = Path(mp)
            if not model_path.exists():
                print(f"Modelo nao encontrado: {model_path}")
                sys.exit(1)
    else:
        gguf = find_model()
        if gguf:
            model_path = gguf
            print(f"Modelo selecionado automaticamente: {gguf.name} ({get_model_label(gguf.stem)})")
        else:
            print("Nenhum modelo GGUF encontrado em models/. O pendrive sera gerado sem LLM local.")

    # Check cache
    needs_download = False
    if not (CACHE / "python" / "python.exe").exists():
        needs_download = True
    if not (CACHE / "python_runtime.zip").exists():
        needs_download = True
    if not (CACHE / "llamafile.exe").exists():
        needs_download = True
    if not (BGE_M3_CACHE / "pytorch_model.bin").exists():
        needs_download = True

    if needs_download:
        print("\nDependencias nao encontradas em cache/")
        yn = input("Preparar agora? (baixa runtime Windows + dependencias) [S/n]: ") or "S"
        if yn.lower() == "s":
            build_python_portable(CACHE)
            download_llamafile(CACHE)
            download_bge_m3_model(BGE_M3_CACHE)
        else:
            sys.exit(1)
    else:
        print("Cache OK")

    copy_to_pendrive(BASE_SRC, CACHE, target, model_path)


if __name__ == "__main__":
    main()
