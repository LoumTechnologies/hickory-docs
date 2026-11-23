<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Build the Compose app and its shared-contract witnesses

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The app needs Compose, activity integration and coroutines, with no browser wrapper. Tests consume the same layout fixture as Swift and TypeScript. Keep API endpoints out of build configuration.

The debug APK stores an opted-in stamp in its generated BuildConfig, making
revision and dirty-state changes build inputs. Packaging also uses the debug
variant, so debug alone cannot turn the badge on.

<ui:copy id="simple-f064d703ee-0">
plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.plugin.compose")
}
val devBuild = providers.environmentVariable("HICKORY_DEV_BUILD").orNull == "1" &&
    providers.environmentVariable("HICKORY_VERSION").orNull.isNullOrEmpty()
val devStamp = if (devBuild) {
    fun git(vararg args: String): String? = runCatching {
        val result = providers.exec {
            workingDir(rootProject.projectDir.resolve("../.."))
            commandLine("git", "--no-optional-locks", *args)
        }
        if (result.result.get().exitValue == 0) result.standardOutput.asText.get().trim() else null
    }.getOrNull()
    val head = git("rev-parse", "--verify", "HEAD")
    val changes = git("status", "--porcelain=v1", "--untracked-files=normal")
    if (head != null && changes != null && git("rev-parse", "--show-prefix") == "" &&
        git("rev-parse", "--verify", "HEAD") == head) head + if (changes.isEmpty()) "" else "+"
    else "unknown"
} else ""

android {
    namespace = "com.hickorydocs.simple"
    compileSdk { version = release(37) { minorApiLevel = 0 } }
    defaultConfig { applicationId = "com.hickorydocs.simple"; minSdk = 26; targetSdk = 36; versionCode = 1; versionName = "0.1.0" }
    buildTypes {
        debug { buildConfigField("String", "DEV_BUILD_STAMP", "\"$devStamp\"") }
        release { buildConfigField("String", "DEV_BUILD_STAMP", "\"\"") }
    }
    buildFeatures { compose = true; buildConfig = true }
    compileOptions { sourceCompatibility = JavaVersion.VERSION_17; targetCompatibility = JavaVersion.VERSION_17 }
    sourceSets["main"].assets.srcDir("../../apple-simple/Sources/Resources")
    sourceSets["test"].resources.srcDir("../../simple-contract")
}
kotlin { compilerOptions { jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17) } }
dependencies {
    implementation(platform("androidx.compose:compose-bom:2025.10.01"))
    implementation("androidx.activity:activity-compose:1.11.0")
    implementation("androidx.compose.foundation:foundation")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.9.4")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.10.2")
    implementation("com.squareup.okhttp3:okhttp:4.12.0")
    testImplementation("junit:junit:4.13.2")
    testImplementation("org.json:json:20250517")
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/android-simple/app/build.gradle.kts"><ui:paste select="#simple-f064d703ee-0"/></ui:file>

</ui:doc>
