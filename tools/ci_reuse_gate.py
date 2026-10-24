#!/usr/bin/env python3
"""Bezpečné opětovné použití PR CI při totožném výsledném Git tree.

Pokud neexistuje ověřitelný úspěšný PR běh, vždy se spustí běžné CI.
Nejde o důvěru v commit message, název větve ani pouhý zelený badge.
"""
from __future__ import annotations

import argparse
import io
import json
from pathlib import Path
import re
import subprocess
import sys
import zipfile

SHA = re.compile(r"^[0-9a-f]{40}$")
WORKFLOWS = {"ci.yml", "g04-postgres-migrations.yml"}
PROOF_FILE = "validated-pr-tree.json"


def gh_json(endpoint: str) -> object:
    return json.loads(subprocess.check_output([
        "gh", "api", "-H", "Accept: application/vnd.github+json", endpoint,
    ], timeout=30))


def read_proof_archive(repo: str, artifact_id: int) -> dict:
    # gh řeší přesměrování a autorizaci GitHub Actions artefaktů.
    archive = subprocess.check_output([
        "gh", "api", "-H", "Accept: application/vnd.github+json",
        f"repos/{repo}/actions/artifacts/{artifact_id}/zip",
    ], timeout=60)
    if len(archive) > 1_000_000:
        raise ValueError("Neúměrně velký CI důkaz")
    with zipfile.ZipFile(io.BytesIO(archive)) as bundle:
        if bundle.namelist() != [PROOF_FILE]:
            raise ValueError("Artefakt obsahuje jiné než očekávané soubory")
        return json.loads(bundle.read(PROOF_FILE))


def make_proof(workflow: str, pr: int, head: str, run: int, attempt: int) -> dict:
    if workflow not in WORKFLOWS or not SHA.fullmatch(head) or pr < 1 or run < 1 or attempt < 1:
        raise ValueError("Neplatné argumenty CI důkazu")
    tree = subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"], text=True).strip()
    if not SHA.fullmatch(tree):
        raise ValueError("Neplatný Git tree checkoutu")
    return {"workflow": workflow, "pr": pr, "head_sha": head,
            "tree_sha": tree, "run_id": run, "run_attempt": attempt}


def verified_pr_for_main(repo: str, main_sha: str) -> list[dict]:
    prs = gh_json(f"repos/{repo}/commits/{main_sha}/pulls?per_page=100")
    if not isinstance(prs, list):
        raise ValueError("Chybná odpověď /commits/.../pulls")
    return [pr for pr in prs if pr.get("merged_at") and
            pr.get("merge_commit_sha") == main_sha and
            pr.get("base", {}).get("ref") == "main" and
            (pr.get("head", {}).get("repo") or {}).get("full_name") == repo]


def can_reuse(repo: str, main_sha: str, workflow: str) -> tuple[bool, str]:
    if not re.fullmatch(r"[a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+", repo):
        raise ValueError("Neplatný název repozitáře")
    if not SHA.fullmatch(main_sha) or workflow not in WORKFLOWS:
        raise ValueError("Neplatná revize nebo workflow")
    main = gh_json(f"repos/{repo}/commits/{main_sha}")
    main_tree = main["commit"]["tree"]["sha"]
    if not SHA.fullmatch(main_tree):
        raise ValueError("Neplatný Git tree hlavní větve")

    for pr in verified_pr_for_main(repo, main_sha):
        pr_no = pr.get("number")
        pr_head = pr.get("head", {}).get("sha")
        if not isinstance(pr_no, int) or not isinstance(pr_head, str) or not SHA.fullmatch(pr_head):
            continue
        # Ověřuje se výhradně běh konkrétního CI workflow pro přesný HEAD PR.
        runs = gh_json(f"repos/{repo}/actions/workflows/{workflow}/runs"
                       f"?event=pull_request&head_sha={pr_head}&per_page=100")
        for run in runs.get("workflow_runs", []):
            if not (run.get("event") == "pull_request" and
                    run.get("head_sha") == pr_head and
                    run.get("status") == "completed" and
                    run.get("conclusion") == "success" and
                    run.get("head_repository", {}).get("full_name") == repo):
                continue
            run_id, attempt = run.get("id"), run.get("run_attempt")
            if not isinstance(run_id, int) or not isinstance(attempt, int):
                continue
            assets = gh_json(f"repos/{repo}/actions/runs/{run_id}/artifacts?per_page=100")
            expected = f"validated-{workflow.removesuffix('.yml')}-{run_id}-{attempt}"
            for artifact in assets.get("artifacts", []):
                if artifact.get("name") != expected or artifact.get("expired"):
                    continue
                try:
                    proof = read_proof_archive(repo, artifact["id"])
                except (ValueError, KeyError, zipfile.BadZipFile, subprocess.CalledProcessError, subprocess.TimeoutExpired):
                    continue
                if proof == {"workflow": workflow, "pr": pr_no,
                             "head_sha": pr_head, "tree_sha": main_tree,
                             "run_id": run_id, "run_attempt": attempt}:
                    return True, f"PR #{pr_no}, CI běh {run_id}, Git tree {main_tree[:12]}"
    return False, "Chybí ověřený PR CI pro totožný Git tree — spustit kompletní testy"


def output_gate(path: str, reusable: bool) -> None:
    # Výstup pravdivý pouze po úspěšném ověření všech atributů artefaktu.
    with open(path, "a", encoding="utf-8") as out:
        out.write(f"reusable={'true' if reusable else 'false'}\n")


def main() -> int:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    issue = sub.add_parser("create")
    issue.add_argument("--workflow", required=True, choices=sorted(WORKFLOWS))
    issue.add_argument("--pr", type=int, required=True)
    issue.add_argument("--head", required=True)
    issue.add_argument("--run", type=int, required=True)
    issue.add_argument("--attempt", type=int, required=True)
    issue.add_argument("--output", type=Path, required=True)
    check = sub.add_parser("verify")
    check.add_argument("--workflow", required=True, choices=sorted(WORKFLOWS))
    check.add_argument("--repo", required=True)
    check.add_argument("--main-sha", required=True)
    check.add_argument("--github-output", required=True)
    args = parser.parse_args()
    if args.command == "create":
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(make_proof(args.workflow, args.pr, args.head,
                                                     args.run, args.attempt), sort_keys=True) + "\n",
                               encoding="utf-8")
        return 0
    try:
        reuse, why = can_reuse(args.repo, args.main_sha, args.workflow)
    except Exception as exc:
        # Jakákoli nedostupnost GitHub API/artefaktu znamená FULL CI, ne release bez testů.
        reuse, why = False, f"Kontrola důkazu selhala ({exc}) — spustit kompletní CI"
    output_gate(args.github_output, reuse)
    print(why)
    return 0


if __name__ == "__main__":
    sys.exit(main())
