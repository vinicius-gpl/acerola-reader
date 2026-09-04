package br.acerola.comic.service.metadata

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.SmallTest
import br.acerola.comic.dto.metadata.comic.ComicMetadataDto
import br.acerola.comic.error.message.ComicInfoError
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import java.io.ByteArrayInputStream

@RunWith(AndroidJUnit4::class)
@SmallTest
class ComicInfoParserTest {
    private lateinit var service: ComicInfoParser

    @Before
    fun setUp() {
        service = ComicInfoParser()
    }

    @Test
    fun parseMangaInfo_should_return_success_with_valid_xml() {
        val xml =
            """
            <?xml version="1.0"?>
            <ComicInfo>
                <Title>One Piece</Title>
                <Series>One Piece Series</Series>
                <Summary>Pirates adventure</Summary>
                <Writer>Eiichiro Oda</Writer>
                <Genre>Action, Adventure, Shonen</Genre>
                <Year>1997</Year>
            </ComicInfo>
            """.trimIndent()

        val inputStream = ByteArrayInputStream(xml.toByteArray())
        val result = service.parseMangaInfo(inputStream)

        assertTrue("Should be a success", result.isRight())
        result.onRight { info ->
            assertEquals("One Piece Series", info.title) // Series tem prioridade sobre Title
            assertEquals("Pirates adventure", info.description)
            assertEquals("Eiichiro Oda", info.authors?.name)
            assertEquals(1997, info.year)
            assertEquals(3, info.genre.size)
            assertEquals("Adventure", info.genre[1].name)
        }
    }

    @Test
    fun parseMangaInfo_should_return_MissingRootElement_if_xml_is_not_comicinfo() {
        val xml =
            """
            <?xml version="1.0"?>
            <WrongRoot>
                <Title>One Piece</Title>
            </WrongRoot>
            """.trimIndent()

        val inputStream = ByteArrayInputStream(xml.toByteArray())
        val result = service.parseMangaInfo(inputStream)

        assertTrue("Should fail", result.isLeft())
        result.onLeft { error ->
            assertTrue(error is ComicInfoError.MissingRootElement)
        }
    }

    @Test
    fun parseMangaInfo_should_return_InvalidXmlFormat_if_xml_is_broken() {
        val xml =
            """
            <?xml version="1.0"?>
            <ComicInfo>
                <Title>One Piece
            </ComicInfo> <!-- Falta fechar a tag Title -->
            """.trimIndent()

        val inputStream = ByteArrayInputStream(xml.toByteArray())
        val result = service.parseMangaInfo(inputStream)

        assertTrue("Should fail with invalid format", result.isLeft())
        result.onLeft { error ->
            assertTrue(error is ComicInfoError.InvalidXmlFormat)
        }
    }

    @Test
    fun serialize_should_generate_valid_xml() {
        val info =
            ComicMetadataDto(
                title = "Berserk",
                description = "Guts struggle",
                status = "ongoing",
                year = 1989,
            )

        val xmlOutput = service.serialize(info)

        assertTrue(xmlOutput.contains("<ComicInfo>"))
        assertTrue(xmlOutput.contains("<Series>Berserk</Series>"))
        assertTrue(xmlOutput.contains("<Summary>Guts struggle</Summary>"))
        assertTrue(xmlOutput.contains("<Year>1989</Year>"))
        assertTrue(xmlOutput.contains("</ComicInfo>"))
    }
}
