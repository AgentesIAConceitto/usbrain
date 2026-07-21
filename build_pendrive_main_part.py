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

    # Check cache
    needs_download = False
    if not (CACHE / "python" / "python.exe").exists():
        needs_download = True
    if not (CACHE / "llamafile.exe").exists():
        needs_download = True

    if needs_download:
        print("\nDependencias nao encontradas em cache/")
        yn = input("Preparar agora? (baixa Python + dependencias) [S/n]: ") or "S"
        if yn.lower() == "s":
            build_python_portable(CACHE)
            download_llamafile(CACHE)
        else:
            sys.exit(1)
    else:
        print("Cache OK")

    copy_to_pendrive(CACHE, target, model_path)


if __name__ == "__main__":
    main()
