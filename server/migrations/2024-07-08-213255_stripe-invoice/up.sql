-- Your SQL goes here
CREATE TABLE stripe_invoices (
    id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
	org_id UUID NOT NULL,
	total INT NOT NULL,
	created_at TIMESTAMP NOT NULL,
	status TEXT NOT NULL,
	hosted_invoice_url TEXT NOT NULL,
    FOREIGN KEY (org_id) REFERENCES organizations(id) ON DELETE CASCADE
);
