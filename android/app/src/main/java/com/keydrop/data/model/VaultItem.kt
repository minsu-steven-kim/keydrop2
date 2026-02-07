package com.keydrop.data.model

data class VaultItem(
    val id: String = "",
    val name: String,
    val url: String? = null,
    val username: String,
    val password: String,
    val notes: String? = null,
    val category: String? = null,
    val favorite: Boolean = false,
    val createdAt: Long = System.currentTimeMillis(),
    val modifiedAt: Long = System.currentTimeMillis()
)

data class KeySet(
    val vaultKey: String,
    val authKey: String,
    val sharingKey: String
)

data class PasswordOptions(
    val length: Int = 20,
    val lowercase: Boolean = true,
    val uppercase: Boolean = true,
    val digits: Boolean = true,
    val symbols: Boolean = true,
    val excludeAmbiguous: Boolean = false,
    val excludeChars: String = ""
)
