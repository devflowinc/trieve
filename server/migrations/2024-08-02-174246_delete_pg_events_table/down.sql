-- This file should undo anything in `up.sql`
CREATE TABLE public.events (
	id uuid NOT NULL,
	created_at timestamp DEFAULT now() NOT NULL,
	updated_at timestamp DEFAULT now() NOT NULL,
	dataset_id uuid NOT NULL,
	event_type varchar(255) NOT NULL,
	event_data jsonb NOT NULL,
	CONSTRAINT file_upload_completed_notifications_pkey PRIMARY KEY (id),
	CONSTRAINT events_dataset_id_fkey FOREIGN KEY (dataset_id) REFERENCES public.datasets(id) ON DELETE CASCADE ON UPDATE CASCADE
);