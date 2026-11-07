<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Resolve Android plugins from their official repositories

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Keep the Compose comparison as an independent Gradle build. It does not alter the existing Tauri mobile workspace.

<ui:copy id="simple-18160dd8be-0">
pluginManagement { repositories { google(); mavenCentral(); gradlePluginPortal() } }
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories { google(); mavenCentral() }
}
rootProject.name = "HickorySimple"
include(":app")
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/android-simple/settings.gradle.kts"><ui:paste select="#simple-18160dd8be-0"/></ui:file>

</ui:doc>
