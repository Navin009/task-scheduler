-- Create custom enum types for job status and schedule type
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

-- Create jobs table with detailed schema
CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Schedule configuration
    schedule_type schedule_type NOT NULL,
    schedule TEXT NOT NULL,  -- Stores cron expression, ISO timestamp, or polling config JSON
    
    -- Execution timing
    scheduled_at TIMESTAMPTZ NOT NULL,  -- When the job is scheduled to run
    started_at TIMESTAMPTZ,             -- When the job actually started
    completed_at TIMESTAMPTZ,           -- When the job finished
    
    -- Job details
    payload JSONB NOT NULL,             -- Job data and parameters
    status job_status NOT NULL DEFAULT 'Pending',
    
    -- Retry configuration
    retries INT NOT NULL DEFAULT 0,
    max_retries INT NOT NULL DEFAULT 3,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Indexes for common queries
    CONSTRAINT valid_schedule_type CHECK (
        (schedule_type = 'one_time' AND schedule ~ '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$') OR
        (schedule_type = 'recurring' AND schedule ~ '^(\*|([0-9]|1[0-9]|2[0-9]|3[0-9]|4[0-9]|5[0-9])) (\*|([0-9]|1[0-9]|2[0-9]|3[0-9]|4[0-9]|5[0-9])) (\*|([0-9]|1[0-9]|2[0-9]|3[0-9]|4[0-9]|5[0-9])) (\*|([0-9]|1[0-9]|2[0-9]|3[0-9]|4[0-9]|5[0-9])) (\*|([0-9]|1[0-9]|2[0-9]|3[0-9]|4[0-9]|5[0-9]))$') OR
        (schedule_type = 'polling' AND schedule ~ '^{.*"interval":\d+.*"max_attempts":\d+.*}$')
    )
);

-- Create indexes for performance
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_jobs_scheduled_at ON jobs(scheduled_at);
CREATE INDEX idx_jobs_created_at ON jobs(created_at);

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
COMMENT ON TABLE jobs IS 'Stores all scheduled jobs with their execution details';
COMMENT ON COLUMN jobs.schedule_type IS 'Type of schedule: one_time, recurring, or polling';
COMMENT ON COLUMN jobs.schedule IS 'Schedule configuration based on type: ISO timestamp, cron expression, or polling config';
COMMENT ON COLUMN jobs.scheduled_at IS 'When the job is scheduled to run';
COMMENT ON COLUMN jobs.started_at IS 'When the job actually started execution';
COMMENT ON COLUMN jobs.completed_at IS 'When the job finished execution';
COMMENT ON COLUMN jobs.payload IS 'Job-specific data and parameters in JSON format';
COMMENT ON COLUMN jobs.retries IS 'Number of times the job has been retried';
COMMENT ON COLUMN jobs.max_retries IS 'Maximum number of retries allowed'; 