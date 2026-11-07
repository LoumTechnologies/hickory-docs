<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Compile the three comparison frontends in CI

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Each platform job runs the same focused recipe available locally. These checks add no publishing or replacement of the existing frontend. Pull requests use the shared path selector; every master push checks all three platforms.

<ui:copy id="simple-9492fd1ec5-0">
name: Simple frontends
on:
  push:
    branches: [master]
  pull_request:
    branches: [master]
jobs:
  changes:
    if: github.event_name == 'pull_request'
    runs-on: ubuntu-latest
    outputs:
      browser: ${{ steps.filter.outputs.browser }}
      apple: ${{ steps.filter.outputs.apple }}
      android: ${{ steps.filter.outputs.android }}
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }
      - id: filter
        shell: bash
        run: |
          checks=$(scripts/affected-checks.sh "origin/${{ github.base_ref }}")
          for platform in browser apple android; do
            if grep -qx "simple-$platform" <<<"$checks"; then
              echo "$platform=true" >> "$GITHUB_OUTPUT"
            else
              echo "$platform=false" >> "$GITHUB_OUTPUT"
            fi
          done
  browser:
    needs: [changes]
    if: always() && (github.event_name == 'push' || needs.changes.outputs.browser == 'true')
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: '22', cache: npm, cache-dependency-path: apps/web/package-lock.json }
      - uses: extractions/setup-just@v4
      - run: npm --prefix apps/web ci
      - run: just simple-test browser
  apple:
    needs: [changes]
    if: always() && (github.event_name == 'push' || needs.changes.outputs.apple == 'true')
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: extractions/setup-just@v4
      - run: brew install xcodegen
      - run: just simple-test apple
      - run: just simple-build ios
  android:
    needs: [changes]
    if: always() && (github.event_name == 'push' || needs.changes.outputs.android == 'true')
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: extractions/setup-just@v4
      - uses: actions/setup-java@v4
        with: { distribution: temurin, java-version: '17' }
      - uses: android-actions/setup-android@v3
      - uses: gradle/actions/setup-gradle@v4
        with: { gradle-version: '9.3.1' }
      - run: sdkmanager 'platforms;android-37.0' 'build-tools;36.0.0'
      - run: just simple-test android
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path=".github/workflows/simple-frontends.yml"><ui:paste select="#simple-9492fd1ec5-0"/></ui:file>

</ui:doc>
