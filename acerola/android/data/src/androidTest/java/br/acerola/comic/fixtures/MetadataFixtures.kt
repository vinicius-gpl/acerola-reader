package br.acerola.comic.fixtures

import br.acerola.comic.dto.metadata.comic.AuthorDto
import br.acerola.comic.dto.metadata.comic.ComicMetadataDto
import br.acerola.comic.dto.metadata.comic.CoverDto
import br.acerola.comic.dto.metadata.comic.GenreDto
import br.acerola.comic.local.entity.metadata.ComicMetadata
import br.acerola.comic.local.entity.metadata.relationship.Author
import br.acerola.comic.local.entity.metadata.relationship.Genre
import br.acerola.comic.local.entity.metadata.relationship.TypeAuthor
import br.acerola.comic.local.entity.metadata.source.AnilistSource
import br.acerola.comic.local.entity.metadata.source.MangadexSource
import br.acerola.comic.local.entity.relation.MetadataRelations

object MetadataFixtures {
    fun createMangaRemoteInfo(
        id: Long = 10,
        title: String = "Naruto",
        description: String = "Ninja story",
        status: String = "ongoing",
        publication: Int = 1999,
        comicDirectoryFk: Long? = null,
        syncSource: String? = null,
        hasComicInfo: Boolean = false,
    ) = ComicMetadata(
        id = id,
        title = title,
        description = description,
        status = status,
        publication = publication,
        comicDirectoryFk = comicDirectoryFk,
        syncSource = syncSource,
        hasComicInfo = hasComicInfo,
    )

    fun createMangaRemoteInfoDto(
        title: String = "Naruto",
        description: String = "Desc",
        status: String = "ongoing",
        year: Int? = null,
        authors: AuthorDto? = null,
        genre: List<GenreDto> = emptyList(),
        cover: CoverDto? = null,
    ) = ComicMetadataDto(
        title = title,
        description = description,
        status = status,
        year = year,
        authors = authors,
        genre = genre,
        cover = cover,
    )

    fun createRemoteInfoRelations(
        remoteInfo: ComicMetadata = createMangaRemoteInfo(),
        mangadexSource: MangadexSource? = null,
        anilistSource: AnilistSource? = null,
        authors: List<Author> = emptyList(),
        genres: List<Genre> = emptyList(),
    ) = MetadataRelations(
        remoteInfo = remoteInfo,
        mangadexSource = mangadexSource,
        anilistSource = anilistSource,
        author = authors,
        genre = genres,
    )

    fun createAuthor(
        id: Long = 1,
        name: String = "Kishimoto",
        type: TypeAuthor = TypeAuthor.AUTHOR,
        comicId: Long = 10,
    ) = Author(id = id, name = name, type = type, comicRemoteInfoFk = comicId)

    fun createGenre(
        id: Long = 1,
        genre: String = "Shonen",
        comicId: Long = 10,
    ) = Genre(id = id, genre = genre, comicRemoteInfoFk = comicId)
}
