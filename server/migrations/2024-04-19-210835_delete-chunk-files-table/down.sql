-- This file should undo anything in `up.sql`
CREATE TABLE chunk_files (
  id uuid PRIMARY KEY,
  chunk_id uuid NOT NULL,
  file_id uuid NOT NULL,
  created_at timestamp with time zone NOT NULL DEFAULT now(),
  updated_at timestamp with time zone NOT NULL DEFAULT now()
);

ALTER TABLE chunk_files
ADD CONSTRAINT chunk_files_chunk_id_fkey
FOREIGN KEY (chunk_id) REFERENCES chunk_metadata(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE chunk_files
ADD CONSTRAINT chunk_files_file_id_fkey
FOREIGN KEY (file_id) REFERENCES files(id) ON UPDATE CASCADE ON DELETE CASCADE;