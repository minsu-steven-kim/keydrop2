package com.keydrop.data.repository

import com.keydrop.crypto.CryptoManager
import com.keydrop.data.local.dao.VaultItemDao
import com.keydrop.data.local.entity.VaultItemEntity
import com.keydrop.data.model.VaultItem
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import java.util.UUID
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class VaultRepository @Inject constructor(
    private val vaultItemDao: VaultItemDao,
    private val cryptoManager: CryptoManager
) {
    private val _isUnlocked = MutableStateFlow(false)
    val isUnlocked: StateFlow<Boolean> = _isUnlocked.asStateFlow()

    private var vaultKey: String? = null

    fun getAllItems(): Flow<List<VaultItem>> {
        return vaultItemDao.getAllItems().map { entities ->
            entities.mapNotNull { entity ->
                try {
                    entity.toVaultItem(vaultKey ?: return@mapNotNull null)
                } catch (e: Exception) {
                    null
                }
            }
        }
    }

    suspend fun getItemById(id: String): VaultItem? {
        val key = vaultKey ?: return null
        return vaultItemDao.getItemById(id)?.toVaultItem(key)
    }

    fun getFavorites(): Flow<List<VaultItem>> {
        return vaultItemDao.getFavorites().map { entities ->
            entities.mapNotNull { entity ->
                try {
                    entity.toVaultItem(vaultKey ?: return@mapNotNull null)
                } catch (e: Exception) {
                    null
                }
            }
        }
    }

    fun searchItems(query: String): Flow<List<VaultItem>> {
        return vaultItemDao.searchItems(query).map { entities ->
            entities.mapNotNull { entity ->
                try {
                    entity.toVaultItem(vaultKey ?: return@mapNotNull null)
                } catch (e: Exception) {
                    null
                }
            }
        }
    }

    suspend fun findByUrl(url: String): List<VaultItem> {
        val key = vaultKey ?: return emptyList()
        val domain = extractDomain(url) ?: return emptyList()
        return vaultItemDao.findByDomain(domain).mapNotNull { entity ->
            try {
                entity.toVaultItem(key)
            } catch (e: Exception) {
                null
            }
        }
    }

    suspend fun addItem(item: VaultItem): String {
        val key = vaultKey ?: throw IllegalStateException("Vault is locked")
        val id = UUID.randomUUID().toString()
        val now = System.currentTimeMillis()

        val encryptedPassword = cryptoManager.encrypt(item.password, key)

        val entity = VaultItemEntity(
            id = id,
            name = item.name,
            url = item.url,
            username = item.username,
            encryptedPassword = encryptedPassword,
            notes = item.notes,
            category = item.category,
            favorite = item.favorite,
            createdAt = now,
            modifiedAt = now,
            pendingSync = true
        )

        vaultItemDao.insert(entity)
        return id
    }

    suspend fun updateItem(item: VaultItem) {
        val key = vaultKey ?: throw IllegalStateException("Vault is locked")
        val existing = vaultItemDao.getItemById(item.id) ?: throw IllegalArgumentException("Item not found")

        val encryptedPassword = cryptoManager.encrypt(item.password, key)

        val updated = existing.copy(
            name = item.name,
            url = item.url,
            username = item.username,
            encryptedPassword = encryptedPassword,
            notes = item.notes,
            category = item.category,
            favorite = item.favorite,
            modifiedAt = System.currentTimeMillis(),
            pendingSync = true
        )

        vaultItemDao.update(updated)
    }

    suspend fun deleteItem(id: String) {
        vaultItemDao.markDeleted(id)
    }

    suspend fun unlock(password: String, salt: String): Boolean {
        return try {
            val masterKey = cryptoManager.deriveMasterKey(password, salt)
            val keys = cryptoManager.deriveKeys(masterKey)
            vaultKey = keys.vaultKey
            _isUnlocked.value = true
            true
        } catch (e: Exception) {
            false
        }
    }

    fun lock() {
        vaultKey = null
        _isUnlocked.value = false
    }

    fun getItemCount(): Flow<Int> = vaultItemDao.getItemCount()

    private fun VaultItemEntity.toVaultItem(key: String): VaultItem {
        val decryptedPassword = cryptoManager.decrypt(encryptedPassword, key)
        return VaultItem(
            id = id,
            name = name,
            url = url,
            username = username,
            password = decryptedPassword,
            notes = notes,
            category = category,
            favorite = favorite,
            createdAt = createdAt,
            modifiedAt = modifiedAt
        )
    }

    private fun extractDomain(url: String): String? {
        return try {
            val cleaned = url.lowercase()
                .removePrefix("http://")
                .removePrefix("https://")
                .substringBefore("/")
                .substringBefore(":")
            cleaned.takeIf { it.isNotBlank() }
        } catch (e: Exception) {
            null
        }
    }
}
