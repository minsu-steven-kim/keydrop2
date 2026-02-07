package com.keydrop.crypto

import com.keydrop.data.model.KeySet
import com.keydrop.data.model.PasswordOptions
import javax.inject.Inject
import javax.inject.Singleton

/**
 * CryptoManager wraps the native crypto-core library via JNI.
 *
 * In a production build, this would call into the uniffi-generated Kotlin bindings.
 * For now, we use a placeholder implementation that will be replaced when the
 * native library is built and integrated.
 */
@Singleton
class CryptoManager @Inject constructor() {

    /**
     * Generate a random salt for key derivation.
     * Returns base64-encoded salt.
     */
    fun generateSalt(): String {
        // TODO: Replace with native call
        // return CryptoCore.generateSalt()
        return generateRandomBase64(16)
    }

    /**
     * Derive master key from password and salt.
     * Uses Argon2id with OWASP parameters.
     * Returns base64-encoded master key.
     */
    fun deriveMasterKey(password: String, saltBase64: String): String {
        // TODO: Replace with native call
        // return CryptoCore.deriveMasterKey(password, saltBase64)
        // Placeholder: simple hash (NOT SECURE - replace with native impl)
        val combined = "$password:$saltBase64"
        return android.util.Base64.encodeToString(
            combined.toByteArray().copyOf(32),
            android.util.Base64.NO_WRAP
        )
    }

    /**
     * Derive encryption keys from master key.
     * Uses HKDF-SHA256.
     */
    fun deriveKeys(masterKeyBase64: String): KeySet {
        // TODO: Replace with native call
        // val keys = CryptoCore.deriveKeys(masterKeyBase64)
        // return KeySet(keys.vaultKey, keys.authKey, keys.sharingKey)

        // Placeholder implementation
        val base = masterKeyBase64.hashCode()
        return KeySet(
            vaultKey = generateDerivedKey(base, "vault"),
            authKey = generateDerivedKey(base, "auth"),
            sharingKey = generateDerivedKey(base, "sharing")
        )
    }

    /**
     * Encrypt plaintext with the given key.
     * Uses AES-256-GCM.
     * Returns base64-encoded ciphertext.
     */
    fun encrypt(plaintext: String, keyBase64: String): String {
        // TODO: Replace with native call
        // return CryptoCore.encrypt(plaintext, keyBase64)

        // Placeholder: XOR-based encryption (NOT SECURE - replace with native impl)
        val keyBytes = android.util.Base64.decode(keyBase64, android.util.Base64.NO_WRAP)
        val plaintextBytes = plaintext.toByteArray(Charsets.UTF_8)
        val nonce = ByteArray(12).also { java.security.SecureRandom().nextBytes(it) }

        val ciphertext = ByteArray(plaintextBytes.size)
        for (i in plaintextBytes.indices) {
            ciphertext[i] = (plaintextBytes[i].toInt() xor keyBytes[i % keyBytes.size].toInt()).toByte()
        }

        val combined = nonce + ciphertext
        return android.util.Base64.encodeToString(combined, android.util.Base64.NO_WRAP)
    }

    /**
     * Decrypt ciphertext with the given key.
     * Uses AES-256-GCM.
     * Returns plaintext string.
     */
    fun decrypt(encryptedBase64: String, keyBase64: String): String {
        // TODO: Replace with native call
        // return CryptoCore.decrypt(encryptedBase64, keyBase64)

        // Placeholder: XOR-based decryption (NOT SECURE - replace with native impl)
        val keyBytes = android.util.Base64.decode(keyBase64, android.util.Base64.NO_WRAP)
        val combined = android.util.Base64.decode(encryptedBase64, android.util.Base64.NO_WRAP)

        val ciphertext = combined.drop(12).toByteArray()
        val plaintext = ByteArray(ciphertext.size)
        for (i in ciphertext.indices) {
            plaintext[i] = (ciphertext[i].toInt() xor keyBytes[i % keyBytes.size].toInt()).toByte()
        }

        return String(plaintext, Charsets.UTF_8)
    }

    /**
     * Generate a random password with the given options.
     */
    fun generatePassword(options: PasswordOptions): String {
        // TODO: Replace with native call
        // return CryptoCore.generatePassword(options)

        val chars = StringBuilder()
        if (options.lowercase) chars.append("abcdefghijklmnopqrstuvwxyz")
        if (options.uppercase) chars.append("ABCDEFGHIJKLMNOPQRSTUVWXYZ")
        if (options.digits) chars.append("0123456789")
        if (options.symbols) chars.append("!@#\$%^&*()_+-=[]{}|;:,.<>?")

        if (options.excludeAmbiguous) {
            val ambiguous = "0O1lI"
            ambiguous.forEach { chars.deleteCharAt(chars.indexOf(it).takeIf { idx -> idx >= 0 } ?: return@forEach) }
        }

        options.excludeChars.forEach { char ->
            val idx = chars.indexOf(char)
            if (idx >= 0) chars.deleteCharAt(idx)
        }

        if (chars.isEmpty()) {
            chars.append("abcdefghijklmnopqrstuvwxyz")
        }

        val random = java.security.SecureRandom()
        return (1..options.length)
            .map { chars[random.nextInt(chars.length)] }
            .joinToString("")
    }

    /**
     * Generate a passphrase with the given word count and separator.
     */
    fun generatePassphrase(wordCount: Int, separator: String): String {
        // TODO: Replace with native call
        // return CryptoCore.generatePassphrase(wordCount, separator)

        // Simple word list (in production, use EFF word list from native code)
        val words = listOf(
            "apple", "banana", "cherry", "dragon", "eagle", "forest", "garden", "harbor",
            "island", "jungle", "kingdom", "lantern", "mountain", "network", "ocean", "palace",
            "quantum", "rainbow", "sunset", "thunder", "universe", "valley", "waterfall", "xenon",
            "yellow", "zenith", "anchor", "breeze", "castle", "diamond", "ember", "falcon"
        )

        val random = java.security.SecureRandom()
        return (1..wordCount)
            .map { words[random.nextInt(words.size)] }
            .joinToString(separator)
    }

    /**
     * Calculate the entropy of a password with the given options.
     */
    fun calculateEntropy(options: PasswordOptions): Double {
        var charsetSize = 0
        if (options.lowercase) charsetSize += 26
        if (options.uppercase) charsetSize += 26
        if (options.digits) charsetSize += 10
        if (options.symbols) charsetSize += 32
        if (options.excludeAmbiguous) charsetSize -= 5

        charsetSize -= options.excludeChars.length
        if (charsetSize <= 0) charsetSize = 26

        return options.length * kotlin.math.log2(charsetSize.toDouble())
    }

    private fun generateRandomBase64(length: Int): String {
        val bytes = ByteArray(length)
        java.security.SecureRandom().nextBytes(bytes)
        return android.util.Base64.encodeToString(bytes, android.util.Base64.NO_WRAP)
    }

    private fun generateDerivedKey(base: Int, purpose: String): String {
        val combined = "$base:$purpose"
        val hash = combined.hashCode().toString().padStart(32, '0').take(32)
        return android.util.Base64.encodeToString(hash.toByteArray(), android.util.Base64.NO_WRAP)
    }
}
