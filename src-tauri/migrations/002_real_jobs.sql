-- Migration 2: real imported jobs may be unscored and discarded.
CREATE TABLE cards_new (
  id TEXT PRIMARY KEY CHECK (length(id) BETWEEN 1 AND 64), title TEXT NOT NULL, company TEXT NOT NULL, source TEXT NOT NULL,
  target TEXT CHECK (target IN ('ai_llm','full_stack','tech_lead','data')), work_model TEXT CHECK (work_model IN ('remote','hybrid','onsite')), location TEXT,
  score INTEGER CHECK (score BETWEEN 0 AND 100), stage TEXT NOT NULL CHECK (stage IN ('inbox','triage','applied','in_process','offer','closed')),
  outcome TEXT CHECK (outcome IN ('hired','rejected','discarded','no_response')), process_step TEXT, salary_pj INTEGER CHECK (salary_pj >= 0), salary_clt INTEGER CHECK (salary_clt >= 0),
  has_cv INTEGER NOT NULL DEFAULT 0 CHECK (has_cv IN (0,1)), has_coverage INTEGER NOT NULL DEFAULT 0 CHECK (has_coverage IN (0,1)), has_interview_prep INTEGER NOT NULL DEFAULT 0 CHECK (has_interview_prep IN (0,1)),
  last_activity TEXT, next_action TEXT, next_action_date TEXT, ai_running TEXT, unread INTEGER NOT NULL DEFAULT 0 CHECK (unread IN (0,1)), recruiter_name TEXT, recruiter_email TEXT, recruiter_linkedin TEXT, job_url TEXT, application_url TEXT, notes TEXT NOT NULL DEFAULT '',
  url_key TEXT, description TEXT NOT NULL DEFAULT '', discard_reason TEXT, discarded_at TEXT,
  CHECK ((stage = 'closed') = (outcome IS NOT NULL))
);
INSERT INTO cards_new (rowid,id,title,company,source,target,work_model,location,score,stage,outcome,process_step,salary_pj,salary_clt,has_cv,has_coverage,has_interview_prep,last_activity,next_action,next_action_date,ai_running,unread,recruiter_name,recruiter_email,recruiter_linkedin,job_url,application_url,notes)
SELECT rowid,id,title,company,source,target,work_model,location,score,stage,outcome,process_step,salary_pj,salary_clt,has_cv,has_coverage,has_interview_prep,last_activity,next_action,next_action_date,ai_running,unread,recruiter_name,recruiter_email,recruiter_linkedin,job_url,application_url,notes FROM cards;
CREATE TABLE timeline_events_new (id INTEGER PRIMARY KEY, card_id TEXT NOT NULL REFERENCES cards_new(id) ON DELETE CASCADE, at TEXT NOT NULL, kind TEXT NOT NULL CHECK (kind IN ('found','applied','response','interview','offer','closed')), label TEXT NOT NULL);
INSERT INTO timeline_events_new SELECT * FROM timeline_events;
DROP TABLE timeline_events;
DROP TABLE cards;
ALTER TABLE cards_new RENAME TO cards;
ALTER TABLE timeline_events_new RENAME TO timeline_events;
CREATE INDEX timeline_events_card ON timeline_events(card_id, at);
CREATE UNIQUE INDEX cards_url_key ON cards(url_key) WHERE url_key IS NOT NULL;
CREATE INDEX cards_discard_reason ON cards(discard_reason);
CREATE TABLE dismissed_jobs (fingerprint TEXT PRIMARY KEY, url_key TEXT, dismissed_at TEXT NOT NULL);
CREATE INDEX dismissed_jobs_url_key ON dismissed_jobs(url_key);
