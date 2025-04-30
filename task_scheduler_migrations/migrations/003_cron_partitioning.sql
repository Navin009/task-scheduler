-- First, drop the trigger if it exists
DROP TRIGGER IF EXISTS update_jobs_next_run_at ON jobs;

-- Drop the functions in reverse order of dependency
DROP FUNCTION IF EXISTS update_next_run_at() CASCADE;
DROP FUNCTION IF EXISTS extract_next_run_time(JSONB) CASCADE;
DROP FUNCTION IF EXISTS calculate_next_run_time(TEXT, TIMESTAMP WITH TIME ZONE) CASCADE;
DROP FUNCTION IF EXISTS parse_cron_expression(TEXT) CASCADE;

-- Create function to parse cron expression
CREATE OR REPLACE FUNCTION parse_cron_expression(cron_expr TEXT)
RETURNS TABLE (
    minute TEXT,
    hour TEXT,
    day_of_month TEXT,
    month TEXT,
    day_of_week TEXT
) AS $$
DECLARE
    parts TEXT[];
BEGIN
    parts := string_to_array(cron_expr, ' ');
    IF array_length(parts, 1) != 5 THEN
        RAISE EXCEPTION 'Invalid cron expression: must have 5 parts';
    END IF;
    
    RETURN QUERY
    SELECT parts[1], parts[2], parts[3], parts[4], parts[5];
END;
$$ LANGUAGE plpgsql;

-- Create function to calculate next run time from cron expression
CREATE OR REPLACE FUNCTION calculate_next_run_time(cron_expr TEXT, from_time TIMESTAMP WITH TIME ZONE)
RETURNS TIMESTAMP WITH TIME ZONE AS $$
DECLARE
    cron_parts RECORD;
    next_time TIMESTAMP WITH TIME ZONE;
    check_time TIMESTAMP WITH TIME ZONE;
    found BOOLEAN;
    max_iterations INTEGER := 1000; -- Prevent infinite loops
    iterations INTEGER := 0;
BEGIN
    -- Parse cron expression
    SELECT * INTO cron_parts FROM parse_cron_expression(cron_expr);
    
    -- Start from the next minute
    check_time := date_trunc('minute', from_time) + interval '1 minute';
    found := FALSE;
    
    WHILE NOT found AND iterations < max_iterations LOOP
        iterations := iterations + 1;
        
        -- Check if current time matches all cron parts
        IF (
            -- Check minute
            (cron_parts.minute = '*' OR 
             cron_parts.minute = extract(minute from check_time)::TEXT OR
             (cron_parts.minute LIKE '*/%' AND 
              extract(minute from check_time)::INTEGER % 
              substring(cron_parts.minute from '/(\d+)')::INTEGER = 0))
            AND
            -- Check hour
            (cron_parts.hour = '*' OR 
             cron_parts.hour = extract(hour from check_time)::TEXT OR
             (cron_parts.hour LIKE '*/%' AND 
              extract(hour from check_time)::INTEGER % 
              substring(cron_parts.hour from '/(\d+)')::INTEGER = 0))
            AND
            -- Check day of month
            (cron_parts.day_of_month = '*' OR 
             cron_parts.day_of_month = extract(day from check_time)::TEXT OR
             (cron_parts.day_of_month LIKE '*/%' AND 
              extract(day from check_time)::INTEGER % 
              substring(cron_parts.day_of_month from '/(\d+)')::INTEGER = 0))
            AND
            -- Check month
            (cron_parts.month = '*' OR 
             cron_parts.month = extract(month from check_time)::TEXT OR
             (cron_parts.month LIKE '*/%' AND 
              extract(month from check_time)::INTEGER % 
              substring(cron_parts.month from '/(\d+)')::INTEGER = 0))
            AND
            -- Check day of week (0-6, where 0 is Sunday)
            (cron_parts.day_of_week = '*' OR 
             cron_parts.day_of_week = extract(dow from check_time)::TEXT OR
             (cron_parts.day_of_week LIKE '*/%' AND 
              extract(dow from check_time)::INTEGER % 
              substring(cron_parts.day_of_week from '/(\d+)')::INTEGER = 0))
        ) THEN
            found := TRUE;
            next_time := check_time;
        ELSE
            check_time := check_time + interval '1 minute';
        END IF;
    END LOOP;
    
    IF NOT found THEN
        RAISE EXCEPTION 'Could not find next run time after % iterations', max_iterations;
    END IF;
    
    RETURN next_time;
END;
$$ LANGUAGE plpgsql;

-- Create function to extract next run time from schedule
CREATE OR REPLACE FUNCTION extract_next_run_time(schedule JSONB)
RETURNS TIMESTAMP WITH TIME ZONE AS $$
DECLARE
    cron_expr TEXT;
    next_run TIMESTAMP WITH TIME ZONE;
BEGIN
    -- Extract cron expression from schedule JSON
    cron_expr := schedule->>'cron_expression';
    
    -- Calculate next run time
    SELECT calculate_next_run_time(cron_expr, CURRENT_TIMESTAMP) INTO next_run;
    
    RETURN next_run;
END;
$$ LANGUAGE plpgsql;

-- Drop existing partitions if they exist
DO $$
BEGIN
    -- Detach partitions if they exist
    IF EXISTS (
        SELECT 1 
        FROM pg_inherits i 
        JOIN pg_class c ON i.inhrelid = c.oid 
        WHERE i.inhparent = 'jobs'::regclass 
        AND c.relname = 'jobs_past'
    ) THEN
        ALTER TABLE jobs DETACH PARTITION jobs_past;
    END IF;
    
    IF EXISTS (
        SELECT 1 
        FROM pg_inherits i 
        JOIN pg_class c ON i.inhrelid = c.oid 
        WHERE i.inhparent = 'jobs'::regclass 
        AND c.relname = 'jobs_future'
    ) THEN
        ALTER TABLE jobs DETACH PARTITION jobs_future;
    END IF;
    
    -- Drop the detached partitions
    DROP TABLE IF EXISTS jobs_past CASCADE;
    DROP TABLE IF EXISTS jobs_future CASCADE;
END $$;

-- Create new partitions based on cron next run time
CREATE TABLE jobs_past PARTITION OF jobs
    FOR VALUES FROM (MINVALUE) TO (CURRENT_TIMESTAMP);

CREATE TABLE jobs_future PARTITION OF jobs
    FOR VALUES FROM (CURRENT_TIMESTAMP) TO (MAXVALUE);

-- Create trigger to update next_run_at based on cron expression
CREATE OR REPLACE FUNCTION update_next_run_at()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.schedule_type = 'recurring' AND NEW.schedule ? 'cron_expression' THEN
        NEW.next_run_at := extract_next_run_time(NEW.schedule);
    ELSIF NEW.schedule_type = 'one_time' THEN
        NEW.next_run_at := NEW.scheduled_at;
    ELSE
        -- For any other case, set next_run_at to scheduled_at to ensure it's not NULL
        NEW.next_run_at := NEW.scheduled_at;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger
CREATE TRIGGER update_jobs_next_run_at
    BEFORE INSERT OR UPDATE ON jobs
    FOR EACH ROW
    EXECUTE FUNCTION update_next_run_at();

-- Add comment to explain the partitioning strategy
COMMENT ON TABLE jobs IS 'Partitioned by next_run_at, which is calculated from cron expressions for recurring jobs'; 