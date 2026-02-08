-- Emergency Access and Remote Commands Schema

-- Emergency contacts table
-- Users can designate trusted contacts who may request emergency access
CREATE TABLE emergency_contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    contact_email VARCHAR(255) NOT NULL,
    contact_name VARCHAR(255),
    contact_user_id UUID REFERENCES users(id),  -- If contact is also a user
    status VARCHAR(50) NOT NULL DEFAULT 'pending',  -- pending, accepted, revoked
    waiting_period_hours INT NOT NULL DEFAULT 48,
    can_view_vault BOOLEAN DEFAULT true,
    invitation_token VARCHAR(255),
    invitation_expires_at TIMESTAMPTZ,
    accepted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_emergency_contacts_user_id ON emergency_contacts(user_id);
CREATE INDEX idx_emergency_contacts_contact_email ON emergency_contacts(contact_email);
CREATE INDEX idx_emergency_contacts_contact_user_id ON emergency_contacts(contact_user_id);
CREATE INDEX idx_emergency_contacts_invitation_token ON emergency_contacts(invitation_token);

-- Emergency access requests table
-- Tracks requests from emergency contacts to access a user's vault
CREATE TABLE emergency_access_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    emergency_contact_id UUID NOT NULL REFERENCES emergency_contacts(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',  -- pending, approved, denied, expired
    request_reason TEXT,
    waiting_period_ends_at TIMESTAMPTZ NOT NULL,
    approved_at TIMESTAMPTZ,
    denied_at TIMESTAMPTZ,
    vault_key_encrypted TEXT,  -- Encrypted with contact's key after approval
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_emergency_access_requests_contact_id ON emergency_access_requests(emergency_contact_id);
CREATE INDEX idx_emergency_access_requests_status ON emergency_access_requests(status);
CREATE INDEX idx_emergency_access_requests_waiting_period ON emergency_access_requests(waiting_period_ends_at);

-- Emergency access logs table
-- Audit trail for all emergency access-related actions
CREATE TABLE emergency_access_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    emergency_contact_id UUID REFERENCES emergency_contacts(id),
    action VARCHAR(100) NOT NULL,
    details JSONB,
    ip_address VARCHAR(45),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_emergency_access_logs_user_id ON emergency_access_logs(user_id);
CREATE INDEX idx_emergency_access_logs_contact_id ON emergency_access_logs(emergency_contact_id);
CREATE INDEX idx_emergency_access_logs_created_at ON emergency_access_logs(created_at);

-- Remote commands table
-- Commands for remotely locking or wiping devices
CREATE TABLE remote_commands (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    command_type VARCHAR(50) NOT NULL,  -- lock, wipe
    status VARCHAR(50) NOT NULL DEFAULT 'pending',  -- pending, delivered, executed, failed
    issued_by_device_id UUID REFERENCES devices(id),
    issued_by_emergency_contact_id UUID REFERENCES emergency_contacts(id),
    executed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_remote_commands_target_device ON remote_commands(target_device_id, status);
CREATE INDEX idx_remote_commands_user_id ON remote_commands(user_id);
CREATE INDEX idx_remote_commands_created_at ON remote_commands(created_at);
