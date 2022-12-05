-- Add up migration script here
CREATE TABLE queue (
    id UUID PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL,
    scheduled_for TIMESTAMP WITH TIME ZONE NOT NULL,
    status INT NOT NULL,
    task_type INT NOT NULL
);
CREATE INDEX index_queue_on_scheduled_for ON queue (scheduled_for);
CREATE INDEX index_queue_on_status ON queue (status);
