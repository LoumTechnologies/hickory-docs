<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Pin the Android build toolchain

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Use explicit compatible plugin versions rather than whichever versions an IDE installation happens to select. AGP 9.1.1 supports the installed API 37.0 platform and supplies Kotlin 2.2.10, so only the matching Compose compiler plugin is applied. See [the compatibility table](https://developer.android.com/build/releases/agp-9-1-0-release-notes).

<ui:copy id="simple-d6a37e2fab-0">
plugins {
    id("com.android.application") version "9.1.1" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.2.10" apply false
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/android-simple/build.gradle.kts"><ui:paste select="#simple-d6a37e2fab-0"/></ui:file>

</ui:doc>
