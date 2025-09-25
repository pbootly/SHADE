-- Create table to store keys
CREATE TABLE IF NOT EXISTS keys (
    id TEXT PRIMARY KEY,
    public_key TEXT NOT NULL,
    private_key TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    expires_at DATETIME
);

-- Create table to store client IP addresses
CREATE TABLE IF NOT EXISTS client_ips (
    ip_address TEXT PRIMARY KEY,
    created_at DATETIME NOT NULL
);

