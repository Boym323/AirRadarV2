"""Bezpečná validace kořenového release tarballu pro DEV deploy.

Archivní nástroj GNU běžně přidává záznam ``.`` nebo ``./``. Tyto záznamy
jsou pouze kořenový adresář a nesmějí být zaměněny za traversal.
"""

from __future__ import annotations

import io
import tarfile


def normalized_member_name(member: tarfile.TarInfo) -> str:
    """Vrátí bezpečný název člena bez neškodného kořenového prefixu."""
    name = member.name
    while name.startswith("./"):
        name = name[2:]
    if name == "." and member.isdir():
        return ""
    return name.rstrip("/")


def validate_archive(data: bytes, expected_sha: str) -> None:
    """Ověří strukturu release archivu před extrakcí pod rootem."""
    with tarfile.open(fileobj=io.BytesIO(data), mode="r:gz") as archive:
        members = archive.getmembers()
        if len(members) > 20_000:
            raise ValueError("Too many archive members")
        found: set[str] = set()
        total = 0
        for entry in members:
            name = normalized_member_name(entry)
            if not name and entry.isdir():
                continue
            parts = name.split("/")
            if (
                not name
                or name.startswith("/")
                or "\\" in name
                or any(part in ("", ".", "..") for part in parts)
                or not (name in ("VERSION", "airradarv2-engine", "web") or name.startswith("web/"))
                or not (entry.isfile() or entry.isdir())
                or name in found
            ):
                raise ValueError(f"Unsafe archive member: {entry.name}")
            found.add(name)
            if entry.isfile():
                total += entry.size
        if total > 300 * 1024 * 1024:
            raise ValueError("Archive uncompressed size too large")
        required = {"VERSION", "airradarv2-engine", "web/index.html"}
        if not required <= found:
            raise ValueError("Missing required files")
        entries = {normalized_member_name(entry): entry for entry in members}
        if not all(entries[name].isfile() for name in required):
            raise ValueError("Invalid required archive members")
        version = archive.extractfile(entries["VERSION"])
        if version is None or version.read(256).decode("ascii", "strict").strip() != expected_sha:
            raise ValueError("Archive VERSION != main commit")