package com.keydrop.data.local

import androidx.room.Database
import androidx.room.RoomDatabase
import com.keydrop.data.local.dao.SyncStateDao
import com.keydrop.data.local.dao.VaultItemDao
import com.keydrop.data.local.entity.SyncStateEntity
import com.keydrop.data.local.entity.VaultItemEntity

@Database(
    entities = [
        VaultItemEntity::class,
        SyncStateEntity::class
    ],
    version = 1,
    exportSchema = true
)
abstract class VaultDatabase : RoomDatabase() {
    abstract fun vaultItemDao(): VaultItemDao
    abstract fun syncStateDao(): SyncStateDao

    companion object {
        const val DATABASE_NAME = "keydrop_vault"
    }
}
