package com.keydrop.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material.icons.filled.Block
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.PersonAdd
import androidx.compose.material.icons.filled.Schedule
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import com.keydrop.R
import com.keydrop.ui.viewmodel.EmergencyAccessRequest
import com.keydrop.ui.viewmodel.EmergencyAccessViewModel
import com.keydrop.ui.viewmodel.EmergencyContact
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.concurrent.TimeUnit

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun EmergencyAccessScreen(
    onNavigateBack: () -> Unit,
    viewModel: EmergencyAccessViewModel = hiltViewModel()
) {
    val uiState by viewModel.uiState.collectAsState()
    val snackbarHostState = remember { SnackbarHostState() }
    var showRemoveDialog by remember { mutableStateOf<String?>(null) }
    var showDenyDialog by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(uiState.error) {
        uiState.error?.let { error ->
            snackbarHostState.showSnackbar(error)
            viewModel.clearError()
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(stringResource(R.string.emergency_access)) },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(
                            imageVector = Icons.Default.ArrowBack,
                            contentDescription = stringResource(R.string.back)
                        )
                    }
                }
            )
        },
        floatingActionButton = {
            FloatingActionButton(onClick = viewModel::showAddContactDialog) {
                Icon(
                    imageVector = Icons.Default.PersonAdd,
                    contentDescription = stringResource(R.string.add_emergency_contact)
                )
            }
        },
        snackbarHost = { SnackbarHost(snackbarHostState) }
    ) { paddingValues ->
        if (uiState.isLoading && uiState.contacts.isEmpty() && uiState.pendingRequests.isEmpty()) {
            LoadingContent(modifier = Modifier.padding(paddingValues))
        } else {
            LazyColumn(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(paddingValues),
                contentPadding = PaddingValues(16.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                // Pending access requests section
                if (uiState.pendingRequests.isNotEmpty()) {
                    item {
                        Text(
                            text = stringResource(R.string.pending_access_requests),
                            style = MaterialTheme.typography.titleMedium,
                            modifier = Modifier.padding(bottom = 8.dp)
                        )
                    }
                    items(uiState.pendingRequests, key = { it.id }) { request ->
                        AccessRequestCard(
                            request = request,
                            onDeny = { showDenyDialog = request.id }
                        )
                    }
                }

                // Emergency contacts section
                item {
                    Text(
                        text = stringResource(R.string.your_emergency_contacts),
                        style = MaterialTheme.typography.titleMedium,
                        modifier = Modifier.padding(top = if (uiState.pendingRequests.isNotEmpty()) 16.dp else 0.dp, bottom = 8.dp)
                    )
                }

                if (uiState.contacts.isEmpty()) {
                    item {
                        EmptyContactsContent()
                    }
                } else {
                    items(uiState.contacts, key = { it.id }) { contact ->
                        EmergencyContactCard(
                            contact = contact,
                            onRemove = { showRemoveDialog = contact.id }
                        )
                    }
                }

                // Granted access section (for contacts who have been granted access to others' vaults)
                if (uiState.grantedAccess.isNotEmpty()) {
                    item {
                        Text(
                            text = stringResource(R.string.vaults_you_can_access),
                            style = MaterialTheme.typography.titleMedium,
                            modifier = Modifier.padding(top = 16.dp, bottom = 8.dp)
                        )
                    }
                    items(uiState.grantedAccess, key = { it.requestId }) { access ->
                        GrantedAccessCard(access = access)
                    }
                }
            }
        }
    }

    // Add contact dialog
    if (uiState.showAddContactDialog) {
        AddContactDialog(
            email = uiState.addContactEmail,
            name = uiState.addContactName,
            waitingPeriod = uiState.addContactWaitingPeriod,
            onEmailChange = viewModel::onAddContactEmailChange,
            onNameChange = viewModel::onAddContactNameChange,
            onWaitingPeriodChange = viewModel::onAddContactWaitingPeriodChange,
            onConfirm = viewModel::addContact,
            onDismiss = viewModel::hideAddContactDialog
        )
    }

    // Remove contact dialog
    showRemoveDialog?.let { contactId ->
        val contact = uiState.contacts.find { it.id == contactId }
        AlertDialog(
            onDismissRequest = { showRemoveDialog = null },
            title = { Text(stringResource(R.string.remove_emergency_contact)) },
            text = {
                Text(stringResource(R.string.remove_contact_message, contact?.email ?: ""))
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        viewModel.removeContact(contactId)
                        showRemoveDialog = null
                    }
                ) {
                    Text(
                        stringResource(R.string.remove),
                        color = MaterialTheme.colorScheme.error
                    )
                }
            },
            dismissButton = {
                TextButton(onClick = { showRemoveDialog = null }) {
                    Text(stringResource(R.string.cancel))
                }
            }
        )
    }

    // Deny request dialog
    showDenyDialog?.let { requestId ->
        val request = uiState.pendingRequests.find { it.id == requestId }
        AlertDialog(
            onDismissRequest = { showDenyDialog = null },
            title = { Text(stringResource(R.string.deny_access_request)) },
            text = {
                Text(stringResource(R.string.deny_request_message, request?.contactEmail ?: ""))
            },
            confirmButton = {
                TextButton(
                    onClick = {
                        viewModel.denyRequest(requestId)
                        showDenyDialog = null
                    }
                ) {
                    Text(
                        stringResource(R.string.deny),
                        color = MaterialTheme.colorScheme.error
                    )
                }
            },
            dismissButton = {
                TextButton(onClick = { showDenyDialog = null }) {
                    Text(stringResource(R.string.cancel))
                }
            }
        )
    }
}

@Composable
private fun EmergencyContactCard(
    contact: EmergencyContact,
    onRemove: () -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        elevation = CardDefaults.cardElevation(defaultElevation = 1.dp)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = contact.name ?: contact.email,
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                if (contact.name != null) {
                    Text(
                        text = contact.email,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                Spacer(modifier = Modifier.height(4.dp))
                Row(verticalAlignment = Alignment.CenterVertically) {
                    StatusBadge(status = contact.status)
                    Spacer(modifier = Modifier.width(8.dp))
                    Icon(
                        imageVector = Icons.Default.Schedule,
                        contentDescription = null,
                        modifier = Modifier.height(14.dp),
                        tint = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = "${contact.waitingPeriodHours}h waiting period",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }

            IconButton(onClick = onRemove) {
                Icon(
                    imageVector = Icons.Default.Delete,
                    contentDescription = stringResource(R.string.remove)
                )
            }
        }
    }
}

@Composable
private fun StatusBadge(status: String) {
    val (color, text) = when (status.lowercase()) {
        "pending" -> MaterialTheme.colorScheme.tertiary to stringResource(R.string.pending)
        "accepted" -> MaterialTheme.colorScheme.primary to stringResource(R.string.accepted)
        "revoked" -> MaterialTheme.colorScheme.error to stringResource(R.string.revoked)
        else -> MaterialTheme.colorScheme.outline to status
    }

    Text(
        text = text,
        style = MaterialTheme.typography.labelSmall,
        color = color
    )
}

@Composable
private fun AccessRequestCard(
    request: EmergencyAccessRequest,
    onDeny: () -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.errorContainer
        ),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = request.contactName ?: request.contactEmail,
                    style = MaterialTheme.typography.titleMedium
                )
                if (request.contactName != null) {
                    Text(
                        text = request.contactEmail,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onErrorContainer
                    )
                }
                Spacer(modifier = Modifier.height(4.dp))
                Text(
                    text = formatTimeRemaining(request.waitingPeriodEndsAt),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onErrorContainer
                )
                request.reason?.let { reason ->
                    Spacer(modifier = Modifier.height(4.dp))
                    Text(
                        text = "Reason: $reason",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onErrorContainer
                    )
                }
            }

            IconButton(onClick = onDeny) {
                Icon(
                    imageVector = Icons.Default.Block,
                    contentDescription = stringResource(R.string.deny),
                    tint = MaterialTheme.colorScheme.error
                )
            }
        }
    }
}

@Composable
private fun GrantedAccessCard(access: com.keydrop.ui.viewmodel.GrantedAccess) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.primaryContainer
        ),
        elevation = CardDefaults.cardElevation(defaultElevation = 1.dp)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            Text(
                text = access.userEmail,
                style = MaterialTheme.typography.titleMedium
            )
            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text = "Approved: ${formatDate(access.approvedAt)}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onPrimaryContainer
            )
        }
    }
}

@Composable
private fun AddContactDialog(
    email: String,
    name: String,
    waitingPeriod: Int,
    onEmailChange: (String) -> Unit,
    onNameChange: (String) -> Unit,
    onWaitingPeriodChange: (Int) -> Unit,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(stringResource(R.string.add_emergency_contact)) },
        text = {
            Column {
                OutlinedTextField(
                    value = email,
                    onValueChange = onEmailChange,
                    label = { Text(stringResource(R.string.email)) },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true
                )
                Spacer(modifier = Modifier.height(8.dp))
                OutlinedTextField(
                    value = name,
                    onValueChange = onNameChange,
                    label = { Text(stringResource(R.string.name_optional)) },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true
                )
                Spacer(modifier = Modifier.height(16.dp))
                Text(
                    text = stringResource(R.string.waiting_period),
                    style = MaterialTheme.typography.labelMedium
                )
                Text(
                    text = stringResource(R.string.waiting_period_explanation),
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                Spacer(modifier = Modifier.height(8.dp))
                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    listOf(24, 48, 72).forEach { hours ->
                        TextButton(
                            onClick = { onWaitingPeriodChange(hours) }
                        ) {
                            Text(
                                text = "${hours}h",
                                color = if (waitingPeriod == hours)
                                    MaterialTheme.colorScheme.primary
                                else
                                    MaterialTheme.colorScheme.onSurface
                            )
                        }
                    }
                }
            }
        },
        confirmButton = {
            TextButton(
                onClick = onConfirm,
                enabled = email.isNotBlank()
            ) {
                Text(stringResource(R.string.add))
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text(stringResource(R.string.cancel))
            }
        }
    )
}

@Composable
private fun LoadingContent(modifier: Modifier = Modifier) {
    Column(
        modifier = modifier.fillMaxSize(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        CircularProgressIndicator()
    }
}

@Composable
private fun EmptyContactsContent() {
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant
        )
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(24.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Icon(
                imageVector = Icons.Default.PersonAdd,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(modifier = Modifier.height(8.dp))
            Text(
                text = stringResource(R.string.no_emergency_contacts),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Spacer(modifier = Modifier.height(4.dp))
            Text(
                text = stringResource(R.string.add_trusted_contact_help),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

private fun formatTimeRemaining(endsAt: Long): String {
    val now = System.currentTimeMillis()
    val remaining = endsAt - now

    if (remaining <= 0) {
        return "Access will be granted soon"
    }

    val hours = TimeUnit.MILLISECONDS.toHours(remaining)
    val minutes = TimeUnit.MILLISECONDS.toMinutes(remaining) % 60

    return if (hours > 0) {
        "Access in ${hours}h ${minutes}m (deny to prevent)"
    } else {
        "Access in ${minutes}m (deny to prevent)"
    }
}

private fun formatDate(timestamp: Long): String {
    val formatter = SimpleDateFormat("MMM d, yyyy", Locale.getDefault())
    return formatter.format(Date(timestamp))
}
