package br.acerola.comic.service.network

import android.content.Context
import android.util.Base64
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import dagger.hilt.android.qualifiers.ApplicationContext
import p2p.SecureBlobStore
import p2p.SecureBlobStoreException
import javax.inject.Inject
import javax.inject.Singleton

private const val PREFS_FILE_NAME = "acerola_p2p_secure"
private const val TAG = "SecureBlobStore"

/**
 * Kotlin implementation of [SecureBlobStore] (foreign `uniffi` trait) used by
 * `SecureP2pStorage`/`SecureTrustedStore` on the Rust side to persist node identity, peer
 * cache and the TOFU trust list behind the Android Keystore instead of plain text on disk —
 * master key managed by [MasterKey] (AES256-GCM), values in [EncryptedSharedPreferences]
 * (AES256-SIV keys, AES256-GCM values).
 *
 * `Vec<u8>`/`ByteArray` are stored as Base64 because `EncryptedSharedPreferences` only
 * accepts `String` values. Real backend failures (corrupted keystore, I/O error) are surfaced
 * as [SecureBlobStoreException] instead of collapsing into `null`/silently swallowed — Rust
 * treats a load failure very differently from "key was never saved" (see the trait doc):
 * a load failure that looked like "never saved" could make the node mint a brand new identity
 * and silently break every existing pairing.
 */
@Singleton
class SecureBlobStoreImpl
    @Inject
    constructor(
        @param:ApplicationContext private val context: Context,
    ) : SecureBlobStore {
        private val prefs by lazy {
            val masterKey =
                MasterKey
                    .Builder(context)
                    .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
                    .build()

            EncryptedSharedPreferences.create(
                context,
                PREFS_FILE_NAME,
                masterKey,
                EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
                EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
            )
        }

        override fun saveBlob(
            key: String,
            value: ByteArray,
        ) {
            val committed =
                try {
                    prefs.edit().putString(key, Base64.encodeToString(value, Base64.NO_WRAP)).commit()
                } catch (error: Exception) {
                    AcerolaLogger.e(TAG, "Failed to save blob for key \"$key\"", LogSource.NETWORK, error)
                    throw SecureBlobStoreException.AccessFailed(error.message ?: error.javaClass.simpleName)
                }
            if (!committed) {
                AcerolaLogger.e(TAG, "SharedPreferences.commit() returned false for key \"$key\"", LogSource.NETWORK)
                throw SecureBlobStoreException.AccessFailed("commit() returned false")
            }
        }

        override fun loadBlob(key: String): ByteArray? =
            try {
                prefs.getString(key, null)?.let { Base64.decode(it, Base64.NO_WRAP) }
            } catch (error: Exception) {
                AcerolaLogger.e(TAG, "Failed to load blob for key \"$key\"", LogSource.NETWORK, error)
                throw SecureBlobStoreException.AccessFailed(error.message ?: error.javaClass.simpleName)
            }

        override fun clearBlob(key: String) {
            try {
                prefs.edit().remove(key).commit()
            } catch (error: Exception) {
                AcerolaLogger.e(TAG, "Failed to clear blob for key \"$key\"", LogSource.NETWORK, error)
                throw SecureBlobStoreException.AccessFailed(error.message ?: error.javaClass.simpleName)
            }
        }
    }
