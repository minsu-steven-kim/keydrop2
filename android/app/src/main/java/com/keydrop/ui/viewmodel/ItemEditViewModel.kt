package com.keydrop.ui.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.keydrop.data.model.VaultItem
import com.keydrop.data.repository.VaultRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

data class ItemEditUiState(
    val isLoading: Boolean = false,
    val isSaved: Boolean = false,
    val name: String = "",
    val username: String = "",
    val password: String = "",
    val url: String = "",
    val notes: String = "",
    val category: String? = null,
    val favorite: Boolean = false,
    val error: String? = null
) {
    val canSave: Boolean
        get() = name.isNotBlank() && username.isNotBlank() && password.isNotBlank()
}

@HiltViewModel
class ItemEditViewModel @Inject constructor(
    private val vaultRepository: VaultRepository
) : ViewModel() {

    private val _uiState = MutableStateFlow(ItemEditUiState())
    val uiState: StateFlow<ItemEditUiState> = _uiState.asStateFlow()

    private var currentItemId: String? = null

    fun loadItem(id: String) {
        currentItemId = id
        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true) }

            val item = vaultRepository.getItemById(id)
            if (item != null) {
                _uiState.update {
                    it.copy(
                        isLoading = false,
                        name = item.name,
                        username = item.username,
                        password = item.password,
                        url = item.url ?: "",
                        notes = item.notes ?: "",
                        category = item.category,
                        favorite = item.favorite
                    )
                }
            } else {
                _uiState.update {
                    it.copy(isLoading = false, error = "Item not found")
                }
            }
        }
    }

    fun onNameChange(name: String) {
        _uiState.update { it.copy(name = name) }
    }

    fun onUsernameChange(username: String) {
        _uiState.update { it.copy(username = username) }
    }

    fun onPasswordChange(password: String) {
        _uiState.update { it.copy(password = password) }
    }

    fun onUrlChange(url: String) {
        _uiState.update { it.copy(url = url) }
    }

    fun onNotesChange(notes: String) {
        _uiState.update { it.copy(notes = notes) }
    }

    fun onFavoriteChange(favorite: Boolean) {
        _uiState.update { it.copy(favorite = favorite) }
    }

    fun save() {
        val state = _uiState.value
        if (!state.canSave) return

        viewModelScope.launch {
            _uiState.update { it.copy(isLoading = true, error = null) }

            try {
                val item = VaultItem(
                    id = currentItemId ?: "",
                    name = state.name,
                    username = state.username,
                    password = state.password,
                    url = state.url.takeIf { it.isNotBlank() },
                    notes = state.notes.takeIf { it.isNotBlank() },
                    category = state.category,
                    favorite = state.favorite
                )

                if (currentItemId != null) {
                    vaultRepository.updateItem(item)
                } else {
                    vaultRepository.addItem(item)
                }

                _uiState.update { it.copy(isLoading = false, isSaved = true) }
            } catch (e: Exception) {
                _uiState.update {
                    it.copy(isLoading = false, error = e.message ?: "Save failed")
                }
            }
        }
    }
}
