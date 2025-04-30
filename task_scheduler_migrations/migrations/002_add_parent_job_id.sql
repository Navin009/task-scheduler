-- Add parent_job_id column to jobs table
ALTER TABLE jobs ADD COLUMN parent_job_id UUID;
ALTER TABLE jobs ADD COLUMN parent_created_at TIMESTAMP WITH TIME ZONE;

-- Add foreign key constraint
ALTER TABLE jobs ADD CONSTRAINT fk_parent_job 
    FOREIGN KEY (parent_job_id, parent_created_at) 
    REFERENCES jobs(id, created_at);

-- Create index for parent_job_id
CREATE INDEX idx_jobs_parent_job_id ON jobs (parent_job_id, parent_created_at); 