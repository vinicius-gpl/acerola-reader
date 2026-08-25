package br.acerola.comic.local.entity.relation

import androidx.room.Embedded
import androidx.room.Relation
import br.acerola.comic.local.entity.archive.ChapterArchive
import br.acerola.comic.local.entity.archive.VolumeArchive

data class ChapterVolumeJoin(
    @Embedded
    val chapter: ChapterArchive,
    @Relation(
        parentColumn = "volume_fk",
        entityColumn = "id",
    )
    val volume: VolumeArchive? = null,
)
