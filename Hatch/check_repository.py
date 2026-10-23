#!/usr/bin/env python3
"""Check project links, Markdown anchors, and pinned workflow actions offline."""
from html.parser import HTMLParser
from pathlib import Path
import re
import subprocess
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]


def prose(text):
    return re.sub(r"^(```|~~~).*?^\1[^\n]*$", "", text, flags=re.M | re.S)


def anchors(text):
    used = {}
    result = set()
    for heading in re.findall(r"^#{1,6}\s+(.+?)\s*#*\s*$", prose(text), re.M):
        slug = re.sub(r"[^\w\- ]", "", heading.lower()).replace(" ", "-")
        number = used.get(slug, 0)
        used[slug] = number + 1
        result.add(slug + (f"-{number}" if number else ""))
    return result


class HtmlLinks(HTMLParser):
    def __init__(self):
        super().__init__()
        self.targets = []

    def handle_starttag(self, tag, attrs):
        for name, value in attrs:
            if name in {"href", "src"} and value:
                self.targets.append(value)


def link_errors(file, root=ROOT):
    text = prose(file.read_text())
    html = HtmlLinks()
    html.feed(text)
    targets = re.findall(r"!?\[[^\]\n]*\]\(([^\s)]+)(?:\s+\"[^\"]*\")?\)", text)
    errors = []
    for target in targets + html.targets:
        url = urlsplit(target)
        if url.scheme or url.netloc:
            continue
        destination = (file.parent / unquote(url.path)).resolve() if url.path else file
        if root.resolve() not in [destination, *destination.parents]:
            errors.append(f"Link escapes repository: {target}")
        elif not destination.exists():
            errors.append(f"Missing link target: {target}")
        elif url.fragment and destination.suffix == ".md" and unquote(url.fragment) not in anchors(destination.read_text()):
            errors.append(f"Missing Markdown anchor: {target}")
    return errors


def action_errors(text):
    errors = []
    for target in re.findall(r"\buses:\s*([^\s#]+)", text):
        if not target.startswith("./") and not re.fullmatch(r"[\w./-]+@[0-9a-f]{40}", target):
            errors.append(f"Action must use a full commit SHA: {target}")
    if "pull_request_target:" in text:
        errors.append("Use pull_request for untrusted contribution checks")
    if re.search(r"permissions:\s*write-all", text):
        errors.append("Workflow must declare scoped token permissions")
    return errors


def main():
    files = [ROOT / name for name in ["README.md", "CONTRIBUTING.md", "SECURITY.md", "SUPPORT.md", "CODE_OF_CONDUCT.md", "CHANGELOG.md"]]
    files += sorted((ROOT / "Hatch").rglob("*.md"))
    files += sorted((ROOT / ".github").glob("*.md"))
    errors = []
    for file in files:
        errors += [f"{file.relative_to(ROOT)}: {error}" for error in link_errors(file)]
    for file in sorted((ROOT / ".github/workflows").glob("*.yml")):
        errors += [f"{file.relative_to(ROOT)}: {error}" for error in action_errors(file.read_text())]
    subprocess.run(["git", "diff", "--check"], cwd=ROOT, check=True)
    if errors:
        raise SystemExit("\n".join(errors))
    print(f"Checked {len(files)} project documents and all workflow action pins.")


if __name__ == "__main__":
    main()
