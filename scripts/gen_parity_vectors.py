import json, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
REPO=Path(sys.argv[1] if len(sys.argv)>1 else ROOT.parent/'job-radar')
sys.path.insert(0,str(REPO/'src'))
from job_radar.identity import company_key, job_fingerprint, title_key, url_key
TITLES=['Backend Sr','Sr Backend','Engenheiro de Software Sênior - Remoto','Tech Lead | Remote','Coração de Dados / São Paulo','Staff AI Engineer (Remote)','']
COMPANIES=['','Acme','ACME Ltda','Grupo Acme S.A.','Café & Cia LTDA ME']
URLS=['http://WWW.Example.com/jobs/1/','https://example.com/a?utm_source=x&b=2&a=1#frag','https://example.com:8080/a?gclid=1','not a url']
OUT=ROOT/'src-tauri/tests/fixtures/parity'; OUT.mkdir(parents=True,exist_ok=True)
(OUT/'identity.json').write_text(json.dumps({'titles':[[x,title_key(x)] for x in TITLES],'companies':[[x,company_key(x)] for x in COMPANIES],'fingerprints':[[c,t,job_fingerprint(c,t)] for c in COMPANIES for t in TITLES],'urls':[[x,url_key(x)] for x in URLS]},ensure_ascii=False,indent=1)+'\n')
