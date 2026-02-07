package com.keydrop.ui.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.keydrop.biometric.BiometricManager
import com.keydrop.sync.SyncManager
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import javax.inject.Inject

data class SettingsUiState(
    val biometricAvailable: Boolean = false,
    val biometricEnabled: Boolean = false,
    val autoLockTimeout: Int = 5, // minutes
    val autoLockText: String = "After 5 minutes",
    val syncEnabled: Boolean = false,
    val lastSyncTime: Long? = null,
    val lastSyncText: String = "Never synced",
    val deviceCount: Int = 1
)

@HiltViewModel
class SettingsViewModel @Inject constructor(
    private val biometricManager: BiometricManager,
    private val syncManager: SyncManager
) : ViewModel() {

    private val _uiState = MutableStateFlow(SettingsUiState())
    val uiState: StateFlow<SettingsUiState> = _uiState.asStateFlow()

    init {
        loadSettings()
    }

    private fun loadSettings() {
        viewModelScope.launch {
            val biometricAvailable = biometricManager.isBiometricAvailable()
            val biometricEnabled = biometricManager.hasStoredCredentials()

            _uiState.update {
                it.copy(
                    biometricAvailable = biometricAvailable,
                    biometricEnabled = biometricEnabled
                )
            }
        }

        viewModelScope.launch {
            syncManager.syncState.collect { state ->
                val lastSyncText = state.lastSyncTimestamp?.let { timestamp ->
                    val formatter = SimpleDateFormat("MMM d, h:mm a", Locale.getDefault())
                    "Last synced: ${formatter.format(Date(timestamp))}"
                } ?: "Never synced"

                _uiState.update {
                    it.copy(
                        syncEnabled = state.isEnabled,
                        lastSyncTime = state.lastSyncTimestamp,
                        lastSyncText = lastSyncText
                    )
                }
            }
        }
    }

    fun onBiometricEnabledChange(enabled: Boolean) {
        viewModelScope.launch {
            if (enabled) {
                // TODO: Prompt for password and enable biometric
            } else {
                biometricManager.clearStoredCredentials()
            }
            _uiState.update { it.copy(biometricEnabled = enabled) }
        }
    }

    fun onSyncEnabledChange(enabled: Boolean) {
        viewModelScope.launch {
            if (enabled) {
                syncManager.enable()
            } else {
                syncManager.disable()
            }
            _uiState.update { it.copy(syncEnabled = enabled) }
        }
    }

    fun syncNow() {
        viewModelScope.launch {
            syncManager.syncNow()
        }
    }
}
