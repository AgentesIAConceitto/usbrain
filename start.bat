@echo off
title USBRAIN - Portable AI Agent
cd /d "%~dp0"
setlocal enabledelayedexpansion
set BOOT_LOG=bootstrap_python.log
del /f /q "!BOOT_LOG!" >nul 2>nul

echo ============================================
echo    USBRAIN - Portable AI Agent
echo    Inicializando...
echo ============================================
echo.

:: Caminho do Python portatil
set LOCAL_ROOT=%LOCALAPPDATA%\USBRAIN
if "%LOCALAPPDATA%"=="" set LOCAL_ROOT=%TEMP%\USBRAIN
set PY=python\python.exe
set PY_OK=0
call :check_python_runtime "python" PY_OK
if "!PY_OK!"=="0" (
    set PY=%LOCAL_ROOT%\python\python.exe
    call :check_python_runtime "%LOCAL_ROOT%\python" PY_OK
)
if "!PY_OK!"=="0" (
    if exist "python_runtime.zip" (
        echo [1/5] Extraindo runtime Python...
        call :extract_python_runtime
        if !errorlevel! neq 0 (
            echo [ERRO] Falha ao extrair o runtime Python.
            if exist "python_extract.log" echo Log: python_extract.log
            pause
            exit /b 1
        )
        echo.
        set PY=%LOCAL_ROOT%\python\python.exe
        call :check_python_runtime "%LOCAL_ROOT%\python" PY_OK
    )
)
if "!PY_OK!"=="0" (
    echo [ERRO] Runtime Python incompleto.
    if exist "python_extract.log" echo Log: python_extract.log
    pause
    exit /b 1
)

:: Verificar llamafile
set LLAMAFILE=
if exist "llamafile.exe" set LLAMAFILE=llamafile.exe

echo [2/5] Python: !PY!
!PY! --version
echo.

echo [3/5] Dependencias: OK (embutidas)
echo.

:: Detectar RAM
echo [4/5] Detectando hardware...
set RAM_MB=8192
for /f "tokens=2 delims==" %%a in ('wmic os get TotalVisibleMemorySize /value 2^>nul') do set /a "ram_kb=%%a" 2>nul
if defined ram_kb set /a RAM_MB=!ram_kb! / 1024
set USBRAIN_RAM_MB=!RAM_MB!
echo RAM: !RAM_MB! MB
set SELECTED_MODEL_PATH=
set SELECTED_MODEL_LABEL=
if defined LLAMAFILE (
    call :choose_model
    if !errorlevel! neq 0 (
        echo [ERRO] Nao foi possivel determinar um modelo compativel para este PC.
        echo Verifique a pasta models\ ou o arquivo installed-models.json.
        pause
        exit /b 1
    )
)
echo.

:: Encontrar porta livre para web UI (8000 ate 8099)
echo [5/5] Verificando portas...
set PORT=8000
:check_port
netstat -an | find "LISTENING" | find ":%PORT% " >nul 2>nul
if !errorlevel! equ 0 (
    set /a PORT=!PORT!+1
    if !PORT! geq 8100 (
        echo AVISO: Nenhuma porta livre de 8000 a 8099, usando 8000
        set PORT=8000
    ) else (
        goto check_port
    )
)
echo   ^> Web UI: http://127.0.0.1:!PORT!
echo.

echo.
echo ============================================
echo  Iniciando interface web na porta !PORT!...
echo ============================================
echo.

>>"!BOOT_LOG!" echo [BOOT] cwd=%cd%
>>"!BOOT_LOG!" echo [BOOT] python=!PY!
>>"!BOOT_LOG!" echo [BOOT] llamafile=!LLAMAFILE!
>>"!BOOT_LOG!" echo [BOOT] port=!PORT!
>>"!BOOT_LOG!" echo [BOOT] date=%date% time=%time%
>>"!BOOT_LOG!" echo [BOOT] import_preflight_start
"!PY!" -X faulthandler -c "import main; print('MAIN_IMPORT_OK')" >>"!BOOT_LOG!" 2>&1
>>"!BOOT_LOG!" echo [BOOT] import_preflight_exit=!errorlevel!
>>"!BOOT_LOG!" echo [BOOT] hardware_preflight_start
"!PY!" -X faulthandler -c "import main; print(main.detect_hardware())" >>"!BOOT_LOG!" 2>&1
>>"!BOOT_LOG!" echo [BOOT] hardware_preflight_exit=!errorlevel!
>>"!BOOT_LOG!" echo [BOOT] runtime_smoke_start
>>"!BOOT_LOG!" echo [BOOT] smoke_ctypes_start
"!PY!" -X faulthandler -c "import ctypes; print('CTYPES_OK')" >>"!BOOT_LOG!" 2>&1
>>"!BOOT_LOG!" echo [BOOT] smoke_ctypes_exit=!errorlevel!
>>"!BOOT_LOG!" echo [BOOT] smoke_numpy_start
"!PY!" -X faulthandler -c "import numpy; print('NUMPY_OK', numpy.__version__)" >>"!BOOT_LOG!" 2>&1
>>"!BOOT_LOG!" echo [BOOT] smoke_numpy_exit=!errorlevel!
>>"!BOOT_LOG!" echo [BOOT] smoke_torch_start
"!PY!" -X faulthandler -c "import torch; print('TORCH_OK', torch.__version__)" >>"!BOOT_LOG!" 2>&1
>>"!BOOT_LOG!" echo [BOOT] smoke_torch_exit=!errorlevel!
>>"!BOOT_LOG!" echo [BOOT] smoke_sentence_transformers_start
"!PY!" -X faulthandler -c "from sentence_transformers import SentenceTransformer; print('SENTENCE_TRANSFORMERS_OK')" >>"!BOOT_LOG!" 2>&1
>>"!BOOT_LOG!" echo [BOOT] smoke_sentence_transformers_exit=!errorlevel!
>>"!BOOT_LOG!" echo [BOOT] runtime_smoke_end

if defined LLAMAFILE (
    echo   ^> LLM selecionada: !SELECTED_MODEL_LABEL!
    >>"!BOOT_LOG!" echo [BOOT] selected_model=!SELECTED_MODEL_LABEL!
    >>"!BOOT_LOG!" echo [BOOT] selected_model_path=!SELECTED_MODEL_PATH!
    >>"!BOOT_LOG!" echo [BOOT] command=!PY! main.py --port !PORT! --model-path "!SELECTED_MODEL_PATH!"
    "!PY!" -X faulthandler main.py --port !PORT! --model-path "!SELECTED_MODEL_PATH!" >>"!BOOT_LOG!" 2>&1
) else (
    echo   ^> LLM pulado: llamafile.exe nao encontrado
    >>"!BOOT_LOG!" echo [BOOT] command=!PY! main.py --no-llm --port !PORT!
    "!PY!" -X faulthandler main.py --no-llm --port !PORT! >>"!BOOT_LOG!" 2>&1
)

if %errorlevel% neq 0 (
    echo.
    echo [ERRO] Falha ao iniciar o agente.
    echo Log: llamafile.log
    echo Boot log: !BOOT_LOG!
    pause
)
goto :eof

:extract_python_runtime
set EXTRACT_ROOT=%LOCAL_ROOT%\python_extract_tmp
set USBRAIN_EXTRACT_ROOT=!EXTRACT_ROOT!
if exist "!EXTRACT_ROOT!" rmdir /s /q "!EXTRACT_ROOT!" >nul 2>nul
del /f /q "python_extract.log" >nul 2>nul

powershell -NoProfile -ExecutionPolicy Bypass -Command "$ErrorActionPreference='Stop'; $zip=Join-Path $PSScriptRoot 'python_runtime.zip'; $dst=$env:USBRAIN_EXTRACT_ROOT; try { Expand-Archive -LiteralPath $zip -DestinationPath $dst -Force } catch { $_ | Out-File -LiteralPath (Join-Path $PSScriptRoot 'python_extract.log') -Encoding utf8; exit 1 }" >nul
if !errorlevel! neq 0 exit /b 1

set EXTRACT_OK=0
call :check_python_runtime "!EXTRACT_ROOT!\python" EXTRACT_OK
if "!EXTRACT_OK!"=="0" (
    >"python_extract.log" echo Runtime Python extraido de forma incompleta.
    >>"python_extract.log" echo Pasta esperada: !EXTRACT_ROOT!\python
    >>"python_extract.log" echo Arquivos criticos ausentes apos a extracao.
    exit /b 1
)

if exist "%LOCAL_ROOT%\python" rmdir /s /q "%LOCAL_ROOT%\python" >nul 2>nul
mkdir "%LOCAL_ROOT%" >nul 2>nul
move /y "!EXTRACT_ROOT!\python" "%LOCAL_ROOT%\python" >nul
if !errorlevel! neq 0 (
    >"python_extract.log" echo Falha ao mover runtime validado para %LOCAL_ROOT%\python
    exit /b 1
)

if exist "!EXTRACT_ROOT!" rmdir /s /q "!EXTRACT_ROOT!" >nul 2>nul
exit /b 0

:check_python_runtime
set "%~2=1"
if not exist "%~1\python.exe" set "%~2=0"
if not exist "%~1\python312.dll" set "%~2=0"
if not exist "%~1\python3.dll" set "%~2=0"
if not exist "%~1\vcruntime140.dll" set "%~2=0"
if not exist "%~1\DLLs\_ctypes.pyd" set "%~2=0"
if not exist "%~1\DLLs\libffi-8.dll" set "%~2=0"
if not exist "%~1\Lib\site.py" set "%~2=0"
if not exist "%~1\Lib\site-packages\aiohttp\__init__.py" set "%~2=0"
if not exist "%~1\Lib\site-packages\httpx\__init__.py" set "%~2=0"
if not exist "%~1\Lib\site-packages\numpy\__init__.py" set "%~2=0"
if not exist "%~1\Lib\site-packages\pypdf\__init__.py" set "%~2=0"
if not exist "%~1\Lib\site-packages\sentence_transformers\__init__.py" set "%~2=0"
if not exist "%~1\Lib\site-packages\transformers\__init__.py" set "%~2=0"
if not exist "%~1\Lib\site-packages\torch\__init__.py" set "%~2=0"
exit /b 0

:choose_model
set MODEL_OPTIONS_FILE=%TEMP%\usbrain-model-options-%RANDOM%%RANDOM%.txt
del /f /q "!MODEL_OPTIONS_FILE!" >nul 2>nul
"!PY!" -X faulthandler main.py --list-model-options > "!MODEL_OPTIONS_FILE!" 2>>"!BOOT_LOG!"
if !errorlevel! neq 0 exit /b 1
if not exist "!MODEL_OPTIONS_FILE!" exit /b 1

set MODEL_COUNT=0
for /f "usebackq tokens=1-5 delims=|" %%a in ("!MODEL_OPTIONS_FILE!") do (
    if not "%%~a"=="" (
        set /a MODEL_COUNT+=1
        set MODEL_OPT_%%a_LABEL=%%b
        set MODEL_OPT_%%a_FILE=%%c
        set MODEL_OPT_%%a_RECOMMENDED=%%d
        set MODEL_OPT_%%a_PATH=%%e
    )
)

if "!MODEL_COUNT!"=="0" (
    del /f /q "!MODEL_OPTIONS_FILE!" >nul 2>nul
    exit /b 1
)

echo Modelos compativeis detectados para este PC:
set DEFAULT_INDEX=1
for /l %%i in (1,1,!MODEL_COUNT!) do (
    set TAG=
    if "!MODEL_OPT_%%i_RECOMMENDED!"=="1" (
        set TAG= [recomendado]
        set DEFAULT_INDEX=%%i
    )
    call echo   %%i^) !MODEL_OPT_%%i_LABEL! - !MODEL_OPT_%%i_FILE!!TAG!
)
echo.

if !MODEL_COUNT! gtr 1 (
    set /p USER_MODEL_INDEX=Escolha a LLM para iniciar ^(Enter = !DEFAULT_INDEX!^): 
) else (
    echo Apenas uma LLM compativel encontrada. Usando a opcao 1.
    set USER_MODEL_INDEX=1
)

if "!USER_MODEL_INDEX!"=="" set USER_MODEL_INDEX=!DEFAULT_INDEX!
for /f "delims=0123456789" %%x in ("!USER_MODEL_INDEX!") do set USER_MODEL_INDEX=
if "!USER_MODEL_INDEX!"=="" set USER_MODEL_INDEX=!DEFAULT_INDEX!
if !USER_MODEL_INDEX! lss 1 set USER_MODEL_INDEX=!DEFAULT_INDEX!
if !USER_MODEL_INDEX! gtr !MODEL_COUNT! set USER_MODEL_INDEX=!DEFAULT_INDEX!

call set SELECTED_MODEL_PATH=%%MODEL_OPT_!USER_MODEL_INDEX!_PATH%%
call set SELECTED_MODEL_LABEL=%%MODEL_OPT_!USER_MODEL_INDEX!_LABEL%%
del /f /q "!MODEL_OPTIONS_FILE!" >nul 2>nul
exit /b 0
