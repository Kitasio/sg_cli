-- Add migration script here
CREATE TABLE metadata
(
    edition int PRIMARY KEY,
    data JSONB NOT NULL
);

CREATE TABLE current_stage
(
    stage SMALLINT NOT NULL DEFAULT 1
);
INSERT INTO current_stage (stage) VALUES (1);