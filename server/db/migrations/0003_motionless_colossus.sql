ALTER TABLE "frames" ADD COLUMN "demo" boolean;--> statement-breakpoint
UPDATE "frames" SET "demo" = true WHERE ("name" = 'Welcome to Bopixi' AND "updated_by" = 'Bopixi') OR "updated_by" = 'seed';
