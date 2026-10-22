import io
import tarfile
import unittest

from deploy.archive_validation import validate_archive


SHA = "a" * 40


def archive(names: list[str]) -> bytes:
    output = io.BytesIO()
    with tarfile.open(fileobj=output, mode="w:gz") as bundle:
        for name in names:
            entry = tarfile.TarInfo(name)
            if name in {".", "./", "./web", "web"} or name.endswith("/"):
                entry.type = tarfile.DIRTYPE
                bundle.addfile(entry)
            else:
                content = SHA.encode() if name.endswith("VERSION") else b"ok"
                entry.size = len(content)
                bundle.addfile(entry, io.BytesIO(content))
    return output.getvalue()


class ArchiveValidationTests(unittest.TestCase):
    def test_gnu_tar_root_dot_is_accepted(self) -> None:
        validate_archive(archive([".", "./VERSION", "./airradarv2-engine", "./web", "./web/index.html"]), SHA)

    def test_traversal_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "Unsafe archive member"):
            validate_archive(archive([".", "VERSION", "airradarv2-engine", "web", "web/index.html", "web/../escape"]), SHA)

    def test_symlink_is_rejected(self) -> None:
        output = io.BytesIO()
        with tarfile.open(fileobj=output, mode="w:gz") as bundle:
            for name in ("VERSION", "airradarv2-engine", "web", "web/index.html"):
                entry = tarfile.TarInfo(name)
                if name == "web":
                    entry.type = tarfile.DIRTYPE
                    bundle.addfile(entry)
                elif name == "airradarv2-engine":
                    entry.type = tarfile.SYMTYPE
                    entry.linkname = "/bin/sh"
                    bundle.addfile(entry)
                else:
                    content = SHA.encode() if name == "VERSION" else b"ok"
                    entry.size = len(content)
                    bundle.addfile(entry, io.BytesIO(content))
        with self.assertRaisesRegex(ValueError, "Unsafe archive member"):
            validate_archive(output.getvalue(), SHA)


if __name__ == "__main__":
    unittest.main()