#!/usr/bin/env python3
"""Publish an exact local linear commit chain with GitHub's Git Data API.

This intentionally never reads or displays an auth token. It requires the
expected remote base SHA, verifies every returned object SHA against the local
Git object, and only updates the ref with --update-ref after a fresh base check.
"""

import argparse
import base64
import datetime as dt
import json
import re
import subprocess
import sys


class PushError(RuntimeError):
    pass


def git(root, *args):
    p = subprocess.run(["git", "-C", root, *args], stdout=subprocess.PIPE,
                       stderr=subprocess.PIPE, check=False)
    if p.returncode:
        raise PushError(f"git {' '.join(args[:2])} failed ({p.returncode})")
    return p.stdout


def api(repo, endpoint, method=None, body=None):
    cmd = ["gh", "api", f"repos/{repo}/{endpoint.lstrip('/')}" ]
    if method:
        cmd += ["--method", method]
    if body is not None:
        cmd += ["--input", "-"]
    p = subprocess.run(cmd, input=(json.dumps(body, ensure_ascii=True).encode()
                                   if body is not None else None),
                       stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                       check=False)
    if p.returncode:
        # Do not echo stderr: it can include environment-specific auth/proxy data.
        raise PushError(f"GitHub API {method or 'GET'} {endpoint} failed ({p.returncode})")
    try:
        return json.loads(p.stdout)
    except Exception as e:
        raise PushError(f"GitHub API {endpoint} returned invalid JSON") from e


def read_commit(root, sha):
    raw = git(root, "cat-file", "commit", sha)
    try:
        headers, message = raw.split(b"\n\n", 1)
    except ValueError as e:
        raise PushError(f"malformed local commit {sha}") from e
    fields = {}
    for line in headers.splitlines():
        if line.startswith(b" "):
            raise PushError(f"continued commit header unsupported in {sha}")
        key, sep, value = line.partition(b" ")
        if not sep:
            raise PushError(f"malformed header in {sha}")
        fields.setdefault(key, []).append(value)
    allowed = {b"tree", b"parent", b"author", b"committer"}
    if set(fields) - allowed:
        raise PushError(f"nonstandard commit headers in {sha}: {sorted(set(fields)-allowed)!r}")
    if len(fields.get(b"tree", [])) != 1 or len(fields.get(b"author", [])) != 1 or len(fields.get(b"committer", [])) != 1:
        raise PushError(f"incomplete commit headers in {sha}")
    try:
        tree = fields[b"tree"][0].decode("ascii")
        parents = [x.decode("ascii") for x in fields.get(b"parent", [])]
    except UnicodeDecodeError as e:
        raise PushError(f"non-ASCII object ID in {sha}") from e
    return {"tree": tree, "parents": parents,
            "author": person(fields[b"author"][0]),
            "committer": person(fields[b"committer"][0]),
            "message": message.decode("utf-8"), "raw": raw}


def person(value):
    m = re.fullmatch(rb"(.*) <([^<>]*)> (-?[0-9]+) ([+-][0-9]{4})", value)
    if not m:
        raise PushError("unsupported local author/committer header")
    name, email, seconds, zone = m.groups()
    offset_minutes = (int(zone[1:3]) * 60 + int(zone[3:5])) * (1 if zone[:1] == b"+" else -1)
    tz = dt.timezone(dt.timedelta(minutes=offset_minutes))
    when = dt.datetime.fromtimestamp(int(seconds), dt.timezone.utc).astimezone(tz)
    date = when.isoformat(timespec="seconds")
    try:
        return {"name": name.decode("utf-8"), "email": email.decode("utf-8"), "date": date}
    except UnicodeDecodeError as e:
        raise PushError("non-UTF-8 author/committer metadata") from e


def tree_entries(root, commit_sha):
    raw = git(root, "ls-tree", "-r", "-z", "--full-tree", commit_sha)
    result = {}
    for entry in raw.split(b"\0"):
        if not entry:
            continue
        meta, path = entry.split(b"\t", 1)
        mode, kind, sha = meta.split(b" ")
        try:
            key = path.decode("utf-8")
        except UnicodeDecodeError as e:
            raise PushError("non-UTF-8 repository path unsupported") from e
        result[key] = (mode.decode("ascii"), kind.decode("ascii"), sha.decode("ascii"))
    return result


def blob_bytes(root, sha):
    return git(root, "cat-file", "blob", sha)


def upload_blob(repo, root, sha):
    content = base64.b64encode(blob_bytes(root, sha)).decode("ascii")
    result = api(repo, "git/blobs", "POST", {"content": content, "encoding": "base64"})
    if result.get("sha") != sha:
        raise PushError(f"blob SHA mismatch: local {sha}, API {result.get('sha')}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--repo", required=True)
    ap.add_argument("--branch", required=True)
    ap.add_argument("--base", required=True)
    ap.add_argument("--head", required=True)
    ap.add_argument("--update-ref", action="store_true",
                     help="perform the final non-force ref update after all SHA checks")
    a = ap.parse_args()

    base_info = read_commit(a.root, a.base)
    head_info = read_commit(a.root, a.head)
    if len(head_info["parents"]) != 1:
        raise PushError("head must be a non-merge commit")

    chain = []
    cursor = a.head
    while cursor != a.base:
        info = read_commit(a.root, cursor)
        if len(info["parents"]) != 1:
            raise PushError(f"merge/root commit in chain at {cursor}")
        chain.append((cursor, info))
        cursor = info["parents"][0]
        if len(chain) > 1000:
            raise PushError("commit chain limit exceeded")
    chain.reverse()
    if not chain:
        raise PushError("head equals base; no commits to publish")

    branch = api(a.repo, f"branches/{a.branch}")
    initial = branch.get("commit", {}).get("sha")
    if initial != a.base:
        raise PushError(f"remote branch moved: expected base {a.base}, saw {initial}")
    remote_base = api(a.repo, f"git/commits/{a.base}")
    if remote_base.get("sha") != a.base or remote_base.get("tree", {}).get("sha") != base_info["tree"]:
        raise PushError("remote base commit/tree does not match the local base object")

    print(f"Remote base verified; publishing {len(chain)} local commits.", flush=True)
    uploaded_blobs = set()
    previous_commit = a.base
    previous_tree = base_info["tree"]
    previous_entries = tree_entries(a.root, a.base)

    for commit_sha, info in chain:
        if info["parents"] != [previous_commit]:
            raise PushError(f"nonlinear local chain at {commit_sha}")
        if info["tree"] == previous_tree:
            tree_sha = previous_tree
        else:
            current_entries = tree_entries(a.root, commit_sha)
            changes = []
            for path in sorted(set(previous_entries) | set(current_entries)):
                old = previous_entries.get(path)
                new = current_entries.get(path)
                if old == new:
                    continue
                if new is None:
                    mode, kind, sha = old
                    changes.append({"path": path, "mode": mode, "type": kind, "sha": None})
                else:
                    mode, kind, sha = new
                    if kind == "blob" and sha not in uploaded_blobs:
                        upload_blob(a.repo, a.root, sha)
                        uploaded_blobs.add(sha)
                    changes.append({"path": path, "mode": mode, "type": kind, "sha": sha})
            result = api(a.repo, "git/trees", "POST",
                         {"base_tree": previous_tree, "tree": changes})
            tree_sha = result.get("sha")
            if tree_sha != info["tree"]:
                raise PushError(f"tree SHA mismatch at {commit_sha}: local {info['tree']}, API {tree_sha}")
            previous_entries = current_entries
            previous_tree = tree_sha

        if tree_sha != info["tree"]:
            raise PushError(f"tree SHA mismatch at {commit_sha}")
        result = api(a.repo, "git/commits", "POST", {
            "message": info["message"], "tree": tree_sha,
            "parents": info["parents"], "author": info["author"],
            "committer": info["committer"],
        })
        returned = result.get("sha")
        if returned != commit_sha:
            raise PushError(f"commit SHA mismatch: local {commit_sha}, API {returned}")
        previous_commit = commit_sha
        print(f"Verified commit {commit_sha}.", flush=True)

    if previous_commit != a.head:
        raise PushError("published chain did not end at requested head")
    if not a.update_ref:
        print("All Git objects verified; ref not changed (pass --update-ref to publish).")
        return

    fresh = api(a.repo, f"branches/{a.branch}").get("commit", {}).get("sha")
    if fresh != a.base:
        raise PushError(f"remote branch changed before update: expected {a.base}, saw {fresh}")
    updated = api(a.repo, f"git/refs/heads/{a.branch}", "PATCH",
                  {"sha": a.head, "force": False})
    actual = updated.get("object", {}).get("sha")
    if actual != a.head:
        raise PushError(f"ref update returned unexpected SHA {actual}")
    print(f"Updated {a.branch} to {a.head} with force=false.")


if __name__ == "__main__":
    try:
        main()
    except PushError as e:
        print(f"STOP: {e}", file=sys.stderr)
        sys.exit(2)
