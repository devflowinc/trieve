-- This file should undo anything in `up.sql`
ALTER TABLE "users" ADD COLUMN "hash" VARCHAR(255) NULL;
ALTER TABLE "users" DROP COLUMN IF EXISTS "name" ;
