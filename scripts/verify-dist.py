#!/usr/bin/env python3
"""Check the local native distribution outputs without installing them."""
import hashlib
import os
import re
from pathlib import Path
import struct
import subprocess
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]
DIST = Path(os.environ.get('KAZZU_DIST_DIR', str(ROOT / 'dist'))).resolve()
VERSION = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']['package']['version']
TOOLS = ROOT / 'target/toolchains'
REPORT = []
# Tool output is parsed below; keep binutils diagnostics independent of the desktop locale.
os.environ['LC_ALL'] = 'C'


def record(text):
    print(text, flush=True)
    REPORT.append(text)


def run(args, env=None, cwd=None):
    result = subprocess.run([str(a) for a in args], env=env, cwd=cwd, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=True)
    record('$ ' + ' '.join(str(a) for a in args) + '\n' + result.stdout.strip())
    return result.stdout


def machine(data):
    assert data[:4] == b'\x7fELF' and data[4:6] == b'\x02\x01', 'Expected little-endian ELF64'
    return struct.unpack_from('<H', data, 18)[0]


def checksum(path):
    digest = hashlib.file_digest(path.open('rb'), 'sha256').hexdigest()
    expected = Path(str(path) + '.sha256').read_text().split()[0]
    assert digest == expected, f'Checksum mismatch: {path}'
    record(f'{path.name}: {path.stat().st_size:,} bytes; SHA-256 OK ({digest})')


def main():
    images = {a: DIST / f'kazzu-{VERSION}-{a}.AppImage' for a in ('x86_64', 'aarch64')}
    cross = TOOLS / 'linux-arm64'
    qemu = cross / 'usr/bin/qemu-aarch64-static'
    if not qemu.exists():
        raise SystemExit('QEMU user-static ARM64 is required to inspect the ARM64 AppImage')
    extracted = {}
    for arch, path in images.items():
        checksum(path)
        with path.open('rb') as handle:
            header = handle.read(64)
        assert machine(header) == (62 if arch == 'x86_64' else 183)
        assert header[8:11] == b'AI\x02', 'Missing Type 2 AppImage marker'
        directory = ROOT / f'target/verification/{arch}'
        directory.mkdir(parents=True, exist_ok=True)
        command = [path, '--appimage-extract'] if arch == 'x86_64' else [qemu, path, '--appimage-extract']
        run(command, cwd=directory)
        extracted[arch] = directory / 'squashfs-root'
        binary = extracted[arch] / 'usr/bin/kazzu'
        staged = ROOT / f'target/appimage/{arch}/Kazzu.AppDir/usr/bin/kazzu'
        assert hashlib.file_digest(binary.open('rb'), 'sha256').digest() == hashlib.file_digest(staged.open('rb'), 'sha256').digest()

        with binary.open('rb') as handle:
            assert machine(handle.read(64)) == (62 if arch == 'x86_64' else 183)
        record(f'{arch}: AppImage runtime and extracted executable architectures match; embedded binary matches the build')
        versions = []
        for obj in [binary, *list((extracted[arch] / 'usr/lib').glob('*.so*'))]:
            output = subprocess.check_output(['readelf', '--version-info', str(obj)], text=True)
            versions.extend(tuple(map(int, version.split('.'))) for version in re.findall(r'Name: GLIBC_([0-9.]+)', output))
        record(f'{arch}: highest referenced GLIBC symbol version: ' + '.'.join(map(str, max(versions))))

    env = dict(os.environ, APPIMAGE_EXTRACT_AND_RUN='1')
    run([images['x86_64'], '--help'], env=env)
    arm_dir = extracted['aarch64']
    if qemu.exists():
        run([qemu, images['aarch64'], '--appimage-offset'])
        run([qemu, '-L', cross / 'usr/aarch64-linux-gnu', '-E', f'LD_LIBRARY_PATH={arm_dir}/usr/lib:{cross}/usr/aarch64-linux-gnu/lib', arm_dir / 'usr/bin/kazzu', '--help'])
    else:
        record('ARM64 execution not checked: QEMU is not available')

    apk = DIST / f'kazzu-{VERSION}-arm64-v8a.apk'
    checksum(apk)
    sdk = Path(os.environ.get('ANDROID_HOME', str(TOOLS / 'android-sdk')))
    bt = sdk / 'build-tools/35.0.1'
    run([bt / 'apksigner', 'verify', '--verbose', '--print-certs', apk])
    run([bt / 'zipalign', '-c', '-P', '16', '4', apk])
    badging = run([bt / 'aapt2', 'dump', 'badging', apk])
    assert f"versionName='{VERSION}'" in badging, 'Unexpected Android versionName'
    with zipfile.ZipFile(apk) as z:
        native = z.read('lib/arm64-v8a/libkazzu_app.so')
        assert machine(native) == 183
        android_dir = ROOT / 'target/verification/android'
        android_dir.mkdir(parents=True, exist_ok=True)
        library = android_dir / 'libkazzu_app.so'
        library.write_bytes(native)
        symbols = subprocess.check_output(['readelf', '--dyn-syms', '--wide', str(library)], text=True)
        for name in ('ANativeActivity_onCreate', 'android_main'):
            assert any(line.rstrip().endswith(' ' + name) and ' UND ' not in line for line in symbols.splitlines()), f'Missing Android entry point: {name}'
        phoff = struct.unpack_from('<Q', native, 32)[0]
        entsize, count = struct.unpack_from('<HH', native, 54)
        loads = [struct.unpack_from('<IIQQQQQQ', native, phoff + i * entsize) for i in range(count)]
        assert all(p[-1] >= 16384 for p in loads if p[0] == 1), 'ELF LOAD alignment below 16 KiB'
    record('APK: AArch64 library, valid signature and 16 KiB load/ZIP alignment confirmed')
    record('All package checks completed successfully')
    record('Hardware validation: x64 Linux host only. QEMU ARM64 startup does not validate real ARM audio/GPU or Android runtime behavior.')


if __name__ == '__main__':
    try:
        main()
    finally:
        DIST.mkdir(parents=True, exist_ok=True)
        (DIST / 'validation.txt').write_text('\n\n'.join(REPORT) + '\n', encoding='utf-8')
