package com.keydrop.data.local.dao

import androidx.room.Dao
import androidx.room.Delete
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Update
import com.keydrop.data.local.entity.VaultItemEntity
import kotlinx.coroutines.flow.Flow

@Dao
interface VaultItemDao {

    @Query("SELECT * FROM vault_items WHERE is_deleted = 0 ORDER BY name ASC")
    fun getAllItems(): Flow<List<VaultItemEntity>>

    @Query("SELECT * FROM vault_items WHERE id = :id AND is_deleted = 0")
    suspend fun getItemById(id: String): VaultItemEntity?

    @Query("SELECT * FROM vault_items WHERE favorite = 1 AND is_deleted = 0 ORDER BY name ASC")
    fun getFavorites(): Flow<List<VaultItemEntity>>

    @Query("""
        SELECT * FROM vault_items
        WHERE is_deleted = 0 AND (
            name LIKE '%' || :query || '%' OR
            username LIKE '%' || :query || '%' OR
            url LIKE '%' || :query || '%'
        )
        ORDER BY name ASC
    """)
    fun searchItems(query: String): Flow<List<VaultItemEntity>>

    @Query("SELECT * FROM vault_items WHERE url LIKE '%' || :domain || '%' AND is_deleted = 0")
    suspend fun findByDomain(domain: String): List<VaultItemEntity>

    @Query("SELECT * FROM vault_items WHERE category = :category AND is_deleted = 0 ORDER BY name ASC")
    fun getByCategory(category: String): Flow<List<VaultItemEntity>>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(item: VaultItemEntity)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertAll(items: List<VaultItemEntity>)

    @Update
    suspend fun update(item: VaultItemEntity)

    @Query("UPDATE vault_items SET is_deleted = 1, pending_sync = 1, modified_at = :modifiedAt WHERE id = :id")
    suspend fun markDeleted(id: String, modifiedAt: Long = System.currentTimeMillis())

    @Delete
    suspend fun delete(item: VaultItemEntity)

    @Query("DELETE FROM vault_items WHERE id = :id")
    suspend fun deleteById(id: String)

    @Query("SELECT * FROM vault_items WHERE pending_sync = 1")
    suspend fun getPendingSyncItems(): List<VaultItemEntity>

    @Query("UPDATE vault_items SET pending_sync = 0, sync_version = :version WHERE id = :id")
    suspend fun markSynced(id: String, version: Long)

    @Query("SELECT COUNT(*) FROM vault_items WHERE is_deleted = 0")
    fun getItemCount(): Flow<Int>

    @Query("DELETE FROM vault_items")
    suspend fun deleteAll()
}
