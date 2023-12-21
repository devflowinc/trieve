CREATE TABLE messages (
    id UUID NOT NULL UNIQUE PRIMARY KEY,
    topic_id UUID NOT NULL REFERENCES topics(id),
    sort_order INT NOT NULL,
    content TEXT NOT NULL,
    role VARCHAR(10) NOT NULL DEFAULT 'assistant' CHECK (role IN ('system', 'user', 'assistant')),
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    prompt_tokens INT DEFAULT 0,
    completion_tokens INT DEFAULT 0,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);