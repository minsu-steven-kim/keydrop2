package com.keydrop.ui.screens

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Checkbox
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.keydrop.R
import com.keydrop.ui.viewmodel.PasswordGeneratorViewModel
import kotlin.math.roundToInt

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PasswordGeneratorScreen(
    onBack: () -> Unit,
    onUsePassword: (String) -> Unit,
    viewModel: PasswordGeneratorViewModel = hiltViewModel()
) {
    val uiState by viewModel.uiState.collectAsState()
    val clipboardManager = LocalClipboardManager.current

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.generate_password)) },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(
                            imageVector = Icons.Default.ArrowBack,
                            contentDescription = "Back"
                        )
                    }
                }
            )
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .padding(16.dp)
                .verticalScroll(rememberScrollState())
        ) {
            // Generated password display
            Card(
                modifier = Modifier.fillMaxWidth(),
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.primaryContainer
                )
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = uiState.generatedPassword,
                        style = MaterialTheme.typography.titleMedium,
                        fontFamily = FontFamily.Monospace,
                        modifier = Modifier.weight(1f)
                    )

                    IconButton(
                        onClick = {
                            clipboardManager.setText(AnnotatedString(uiState.generatedPassword))
                        }
                    ) {
                        Icon(
                            imageVector = Icons.Default.ContentCopy,
                            contentDescription = stringResource(R.string.copy_password)
                        )
                    }

                    IconButton(onClick = viewModel::regenerate) {
                        Icon(
                            imageVector = Icons.Default.Refresh,
                            contentDescription = stringResource(R.string.regenerate)
                        )
                    }
                }
            }

            Spacer(modifier = Modifier.height(8.dp))

            // Entropy indicator
            Text(
                text = "Strength: ${uiState.entropy.roundToInt()} bits",
                style = MaterialTheme.typography.bodySmall,
                color = when {
                    uiState.entropy < 40 -> MaterialTheme.colorScheme.error
                    uiState.entropy < 60 -> MaterialTheme.colorScheme.tertiary
                    else -> MaterialTheme.colorScheme.primary
                }
            )

            Spacer(modifier = Modifier.height(24.dp))

            // Length slider
            Text(
                text = "${stringResource(R.string.password_length)}: ${uiState.length}",
                style = MaterialTheme.typography.titleSmall
            )

            Slider(
                value = uiState.length.toFloat(),
                onValueChange = { viewModel.onLengthChange(it.roundToInt()) },
                valueRange = 8f..64f,
                steps = 55,
                modifier = Modifier.fillMaxWidth()
            )

            Spacer(modifier = Modifier.height(16.dp))

            // Character options
            OptionRow(
                label = stringResource(R.string.include_lowercase),
                checked = uiState.lowercase,
                onCheckedChange = viewModel::onLowercaseChange
            )

            OptionRow(
                label = stringResource(R.string.include_uppercase),
                checked = uiState.uppercase,
                onCheckedChange = viewModel::onUppercaseChange
            )

            OptionRow(
                label = stringResource(R.string.include_digits),
                checked = uiState.digits,
                onCheckedChange = viewModel::onDigitsChange
            )

            OptionRow(
                label = stringResource(R.string.include_symbols),
                checked = uiState.symbols,
                onCheckedChange = viewModel::onSymbolsChange
            )

            OptionRow(
                label = stringResource(R.string.exclude_ambiguous),
                checked = uiState.excludeAmbiguous,
                onCheckedChange = viewModel::onExcludeAmbiguousChange
            )

            Spacer(modifier = Modifier.height(32.dp))

            // Use password button
            Button(
                onClick = { onUsePassword(uiState.generatedPassword) },
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(stringResource(R.string.use_password))
            }
        }
    }
}

@Composable
private fun OptionRow(
    label: String,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp)
    ) {
        Checkbox(
            checked = checked,
            onCheckedChange = onCheckedChange
        )
        Text(
            text = label,
            style = MaterialTheme.typography.bodyLarge
        )
    }
}
