-- Drop existing schema and recreate
DROP SCHEMA IF EXISTS public CASCADE;
CREATE SCHEMA public;
GRANT ALL ON SCHEMA public TO public;

-- Drop existing tables and types
DROP TABLE IF EXISTS jobs CASCADE;
DROP TABLE IF EXISTS templates CASCADE;
DROP TYPE IF EXISTS schedule_type CASCADE;
DROP TYPE IF EXISTS job_status CASCADE;
DROP TYPE IF EXISTS job_type CASCADE;
DROP FUNCTION IF EXISTS update_updated_at_column CASCADE; 