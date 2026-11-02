CREATE TABLE cards (
  id TEXT PRIMARY KEY CHECK (length(id) BETWEEN 1 AND 64), title TEXT NOT NULL, company TEXT NOT NULL, source TEXT NOT NULL,
  target TEXT NOT NULL CHECK (target IN ('ai_llm','full_stack','tech_lead','data')),
  work_model TEXT NOT NULL CHECK (work_model IN ('remote','hybrid','onsite')), location TEXT,
  score INTEGER NOT NULL CHECK (score BETWEEN 0 AND 100), stage TEXT NOT NULL CHECK (stage IN ('inbox','triage','applied','in_process','offer','closed')),
  outcome TEXT CHECK (outcome IN ('hired','rejected','discarded','no_response')), process_step TEXT,
  salary_pj INTEGER CHECK (salary_pj >= 0), salary_clt INTEGER CHECK (salary_clt >= 0),
  has_cv INTEGER NOT NULL DEFAULT 0 CHECK (has_cv IN (0,1)), has_coverage INTEGER NOT NULL DEFAULT 0 CHECK (has_coverage IN (0,1)), has_interview_prep INTEGER NOT NULL DEFAULT 0 CHECK (has_interview_prep IN (0,1)),
  last_activity TEXT, next_action TEXT, next_action_date TEXT, ai_running TEXT, unread INTEGER NOT NULL DEFAULT 0 CHECK (unread IN (0,1)),
  recruiter_name TEXT, recruiter_email TEXT, recruiter_linkedin TEXT, job_url TEXT, application_url TEXT, notes TEXT NOT NULL DEFAULT '',
  CHECK ((stage = 'closed') = (outcome IS NOT NULL))
);
CREATE TABLE timeline_events (id INTEGER PRIMARY KEY, card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE, at TEXT NOT NULL, kind TEXT NOT NULL CHECK (kind IN ('found','applied','response','interview','offer','closed')), label TEXT NOT NULL);
CREATE INDEX timeline_events_card ON timeline_events(card_id, at);
