package com.keydrop.di

import android.content.Context
import androidx.room.Room
import com.keydrop.data.local.VaultDatabase
import com.keydrop.data.local.dao.SyncStateDao
import com.keydrop.data.local.dao.VaultItemDao
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object AppModule {

    @Provides
    @Singleton
    fun provideVaultDatabase(
        @ApplicationContext context: Context
    ): VaultDatabase {
        return Room.databaseBuilder(
            context,
            VaultDatabase::class.java,
            VaultDatabase.DATABASE_NAME
        )
            .fallbackToDestructiveMigration()
            .build()
    }

    @Provides
    @Singleton
    fun provideVaultItemDao(database: VaultDatabase): VaultItemDao {
        return database.vaultItemDao()
    }

    @Provides
    @Singleton
    fun provideSyncStateDao(database: VaultDatabase): SyncStateDao {
        return database.syncStateDao()
    }
}
