package br.acerola.comic.common.ux.component

import androidx.compose.runtime.remember
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.navigation.compose.rememberNavController
import br.acerola.comic.common.ux.Acerola
import br.acerola.comic.common.ux.theme.AcerolaTheme
import dev.chrisbanes.haze.HazeState
import org.junit.Rule
import org.junit.Test

class NavigationBarsTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @Test
    fun `BottomBar_should_display_main_destinations`() {
        composeTestRule.setContent {
            val navController = rememberNavController()
            val hazeState = remember { HazeState() }

            AcerolaTheme {
                Acerola.Component.BottomBar(
                    navController = navController,
                    hazeState = hazeState, // Passado conforme exigido pelo componente
                )
            }
        }

        // Como o BottomBar usa label = null, precisamos buscar pela contentDescription dos ícones
        // Substitua os textos abaixo pelas strings exatas retornadas por `stringResource(destination.contentDescriptionRes)`
        composeTestRule.onNodeWithContentDescription("home", ignoreCase = true).assertIsDisplayed()
        composeTestRule.onNodeWithContentDescription("Histórico", ignoreCase = true).assertIsDisplayed()
        composeTestRule.onNodeWithContentDescription("Configurações", ignoreCase = true).assertIsDisplayed()
    }

    @Test
    fun `TopBar_should_display_given_title`() {
        composeTestRule.setContent {
            AcerolaTheme {
                Acerola.Component.TopBar(title = "Test Title")
            }
        }

        composeTestRule.onNodeWithText("Test Title").assertIsDisplayed()
    }

    @Test
    fun `SideBar_should_render_in_landscape_mode`() {
        composeTestRule.setContent {
            val navController = rememberNavController()

            AcerolaTheme {
                Acerola.Component.SideBar(navController = navController)
            }
        }

        // A SideBar possui o Text(), então onNodeWithText vai funcionar
        composeTestRule.onNodeWithText("home", ignoreCase = true).assertIsDisplayed()
    }
}
