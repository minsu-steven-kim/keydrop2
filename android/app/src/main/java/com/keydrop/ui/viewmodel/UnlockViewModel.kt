package com.keydrop.ui.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.keydrop.biometric.BiometricManager
import com.keydrop.data.repository.VaultRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

data class UnlockUiState(
    val isLoading: Boolean = false,
    val isUnlocked: Boolean = false,
    val error: String? = null,
    val biometricAvailable: Boolean = false
)

@HiltViewModel
class UnlockViewModel @Inject constructor(
    private val vaultRepository: VaultRepository,
    private val biometricManager: BiometricManager
) : ViewModel() {

    private val _uiState = MutableStateFlow(UnlockUiState())
    val uiState: StateFlow<UnlockUiState> = _uiState.asStateFlow()

    init {
        checkBiometricAvailability()
    }

    private fun checkBiometricAvailability() {
        viewModelScope.launch {
            val available = biometricManager.isBiometricAvailable() &&
                    biometricManager.hasStoredCredentials()
            _uiState.update { it.copy(biometricAvailable = available) }
        }
    }

    fun unlock(password: String) {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null) }

            try {
                // For now, use a default salt. In production, this would be retrieved
                // from local storage or the server
                val salt = getSavedSalt() ?: return@launch

                val success = vaultRepository.unlock(password, salt)

                if (success) {
                    _uiState.update { it.copy(isLoading = false, isUnlocked = true) }
                } else {
                    _uiState.update {
                        it.copy(
                            isLoading = false,
                            error = "Invalid master password"
                        )
                    }
                }
            } catch (e: Exception) {
                _uiState.update {
                    it.copy(
                        isLoading = false,
                        error = e.message ?: "Unlock failed"
                    )
                }
            }
        }
    }

    fun unlockWithBiometric() {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null) }

            try {
                val credentials = biometricManager.retrieveCredentials()
                if (credentials != null) {
                    val success = vaultRepository.unlock(credentials.password, credentials.salt)
                    if (success) {
                        _uiState.update { it.copy(isLoading = false, isUnlocked = true) }
                    } else {
                        _uiState.update {
                            it.copy(isLoading = false, error = "Biometric unlock failed")
                        }
                    }
                } else {
                    _uiState.update {
                        it.copy(isLoading = false, error = "No stored credentials")
                    }
                }
            } catch (e: Exception) {
                _uiState.update {
                    it.copy(isLoading = false, error = e.message ?: "Biometric unlock failed")
                }
            }
        }
    }

    private suspend fun getSavedSalt(): String? {
        // TODO: Retrieve saved salt from secure storage
        // For demo purposes, return a placeholder
        return "AAAAAAAAAAAAAAAAAAAAAA==" // 16 bytes of zeros in base64
    }
}
