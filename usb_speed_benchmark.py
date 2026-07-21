#!/usr/bin/env python3
from __future__ import annotations

import fcntl
import os
import time
from pathlib import Path


F_NOCACHE = 48
TEST_SIZE_BYTES = 1024 * 1024 * 1024  # 1 GiB
CHUNK_SIZE = 8 * 1024 * 1024  # 8 MiB


def set_nocache(fd: int) -> None:
    try:
        fcntl.fcntl(fd, F_NOCACHE, 1)
    except Exception:
        pass


def benchmark(volume: Path) -> tuple[float, float, Path]:
    test_file = volume / "usb_speed_test.bin"
    chunk = b"\0" * CHUNK_SIZE

    if test_file.exists():
        test_file.unlink()

    fd = os.open(str(test_file), os.O_CREAT | os.O_WRONLY | os.O_TRUNC, 0o644)
    try:
        set_nocache(fd)
        written = 0
        start = time.perf_counter()
        while written < TEST_SIZE_BYTES:
            to_write = min(CHUNK_SIZE, TEST_SIZE_BYTES - written)
            os.write(fd, chunk[:to_write])
            written += to_write
        os.fsync(fd)
        write_elapsed = time.perf_counter() - start
    finally:
        os.close(fd)

    fd = os.open(str(test_file), os.O_RDONLY)
    try:
        set_nocache(fd)
        read_total = 0
        start = time.perf_counter()
        while True:
            data = os.read(fd, CHUNK_SIZE)
            if not data:
                break
            read_total += len(data)
        read_elapsed = time.perf_counter() - start
    finally:
        os.close(fd)

    write_mbps = (TEST_SIZE_BYTES / (1024 * 1024)) / write_elapsed
    read_mbps = (read_total / (1024 * 1024)) / read_elapsed
    return write_mbps, read_mbps, test_file


def main() -> None:
    volume = Path("/Volumes/USBRAIN")
    write_mbps = 0.0
    read_mbps = 0.0
    test_file = volume / "usb_speed_test.bin"

    try:
        write_mbps, read_mbps, test_file = benchmark(volume)
        print(f"VOLUME={volume}")
        print(f"TEST_SIZE_GIB={TEST_SIZE_BYTES / (1024**3):.2f}")
        print(f"WRITE_MBPS={write_mbps:.2f}")
        print(f"READ_MBPS={read_mbps:.2f}")
    finally:
        if test_file.exists():
            test_file.unlink()


if __name__ == "__main__":
    main()
