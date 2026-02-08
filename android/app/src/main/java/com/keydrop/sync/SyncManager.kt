package com.keydrop.sync

import android.content.Context
import androidx.work.Constraints
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import com.keydrop.data.local.dao.SyncStateDao
import com.keydrop.data.local.dao.VaultItemDao
import com.keydrop.data.local.entity.SyncStateEntity
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import java.util.concurrent.TimeUnit
import javax.inject.Inject
import javax.inject.Singleton

data class SyncState(
    val isEnabled: Boolean = false,
    val isSyncing: Boolean = false,
    val lastSyncVersion: Long = 0,
    val lastSyncTimestamp: Long? = null,
    val error: String? = null,
    val pendingCommand: RemoteCommand? = null
)

@Singleton
class SyncManager @Inject constructor(
    @ApplicationContext private val context: Context,
    private val syncStateDao: SyncStateDao,
    private val vaultItemDao: VaultItemDao,
    private val syncApi: SyncApi
) {
    private val workManager = WorkManager.getInstance(context)

    private val _syncState = MutableStateFlow(SyncState())
    val syncState: StateFlow<SyncState> = _syncState.asStateFlow()

    private val _isSyncing = MutableStateFlow(false)
    val isSyncing: StateFlow<Boolean> = _isSyncing.asStateFlow()

    // Callback for handling remote commands
    private var onLockCommand: (() -> Unit)? = null
    private var onWipeCommand: (() -> Unit)? = null

    companion object {
        private const val SYNC_WORK_NAME = "keydrop_sync"
        private const val COMMAND_CHECK_WORK_NAME = "keydrop_command_check"
        private const val SYNC_INTERVAL_MINUTES = 15L
        private const val COMMAND_CHECK_INTERVAL_MINUTES = 5L
    }

    fun setCommandHandlers(onLock: () -> Unit, onWipe: () -> Unit) {
        onLockCommand = onLock
        onWipeCommand = onWipe
    }

    suspend fun initialize() {
        val savedState = syncStateDao.getSyncState()
        if (savedState != null) {
            _syncState.value = SyncState(
                isEnabled = true,
                lastSyncVersion = savedState.lastSyncVersion,
                lastSyncTimestamp = savedState.lastSyncTimestamp
            )
        }
    }

    fun enable() {
        _syncState.value = _syncState.value.copy(isEnabled = true)
        schedulePeriodicSync()
    }

    fun disable() {
        _syncState.value = _syncState.value.copy(isEnabled = false)
        workManager.cancelUniqueWork(SYNC_WORK_NAME)
    }

    fun schedulePeriodicSync() {
        val constraints = Constraints.Builder()
            .setRequiredNetworkType(NetworkType.CONNECTED)
            .build()

        val syncRequest = PeriodicWorkRequestBuilder<SyncWorker>(
            SYNC_INTERVAL_MINUTES, TimeUnit.MINUTES
        )
            .setConstraints(constraints)
            .build()

        workManager.enqueueUniquePeriodicWork(
            SYNC_WORK_NAME,
            ExistingPeriodicWorkPolicy.KEEP,
            syncRequest
        )
    }

    suspend fun checkRemoteCommands() {
        if (!_syncState.value.isEnabled) return

        try {
            val commands = syncApi.getCommands()
            for (command in commands) {
                handleRemoteCommand(command)
            }
        } catch (e: Exception) {
            // Log error but don't fail - command checking is non-critical
        }
    }

    private suspend fun handleRemoteCommand(command: RemoteCommand) {
        _syncState.value = _syncState.value.copy(pendingCommand = command)

        try {
            when (command.commandType.lowercase()) {
                "lock" -> {
                    onLockCommand?.invoke()
                    syncApi.acknowledgeCommand(command.id, true)
                }
                "wipe" -> {
                    onWipeCommand?.invoke()
                    syncApi.acknowledgeCommand(command.id, true)
                }
                else -> {
                    syncApi.acknowledgeCommand(command.id, false)
                }
            }
        } catch (e: Exception) {
            syncApi.acknowledgeCommand(command.id, false)
        } finally {
            _syncState.value = _syncState.value.copy(pendingCommand = null)
        }
    }

    suspend fun syncNow() {
        if (_isSyncing.value) return

        _isSyncing.value = true
        _syncState.value = _syncState.value.copy(isSyncing = true, error = null)

        try {
            // 1. Get current sync state
            val currentState = syncStateDao.getSyncState() ?: SyncStateEntity()
            val sinceVersion = currentState.lastSyncVersion

            // 2. Pull changes from server
            val pullResponse = syncApi.pull(sinceVersion)

            // 3. Apply server changes locally
            for (item in pullResponse.items) {
                // Decrypt and store item
                // This is handled by SyncWorker in detail
            }

            // 4. Push local changes to server
            val pendingItems = vaultItemDao.getPendingSyncItems()
            if (pendingItems.isNotEmpty()) {
                val pushRequest = SyncPushRequest(
                    baseVersion = sinceVersion,
                    items = pendingItems.map { entity ->
                        SyncItem(
                            id = entity.id,
                            encryptedData = entity.encryptedPassword, // Full encrypted blob in real impl
                            version = entity.syncVersion,
                            isDeleted = entity.isDeleted,
                            modifiedAt = entity.modifiedAt
                        )
                    }
                )

                val pushResponse = syncApi.push(pushRequest)

                // Mark items as synced
                for (item in pendingItems) {
                    vaultItemDao.markSynced(item.id, pushResponse.newVersion)
                }
            }

            // 5. Update sync state
            val newVersion = pullResponse.currentVersion
            syncStateDao.updateSyncVersion(newVersion, System.currentTimeMillis())

            _syncState.value = _syncState.value.copy(
                isSyncing = false,
                lastSyncVersion = newVersion,
                lastSyncTimestamp = System.currentTimeMillis()
            )
        } catch (e: Exception) {
            _syncState.value = _syncState.value.copy(
                isSyncing = false,
                error = e.message
            )
        } finally {
            _isSyncing.value = false
        }
    }
}

// API models
data class SyncItem(
    val id: String,
    val encryptedData: String,
    val version: Long,
    val isDeleted: Boolean,
    val modifiedAt: Long
)

data class SyncPullResponse(
    val currentVersion: Long,
    val items: List<SyncItem>,
    val hasMore: Boolean
)

data class SyncPushRequest(
    val baseVersion: Long,
    val items: List<SyncItem>
)

data class SyncPushResponse(
    val newVersion: Long,
    val hadConflicts: Boolean,
    val conflicts: List<SyncItem>
)

interface SyncApi {
    suspend fun pull(sinceVersion: Long): SyncPullResponse
    suspend fun push(request: SyncPushRequest): SyncPushResponse
    suspend fun getCommands(): List<RemoteCommand>
    suspend fun acknowledgeCommand(commandId: String, success: Boolean)
}

// Remote command models
data class RemoteCommand(
    val id: String,
    val commandType: String,  // "lock" or "wipe"
    val status: String,
    val createdAt: Long
)

// Command types
sealed class RemoteCommandType {
    object Lock : RemoteCommandType()
    object Wipe : RemoteCommandType()
}
