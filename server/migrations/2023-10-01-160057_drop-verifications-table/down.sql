-- This file should undo anything in `up.sql`
CREATE TABLE verification_notifications (
    id UUID PRIMARY KEY,
    user_uuid UUID NOT NULL REFERENCES users (id),
    chunk_uuid UUID NOT NULL REFERENCES chunk_metadata (id),
    verification_uuid UUID NOT NULL REFERENCES chunk_verification (id),
    similarity_score BigInt NOT NULL,
    user_read boolean NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

ALTER TABLE
    chunk_verification
ADD
    CONSTRAINT uq_chunk_id UNIQUE(chunk_id);

CREATE TABLE public.chunk_verification (
    id uuid NOT NULL,
    chunk_id uuid NOT NULL,
    similarity_score bigint NOT NULL,
    created_at timestamp without time zone NOT NULL DEFAULT now(),
    updated_at timestamp without time zone NOT NULL DEFAULT now()
);

ALTER TABLE
    public.chunk_verification
ADD
    CONSTRAINT chunk_verification_pkey PRIMARY KEY (id);