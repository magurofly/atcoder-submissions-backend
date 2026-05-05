CREATE TABLE submissions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    language_id INTEGER NOT NULL,
    timestamp INTEGER NOT NULL,
    score INTEGER,
    code_size INTEGER NOT NULL,
    status TEXT NOT NULL,
    execution_time INTEGER,
    memory_usage INTEGER
);

CREATE INDEX idx_submissions_task_id ON submissions (task_id);
CREATE INDEX idx_submissions_user_id ON submissions (user_id);
CREATE INDEX idx_submissions_task_language_id_timestamp ON submissions (task_id, language_id, timestamp);
CREATE INDEX idx_submissions_task_timestamp ON submissions (task_id, timestamp);
CREATE INDEX idx_submissions_task_score_timestamp ON submissions (task_id, score, timestamp);
CREATE INDEX idx_submissions_task_code_size_timestamp ON submissions (task_id, code_size, timestamp);
CREATE INDEX idx_submissions_task_status_timestamp ON submissions (task_id, status, timestamp);
CREATE INDEX idx_submissions_task_execution_time_timestamp ON submissions (task_id, execution_time, timestamp);
CREATE INDEX idx_submissions_task_memory_usage_timestamp ON submissions (task_id, memory_usage, timestamp);

CREATE INDEX idx_submissions_task_status_score_timestamp ON submissions (task_id, status, score, timestamp);
CREATE INDEX idx_submissions_task_status_code_size_timestamp ON submissions (task_id, status, code_size, timestamp);
CREATE INDEX idx_submissions_task_status_execution_time_timestamp ON submissions (task_id, status, execution_time, timestamp);
CREATE INDEX idx_submissions_task_status_memory_usage_timestamp ON submissions (task_id, status, memory_usage, timestamp);

CREATE INDEX idx_submissions_task_status_language_timestamp ON submissions (task_id, status, language_id, timestamp);
CREATE INDEX idx_submissions_task_status_language_score_timestamp ON submissions (task_id, status, language_id, score, timestamp);
CREATE INDEX idx_submissions_task_status_language_code_size_timestamp ON submissions (task_id, status, language_id, code_size, timestamp);
CREATE INDEX idx_submissions_task_status_language_execution_time_timestamp ON submissions (task_id, status, language_id, execution_time, timestamp);
CREATE INDEX idx_submissions_task_status_language_memory_usage_timestamp ON submissions (task_id, status, language_id, memory_usage, timestamp);

CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contest_id INTEGER NOT NULL,
    name TEXT NOT NULL
);

CREATE INDEX idx_tasks_contest_id ON tasks (contest_id);
CREATE INDEX idx_tasks_name ON tasks (name);

CREATE TABLE contests (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);

CREATE INDEX idx_contests_name ON contests (name);

CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);

CREATE INDEX idx_users_name ON users (name);

CREATE TABLE languages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT NOT NULL
);

CREATE INDEX idx_languages_description ON languages (description);