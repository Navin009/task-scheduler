-- Add new columns to jobs table for job types and tracking
ALTER TABLE jobs
ADD COLUMN job_type VARCHAR(20) NOT NULL DEFAULT 'one_time',
ADD COLUMN parent_job_id UUID REFERENCES jobs(id),
ADD COLUMN max_attempts INTEGER DEFAULT 1,
ADD COLUMN attempts INTEGER DEFAULT 0,
ADD COLUMN priority INTEGER DEFAULT 0,
ADD COLUMN payload JSONB DEFAULT '{}'::jsonb;

-- Add index for job type and parent job
CREATE INDEX idx_jobs_type ON jobs(job_type);
CREATE INDEX idx_jobs_parent ON jobs(parent_job_id);
CREATE INDEX idx_jobs_status_scheduled ON jobs(status, scheduled_at);

-- Add check constraint for job types
ALTER TABLE jobs
ADD CONSTRAINT valid_job_type CHECK (job_type IN ('one_time', 'recurring', 'polling'));

-- Add check constraint for attempts
ALTER TABLE jobs
ADD CONSTRAINT valid_attempts CHECK (attempts <= max_attempts);

-- Add check constraint for priority
ALTER TABLE jobs
ADD CONSTRAINT valid_priority CHECK (priority >= 0 AND priority <= 10);

-- Add check constraint for status
ALTER TABLE jobs
ADD CONSTRAINT valid_status CHECK (status IN ('pending', 'running', 'completed', 'failed', 'retrying'));

-- Update existing jobs to have one_time type
UPDATE jobs SET job_type = 'one_time' WHERE job_type IS NULL;

-- Add comment to explain job types
COMMENT ON COLUMN jobs.job_type IS 'Type of job: one_time, recurring, or polling';
COMMENT ON COLUMN jobs.parent_job_id IS 'Reference to parent job for recurring or polling jobs';
COMMENT ON COLUMN jobs.max_attempts IS 'Maximum number of attempts for polling jobs';
COMMENT ON COLUMN jobs.attempts IS 'Current number of attempts for polling jobs';
COMMENT ON COLUMN jobs.priority IS 'Job priority (0-10)';
COMMENT ON COLUMN jobs.payload IS 'Job payload in JSON format'; 