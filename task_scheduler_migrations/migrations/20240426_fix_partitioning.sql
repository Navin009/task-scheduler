-- Drop existing tables and types
DROP TABLE IF EXISTS jobs CASCADE;
DROP TABLE IF EXISTS templates CASCADE;
DROP TYPE IF EXISTS schedule_type CASCADE;
DROP TYPE IF EXISTS job_status CASCADE;
DROP FUNCTION IF EXISTS update_updated_at_column CASCADE;

-- Create custom enum types
CREATE TYPE schedule_type AS ENUM ('one_time', 'recurring', 'polling');
CREATE TYPE job_status AS ENUM ('pending', 'running', 'completed', 'failed', 'cancelled');

-- Create base jobs table with partitioning
CREATE TABLE jobs (
    id UUID PRIMARY KEY,
    schedule_type schedule_type NOT NULL,
    status job_status NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    scheduled_for TIMESTAMP WITH TIME ZONE NOT NULL,
    completed_at TIMESTAMP WITH TIME ZONE,
    error_message TEXT,
    metadata JSONB,
    CONSTRAINT jobs_pkey PRIMARY KEY (schedule_type, id)
) PARTITION BY LIST (schedule_type);

-- Create partitions for each job type
CREATE TABLE one_time_jobs PARTITION OF jobs
    FOR VALUES IN ('one_time');

CREATE TABLE recurring_jobs PARTITION OF jobs
    FOR VALUES IN ('recurring');

CREATE TABLE polling_jobs PARTITION OF jobs
    FOR VALUES IN ('polling');

-- Add type-specific columns to each partition
ALTER TABLE one_time_jobs ADD COLUMN execution_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE recurring_jobs ADD COLUMN cron_expression TEXT NOT NULL;
ALTER TABLE recurring_jobs ADD COLUMN next_run TIMESTAMP WITH TIME ZONE NOT NULL;
ALTER TABLE polling_jobs ADD COLUMN endpoint_url TEXT NOT NULL;
ALTER TABLE polling_jobs ADD COLUMN poll_interval INTEGER NOT NULL;  -- in seconds
ALTER TABLE polling_jobs ADD COLUMN last_polled_at TIMESTAMP WITH TIME ZONE;
ALTER TABLE polling_jobs ADD COLUMN response_status INTEGER;
ALTER TABLE polling_jobs ADD COLUMN response_body TEXT;

-- Create indexes for performance
CREATE INDEX idx_jobs_scheduled_for ON jobs (scheduled_for);
CREATE INDEX idx_jobs_status ON jobs (status);
CREATE INDEX idx_recurring_jobs_next_run ON recurring_jobs (next_run);
CREATE INDEX idx_polling_jobs_endpoint_url ON polling_jobs (endpoint_url);

-- Add check constraints
ALTER TABLE jobs ADD CONSTRAINT valid_job_type CHECK (
    (schedule_type = 'one_time' AND id IN (SELECT id FROM one_time_jobs)) OR
    (schedule_type = 'recurring' AND id IN (SELECT id FROM recurring_jobs)) OR
    (schedule_type = 'polling' AND id IN (SELECT id FROM polling_jobs))
);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to automatically update updated_at
CREATE TRIGGER update_jobs_updated_at
    BEFORE UPDATE ON jobs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Add helpful comments
COMMENT ON TABLE jobs IS 'Base table for all job types, partitioned by schedule_type';
COMMENT ON COLUMN jobs.id IS 'Unique identifier for the job';
COMMENT ON COLUMN jobs.schedule_type IS 'Type of job schedule (one_time, recurring, polling)';
COMMENT ON COLUMN jobs.status IS 'Current status of the job';
COMMENT ON COLUMN jobs.scheduled_for IS 'When the job is scheduled to run';
COMMENT ON COLUMN jobs.completed_at IS 'When the job was completed';
COMMENT ON COLUMN jobs.error_message IS 'Error message if the job failed';
COMMENT ON COLUMN jobs.metadata IS 'Additional job metadata in JSON format';

-- Create partition for one-time jobs
CREATE TABLE one_time_jobs PARTITION OF jobs
    FOR VALUES IN ('one_time')
    PARTITION BY RANGE (created_at);

-- Create monthly partitions for one-time jobs
CREATE TABLE one_time_jobs_y2024m01 PARTITION OF one_time_jobs
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
CREATE TABLE one_time_jobs_y2024m02 PARTITION OF one_time_jobs
    FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');

-- Create partition for recurring jobs
CREATE TABLE recurring_jobs PARTITION OF jobs
    FOR VALUES IN ('recurring')
    PARTITION BY RANGE (created_at);

-- Create monthly partitions for recurring jobs
CREATE TABLE recurring_jobs_y2024m01 PARTITION OF recurring_jobs
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
CREATE TABLE recurring_jobs_y2024m02 PARTITION OF recurring_jobs
    FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');

-- Create partition for polling jobs
CREATE TABLE polling_jobs PARTITION OF jobs
    FOR VALUES IN ('polling')
    PARTITION BY RANGE (created_at);

-- Create monthly partitions for polling jobs
CREATE TABLE polling_jobs_y2024m01 PARTITION OF polling_jobs
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
CREATE TABLE polling_jobs_y2024m02 PARTITION OF polling_jobs
    FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');

-- Add type-specific columns to each partition
ALTER TABLE one_time_jobs ADD COLUMN scheduled_at TIMESTAMPTZ NOT NULL;
ALTER TABLE one_time_jobs ADD COLUMN started_at TIMESTAMPTZ;
ALTER TABLE one_time_jobs ADD COLUMN completed_at TIMESTAMPTZ;

ALTER TABLE recurring_jobs ADD COLUMN last_run_at TIMESTAMPTZ;
ALTER TABLE recurring_jobs ADD COLUMN next_run_at TIMESTAMPTZ;

ALTER TABLE polling_jobs ADD COLUMN interval_seconds INT NOT NULL;
ALTER TABLE polling_jobs ADD COLUMN last_check_at TIMESTAMPTZ;
ALTER TABLE polling_jobs ADD COLUMN next_check_at TIMESTAMPTZ;

-- Create indexes for performance
CREATE INDEX idx_jobs_status ON jobs(schedule_type, status);
CREATE INDEX idx_jobs_created_at ON jobs(schedule_type, created_at);
CREATE INDEX idx_jobs_type ON jobs(schedule_type);
CREATE INDEX idx_one_time_jobs_scheduled_at ON one_time_jobs(schedule_type, scheduled_at);
CREATE INDEX idx_recurring_jobs_next_run_at ON recurring_jobs(schedule_type, next_run_at);
CREATE INDEX idx_polling_jobs_next_check_at ON polling_jobs(schedule_type, next_check_at);

-- Add comments to explain the schema
COMMENT ON TABLE one_time_jobs IS 'Partition for one-time scheduled jobs';
COMMENT ON TABLE recurring_jobs IS 'Partition for recurring jobs using cron expressions';
COMMENT ON TABLE polling_jobs IS 'Partition for polling jobs with retry logic';
COMMENT ON COLUMN jobs.schedule_type IS 'Type of job schedule (one_time, recurring, polling)'; 