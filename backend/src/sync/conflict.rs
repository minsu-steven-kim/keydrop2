use super::SyncItem;

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConflictStrategy {
    /// Server wins - use server's version
    ServerWins,
    /// Client wins - use client's version
    ClientWins,
    /// Last write wins based on modified_at timestamp
    LastWriteWins,
}

/// Resolve a conflict between server and client versions
pub fn resolve_conflict(
    server_item: &SyncItem,
    client_item: &SyncItem,
    strategy: ConflictStrategy,
) -> ConflictResolution {
    match strategy {
        ConflictStrategy::ServerWins => ConflictResolution::UseServer,
        ConflictStrategy::ClientWins => ConflictResolution::UseClient,
        ConflictStrategy::LastWriteWins => {
            if client_item.modified_at > server_item.modified_at {
                ConflictResolution::UseClient
            } else {
                ConflictResolution::UseServer
            }
        }
    }
}

/// Result of conflict resolution
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConflictResolution {
    /// Use the server's version
    UseServer,
    /// Use the client's version
    UseClient,
}

/// Detect if there's a conflict between base version and current server state
pub fn has_conflict(client_base_version: i64, item_server_version: i64) -> bool {
    // Conflict if the item has been modified since client's base version
    item_server_version > client_base_version
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_item(modified_at: i64) -> SyncItem {
        SyncItem {
            id: Uuid::new_v4(),
            encrypted_data: "test".to_string(),
            version: 1,
            is_deleted: false,
            modified_at,
        }
    }

    #[test]
    fn test_last_write_wins_client_newer() {
        let server = make_item(1000);
        let client = make_item(2000);

        let result = resolve_conflict(&server, &client, ConflictStrategy::LastWriteWins);
        assert_eq!(result, ConflictResolution::UseClient);
    }

    #[test]
    fn test_last_write_wins_server_newer() {
        let server = make_item(2000);
        let client = make_item(1000);

        let result = resolve_conflict(&server, &client, ConflictStrategy::LastWriteWins);
        assert_eq!(result, ConflictResolution::UseServer);
    }

    #[test]
    fn test_server_wins_strategy() {
        let server = make_item(1000);
        let client = make_item(2000);

        let result = resolve_conflict(&server, &client, ConflictStrategy::ServerWins);
        assert_eq!(result, ConflictResolution::UseServer);
    }
}
