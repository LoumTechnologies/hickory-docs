<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Pin the Gradle distribution used by Android Studio

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Android Studio can resolve this exact distribution when importing the project. Command-line recipes use Gradle 9.3.1 installed by the developer or CI setup action; no wrapper JAR is vendored in this initial source-only scaffold. The checksum is [Gradle’s published 9.3.1 binary distribution checksum](https://gradle.org/release-checksums/).

<ui:copy id="simple-4dc43924f5-0">
distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-9.3.1-bin.zip
distributionSha256Sum=b266d5ff6b90eada6dc3b20cb090e3731302e553a27c5d3e4df1f0d76beaff06
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/android-simple/gradle/wrapper/gradle-wrapper.properties"><ui:paste select="#simple-4dc43924f5-0"/></ui:file>

</ui:doc>
