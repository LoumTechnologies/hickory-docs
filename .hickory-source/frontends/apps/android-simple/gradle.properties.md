<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Enable AndroidX without machine-specific paths

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

JDK and SDK locations remain local developer configuration. Build caches and local.properties are excluded from literate outputs.

<ui:copy id="simple-be0b3784a0-0">
android.useAndroidX=true
kotlin.code.style=official
org.gradle.jvmargs=-Xmx2048m -Dfile.encoding=UTF-8
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/android-simple/gradle.properties"><ui:paste select="#simple-be0b3784a0-0"/></ui:file>

</ui:doc>
