package com.keydrop.sync

import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.Header
import retrofit2.http.POST
import retrofit2.http.Query
import javax.inject.Inject
import javax.inject.Singleton

interface SyncApiService {
    @GET("api/v1/sync/pull")
    suspend fun pull(
        @Header("Authorization") token: String,
        @Query("since_version") sinceVersion: Long
    ): SyncPullResponse

    @POST("api/v1/sync/push")
    suspend fun push(
        @Header("Authorization") token: String,
        @Body request: SyncPushRequest
    ): SyncPushResponse
}

@Singleton
class SyncApiImpl @Inject constructor(
    private val apiService: SyncApiService,
    private val tokenManager: TokenManager
) : SyncApi {

    override suspend fun pull(sinceVersion: Long): SyncPullResponse {
        val token = tokenManager.getAccessToken()
            ?: throw IllegalStateException("Not authenticated")
        return apiService.pull("Bearer $token", sinceVersion)
    }

    override suspend fun push(request: SyncPushRequest): SyncPushResponse {
        val token = tokenManager.getAccessToken()
            ?: throw IllegalStateException("Not authenticated")
        return apiService.push("Bearer $token", request)
    }
}

/**
 * Manages access and refresh tokens for API authentication.
 */
@Singleton
class TokenManager @Inject constructor() {
    private var accessToken: String? = null
    private var refreshToken: String? = null
    private var tokenExpiry: Long = 0

    fun setTokens(access: String, refresh: String, expiresIn: Long) {
        accessToken = access
        refreshToken = refresh
        tokenExpiry = System.currentTimeMillis() + (expiresIn * 1000)
    }

    fun getAccessToken(): String? {
        // Check if token is expired
        if (System.currentTimeMillis() > tokenExpiry - 60000) {
            // Token expired or expiring soon, should refresh
            return null
        }
        return accessToken
    }

    fun getRefreshToken(): String? = refreshToken

    fun clear() {
        accessToken = null
        refreshToken = null
        tokenExpiry = 0
    }
}
