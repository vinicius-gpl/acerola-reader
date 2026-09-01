package br.acerola.comic.service.artwork

import android.content.Context
import android.net.Uri
import androidx.core.net.toUri
import androidx.documentfile.provider.DocumentFile
import arrow.core.Either
import arrow.core.flatMap
import arrow.core.left
import arrow.core.right
import br.acerola.comic.error.message.IoError
import br.acerola.comic.local.dao.archive.ComicDirectoryDao
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.pattern.media.MediaFile
import br.acerola.comic.service.file.FileStorageHandler
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class BannerSaver
    @Inject
    constructor(
        private val directoryDao: ComicDirectoryDao,
        private val fileStorageHandler: FileStorageHandler,
        @param:ApplicationContext private val context: Context,
    ) {
        suspend fun processBanner(
            rootUri: Uri,
            folderId: Long,
            bytes: ByteArray,
            bannerUrl: String,
            comicFolderName: String,
            comicRemoteInfoFk: Long,
        ): Either<IoError, Long> =
            withContext(Dispatchers.IO) {
                try {
                    val directory =
                        directoryDao.getDirectoryById(comicId = folderId)
                            ?: return@withContext IoError.FileNotFound("Directory not found in database").left()

                    val comicDir = runCatching { DocumentFile.fromTreeUri(context, directory.path.toUri()) }.getOrNull()

                    if (comicDir != null && comicDir.isDirectory) {
                        AcerolaLogger.d(TAG, "Saving banner to directory: ${comicDir.uri}", LogSource.REPOSITORY)

                        comicDir.listFiles().forEach { file ->
                            val fileName = file.name ?: return@forEach
                            if (MediaFile.isBanner(fileName)) {
                                file.delete()
                            }
                        }

                        val fileName = MediaFile.BANNER.defaultFileName

                        fileStorageHandler
                            .saveFile(
                                folder = comicDir,
                                fileName = fileName,
                                mimeType = "image/jpeg",
                                bytes = bytes,
                            ).flatMap {
                                val savedFile = comicDir.findFile(fileName)
                                val savedUriString = savedFile?.uri?.toString() ?: bannerUrl

                                directoryDao.update(directory.copy(banner = savedUriString, lastModified = System.currentTimeMillis()))
                                folderId.right()
                            }
                    } else {
                        val internalStorageResult =
                            runCatching {
                                val internalDir = java.io.File(context.filesDir, "banners")
                                if (!internalDir.exists()) internalDir.mkdirs()
                                val bannerFile = java.io.File(internalDir, "$folderId.jpg")
                                bannerFile.writeBytes(bytes)
                                val savedUriString = Uri.fromFile(bannerFile).toString()
                                directoryDao.update(directory.copy(banner = savedUriString, lastModified = System.currentTimeMillis()))
                                folderId.right()
                            }

                        internalStorageResult.getOrElse {
                            AcerolaLogger.e(TAG, "Comic directory not accessible for ${directory.path}", LogSource.REPOSITORY)
                            IoError.FileNotFound("Comic directory not accessible").left()
                        }
                    }
                } catch (exception: Exception) {
                    AcerolaLogger.e(
                        TAG,
                        "Critical error processing banner for $comicFolderName",
                        LogSource.REPOSITORY,
                        exception,
                    )
                    IoError.FileWriteError(comicFolderName, exception).left()
                }
            }

        companion object {
            private const val TAG = "MangaSaveBannerService"
        }
    }
