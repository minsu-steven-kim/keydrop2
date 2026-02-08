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

data class EmergencyContact(
    val id: String,
    val email: String,
    val name: String?,
    val status: String,  // pending, accepted, revoked
    val waitingPeriodHours: Int,
    val canViewVault: Boolean,
    val acceptedAt: Long?,
    val createdAt: Long
)

data class EmergencyAccessRequest(
    val id: String,
    val contactId: String,
    val contactEmail: String,
    val contactName: String?,
    val reason: String?,
    val waitingPeriodEndsAt: Long,
    val createdAt: Long
)

data class GrantedAccess(
    val contactId: String,
    val userEmail: String,
    val requestId: String,
    val approvedAt: Long
)

data class EmergencyAccessUiState(
    val contacts: List<EmergencyContact> = emptyList(),
    val pendingRequests: List<EmergencyAccessRequest> = emptyList(),
    val grantedAccess: List<GrantedAccess> = emptyList(),
    val isLoading: Boolean = false,
    val error: String? = null,
    val showAddContactDialog: Boolean = false,
    val addContactEmail: String = "",
    val addContactName: String = "",
    val addContactWaitingPeriod: Int = 48
)

interface EmergencyAccessApi {
    suspend fun getContacts(): List<EmergencyContact>
    suspend fun addContact(email: String, name: String?, waitingPeriodHours: Int): EmergencyContact
    suspend fun removeContact(contactId: String)
    suspend fun acceptInvitation(contactId: String, token: String)
    suspend fun getPendingRequests(): List<EmergencyAccessRequest>
    suspend fun denyRequest(requestId: String)
    suspend fun requestAccess(contactId: String, reason: String?)
    suspend fun getGrantedAccess(): List<GrantedAccess>
}

@HiltViewModel
class EmergencyAccessViewModel @Inject constructor(
    private val emergencyAccessApi: EmergencyAccessApi
) : ViewModel() {

    private val _uiState = MutableStateFlow(EmergencyAccessUiState())
    val uiState: StateFlow<EmergencyAccessUiState> = _uiState.asStateFlow()

    init {
        loadData()
    }

    fun loadData() {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null) }
            try {
                val contacts = emergencyAccessApi.getContacts()
                val requests = emergencyAccessApi.getPendingRequests()
                val granted = emergencyAccessApi.getGrantedAccess()

                _uiState.update {
                    it.copy(
                        contacts = contacts,
                        pendingRequests = requests,
                        grantedAccess = granted,
                        isLoading = false
                    )
                }
            } catch (e: Exception) {
                _uiState.update { it.copy(isLoading = false, error = e.message) }
            }
        }
    }

    fun showAddContactDialog() {
        _uiState.update {
            it.copy(
                showAddContactDialog = true,
                addContactEmail = "",
                addContactName = "",
                addContactWaitingPeriod = 48
            )
        }
    }

    fun hideAddContactDialog() {
        _uiState.update { it.copy(showAddContactDialog = false) }
    }

    fun onAddContactEmailChange(email: String) {
        _uiState.update { it.copy(addContactEmail = email) }
    }

    fun onAddContactNameChange(name: String) {
        _uiState.update { it.copy(addContactName = name) }
    }

    fun onAddContactWaitingPeriodChange(hours: Int) {
        _uiState.update { it.copy(addContactWaitingPeriod = hours) }
    }

    fun addContact() {
        val state = _uiState.value
        if (state.addContactEmail.isBlank()) return

        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null) }
            try {
                val contact = emergencyAccessApi.addContact(
                    email = state.addContactEmail,
                    name = state.addContactName.takeIf { it.isNotBlank() },
                    waitingPeriodHours = state.addContactWaitingPeriod
                )
                _uiState.update {
                    it.copy(
                        contacts = it.contacts + contact,
                        isLoading = false,
                        showAddContactDialog = false
                    )
                }
            } catch (e: Exception) {
                _uiState.update { it.copy(isLoading = false, error = e.message) }
            }
        }
    }

    fun removeContact(contactId: String) {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null) }
            try {
                emergencyAccessApi.removeContact(contactId)
                _uiState.update {
                    it.copy(
                        contacts = it.contacts.filter { c -> c.id != contactId },
                        isLoading = false
                    )
                }
            } catch (e: Exception) {
                _uiState.update { it.copy(isLoading = false, error = e.message) }
            }
        }
    }

    fun denyRequest(requestId: String) {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null) }
            try {
                emergencyAccessApi.denyRequest(requestId)
                _uiState.update {
                    it.copy(
                        pendingRequests = it.pendingRequests.filter { r -> r.id != requestId },
                        isLoading = false
                    )
                }
            } catch (e: Exception) {
                _uiState.update { it.copy(isLoading = false, error = e.message) }
            }
        }
    }

    fun requestAccess(contactId: String, reason: String?) {
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null) }
            try {
                emergencyAccessApi.requestAccess(contactId, reason)
                loadData()  // Reload to get updated state
            } catch (e: Exception) {
                _uiState.update { it.copy(isLoading = false, error = e.message) }
            }
        }
    }

    fun clearError() {
        _uiState.update { it.copy(error = null) }
    }
}
