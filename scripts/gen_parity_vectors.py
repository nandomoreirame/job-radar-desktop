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

import os
from job_radar.filters import count_skill_hits, detect_contract, detect_seniority, detect_work_mode, parse_salary, passes_hard_filters
from job_radar.models import Job, clean_html

SKILLS = {"python", "aws", "typescript", "node.js", "c#x", "react", "ci/cd", "docker"}
JOBS = [
    ("Engenheiro de Software Sênior", "Python, AWS e TypeScript em produção.", "", "remote"),
    ("Desenvolvedor Pleno", "Python AWS Docker", "", "remote"),
    ("Dev PL/SQL Sênior", "Python AWS Docker", "", "remote"),
    ("Desenvolvedor Java Sênior", "Python AWS Docker", "", "remote"),
    ("Full Stack (React / Python / Go or Node.js or Java) Sênior", "AWS", "", "remote"),
    ("Senior Developer", "JavaScript, python, aws, docker. Sem java no texto? javascript.", "", "remote"),
    ("QA Engineer Sênior", "Python AWS Docker", "", "remote"),
    ("Product Engineer Sênior", "Python AWS Docker", "", "remote"),
    ("Product Owner Sênior", "Python AWS Docker", "", "remote"),
    ("Banco de Talentos Tecnologia", "Python AWS Docker", "", "remote"),
    ("Vaga afirmativa para mulheres - Dev Sênior", "Python AWS Docker", "", "remote"),
    ("Dev Sênior", "Esta vaga é exclusiva para pessoas com deficiência. Python AWS Docker", "", "remote"),
    ("Dev Sênior", "Vagas sinalizadas como exclusivas ou não para pessoas com deficiência. Python AWS Docker", "", "remote"),
    ("Dev Sênior", "Trabalho presencial. Python AWS Docker", "São Paulo, SP", None),
    ("Dev Sênior", "Modelo híbrido. Python AWS Docker", "Campinas, SP", None),
    ("Dev Sênior", "Home office. Python AWS Docker", "", None),
    ("Dev Sênior", "Só Python", "", "remote"),
    ("Dev Sênior", "Python AWS Docker. Salário R$ 5.000,00 PJ", "", "remote"),
    ("Dev Sênior", "Python AWS Docker. VR de R$ 1.000,00", "", "remote"),
    ("Dev Sênior", "Python AWS Docker ci/cd node.js. 15k CLT", "", "remote"),
    ("Consultor SAP Sênior", "SAP ABAP SAP MM. Python AWS", "", "remote"),
    ("", "", "", None),
]
SETTINGS = {"MIN_SALARY_PJ": "10000", "MIN_SALARY_CLT": "9000", "COMMUTE_CITIES": "São Paulo", "MAX_JOB_AGE_DAYS": "0", "BLOCKED_COMPANIES": "Bloqueada S.A."}

def job_of(title, desc, loc, hint, company="Acme"):
    return Job(source="gupy", external_id="1", title=title, company=company, url="https://a.example/1", description=desc, location=loc, work_mode_hint=hint)

def filter_vectors():
    os.environ.update(SETTINGS)
    rows = []
    for title, desc, loc, hint in JOBS:
        job = job_of(title, desc, loc, hint)
        ok, reason = passes_hard_filters(job, SKILLS, 3)
        rows.append({"title": title, "description": desc, "location": loc, "hint": hint, "ok": ok, "reason": reason, "hits": sorted(count_skill_hits(job, SKILLS))})
    blocked = passes_hard_filters(job_of("Dev Sênior", "Python AWS Docker", "", "remote", company="BLOQUEADA"), SKILLS, 3)
    html = ["<p>Olá&nbsp;<b>mundo</b></p>", "&quot;a&quot; &amp; b", "", "a\n\n b"]
    return {
        "settings": SETTINGS, "skills": sorted(SKILLS), "jobs": rows, "blocked": list(blocked),
        "clean_html": [[h, clean_html(h)] for h in html],
        "salaries": [[t, parse_salary(t)] for t in ["R$ 14.000,00", "até 14k", "VR R$ 900", "R$ 14.000 a R$ 20.000", "ano de 2026", ""]],
        "contracts": [[t, detect_contract(t)] for t in ["vaga PJ", "CLT", "regime celetista", "nada", ""]],
    }


import datetime as dt
import job_radar.scoring as scoring
from job_radar.profile import expand_synonyms, extract_skills


class FixedDatetime(dt.datetime):
    """Frozen clock; published dates are built from it so isinstance() still holds."""

    @classmethod
    def now(cls, tz=None):
        base = cls(2026, 11, 2, 15, 0, 0, tzinfo=dt.timezone.utc)
        return base.astimezone(tz) if tz else cls(2026, 11, 2, 12, 0, 0)


def scoring_vectors():
    scoring.datetime = FixedDatetime
    utc = dt.timezone.utc
    profile = (ROOT / "src-tauri" / "tests" / "fixtures" / "perfil.md").read_text(encoding="utf-8")
    skills = expand_synonyms(extract_skills(profile))
    dates = [None, FixedDatetime(2026, 11, 1, tzinfo=utc), FixedDatetime(2026, 10, 5, tzinfo=utc), FixedDatetime(2026, 9, 1, tzinfo=utc)]
    extra = [
        ("Staff AI Engineer", "Python TypeScript AWS FastAPI Docker SQL Polars Rust React Next.js MCP", "", "remote"),
        ("Engenheiro de Dados Sênior", "Python SQL Polars ETL", "", "hybrid"),
        ("Arquiteto de Software", "Salesforce SAP Dynamics. Python", "", None),
        ("Tech Lead", "Node.js e TypeScript", "", "onsite"),
    ]
    rows = []
    for title, desc, loc, hint in JOBS + extra:
        for published in dates:
            job = job_of(title, desc, loc, hint)
            job.published_at = published
            hits = count_skill_hits(job, skills)
            rows.append({"title": title, "description": desc, "location": loc, "hint": hint,
                         "published_at": published.isoformat() if published else None,
                         "hits": len(hits), "score": scoring.score_job(job, skills, hits)})
    return {"skills": sorted(skills), "extracted": sorted(extract_skills(profile)), "jobs": rows}


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    identity = {
        "titles": [[t, title_key(t)] for t in TITLES],
        "companies": [[c, company_key(c)] for c in COMPANIES],
        "fingerprints": [[c, t, job_fingerprint(c, t)] for c in COMPANIES for t in TITLES],
        "urls": [[u, url_key(u)] for u in URLS],
    }
    (OUT / "identity.json").write_text(json.dumps(identity, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    (OUT / "scoring.json").write_text(json.dumps(scoring_vectors(), ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    (OUT / "filters.json").write_text(json.dumps(filter_vectors(), ensure_ascii=False, indent=1) + "\n", encoding="utf-8")

if __name__ == "__main__":
    main()
