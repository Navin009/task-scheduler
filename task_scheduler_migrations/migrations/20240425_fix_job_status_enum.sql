-- Drop existing job_status type
DROP TYPE IF EXISTS job_status CASCADE;

-- Recreate job_status type with lowercase values
CREATE TYPE job_status AS ENUM (
    'pending',     -- Job is waiting to be executed
    'running',     -- Job is currently being executed
    'completed',   -- Job has finished successfully
    'failed',      -- Job has failed
    'retrying'     -- Job is being retried after a failure
);

-- Update existing jobs to use lowercase status values
UPDATE jobs SET status = LOWER(status)::job_status;

-- Update check constraint for status
ALTER TABLE jobs
DROP CONSTRAINT IF EXISTS valid_status;

ALTER TABLE jobs
ADD CONSTRAINT valid_status CHECK (status IN ('pending', 'running', 'completed', 'failed', 'retrying')); 