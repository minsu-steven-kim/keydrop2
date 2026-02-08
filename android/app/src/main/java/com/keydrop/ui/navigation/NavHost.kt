package com.keydrop.ui.navigation

import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import com.keydrop.ui.screens.DevicesScreen
import com.keydrop.ui.screens.EmergencyAccessScreen
import com.keydrop.ui.screens.ItemDetailScreen
import com.keydrop.ui.screens.ItemEditScreen
import com.keydrop.ui.screens.PasswordGeneratorScreen
import com.keydrop.ui.screens.SettingsScreen
import com.keydrop.ui.screens.UnlockScreen
import com.keydrop.ui.screens.VaultListScreen
import com.keydrop.ui.viewmodel.AppViewModel

sealed class Screen(val route: String) {
    object Unlock : Screen("unlock")
    object VaultList : Screen("vault")
    object ItemDetail : Screen("item/{itemId}") {
        fun createRoute(itemId: String) = "item/$itemId"
    }
    object ItemEdit : Screen("item/{itemId}/edit") {
        fun createRoute(itemId: String) = "item/$itemId/edit"
    }
    object ItemCreate : Screen("item/new")
    object PasswordGenerator : Screen("password-generator")
    object Settings : Screen("settings")
    object Devices : Screen("devices")
    object EmergencyAccess : Screen("emergency-access")
}

@Composable
fun KeydropNavHost() {
    val navController = rememberNavController()
    val appViewModel: AppViewModel = hiltViewModel()
    val isUnlocked by appViewModel.isUnlocked.collectAsState()

    val startDestination = if (isUnlocked) Screen.VaultList.route else Screen.Unlock.route

    NavHost(
        navController = navController,
        startDestination = startDestination
    ) {
        composable(Screen.Unlock.route) {
            UnlockScreen(
                onUnlockSuccess = {
                    navController.navigate(Screen.VaultList.route) {
                        popUpTo(Screen.Unlock.route) { inclusive = true }
                    }
                }
            )
        }

        composable(Screen.VaultList.route) {
            VaultListScreen(
                onItemClick = { itemId ->
                    navController.navigate(Screen.ItemDetail.createRoute(itemId))
                },
                onAddClick = {
                    navController.navigate(Screen.ItemCreate.route)
                },
                onSettingsClick = {
                    navController.navigate(Screen.Settings.route)
                },
                onLocked = {
                    navController.navigate(Screen.Unlock.route) {
                        popUpTo(0) { inclusive = true }
                    }
                }
            )
        }

        composable(
            route = Screen.ItemDetail.route,
            arguments = listOf(navArgument("itemId") { type = NavType.StringType })
        ) { backStackEntry ->
            val itemId = backStackEntry.arguments?.getString("itemId") ?: return@composable
            ItemDetailScreen(
                itemId = itemId,
                onBack = { navController.popBackStack() },
                onEdit = { navController.navigate(Screen.ItemEdit.createRoute(itemId)) },
                onDeleted = { navController.popBackStack() }
            )
        }

        composable(
            route = Screen.ItemEdit.route,
            arguments = listOf(navArgument("itemId") { type = NavType.StringType })
        ) { backStackEntry ->
            val itemId = backStackEntry.arguments?.getString("itemId") ?: return@composable
            ItemEditScreen(
                itemId = itemId,
                onBack = { navController.popBackStack() },
                onSaved = { navController.popBackStack() },
                onGeneratePassword = {
                    navController.navigate(Screen.PasswordGenerator.route)
                }
            )
        }

        composable(Screen.ItemCreate.route) {
            ItemEditScreen(
                itemId = null,
                onBack = { navController.popBackStack() },
                onSaved = { navController.popBackStack() },
                onGeneratePassword = {
                    navController.navigate(Screen.PasswordGenerator.route)
                }
            )
        }

        composable(Screen.PasswordGenerator.route) {
            PasswordGeneratorScreen(
                onBack = { navController.popBackStack() },
                onUsePassword = { password ->
                    // Return the generated password to the previous screen
                    navController.previousBackStackEntry?.savedStateHandle?.set("generatedPassword", password)
                    navController.popBackStack()
                }
            )
        }

        composable(Screen.Settings.route) {
            SettingsScreen(
                onBack = { navController.popBackStack() },
                onLogout = {
                    appViewModel.lock()
                    navController.navigate(Screen.Unlock.route) {
                        popUpTo(0) { inclusive = true }
                    }
                },
                onDevicesClick = {
                    navController.navigate(Screen.Devices.route)
                },
                onEmergencyAccessClick = {
                    navController.navigate(Screen.EmergencyAccess.route)
                }
            )
        }

        composable(Screen.Devices.route) {
            DevicesScreen(
                onNavigateBack = { navController.popBackStack() }
            )
        }

        composable(Screen.EmergencyAccess.route) {
            EmergencyAccessScreen(
                onNavigateBack = { navController.popBackStack() }
            )
        }
    }
}
