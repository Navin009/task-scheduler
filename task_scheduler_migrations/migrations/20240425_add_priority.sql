-- Add priority column to jobs table
ALTER TABLE jobs
ADD COLUMN priority INT NOT NULL DEFAULT 0;

-- Create index for priority
CREATE INDEX idx_jobs_priority ON jobs(priority);

-- Add check constraint for priority
ALTER TABLE jobs
ADD CONSTRAINT valid_priority CHECK (priority >= 0 AND priority <= 10);

-- Add comment to explain priority
COMMENT ON COLUMN jobs.priority IS 'Job priority (0-10, higher is more important)'; 