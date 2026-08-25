package br.acerola.comic.local.translator.ui

import br.acerola.comic.dto.metadata.category.CategoryDto
import br.acerola.comic.dto.metadata.comic.AuthorDto
import br.acerola.comic.dto.metadata.comic.ComicMetadataDto
import br.acerola.comic.dto.metadata.comic.GenreDto
import br.acerola.comic.dto.metadata.comic.source.AnilistSourceDto
import br.acerola.comic.dto.metadata.comic.source.ComicSourcesDto
import br.acerola.comic.dto.metadata.comic.source.MangadexSourceDto
import br.acerola.comic.dto.view.ComicSummaryDto
import br.acerola.comic.local.entity.category.Category
import br.acerola.comic.local.entity.metadata.relationship.Author
import br.acerola.comic.local.entity.metadata.relationship.Genre
import br.acerola.comic.local.entity.metadata.source.AnilistSource
import br.acerola.comic.local.entity.metadata.source.MangadexSource
import br.acerola.comic.local.entity.relation.MetadataRelations
import br.acerola.comic.local.entity.view.ComicSummaryView
import br.acerola.comic.pattern.metadata.MetadataSource

fun MetadataRelations.toViewDto(): ComicMetadataDto =
    ComicMetadataDto(
        id = this.remoteInfo.id,
        title = this.remoteInfo.title,
        description = this.remoteInfo.description,
        romanji = null,
        year = this.remoteInfo.publication,
        status = this.remoteInfo.status,
        authors = this.author.firstOrNull()?.toViewDto(),
        cover = null,
        banner = null,
        genre = this.genre.map { it.toViewDto() },
        comicDirectoryFk = this.remoteInfo.comicDirectoryFk,
        syncSource = MetadataSource.from(this.remoteInfo.syncSource),
        sources =
            ComicSourcesDto(
                mangadex = this.mangadexSource?.toViewDto(),
                anilist = this.anilistSource?.toViewDto(),
                comicInfo = null,
            ),
    )

fun ComicSummaryView.toViewDto(): ComicSummaryDto =
    ComicSummaryDto(
        directoryId = directoryFk,
        folderName = folderName,
        folderCover = folderCover,
        folderBanner = folderBanner,
        externalSync = externalSync,
        metadataTitle = metadataTitle,
        activeSource = MetadataSource.from(activeSource),
        metadataId = metadataFk,
    )

fun MangadexSource.toViewDto(): MangadexSourceDto =
    MangadexSourceDto(
        mangadexId = mangadexId,
        anilistId = null,
        amazonUrl = null,
        ebookjapanUrl = null,
        rawUrl = null,
        engtlUrl = null,
    )

fun AnilistSource.toViewDto(): AnilistSourceDto =
    AnilistSourceDto(
        anilistId = anilistId,
        averageScore = averageScore,
        popularity = popularity,
        trending = trending,
        coverImage = coverImage,
        bannerImage = bannerImage,
    )

fun Category.toViewDto(): CategoryDto =
    CategoryDto(
        id = id,
        name = name,
        color = color,
    )

fun Author.toViewDto(): AuthorDto =
    AuthorDto(
        id = id.toString(),
        name = name,
        type = type.type,
    )

fun Genre.toViewDto(): GenreDto =
    GenreDto(
        id = id.toString(),
        name = genre,
    )
