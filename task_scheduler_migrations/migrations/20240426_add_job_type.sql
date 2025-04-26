-- Add job_type column to jobs table
ALTER TABLE jobs ADD COLUMN job_type VARCHAR(20) NOT NULL DEFAULT 'one_time';

-- Add index for job type
CREATE INDEX idx_jobs_type ON jobs(job_type);

-- Add check constraint for job types
ALTER TABLE jobs
ADD CONSTRAINT valid_job_type CHECK (job_type IN ('one_time', 'recurring', 'polling'));

-- Add comment to explain job type
COMMENT ON COLUMN jobs.job_type IS 'Type of job: one_time, recurring, or polling'; 