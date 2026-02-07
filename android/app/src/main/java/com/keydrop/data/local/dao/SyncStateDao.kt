package com.keydrop.data.local.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import com.keydrop.data.local.entity.SyncStateEntity
import kotlinx.coroutines.flow.Flow

@Dao
interface SyncStateDao {

    @Query("SELECT * FROM sync_state WHERE id = 1")
    suspend fun getSyncState(): SyncStateEntity?

    @Query("SELECT * FROM sync_state WHERE id = 1")
    fun getSyncStateFlow(): Flow<SyncStateEntity?>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun updateSyncState(state: SyncStateEntity)

    @Query("UPDATE sync_state SET last_sync_version = :version, last_sync_timestamp = :timestamp WHERE id = 1")
    suspend fun updateSyncVersion(version: Long, timestamp: Long = System.currentTimeMillis())

    @Query("UPDATE sync_state SET device_id = :deviceId WHERE id = 1")
    suspend fun updateDeviceId(deviceId: String)

    @Query("DELETE FROM sync_state")
    suspend fun clear()
}
