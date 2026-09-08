package br.acerola.comic.util.notification

import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import androidx.core.app.NotificationCompat
import br.acerola.comic.infra.R
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class NotificationHelper
    @Inject
    constructor(
        @param:ApplicationContext private val context: Context,
    ) {
        companion object {
            const val SYNC_CHANNEL_ID = "sync_channel"
            const val SYNC_NOTIFICATION_ID = 1001
            const val DOWNLOAD_NOTIFICATION_ID = 1002
            const val METADATA_NOTIFICATION_ID = 1003
            const val P2P_SYNC_NOTIFICATION_ID = 1004

            /** ID separado do `P2P_SYNC_NOTIFICATION_ID` acima de propósito: aquele é a
             *  notificação ONGOING (indeterminada, controlada por `P2pSyncForegroundService`,
             *  ligada/desligada por transições de `syncingKeys`) — reusar o mesmo id aqui faria
             *  `showFinishedNotification` (auto-cancel, não ongoing) cancelar/substituir a
             *  notificação de progresso ainda ativa se outra sessão P2P (ex.: sync de histórico)
             *  seguir rodando ao mesmo tempo pro mesmo ou outro peer. */
            const val P2P_FILE_SYNC_RESULT_NOTIFICATION_ID = 1005
        }

        private val notificationManager =
            context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

        init {
            createNotificationChannel()
        }

        private fun createNotificationChannel() {
            val name = context.getString(R.string.sync_notification_channel_name)
            val descriptionText = context.getString(R.string.sync_notification_channel_description)
            val importance = NotificationManager.IMPORTANCE_LOW
            val channel =
                NotificationChannel(SYNC_CHANNEL_ID, name, importance).apply {
                    description = descriptionText
                }

            notificationManager.createNotificationChannel(channel)
        }

        fun createBaseNotification(
            title: String,
            content: String,
        ): NotificationCompat.Builder =
            NotificationCompat
                .Builder(context, SYNC_CHANNEL_ID)
                .setSmallIcon(R.drawable.ic_notification)
                .setContentTitle(title)
                .setContentText(content)
                .setPriority(NotificationCompat.PRIORITY_LOW)
                .setOngoing(true)
                .setProgress(100, 0, true)

        fun updateProgress(
            builder: NotificationCompat.Builder,
            progress: Int,
            notificationId: Int = SYNC_NOTIFICATION_ID,
            max: Int = 100,
        ) {
            val isIndeterminate = progress < 0
            builder.setProgress(max, if (isIndeterminate) 0 else progress, isIndeterminate)
            notificationManager.notify(notificationId, builder.build())
        }

        fun cancelNotification(notificationId: Int = SYNC_NOTIFICATION_ID) {
            notificationManager.cancel(notificationId)
        }

        fun showFinishedNotification(
            title: String,
            content: String,
            notificationId: Int = SYNC_NOTIFICATION_ID,
        ) {
            val builder =
                NotificationCompat
                    .Builder(context, SYNC_CHANNEL_ID)
                    .setSmallIcon(R.drawable.ic_notification)
                    .setContentTitle(title)
                    .setContentText(content)
                    .setPriority(NotificationCompat.PRIORITY_DEFAULT)
                    .setAutoCancel(true)
                    .setOngoing(false)

            notificationManager.notify(notificationId, builder.build())
        }
    }
