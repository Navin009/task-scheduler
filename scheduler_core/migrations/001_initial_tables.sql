CREATE TYPE job_status AS ENUM ('Pending', 'Running', 'Completed', 'Failed', 'Retrying');
CREATE TYPE schedule_type AS ENUM ('Immediate', 'Cron', 'Interval');

CREATE TABLE jobs (
    id VARCHAR(255) PRIMARY KEY,
    schedule_type schedule_type NOT NULL,
    schedule TEXT NOT NULL,
    payload JSONB NOT NULL,
    status job_status NOT NULL DEFAULT 'Pending',
    retries INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE templates (
    id VARCHAR(255) PRIMARY KEY,
    cron_pattern TEXT NOT NULL,
    payload_template JSONB NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
