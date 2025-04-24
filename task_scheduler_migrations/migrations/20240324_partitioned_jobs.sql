-- Drop existing tables and types
DROP TABLE IF EXISTS jobs CASCADE;
DROP TABLE IF EXISTS one_time_jobs CASCADE;
DROP TABLE IF EXISTS recurring_jobs CASCADE;
DROP TABLE IF EXISTS polling_jobs CASCADE;
DROP TYPE IF EXISTS schedule_type CASCADE;
DROP TYPE IF EXISTS job_status CASCADE;
DROP FUNCTION IF EXISTS update_updated_at_column CASCADE;

-- Create custom enum types
CREATE TYPE schedule_type AS ENUM (
    'one_time',    -- For single execution at a specific time
    'recurring',   -- For recurring executions using cron expressions
    'polling'      -- For status checking with retries
);

CREATE TYPE job_status AS ENUM (
    'Pending',     -- Job is waiting to be executed
    'Running',     -- Job is currently being executed
    'Completed',   -- Job has finished successfully
    'Failed',      -- Job has failed
    'Retrying'     -- Job is being retried after a failure
);

-- Create base jobs table for common fields
CREATE TABLE jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    schedule_type schedule_type NOT NULL,
    status job_status NOT NULL DEFAULT 'Pending',
    payload JSONB NOT NULL,
    retries INT NOT NULL DEFAULT 0,
    max_retries INT NOT NULL DEFAULT 3,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
) PARTITION BY LIST (schedule_type);

-- Create partition for one-time jobs
CREATE TABLE one_time_jobs PARTITION OF jobs
    FOR VALUES IN ('one_time')
    PARTITION BY RANGE (created_at);

-- Create monthly partitions for one-time jobs
CREATE TABLE one_time_jobs_y2024m01 PARTITION OF one_time_jobs
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
CREATE TABLE one_time_jobs_y2024m02 PARTITION OF one_time_jobs
    FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');
-- Add more monthly partitions as needed

-- Create partition for recurring jobs
CREATE TABLE recurring_jobs PARTITION OF jobs
    FOR VALUES IN ('recurring')
    PARTITION BY RANGE (created_at);

-- Create monthly partitions for recurring jobs
CREATE TABLE recurring_jobs_y2024m01 PARTITION OF recurring_jobs
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
CREATE TABLE recurring_jobs_y2024m02 PARTITION OF recurring_jobs
    FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');
-- Add more monthly partitions as needed

-- Create partition for polling jobs
CREATE TABLE polling_jobs PARTITION OF jobs
    FOR VALUES IN ('polling')
    PARTITION BY RANGE (created_at);

-- Create monthly partitions for polling jobs
CREATE TABLE polling_jobs_y2024m01 PARTITION OF polling_jobs
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
CREATE TABLE polling_jobs_y2024m02 PARTITION OF polling_jobs
    FOR VALUES FROM ('2024-02-01') TO ('2024-03-01');
-- Add more monthly partitions as needed

-- Add type-specific columns to each partition
ALTER TABLE one_time_jobs ADD COLUMN scheduled_at TIMESTAMPTZ NOT NULL;
ALTER TABLE one_time_jobs ADD COLUMN started_at TIMESTAMPTZ;
ALTER TABLE one_time_jobs ADD COLUMN completed_at TIMESTAMPTZ;

ALTER TABLE recurring_jobs ADD COLUMN cron_expression TEXT NOT NULL;
ALTER TABLE recurring_jobs ADD COLUMN last_run_at TIMESTAMPTZ;
ALTER TABLE recurring_jobs ADD COLUMN next_run_at TIMESTAMPTZ;

ALTER TABLE polling_jobs ADD COLUMN interval_seconds INT NOT NULL;
ALTER TABLE polling_jobs ADD COLUMN max_attempts INT NOT NULL;
ALTER TABLE polling_jobs ADD COLUMN last_check_at TIMESTAMPTZ;
ALTER TABLE polling_jobs ADD COLUMN next_check_at TIMESTAMPTZ;

-- Create indexes for performance
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_created_at ON jobs(created_at);
CREATE INDEX idx_one_time_jobs_scheduled_at ON one_time_jobs(scheduled_at);
CREATE INDEX idx_recurring_jobs_next_run_at ON recurring_jobs(next_run_at);
CREATE INDEX idx_polling_jobs_next_check_at ON polling_jobs(next_check_at);

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

-- Add comments to explain the schema
COMMENT ON TABLE jobs IS 'Base table for all job types with common fields';
COMMENT ON TABLE one_time_jobs IS 'Partition for one-time scheduled jobs';
COMMENT ON TABLE recurring_jobs IS 'Partition for recurring jobs using cron expressions';
COMMENT ON TABLE polling_jobs IS 'Partition for polling jobs with retry logic'; 