-- Add down migration script here
DROP INDEX index_queue_on_scheduled_for;
DROP INDEX index_queue_on_status
DROP TABLE queue;
