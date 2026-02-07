package com.keydrop.autofill

import android.app.assist.AssistStructure
import android.os.Build
import android.os.CancellationSignal
import android.service.autofill.AutofillService
import android.service.autofill.Dataset
import android.service.autofill.FillCallback
import android.service.autofill.FillRequest
import android.service.autofill.FillResponse
import android.service.autofill.SaveCallback
import android.service.autofill.SaveInfo
import android.service.autofill.SaveRequest
import android.view.View
import android.view.autofill.AutofillId
import android.view.autofill.AutofillValue
import android.widget.RemoteViews
import com.keydrop.R
import com.keydrop.data.repository.VaultRepository
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import javax.inject.Inject

@AndroidEntryPoint
class KeydropAutofillService : AutofillService() {

    @Inject
    lateinit var vaultRepository: VaultRepository

    private val serviceScope = CoroutineScope(SupervisorJob() + Dispatchers.Main)

    override fun onFillRequest(
        request: FillRequest,
        cancellationSignal: CancellationSignal,
        callback: FillCallback
    ) {
        // Check if vault is unlocked
        if (!vaultRepository.isUnlocked.value) {
            // Return authentication request
            callback.onSuccess(null)
            return
        }

        val structure = request.fillContexts.lastOrNull()?.structure ?: run {
            callback.onSuccess(null)
            return
        }

        // Parse the structure to find autofill fields
        val parsedFields = parseStructure(structure)

        if (parsedFields.usernameId == null && parsedFields.passwordId == null) {
            callback.onSuccess(null)
            return
        }

        // Find matching credentials
        val webDomain = parsedFields.webDomain
        serviceScope.launch {
            try {
                val matches = if (webDomain != null) {
                    vaultRepository.findByUrl(webDomain)
                } else {
                    emptyList()
                }

                if (matches.isEmpty()) {
                    callback.onSuccess(null)
                    return@launch
                }

                val responseBuilder = FillResponse.Builder()

                for (item in matches.take(5)) { // Limit to 5 suggestions
                    val presentation = RemoteViews(packageName, R.layout.autofill_item).apply {
                        setTextViewText(R.id.autofill_item_title, item.name)
                        setTextViewText(R.id.autofill_item_subtitle, item.username)
                    }

                    val datasetBuilder = Dataset.Builder()

                    parsedFields.usernameId?.let { id ->
                        datasetBuilder.setValue(
                            id,
                            AutofillValue.forText(item.username),
                            presentation
                        )
                    }

                    parsedFields.passwordId?.let { id ->
                        datasetBuilder.setValue(
                            id,
                            AutofillValue.forText(item.password),
                            presentation
                        )
                    }

                    responseBuilder.addDataset(datasetBuilder.build())
                }

                // Add save info for capturing new credentials
                val saveInfoBuilder = SaveInfo.Builder(
                    SaveInfo.SAVE_DATA_TYPE_USERNAME or SaveInfo.SAVE_DATA_TYPE_PASSWORD,
                    listOfNotNull(parsedFields.usernameId, parsedFields.passwordId).toTypedArray()
                )

                responseBuilder.setSaveInfo(saveInfoBuilder.build())

                callback.onSuccess(responseBuilder.build())
            } catch (e: Exception) {
                callback.onFailure(e.message)
            }
        }
    }

    override fun onSaveRequest(request: SaveRequest, callback: SaveCallback) {
        val structure = request.fillContexts.lastOrNull()?.structure ?: run {
            callback.onSuccess()
            return
        }

        val parsedFields = parseStructure(structure)
        val username = parsedFields.usernameValue
        val password = parsedFields.passwordValue
        val webDomain = parsedFields.webDomain

        if (username.isNullOrBlank() || password.isNullOrBlank()) {
            callback.onSuccess()
            return
        }

        // TODO: Show save credential dialog
        // For now, just acknowledge the save request
        callback.onSuccess()
    }

    override fun onDestroy() {
        super.onDestroy()
        serviceScope.cancel()
    }

    private fun parseStructure(structure: AssistStructure): ParsedFields {
        val result = ParsedFields()

        for (i in 0 until structure.windowNodeCount) {
            val windowNode = structure.getWindowNodeAt(i)
            parseNode(windowNode.rootViewNode, result)
        }

        return result
    }

    private fun parseNode(node: AssistStructure.ViewNode?, result: ParsedFields) {
        if (node == null) return

        // Check for web domain
        if (result.webDomain == null) {
            node.webDomain?.let { result.webDomain = it }
        }

        // Check autofill hints
        val autofillHints = node.autofillHints
        val autofillId = node.autofillId

        if (autofillHints != null && autofillId != null) {
            for (hint in autofillHints) {
                when (hint) {
                    View.AUTOFILL_HINT_USERNAME,
                    View.AUTOFILL_HINT_EMAIL_ADDRESS -> {
                        result.usernameId = autofillId
                        result.usernameValue = node.autofillValue?.textValue?.toString()
                    }
                    View.AUTOFILL_HINT_PASSWORD -> {
                        result.passwordId = autofillId
                        result.passwordValue = node.autofillValue?.textValue?.toString()
                    }
                }
            }
        }

        // Check input type as fallback
        if (autofillId != null && result.usernameId == null && result.passwordId == null) {
            val inputType = node.inputType
            val className = node.className

            if (className?.contains("EditText") == true) {
                when {
                    inputType and android.text.InputType.TYPE_TEXT_VARIATION_PASSWORD != 0 ||
                    inputType and android.text.InputType.TYPE_TEXT_VARIATION_WEB_PASSWORD != 0 -> {
                        result.passwordId = autofillId
                        result.passwordValue = node.autofillValue?.textValue?.toString()
                    }
                    inputType and android.text.InputType.TYPE_TEXT_VARIATION_EMAIL_ADDRESS != 0 ||
                    inputType and android.text.InputType.TYPE_TEXT_VARIATION_WEB_EMAIL_ADDRESS != 0 -> {
                        result.usernameId = autofillId
                        result.usernameValue = node.autofillValue?.textValue?.toString()
                    }
                }
            }
        }

        // Recursively parse children
        for (i in 0 until node.childCount) {
            parseNode(node.getChildAt(i), result)
        }
    }

    private data class ParsedFields(
        var webDomain: String? = null,
        var usernameId: AutofillId? = null,
        var usernameValue: String? = null,
        var passwordId: AutofillId? = null,
        var passwordValue: String? = null
    )
}
