package com.keydrop.ui.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

data class Device(
    val id: String,
    val name: String,
    val type: String,
    val lastSeenAt: Long,
    val isCurrent: Boolean
)

data class DevicesUiState(
    val devices: List<Device> = emptyList(),
    val isLoading: Boolean = false,
    val error: String? = null,
    val actionInProgress: String? = null  // Device ID that has action in progress
)

interface DevicesApi {
    suspend fun getDevices(): List<Device>
    suspend fun lockDevice(deviceId: String)
    suspend fun wipeDevice(deviceId: String)
    suspend fun deleteDevice(deviceId: String)
}

@HiltViewModel
class DevicesViewModel @Inject constructor(
    private val devicesApi: DevicesApi
) : ViewModel() {

    private val _uiState = MutableStateFlow(DevicesUiState())
    val uiState: StateFlow<DevicesUiState> = _uiState.asStateFlow()

    init {
        loadDevices()
    }

    fun loadDevices() {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null) }
            try {
                val devices = devicesApi.getDevices()
                _uiState.update { it.copy(devices = devices, isLoading = false) }
            } catch (e: Exception) {
                _uiState.update { it.copy(isLoading = false, error = e.message) }
            }
        }
    }

    fun lockDevice(deviceId: String) {
        viewModelScope.launch {
            _uiState.update { it.copy(actionInProgress = deviceId, error = null) }
            try {
                devicesApi.lockDevice(deviceId)
                _uiState.update { it.copy(actionInProgress = null) }
            } catch (e: Exception) {
                _uiState.update { it.copy(actionInProgress = null, error = e.message) }
            }
        }
    }

    fun wipeDevice(deviceId: String) {
        viewModelScope.launch {
            _uiState.update { it.copy(actionInProgress = deviceId, error = null) }
            try {
                devicesApi.wipeDevice(deviceId)
                // After wipe, reload devices list
                loadDevices()
            } catch (e: Exception) {
                _uiState.update { it.copy(actionInProgress = null, error = e.message) }
            }
        }
    }

    fun deleteDevice(deviceId: String) {
        viewModelScope.launch {
            _uiState.update { it.copy(actionInProgress = deviceId, error = null) }
            try {
                devicesApi.deleteDevice(deviceId)
                // Remove from list
                _uiState.update { state ->
                    state.copy(
                        devices = state.devices.filter { it.id != deviceId },
                        actionInProgress = null
                    )
                }
            } catch (e: Exception) {
                _uiState.update { it.copy(actionInProgress = null, error = e.message) }
            }
        }
    }

    fun clearError() {
        _uiState.update { it.copy(error = null) }
    }
}
