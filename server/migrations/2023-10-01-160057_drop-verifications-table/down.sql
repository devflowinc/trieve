-- This file should undo anything in `up.sql`
CREATE TABLE verification_notifications (
    id UUID PRIMARY KEY,
    user_uuid UUID NOT NULL REFERENCES users (id),
    card_uuid UUID NOT NULL REFERENCES card_metadata (id),
    verification_uuid UUID NOT NULL REFERENCES card_verification (id),
    similarity_score BigInt NOT NULL,
    user_read boolean NOT NULL DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

ALTER TABLE
    card_verification
ADD
    CONSTRAINT uq_card_id UNIQUE(card_id);

CREATE TABLE public.card_verification (
    id uuid NOT NULL,
    card_id uuid NOT NULL,
    similarity_score bigint NOT NULL,
    created_at timestamp without time zone NOT NULL DEFAULT now(),
    updated_at timestamp without time zone NOT NULL DEFAULT now()
);

ALTER TABLE
    public.card_verification
ADD
    CONSTRAINT card_verification_pkey PRIMARY KEY (id);