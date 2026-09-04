package br.acerola.comic.module.comic.template
import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import br.acerola.comic.common.ux.theme.AcerolaTheme
import br.acerola.comic.module.comic.Comic
import br.acerola.comic.module.comic.state.MainTab

@Composable
fun Comic.Template.Tabs(
    totalChapters: Int,
    activeTab: MainTab,
    onTabSelected: (MainTab) -> Unit,
) {
    val tabs = MainTab.entries.toTypedArray()

    Row(
        horizontalArrangement = Arrangement.Start,
        modifier =
            Modifier
                .fillMaxWidth()
                .padding(vertical = 16.dp, horizontal = 20.dp),
    ) {
        tabs.forEach { tab ->
            val isActive = tab == activeTab

            val title =
                tab.takeIf { it == MainTab.CHAPTERS }?.let { stringResource(id = it.titleRes, totalChapters) }
                    ?: stringResource(id = tab.titleRes)

            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                modifier =
                    Modifier
                        .padding(end = 24.dp)
                        .clickable { onTabSelected(tab) },
            ) {
                Text(
                    text = title,
                    color =
                        if (isActive) {
                            MaterialTheme.colorScheme.onBackground
                        } else {
                            MaterialTheme.colorScheme.onSurfaceVariant
                        },
                    style =
                        MaterialTheme.typography.titleMedium.copy(
                            fontWeight = if (isActive) FontWeight.Bold else FontWeight.Normal,
                        ),
                )

                if (isActive) {
                    Spacer(modifier = Modifier.height(height = 4.dp))
                    Box(
                        modifier =
                            Modifier
                                .width(width = 20.dp)
                                .height(height = 3.dp)
                                .background(
                                    color = MaterialTheme.colorScheme.primary,
                                    shape = RoundedCornerShape(size = 2.dp),
                                ),
                    )
                }
            }
        }
    }
}

@Preview(name = "Light", showBackground = true)
@Preview(name = "Dark", showBackground = true, uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun TabsPreview() {
    AcerolaTheme {
        Comic.Template.Tabs(
            totalChapters = 10,
            activeTab = MainTab.CHAPTERS,
            onTabSelected = {},
        )
    }
}
