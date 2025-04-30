-- Create custom types
CREATE TYPE schedule_type AS ENUM ('one_time', 'recurring', 'polling');
CREATE TYPE job_status AS ENUM ('pending', 'running', 'completed', 'failed', 'cancelled', 'retrying');
CREATE TYPE job_type AS ENUM ('one_time', 'recurring', 'polling');

-- Create templates table
CREATE TABLE templates (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    job_type job_type NOT NULL,
    schedule_type schedule_type NOT NULL,
    schedule VARCHAR(30) NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    interval INTEGER,
    cron VARCHAR(30),
    schedule_at TIMESTAMP WITH TIME ZONE,
    max_attempts INTEGER NOT NULL DEFAULT 1,
    metadata JSONB,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create jobs table with partitioning
CREATE TABLE jobs (
    id UUID NOT NULL,
    description TEXT,
    parent_job_id UUID,
    reference_id text,
    status job_status NOT NULL DEFAULT 'pending',
    priority INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    retries INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    payload JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    next_run_at TIMESTAMP WITH TIME ZONE,
    last_run_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    scheduled_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    metadata JSONB,
    template_id UUID REFERENCES templates(id),
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- Create default partition
CREATE TABLE jobs_default PARTITION OF jobs default;

-- Create indexes
CREATE INDEX idx_jobs_status ON jobs (status);
CREATE INDEX idx_jobs_next_run_at ON jobs (next_run_at);
CREATE INDEX idx_templates_name ON templates (name);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger for updated_at
CREATE TRIGGER update_jobs_updated_at
    BEFORE UPDATE ON jobs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_templates_updated_at
    BEFORE UPDATE ON templates
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column(); 