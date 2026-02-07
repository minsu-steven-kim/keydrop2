package com.keydrop.biometric

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import androidx.biometric.BiometricManager as AndroidBiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.suspendCancellableCoroutine
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.inject.Inject
import javax.inject.Singleton
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException

data class StoredCredentials(
    val password: String,
    val salt: String
)

@Singleton
class BiometricManager @Inject constructor(
    @ApplicationContext private val context: Context
) {
    private val keyStore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
    private val keyAlias = "keydrop_biometric_key"
    private val prefsName = "keydrop_biometric_prefs"

    private val masterKey by lazy {
        MasterKey.Builder(context)
            .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
            .build()
    }

    private val encryptedPrefs by lazy {
        EncryptedSharedPreferences.create(
            context,
            prefsName,
            masterKey,
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
        )
    }

    /**
     * Check if biometric authentication is available on this device.
     */
    fun isBiometricAvailable(): Boolean {
        val biometricManager = AndroidBiometricManager.from(context)
        return when (biometricManager.canAuthenticate(
            AndroidBiometricManager.Authenticators.BIOMETRIC_STRONG
        )) {
            AndroidBiometricManager.BIOMETRIC_SUCCESS -> true
            else -> false
        }
    }

    /**
     * Check if there are stored credentials for biometric unlock.
     */
    fun hasStoredCredentials(): Boolean {
        return encryptedPrefs.contains("encrypted_password") &&
               encryptedPrefs.contains("salt")
    }

    /**
     * Store credentials for biometric unlock.
     * Must be called after successful password authentication.
     */
    suspend fun storeCredentials(
        activity: FragmentActivity,
        password: String,
        salt: String
    ): Boolean {
        return try {
            // Authenticate with biometric first
            val authenticated = authenticateWithBiometric(activity)
            if (!authenticated) return false

            // Generate or get the encryption key
            val key = getOrCreateKey()

            // Encrypt and store the password
            val cipher = Cipher.getInstance("AES/GCM/NoPadding")
            cipher.init(Cipher.ENCRYPT_MODE, key)

            val encryptedPassword = cipher.doFinal(password.toByteArray(Charsets.UTF_8))
            val iv = cipher.iv

            encryptedPrefs.edit()
                .putString("encrypted_password", android.util.Base64.encodeToString(encryptedPassword, android.util.Base64.NO_WRAP))
                .putString("password_iv", android.util.Base64.encodeToString(iv, android.util.Base64.NO_WRAP))
                .putString("salt", salt)
                .apply()

            true
        } catch (e: Exception) {
            false
        }
    }

    /**
     * Retrieve stored credentials after biometric authentication.
     */
    suspend fun retrieveCredentials(activity: FragmentActivity? = null): StoredCredentials? {
        if (!hasStoredCredentials()) return null

        return try {
            // If activity provided, authenticate first
            if (activity != null) {
                val authenticated = authenticateWithBiometric(activity)
                if (!authenticated) return null
            }

            val encryptedPasswordBase64 = encryptedPrefs.getString("encrypted_password", null) ?: return null
            val ivBase64 = encryptedPrefs.getString("password_iv", null) ?: return null
            val salt = encryptedPrefs.getString("salt", null) ?: return null

            val encryptedPassword = android.util.Base64.decode(encryptedPasswordBase64, android.util.Base64.NO_WRAP)
            val iv = android.util.Base64.decode(ivBase64, android.util.Base64.NO_WRAP)

            val key = keyStore.getKey(keyAlias, null) as? SecretKey ?: return null

            val cipher = Cipher.getInstance("AES/GCM/NoPadding")
            cipher.init(Cipher.DECRYPT_MODE, key, javax.crypto.spec.GCMParameterSpec(128, iv))

            val decryptedPassword = String(cipher.doFinal(encryptedPassword), Charsets.UTF_8)

            StoredCredentials(decryptedPassword, salt)
        } catch (e: Exception) {
            null
        }
    }

    /**
     * Clear stored credentials.
     */
    fun clearStoredCredentials() {
        encryptedPrefs.edit()
            .remove("encrypted_password")
            .remove("password_iv")
            .remove("salt")
            .apply()

        try {
            keyStore.deleteEntry(keyAlias)
        } catch (e: Exception) {
            // Key might not exist
        }
    }

    /**
     * Show biometric prompt and authenticate.
     */
    suspend fun authenticateWithBiometric(activity: FragmentActivity): Boolean {
        return suspendCancellableCoroutine { continuation ->
            val executor = ContextCompat.getMainExecutor(activity)

            val callback = object : BiometricPrompt.AuthenticationCallback() {
                override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                    continuation.resume(true)
                }

                override fun onAuthenticationFailed() {
                    // This is called for each failed attempt, not final failure
                }

                override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                    when (errorCode) {
                        BiometricPrompt.ERROR_NEGATIVE_BUTTON,
                        BiometricPrompt.ERROR_USER_CANCELED -> {
                            continuation.resume(false)
                        }
                        else -> {
                            continuation.resumeWithException(
                                BiometricException(errorCode, errString.toString())
                            )
                        }
                    }
                }
            }

            val promptInfo = BiometricPrompt.PromptInfo.Builder()
                .setTitle("Unlock Keydrop")
                .setSubtitle("Use your fingerprint or face to unlock")
                .setNegativeButtonText("Use Password")
                .setAllowedAuthenticators(AndroidBiometricManager.Authenticators.BIOMETRIC_STRONG)
                .build()

            val biometricPrompt = BiometricPrompt(activity, executor, callback)
            biometricPrompt.authenticate(promptInfo)

            continuation.invokeOnCancellation {
                biometricPrompt.cancelAuthentication()
            }
        }
    }

    private fun getOrCreateKey(): SecretKey {
        // Check if key already exists
        keyStore.getKey(keyAlias, null)?.let {
            return it as SecretKey
        }

        // Generate new key
        val keyGenerator = KeyGenerator.getInstance(
            KeyProperties.KEY_ALGORITHM_AES,
            "AndroidKeyStore"
        )

        val spec = KeyGenParameterSpec.Builder(
            keyAlias,
            KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
        )
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setKeySize(256)
            .setUserAuthenticationRequired(true)
            .setUserAuthenticationParameters(
                0, // Require authentication for every use
                KeyProperties.AUTH_BIOMETRIC_STRONG
            )
            .setInvalidatedByBiometricEnrollment(true)
            .build()

        keyGenerator.init(spec)
        return keyGenerator.generateKey()
    }
}

class BiometricException(
    val errorCode: Int,
    message: String
) : Exception(message)
