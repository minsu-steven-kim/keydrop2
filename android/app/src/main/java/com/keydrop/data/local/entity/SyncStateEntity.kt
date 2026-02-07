package com.keydrop.data.local.entity

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "sync_state")
data class SyncStateEntity(
    @PrimaryKey
    val id: Int = 1, // Single row table

    @ColumnInfo(name = "last_sync_version")
    val lastSyncVersion: Long = 0,

    @ColumnInfo(name = "last_sync_timestamp")
    val lastSyncTimestamp: Long = 0,

    @ColumnInfo(name = "device_id")
    val deviceId: String? = null
)
