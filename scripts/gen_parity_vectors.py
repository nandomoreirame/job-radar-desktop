#!/usr/bin/env python3
"""Generates parity vectors from the Python crawler. Inputs are fictional.

Usage: python3 scripts/gen_parity_vectors.py [path-to-job-radar]
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REPO = Path(sys.argv[1] if len(sys.argv) > 1 else ROOT.parent / "job-radar")
sys.path.insert(0, str(REPO / "src"))
OUT = ROOT / "src-tauri" / "tests" / "fixtures" / "parity"

from job_radar.identity import company_key, job_fingerprint, title_key, url_key  # noqa: E402

TITLES = [
    "Backend Sr", "Sr Backend", "Engenheiro de Software Sênior - Remoto", "Desenvolvedor (Híbrido - São Paulo/SP)",
    "Dev Go - GO", "Analista - Goiânia - GO", "Tech Lead | Remote", "Coração de Dados / São Paulo", "Straße Developer",
    "Pessoa Desenvolvedora Back-end PL/SQL", "", "   ", "Staff AI Engineer (Remote)", "C++ Developer - Porto",
    "Engenheira de Machine Learning [Remoto]", "ＦＵＬＬ Stack",
]
COMPANIES = ["", "Acme", "ACME Ltda", "Grupo Acme S.A.", "Acme S/A", "Acme Inc", "ME", "Grupo", "Café & Cia LTDA ME", "Coração Labs EIRELI"]
URLS = [
    "http://WWW.Example.com/jobs/1/", "https://example.com/a?utm_source=x&b=2&a=1#frag", "https://example.com:8080/a?gclid=1",
    "https://example.com:443/a", "https://boards.greenhouse.io/x/jobs/1?gh_src=abc", "https://example.com/a?gh_src=keep",
    "https://x.lever.co/a/1?lever-source=li", "https://www.linkedin.com/jobs/view/1?trk=a&refId=b", "https://example.com/a?q=caf%C3%A9+bar",
    "https://example.com/?b=&a=1", "https://example.com/", "not a url", "", "ftp://example.com/a", "HTTPS://Example.COM/Path/",
]

def main():
    OUT.mkdir(parents=True, exist_ok=True)
    identity = {
        "titles": [[t, title_key(t)] for t in TITLES],
        "companies": [[c, company_key(c)] for c in COMPANIES],
        "fingerprints": [[c, t, job_fingerprint(c, t)] for c in COMPANIES for t in TITLES],
        "urls": [[u, url_key(u)] for u in URLS],
    }
    (OUT / "identity.json").write_text(json.dumps(identity, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

if __name__ == "__main__":
    main()
