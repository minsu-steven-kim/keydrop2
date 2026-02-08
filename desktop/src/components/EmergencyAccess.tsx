import { useState, useEffect } from 'react';

interface EmergencyContact {
  id: string;
  email: string;
  name: string | null;
  status: 'pending' | 'accepted' | 'revoked';
  waitingPeriodHours: number;
  canViewVault: boolean;
  acceptedAt: number | null;
  createdAt: number;
}

interface EmergencyAccessRequest {
  id: string;
  contactId: string;
  contactEmail: string;
  contactName: string | null;
  reason: string | null;
  waitingPeriodEndsAt: number;
  createdAt: number;
}

interface GrantedAccess {
  contactId: string;
  userEmail: string;
  requestId: string;
  approvedAt: number;
}

interface EmergencyAccessProps {
  onClose: () => void;
}

// Mock API functions
async function fetchContacts(): Promise<EmergencyContact[]> {
  return [
    {
      id: '1',
      email: 'trusted@example.com',
      name: 'Trusted Person',
      status: 'accepted',
      waitingPeriodHours: 48,
      canViewVault: true,
      acceptedAt: Date.now() - 86400000,
      createdAt: Date.now() - 172800000,
    },
  ];
}

async function fetchPendingRequests(): Promise<EmergencyAccessRequest[]> {
  return [];
}

async function fetchGrantedAccess(): Promise<GrantedAccess[]> {
  return [];
}

async function addContact(email: string, name: string | null, waitingPeriodHours: number): Promise<EmergencyContact> {
  return {
    id: String(Date.now()),
    email,
    name,
    status: 'pending',
    waitingPeriodHours,
    canViewVault: true,
    acceptedAt: null,
    createdAt: Date.now(),
  };
}

async function removeContact(contactId: string): Promise<void> {
  console.log('Removing contact:', contactId);
}

async function denyRequest(requestId: string): Promise<void> {
  console.log('Denying request:', requestId);
}

function formatTimeRemaining(endsAt: number): string {
  const now = Date.now();
  const remaining = endsAt - now;

  if (remaining <= 0) {
    return 'Access will be granted soon';
  }

  const hours = Math.floor(remaining / 3600000);
  const minutes = Math.floor((remaining % 3600000) / 60000);

  if (hours > 0) {
    return `Access in ${hours}h ${minutes}m`;
  }
  return `Access in ${minutes}m`;
}

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleDateString();
}

export default function EmergencyAccess({ onClose }: EmergencyAccessProps) {
  const [contacts, setContacts] = useState<EmergencyContact[]>([]);
  const [pendingRequests, setPendingRequests] = useState<EmergencyAccessRequest[]>([]);
  const [grantedAccess, setGrantedAccess] = useState<GrantedAccess[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [addEmail, setAddEmail] = useState('');
  const [addName, setAddName] = useState('');
  const [addWaitingPeriod, setAddWaitingPeriod] = useState(48);
  const [confirmRemove, setConfirmRemove] = useState<string | null>(null);
  const [confirmDeny, setConfirmDeny] = useState<string | null>(null);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setLoading(true);
    try {
      const [contactsData, requestsData, accessData] = await Promise.all([
        fetchContacts(),
        fetchPendingRequests(),
        fetchGrantedAccess(),
      ]);
      setContacts(contactsData);
      setPendingRequests(requestsData);
      setGrantedAccess(accessData);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleAddContact = async () => {
    if (!addEmail.trim()) return;

    try {
      const contact = await addContact(addEmail, addName || null, addWaitingPeriod);
      setContacts([...contacts, contact]);
      setShowAddDialog(false);
      setAddEmail('');
      setAddName('');
      setAddWaitingPeriod(48);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleRemoveContact = async (contactId: string) => {
    try {
      await removeContact(contactId);
      setContacts(contacts.filter(c => c.id !== contactId));
      setConfirmRemove(null);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleDenyRequest = async (requestId: string) => {
    try {
      await denyRequest(requestId);
      setPendingRequests(pendingRequests.filter(r => r.id !== requestId));
      setConfirmDeny(null);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content emergency-access" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Emergency Access</h2>
          <button className="btn btn-ghost" onClick={onClose}>×</button>
        </div>

        <div className="modal-body">
          {loading ? (
            <div className="loading-small">
              <div className="spinner" />
            </div>
          ) : error ? (
            <div className="error-message">
              {error}
              <button className="btn btn-ghost" onClick={() => setError(null)}>×</button>
            </div>
          ) : (
            <>
              {/* Pending Access Requests */}
              {pendingRequests.length > 0 && (
                <section className="section">
                  <h3 className="section-title warning">Pending Access Requests</h3>
                  <div className="request-list">
                    {pendingRequests.map(request => (
                      <div key={request.id} className="request-item warning">
                        <div className="request-info">
                          <div className="request-contact">
                            {request.contactName || request.contactEmail}
                          </div>
                          <div className="request-time">
                            {formatTimeRemaining(request.waitingPeriodEndsAt)}
                          </div>
                          {request.reason && (
                            <div className="request-reason">
                              Reason: {request.reason}
                            </div>
                          )}
                        </div>
                        <button
                          className="btn btn-danger"
                          onClick={() => setConfirmDeny(request.id)}
                        >
                          Deny
                        </button>
                      </div>
                    ))}
                  </div>
                </section>
              )}

              {/* Your Emergency Contacts */}
              <section className="section">
                <div className="section-header">
                  <h3 className="section-title">Your Emergency Contacts</h3>
                  <button className="btn btn-primary" onClick={() => setShowAddDialog(true)}>
                    + Add Contact
                  </button>
                </div>

                {contacts.length === 0 ? (
                  <div className="empty-state">
                    <p>No emergency contacts configured</p>
                    <p className="hint">
                      Add trusted contacts who can request access to your vault in case of emergency.
                    </p>
                  </div>
                ) : (
                  <div className="contact-list">
                    {contacts.map(contact => (
                      <div key={contact.id} className="contact-item">
                        <div className="contact-info">
                          <div className="contact-name">
                            {contact.name || contact.email}
                          </div>
                          {contact.name && (
                            <div className="contact-email">{contact.email}</div>
                          )}
                          <div className="contact-meta">
                            <span className={`status-badge ${contact.status}`}>
                              {contact.status}
                            </span>
                            <span>{contact.waitingPeriodHours}h waiting period</span>
                          </div>
                        </div>
                        <button
                          className="btn btn-icon"
                          title="Remove contact"
                          onClick={() => setConfirmRemove(contact.id)}
                        >
                          ✕
                        </button>
                      </div>
                    ))}
                  </div>
                )}
              </section>

              {/* Vaults You Can Access */}
              {grantedAccess.length > 0 && (
                <section className="section">
                  <h3 className="section-title">Vaults You Can Access</h3>
                  <div className="access-list">
                    {grantedAccess.map(access => (
                      <div key={access.requestId} className="access-item">
                        <div className="access-email">{access.userEmail}</div>
                        <div className="access-date">
                          Approved: {formatDate(access.approvedAt)}
                        </div>
                      </div>
                    ))}
                  </div>
                </section>
              )}
            </>
          )}
        </div>

        {/* Add Contact Dialog */}
        {showAddDialog && (
          <div className="confirm-overlay">
            <div className="confirm-dialog">
              <h3>Add Emergency Contact</h3>
              <div className="form-group">
                <label htmlFor="contact-email">Email</label>
                <input
                  id="contact-email"
                  type="email"
                  value={addEmail}
                  onChange={e => setAddEmail(e.target.value)}
                  placeholder="contact@example.com"
                />
              </div>
              <div className="form-group">
                <label htmlFor="contact-name">Name (optional)</label>
                <input
                  id="contact-name"
                  type="text"
                  value={addName}
                  onChange={e => setAddName(e.target.value)}
                  placeholder="Contact name"
                />
              </div>
              <div className="form-group">
                <label>Waiting Period</label>
                <p className="hint">
                  Time you have to deny an access request before it's automatically approved.
                </p>
                <div className="waiting-period-options">
                  {[24, 48, 72].map(hours => (
                    <button
                      key={hours}
                      className={`btn ${addWaitingPeriod === hours ? 'btn-primary' : 'btn-secondary'}`}
                      onClick={() => setAddWaitingPeriod(hours)}
                    >
                      {hours}h
                    </button>
                  ))}
                </div>
              </div>
              <div className="confirm-actions">
                <button className="btn btn-secondary" onClick={() => setShowAddDialog(false)}>
                  Cancel
                </button>
                <button
                  className="btn btn-primary"
                  onClick={handleAddContact}
                  disabled={!addEmail.trim()}
                >
                  Add Contact
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Remove Confirmation */}
        {confirmRemove && (
          <div className="confirm-overlay">
            <div className="confirm-dialog">
              <h3>Remove Emergency Contact?</h3>
              <p>
                This contact will no longer be able to request emergency access to your vault.
              </p>
              <div className="confirm-actions">
                <button className="btn btn-secondary" onClick={() => setConfirmRemove(null)}>
                  Cancel
                </button>
                <button
                  className="btn btn-danger"
                  onClick={() => handleRemoveContact(confirmRemove)}
                >
                  Remove
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Deny Confirmation */}
        {confirmDeny && (
          <div className="confirm-overlay">
            <div className="confirm-dialog">
              <h3>Deny Access Request?</h3>
              <p>
                The emergency contact will be notified that their request was denied.
              </p>
              <div className="confirm-actions">
                <button className="btn btn-secondary" onClick={() => setConfirmDeny(null)}>
                  Cancel
                </button>
                <button
                  className="btn btn-danger"
                  onClick={() => handleDenyRequest(confirmDeny)}
                >
                  Deny Request
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
