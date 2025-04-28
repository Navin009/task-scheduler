-- Create custom types
CREATE TYPE schedule_type AS ENUM ('one_time', 'recurring');
CREATE TYPE job_status AS ENUM ('pending', 'running', 'completed', 'failed', 'cancelled');
CREATE TYPE job_type AS ENUM ('http', 'shell', 'email');

-- Create templates table
CREATE TABLE templates (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    job_type job_type NOT NULL,
    schedule_type schedule_type NOT NULL,
    schedule JSONB NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT valid_template_schedule CHECK (
        (schedule_type = 'one_time' AND schedule ? 'run_at') OR
        (schedule_type = 'recurring' AND schedule ? 'cron_expression')
    )
);

-- Create jobs table with partitioning
CREATE TABLE jobs (
    id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    schedule_type schedule_type NOT NULL,
    schedule JSONB NOT NULL,
    status job_status NOT NULL DEFAULT 'pending',
    job_type job_type NOT NULL,
    priority INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    retry_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    next_run_at TIMESTAMP WITH TIME ZONE,
    last_run_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB,
    template_id UUID REFERENCES templates(id),
    CONSTRAINT valid_schedule CHECK (
        (schedule_type = 'one_time' AND schedule ? 'run_at') OR
        (schedule_type = 'recurring' AND schedule ? 'cron_expression')
    ),
    PRIMARY KEY (id, next_run_at)
) PARTITION BY RANGE (next_run_at);

-- Create partitions
CREATE TABLE jobs_past PARTITION OF jobs
    FOR VALUES FROM (MINVALUE) TO (CURRENT_TIMESTAMP);

CREATE TABLE jobs_future PARTITION OF jobs
    FOR VALUES FROM (CURRENT_TIMESTAMP) TO (MAXVALUE);

-- Create indexes
CREATE INDEX idx_jobs_status ON jobs (status);
CREATE INDEX idx_jobs_next_run_at ON jobs (next_run_at);
CREATE INDEX idx_jobs_type ON jobs (job_type);
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