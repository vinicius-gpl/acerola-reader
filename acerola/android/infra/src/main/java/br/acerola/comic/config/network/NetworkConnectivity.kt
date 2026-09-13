package br.acerola.comic.config.network

import android.content.Context
import android.net.ConnectivityManager
import android.net.NetworkCapabilities

/** `true` quando a rede ativa é celular (dados móveis) — usado pra pedir confirmação antes de
 *  disparar uma sincronização P2P (ver `SyncViewModel`), já que isso pode consumir a franquia de
 *  dados do usuário. Requer `ACCESS_NETWORK_STATE` (declarada em `app/src/main/AndroidManifest.xml`). */
fun isOnCellularConnection(context: Context): Boolean {
    val connectivityManager = context.getSystemService(ConnectivityManager::class.java) ?: return false
    val network = connectivityManager.activeNetwork ?: return false
    val capabilities = connectivityManager.getNetworkCapabilities(network) ?: return false
    return capabilities.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR)
}
