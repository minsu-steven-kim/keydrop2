package com.keydrop.data.local.entity

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "vault_items")
data class VaultItemEntity(
    @PrimaryKey
    val id: String,

    val name: String,

    val url: String?,

    val username: String,

    @ColumnInfo(name = "encrypted_password")
    val encryptedPassword: String,

    val notes: String?,

    val category: String?,

    val favorite: Boolean = false,

    @ColumnInfo(name = "created_at")
    val createdAt: Long,

    @ColumnInfo(name = "modified_at")
    val modifiedAt: Long,

    @ColumnInfo(name = "sync_version")
    val syncVersion: Long = 0,

    @ColumnInfo(name = "is_deleted")
    val isDeleted: Boolean = false,

    @ColumnInfo(name = "pending_sync")
    val pendingSync: Boolean = false
)
