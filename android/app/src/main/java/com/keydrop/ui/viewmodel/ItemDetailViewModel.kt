package com.keydrop.ui.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.keydrop.data.model.VaultItem
import com.keydrop.data.repository.VaultRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class ItemDetailViewModel @Inject constructor(
    private val vaultRepository: VaultRepository
) : ViewModel() {

    private val _item = MutableStateFlow<VaultItem?>(null)
    val item: StateFlow<VaultItem?> = _item.asStateFlow()

    private val _isDeleted = MutableStateFlow(false)
    val isDeleted: StateFlow<Boolean> = _isDeleted.asStateFlow()

    private var currentItemId: String? = null

    fun loadItem(id: String) {
        currentItemId = id
        viewModelScope.launch {
            _item.value = vaultRepository.getItemById(id)
        }
    }

    fun deleteItem() {
        val id = currentItemId ?: return
        viewModelScope.launch {
            vaultRepository.deleteItem(id)
            _isDeleted.value = true
        }
    }
}
