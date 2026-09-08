package br.acerola.comic.module.comic.template

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import br.acerola.comic.fixtures.ComicFixtures
import br.acerola.comic.module.comic.Comic
import org.junit.Rule
import org.junit.Test

class ConfigSectionTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @Test
    fun should_display_all_comic_config_sections() {
        val uiState = ComicFixtures.createMangaUiState()

        composeTestRule.setContent {
            LazyColumn {
                Comic.Template.configSection(
                    scope = this,
                    uiState = uiState,
                    // "Leitura" colapsa por padrão (ver Acerola.Component.AccordionCard) — expande
                    // pra o teste inspecionar o conteúdo. "Sincronização" é sempre flat/aberta.
                    expandedCategories = setOf("reading"),
                    onToggleCategory = {},
                    onAction = {},
                    onSyncAction = {},
                )
            }
        }

        // Títulos das 3 categorias (Leitura/Sincronização/Avançado) são cabeçalhos de
        // Acerola.Component.AccordionCard, texto exato do strings.xml.
        composeTestRule.onNodeWithText("Leitura", ignoreCase = true).assertIsDisplayed()
        composeTestRule.onNodeWithText("Sincronização", ignoreCase = true).assertIsDisplayed()

        // MangaDex é exibido dentro do content expansível do ComicExternalSyncToggle, na
        // categoria "Sincronização" (sempre flat) — o fixture já tem externalSyncEnabled = true.
        composeTestRule.onNodeWithText("MangaDex", ignoreCase = true).assertIsDisplayed()
    }
}
