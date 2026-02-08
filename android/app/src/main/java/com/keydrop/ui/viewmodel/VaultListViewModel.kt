package com.keydrop.ui.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.keydrop.data.model.VaultItem
import com.keydrop.data.repository.VaultRepository
import com.keydrop.sync.SyncManager
import com.keydrop.sync.SyncState
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class VaultListViewModel @Inject constructor(
    private val vaultRepository: VaultRepository,
    private val syncManager: SyncManager
) : ViewModel() {

    private val _searchQuery = MutableStateFlow("")
    val searchQuery: StateFlow<String> = _searchQuery.asStateFlow()

    private val _isSearchActive = MutableStateFlow(false)
    val isSearchActive: StateFlow<Boolean> = _isSearchActive.asStateFlow()

    val syncState: StateFlow<SyncState> = syncManager.syncState

    @OptIn(ExperimentalCoroutinesApi::class)
    val items: StateFlow<List<VaultItem>> = _searchQuery
        .flatMapLatest { query ->
            if (query.isBlank()) {
                vaultRepository.getAllItems()
            } else {
                vaultRepository.searchItems(query)
            }
        }
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = emptyList()
        )

    fun onSearchQueryChange(query: String) {
        _searchQuery.value = query
    }

    fun onSearchActiveChange(active: Boolean) {
        _isSearchActive.value = active
        if (!active) {
            _searchQuery.value = ""
        }
    }

    fun triggerSync() {
        viewModelScope.launch {
            syncManager.syncNow()
        }
    }
}
