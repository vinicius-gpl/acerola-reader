package br.acerola.comic.usecase

import android.content.Context
import br.acerola.comic.config.preference.DeviceAliasPreference
import br.acerola.comic.config.preference.RelayPreference
import br.acerola.comic.service.P2pService
import br.acerola.comic.service.network.CoverBrowseProviderImpl
import br.acerola.comic.service.network.FileSyncProviderImpl
import br.acerola.comic.service.network.HistorySyncProviderImpl
import br.acerola.comic.service.network.P2pEventBus
import br.acerola.comic.service.network.SecureBlobStoreImpl
import br.acerola.comic.usecase.network.P2pUseCase
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object NetworkCaseModule {
    @Provides
    @Singleton
    fun provideP2pService(
        @ApplicationContext context: Context,
        eventBus: P2pEventBus,
        secureStore: SecureBlobStoreImpl,
        historyProvider: HistorySyncProviderImpl,
        fileProvider: FileSyncProviderImpl,
        coverProvider: CoverBrowseProviderImpl,
    ): P2pService {
        // Read once at startup (no runtime hot-swap) — changing the relay requires restarting the app.
        val relayUrlOverride = runBlocking { RelayPreference.relayUrlOverrideFlow(context).first() }
        // Renaming the device afterwards doesn't need a restart (see
        // `P2pService.setLocalDeviceName`) — this initial read only seeds the very first
        // `DeviceInfo` the node boots with, matching whatever was last saved.
        val deviceNameOverride = runBlocking { DeviceAliasPreference.deviceAliasFlow(context).first() }
        return P2pService(
            context,
            relayUrlOverride,
            deviceNameOverride,
            secureStore,
            historyProvider,
            fileProvider,
            coverProvider,
        ) { event, data ->
            eventBus.emit(event, data)
        }
    }

    @Provides
    @Singleton
    fun provideP2pUseCase(p2pService: P2pService): P2pUseCase = P2pUseCase(p2pService)
}
