package com.keydrop.ui.viewmodel

import androidx.lifecycle.ViewModel
import com.keydrop.crypto.CryptoManager
import com.keydrop.data.model.PasswordOptions
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import javax.inject.Inject

data class PasswordGeneratorUiState(
    val generatedPassword: String = "",
    val length: Int = 20,
    val lowercase: Boolean = true,
    val uppercase: Boolean = true,
    val digits: Boolean = true,
    val symbols: Boolean = true,
    val excludeAmbiguous: Boolean = false,
    val entropy: Double = 0.0
)

@HiltViewModel
class PasswordGeneratorViewModel @Inject constructor(
    private val cryptoManager: CryptoManager
) : ViewModel() {

    private val _uiState = MutableStateFlow(PasswordGeneratorUiState())
    val uiState: StateFlow<PasswordGeneratorUiState> = _uiState.asStateFlow()

    init {
        regenerate()
    }

    fun regenerate() {
        val options = currentOptions()
        val password = cryptoManager.generatePassword(options)
        val entropy = cryptoManager.calculateEntropy(options)

        _uiState.update {
            it.copy(
                generatedPassword = password,
                entropy = entropy
            )
        }
    }

    fun onLengthChange(length: Int) {
        _uiState.update { it.copy(length = length) }
        regenerate()
    }

    fun onLowercaseChange(enabled: Boolean) {
        _uiState.update { it.copy(lowercase = enabled) }
        ensureAtLeastOneOption()
        regenerate()
    }

    fun onUppercaseChange(enabled: Boolean) {
        _uiState.update { it.copy(uppercase = enabled) }
        ensureAtLeastOneOption()
        regenerate()
    }

    fun onDigitsChange(enabled: Boolean) {
        _uiState.update { it.copy(digits = enabled) }
        ensureAtLeastOneOption()
        regenerate()
    }

    fun onSymbolsChange(enabled: Boolean) {
        _uiState.update { it.copy(symbols = enabled) }
        ensureAtLeastOneOption()
        regenerate()
    }

    fun onExcludeAmbiguousChange(enabled: Boolean) {
        _uiState.update { it.copy(excludeAmbiguous = enabled) }
        regenerate()
    }

    private fun ensureAtLeastOneOption() {
        val state = _uiState.value
        if (!state.lowercase && !state.uppercase && !state.digits && !state.symbols) {
            _uiState.update { it.copy(lowercase = true) }
        }
    }

    private fun currentOptions(): PasswordOptions {
        val state = _uiState.value
        return PasswordOptions(
            length = state.length,
            lowercase = state.lowercase,
            uppercase = state.uppercase,
            digits = state.digits,
            symbols = state.symbols,
            excludeAmbiguous = state.excludeAmbiguous
        )
    }
}
