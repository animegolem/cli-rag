#!/usr/bin/env python3
import subprocess
import sys
from pathlib import Path

try:
    import yaml  # type: ignore
except ModuleNotFoundError:
    sys.stderr.write(
        "[yaml-fmt] PyYAML not installed; skipping YAML check (install via `pip install pyyaml` to enable).\n"
    )
    sys.exit(0)


def git_tracked_yaml() -> list[Path]:
    try:
        output = subprocess.check_output(
            [
                "git",
                "ls-files",
                "*.yml",
                "*.yaml",
            ],
            text=True,
        )
    except subprocess.CalledProcessError as exc:  # pragma: no cover - git failure should be rare
        sys.stderr.write(f"[yaml-fmt] git ls-files failed: {exc}\n")
        sys.exit(exc.returncode)
    paths: list[Path] = []
    for line in output.splitlines():
        if not line.strip():
            continue
        path = Path(line.strip())
        if not path.exists():
            continue
        paths.append(path)
    return paths


def main() -> int:
    problems: list[str] = []
    for path in git_tracked_yaml():
        try:
            with path.open("r", encoding="utf-8") as fh:
                yaml.safe_load(fh)
        except yaml.YAMLError as err:
            problems.append(f"{path}: {err}")
    if problems:
        sys.stderr.write("[yaml-fmt] YAML parsing failed for the following files:\n")
        for item in problems:
            sys.stderr.write(f"  - {item}\n")
        sys.stderr.write("[yaml-fmt] Please fix the YAML (indentation / syntax) before committing.\n")
        return 1
    print("[yaml-fmt] YAML parse check passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
