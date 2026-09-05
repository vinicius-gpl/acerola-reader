package br.acerola.comic.service

import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.content.ContextCompat
import br.acerola.comic.core.R
import br.acerola.comic.util.notification.NotificationHelper
import dagger.hilt.android.AndroidEntryPoint
import javax.inject.Inject

/**
 * Serviço "vazio" — não faz nenhuma transferência sozinho, só sinaliza pro sistema que existe
 * trabalho de rede ativo em andamento (o node P2P/`P2pService` já rodam independente disso,
 * atrelados ao ciclo de vida do Hilt). Sem isso, uma sincronização longa (minutos, capítulos
 * grandes) roda sem NENHUMA isenção do Doze/App Standby — em fabricantes com gerenciador de
 * energia próprio (MIUI/EMUI/ColorOS) isso é ainda mais agressivo que o Android puro, e pode
 * suspender a conexão mesmo com a tela ligada (ver TODO.md).
 *
 * Início/fim são disparados por `SyncViewModel` a cada transição de `syncingKeys` vazio<->não
 * vazio — este serviço nunca decide sozinho quando começar ou parar.
 */
@AndroidEntryPoint
class P2pSyncForegroundService : Service() {
    @Inject
    lateinit var notificationHelper: NotificationHelper

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(
        intent: Intent?,
        flags: Int,
        startId: Int,
    ): Int {
        val notification =
            notificationHelper
                .createBaseNotification(
                    getString(R.string.p2p_sync_notification_title),
                    getString(R.string.p2p_sync_notification_description),
                ).setProgress(0, 0, false)
                .build()

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            startForeground(
                NotificationHelper.P2P_SYNC_NOTIFICATION_ID,
                notification,
                ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC,
            )
        } else {
            startForeground(NotificationHelper.P2P_SYNC_NOTIFICATION_ID, notification)
        }

        return START_NOT_STICKY
    }

    override fun onDestroy() {
        notificationHelper.cancelNotification(NotificationHelper.P2P_SYNC_NOTIFICATION_ID)
        super.onDestroy()
    }

    companion object {
        /** Idempotente — chamar de novo enquanto já ativo só reemite a mesma notificação. */
        fun start(context: Context) {
            ContextCompat.startForegroundService(context, Intent(context, P2pSyncForegroundService::class.java))
        }

        fun stop(context: Context) {
            context.stopService(Intent(context, P2pSyncForegroundService::class.java))
        }
    }
}
