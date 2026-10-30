#!/usr/bin/env python3
"""Fetch legacy Minecraft modpack mods and extract each mod JAR's assets.

Only mod/coremod JARs and asset files are copied to the output tree. The
original pack archives and download cache are kept separately under --cache.
Uses only the Python standard library.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import ssl
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
import zipfile
from pathlib import Path, PurePosixPath
from typing import Any

# Disable SSL certificate verification
SSL_CONTEXT = ssl._create_unverified_context()
USER_AGENT = "legacy-modpack-assets-fetcher/1.0 (local mod research)"

PACKS = {
    "tekkit": {
        "display": "Tekkit Classic",
        "minecraft": "1.2.5",
        "version": "3.1.2",
        "kind": "zip",
        "url": "https://servers.technicpack.net/Technic/servers/tekkit/Tekkit_Server_3.1.2.zip",
        "archive": "Tekkit_Server_3.1.2.zip",
    },
    "direwolf20": {
        "display": "Direwolf20",
        "minecraft": "1.4.7",
        "version": "5.3.2",
        "kind": "ftb-api",
        "pack_id": 70,
        "version_id": 185,
        "api_url": "https://api.feed-the-beast.com/v1/modpacks/public/modpack/70/185",
    },
    "agrarian": {
        "display": "Agrarian Skies 2",
        "minecraft": "1.7.10",
        "version": "2.0.6a",
        "kind": "curseforge-manifest",
        "project_id": 225550,
        "file_id": 3045516,
        "archive": "Agrarian-Skies-2-2.0.6a.zip",
        "api_url": "https://api.curseforge.com/v1",
    },
}


def log(message: str) -> None:
    print(message, flush=True)


def sha1_file(path: Path) -> str:
    digest = hashlib.sha1()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def sanitize_url(url: str) -> str:
    parts = urllib.parse.urlsplit(url)
    encoded_path = urllib.parse.quote(parts.path, safe="/:")
    return urllib.parse.urlunsplit((parts.scheme, parts.netloc, encoded_path, parts.query, parts.fragment))


def request_bytes(url: str, *, headers: dict[str, str] | None = None,
                  retries: int = 4) -> bytes:
    request_headers = {"User-Agent": USER_AGENT}
    if headers:
        request_headers.update(headers)
    req = urllib.request.Request(sanitize_url(url), headers=request_headers)
    last_error: Exception | None = None
    for attempt in range(retries):
        try:
            with urllib.request.urlopen(req, context=SSL_CONTEXT, timeout=120) as response:
                return response.read()
        except (urllib.error.URLError, TimeoutError, OSError) as exc:
            last_error = exc
            if attempt + 1 < retries:
                time.sleep(1.5 * (attempt + 1))
    raise RuntimeError(f"Download failed: {url}\n  {last_error}") from last_error


def download_file(
    url: str,
    destination: str | Path,
    force: bool = False,
    headers: dict[str, str] | None = None,
    expected_sha1: str | None = None,
) -> Path:
    destination = Path(destination)
    destination.parent.mkdir(parents=True, exist_ok=True)

    if destination.exists() and not force:
        if expected_sha1 and sha1_file(destination).lower() != expected_sha1.lower():
            print(f"  Cached file failed SHA-1; downloading again: {destination.name}")
        else:
            print(f"  SKIP {destination} (already exists)")
            return destination

    print(f"  GET {url}")

    req_headers = {"User-Agent": "Mozilla/5.0"}
    if headers:
        req_headers.update(headers)

    request = urllib.request.Request(
        sanitize_url(url),
        headers=req_headers,
    )

    try:
        with urllib.request.urlopen(
            request,
            context=SSL_CONTEXT,
            timeout=120,
        ) as response:
            hasher = hashlib.sha1() if expected_sha1 else None
            with open(destination, "wb") as output:
                while chunk := response.read(1024 * 1024):
                    output.write(chunk)
                    if hasher:
                        hasher.update(chunk)

        if expected_sha1 and hasher:
            actual = hasher.hexdigest()
            if actual.lower() != expected_sha1.lower():
                if destination.exists():
                    destination.unlink()
                raise RuntimeError(
                    f"SHA-1 mismatch for {destination.name}: expected {expected_sha1}, got {actual}"
                )

        print(f"  OK {destination}")
        return destination

    except Exception as exc:
        if destination.exists():
            destination.unlink()

        print(f"  ERROR: Download failed: {url}")
        print(f"  {exc}")
        raise


download = download_file


def json_request(url: str, *, headers: dict[str, str] | None = None) -> dict[str, Any]:
    raw = request_bytes(url, headers=headers)
    try:
        value = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"Expected JSON from {url}, received invalid JSON") from exc
    if not isinstance(value, dict):
        raise RuntimeError(f"Expected a JSON object from {url}")
    return value


def safe_relative_path(raw: str) -> Path:
    """Convert an archive/API path to a safe relative path."""
    posix = PurePosixPath(raw.replace("\\", "/"))
    if posix.is_absolute() or any(part in ("..", "") for part in posix.parts):
        raise RuntimeError(f"Unsafe archive path: {raw!r}")
    return Path(*posix.parts)


def is_mod_archive_path(name: str) -> bool:
    path = PurePosixPath(name.replace("\\", "/"))
    parts = {part.lower() for part in path.parts[:-1]}
    # 1.2.5-era Forge packs commonly shipped mods as .zip instead of .jar.
    return path.suffix.lower() in {".jar", ".zip"} and bool(parts.intersection({"mods", "coremods"}))


def normalize_mod_folder(name: str) -> Path:
    path = PurePosixPath(name.replace("\\", "/"))
    # Preserve mods/ versus coremods/, but do not recreate arbitrary archive paths.
    folder = "coremods" if "coremods" in {p.lower() for p in path.parts[:-1]} else "mods"
    return Path(folder) / Path(path.name)


def extract_mod_assets(jar_path: Path, assets_root: Path) -> int:
    """Extract embedded assets without unpacking code or other mod-archive contents.

    Per-JAR folders avoid collisions between mods that happen to ship the same
    asset namespace or file path.
    """
    extracted = 0
    mod_name = re.sub(r"[^A-Za-z0-9._-]+", "_", jar_path.stem)
    try:
        with zipfile.ZipFile(jar_path) as jar:
            for member in jar.infolist():
                if member.is_dir() or not member.filename.startswith("assets/"):
                    continue
                relative = safe_relative_path(member.filename)
                target = assets_root / "mod-assets" / mod_name / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                with jar.open(member) as src, target.open("wb") as dst:
                    shutil.copyfileobj(src, dst)
                extracted += 1
    except zipfile.BadZipFile:
        log(f"  WARNING: not a readable JAR/ZIP, skipped assets: {jar_path.name}")
    return extracted


def register_jar(jar_path: Path, assets_root: Path, records: list[dict[str, Any]]) -> None:
    count = extract_mod_assets(jar_path, assets_root)
    records.append({
        "archive": str(jar_path.relative_to(assets_root)),
        "sha1": sha1_file(jar_path),
        "embedded_asset_files": count,
    })


def harvest_modpack_zip(archive: Path, pack_root: Path, *, force: bool = False) -> list[dict[str, Any]]:
    mods_root = pack_root / "mods"
    mods_root.mkdir(parents=True, exist_ok=True)
    records: list[dict[str, Any]] = []
    seen: set[str] = set()
    with zipfile.ZipFile(archive) as zf:
        for member in zf.infolist():
            if member.is_dir():
                continue
            normalized = member.filename.replace("\\", "/")
            low = normalized.lower()

            if is_mod_archive_path(normalized):
                relative = normalize_mod_folder(normalized)
                target = pack_root / relative
                if target.name.lower() in seen and not force:
                    log(f"  WARNING: duplicate mod filename, keeping first: {target.name}")
                    continue
                seen.add(target.name.lower())
                target.parent.mkdir(parents=True, exist_ok=True)
                if not target.exists() or force:
                    with zf.open(member) as src, target.open("wb") as dst:
                        shutil.copyfileobj(src, dst)
                register_jar(target, pack_root, records)

            # Keep pack-provided assets separate from assets embedded in mod JARs.
            marker = "/overrides/assets/"
            if marker in f"/{low}":
                # Find the path after the overrides directory; preserve the assets/ prefix.
                parts = PurePosixPath(normalized).parts
                try:
                    overrides_index = next(i for i, part in enumerate(parts) if part.lower() == "overrides")
                    relative_parts = parts[overrides_index + 1:]
                    if relative_parts and relative_parts[0].lower() == "assets":
                        relative = safe_relative_path("/".join(relative_parts))
                        dest = pack_root / "pack-assets" / relative
                        dest.parent.mkdir(parents=True, exist_ok=True)
                        if not dest.exists() or force:
                            with zf.open(member) as src, dest.open("wb") as dst:
                                shutil.copyfileobj(src, dst)
                except StopIteration:
                    pass
    return records


def fetch_tekkit(pack_root: Path, cache_root: Path, force: bool) -> list[dict[str, Any]]:
    spec = PACKS["tekkit"]
    archive = cache_root / spec["archive"]
    download(spec["url"], archive, force=force)
    log(f"Extracting {spec['display']} {spec['version']} mod JARs/assets...")
    return harvest_modpack_zip(archive, pack_root, force=force)


def fetch_direwolf20(pack_root: Path, cache_root: Path, force: bool) -> list[dict[str, Any]]:
    spec = PACKS["direwolf20"]
    manifest_path = cache_root / "direwolf20-5.3.2-manifest.json"
    manifest = json_request(spec["api_url"])
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    files = manifest.get("files")
    if not isinstance(files, list):
        raise RuntimeError("FTB API response did not contain a 'files' array")

    records: list[dict[str, Any]] = []
    downloaded = 0
    for entry in files:
        if not isinstance(entry, dict):
            continue
        path = str(entry.get("path", ""))
        name = str(entry.get("name", ""))
        kind = str(entry.get("type", "")).lower()
        combined = f"{path.rstrip('/')}/{name}" if name else path
        is_mod = kind == "mod" or is_mod_archive_path(combined)
        if not is_mod or PurePosixPath(combined).suffix.lower() not in {".jar", ".zip"}:
            continue

        url = entry.get("url")
        if not isinstance(url, str) or not url.startswith("https://"):
            mirrors = entry.get("mirrors") or []
            url = next((m for m in mirrors if isinstance(m, str) and m.startswith("https://")), None)
        if not url:
            log(f"  WARNING: no download URL for {name or combined}; skipped")
            continue

        relative = normalize_mod_folder(combined)
        target = pack_root / relative
        expected_sha1 = entry.get("sha1") if isinstance(entry.get("sha1"), str) else None
        download(url, target, force=force, expected_sha1=expected_sha1)
        register_jar(target, pack_root, records)
        downloaded += 1

    if downloaded == 0:
        raise RuntimeError("No mod JARs were found in the Direwolf20 FTB manifest")
    return records


def curseforge_headers(api_key: str) -> dict[str, str]:
    return {"x-api-key": api_key, "Accept": "application/json"}


def curseforge_download_url(api_base: str, project_id: int, file_id: int,
                            api_key: str) -> str:
    endpoint = f"{api_base}/mods/{project_id}/files/{file_id}/download-url"
    response = json_request(endpoint, headers=curseforge_headers(api_key))
    url = response.get("data")
    if not isinstance(url, str) or not url.startswith("https://"):
        raise RuntimeError(f"CurseForge did not return a downloadable URL for {project_id}/{file_id}")
    return url


def download_curseforge_file(api_base: str, project_id: int, file_id: int,
                             destination: Path, api_key: str, force: bool) -> Path:
    if destination.exists() and not force:
        return destination
    url = curseforge_download_url(api_base, project_id, file_id, api_key)
    # CurseForge's CDN download authentication uses the same key header.
    headers = {"x-api-key": api_key, "User-Agent": USER_AGENT}
    return download(url, destination, headers=headers, force=force)


def fetch_agrarian(pack_root: Path, cache_root: Path, force: bool) -> list[dict[str, Any]]:
    spec = PACKS["agrarian"]
    api_key = os.environ.get("CURSEFORGE_API_KEY", "").strip()
    if not api_key:
        log("  NOTE: CURSEFORGE_API_KEY not configured or ignored. Skipping Agrarian Skies 2.")
        return []

    try:
        archive = cache_root / spec["archive"]
        download_curseforge_file(
            spec["api_url"], spec["project_id"], spec["file_id"], archive, api_key, force
        )

        with zipfile.ZipFile(archive) as zf:
            try:
                manifest = json.loads(zf.read("manifest.json"))
            except KeyError as exc:
                raise RuntimeError("Agrarian Skies archive did not contain manifest.json") from exc

            files = manifest.get("files")
            if not isinstance(files, list):
                raise RuntimeError("Agrarian Skies manifest did not contain a files array")

            # Extract only pack asset overrides, not configs, worlds, scripts, or other files.
            overrides = str(manifest.get("overrides", "overrides"))
            for member in zf.infolist():
                if member.is_dir():
                    continue
                parts = PurePosixPath(member.filename.replace("\\", "/")).parts
                try:
                    index = next(i for i, part in enumerate(parts) if part == overrides)
                except StopIteration:
                    continue
                relative_parts = parts[index + 1:]
                if not relative_parts or relative_parts[0].lower() != "assets":
                    continue
                relative = safe_relative_path("/".join(relative_parts))
                target = pack_root / "pack-assets" / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                if not target.exists() or force:
                    with zf.open(member) as src, target.open("wb") as dst:
                        shutil.copyfileobj(src, dst)

        mods_root = pack_root / "mods"
        mods_root.mkdir(parents=True, exist_ok=True)
        records: list[dict[str, Any]] = []
        total = len(files)
        for index, item in enumerate(files, start=1):
            if not isinstance(item, dict):
                continue
            if item.get("required", True) is False:
                continue
            project_id = item.get("projectID")
            file_id = item.get("fileID")
            if not isinstance(project_id, int) or not isinstance(file_id, int):
                log(f"  WARNING: malformed CurseForge manifest item: {item}")
                continue
            url = curseforge_download_url(spec["api_url"], project_id, file_id, api_key)
            filename = urllib.parse.unquote(PurePosixPath(urllib.parse.urlparse(url).path).name)
            if not filename or "." not in filename:
                filename = f"curseforge-{project_id}-{file_id}.jar"
            target = mods_root / filename
            log(f"  [{index}/{total}] {filename}")
            download(url, target, headers={"x-api-key": api_key}, force=force)
            # CurseForge manifests may include non-JAR files, but modpack mod files are JARs.
            if target.suffix.lower() in {".jar", ".zip"}:
                register_jar(target, pack_root, records)

        return records
    except Exception as exc:
        log(f"  WARNING: CurseForge API key not working ({exc}). Ignoring Agrarian Skies 2.")
        return []


def write_metadata(pack_key: str, pack_root: Path, records: list[dict[str, Any]]) -> None:
    spec = PACKS[pack_key]
    payload = {
        "pack": spec["display"],
        "minecraft_version": spec["minecraft"],
        "pack_version": spec["version"],
        "mod_jars": len(records),
        "embedded_asset_files": sum(r["embedded_asset_files"] for r in records),
        "mods": records,
        "notes": [
            "Only mod/coremod JARs and their embedded assets are collected.",
            "Mod archive assets are preserved per archive under mod-assets/<archive-name>/assets/ to avoid collisions.",
            "Pack-provided assets are stored under pack-assets/assets/.",
            "Minecraft's vanilla asset index/object files are not included.",
        ],
    }
    pack_root.mkdir(parents=True, exist_ok=True)
    (pack_root / "pack-info.json").write_text(json.dumps(payload, indent=2), encoding="utf-8")


def run_pack(pack_key: str, output: Path, cache: Path, force: bool) -> None:
    pack_root = output / pack_key
    pack_root.mkdir(parents=True, exist_ok=True)
    log(f"\n== {PACKS[pack_key]['display']} ({PACKS[pack_key]['minecraft']}, {PACKS[pack_key]['version']}) ==")

    if pack_key == "tekkit":
        records = fetch_tekkit(pack_root, cache, force)
    elif pack_key == "direwolf20":
        records = fetch_direwolf20(pack_root, cache, force)
    elif pack_key == "agrarian":
        records = fetch_agrarian(pack_root, cache, force)
    else:
        raise ValueError(pack_key)

    write_metadata(pack_key, pack_root, records)
    log(
        f"Done: {len(records)} JARs; "
        f"{sum(r['embedded_asset_files'] for r in records)} embedded asset files -> {pack_root}"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", choices=["tekkit", "direwolf20", "agrarian", "list"])
    parser.add_argument("--output", type=Path, default=Path("./assets"))
    parser.add_argument("--cache", type=Path, default=Path("./.cache/modpacks"))
    parser.add_argument("--force", action="store_true", help="redownload cached source files")
    args = parser.parse_args()

    if args.pack == "list":
        for key, spec in PACKS.items():
            print(f"{key:12} {spec['display']} | Minecraft {spec['minecraft']} | pack {spec['version']}")
        return 0

    try:
        run_pack(args.pack, args.output.resolve(), args.cache.resolve(), args.force)
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
