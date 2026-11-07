<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Allow explicit connections to a workspace server

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

This comparison app connects to a user-entered engine address, including HTTP on local development hosts. It requests no filesystem permission and embeds no provider keys. Configuration changes retain the current editing session.

<ui:copy id="simple-edf5034e37-0">
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <uses-permission android:name="android.permission.INTERNET" />
    <application android:label="Hickory Simple" android:theme="@android:style/Theme.Material.Light.NoActionBar" android:usesCleartextTraffic="true" android:allowBackup="false" android:supportsRtl="true">
        <activity android:name=".MainActivity" android:exported="true" android:configChanges="orientation|screenSize|keyboardHidden" android:windowSoftInputMode="adjustResize">
            <intent-filter><action android:name="android.intent.action.MAIN" /><category android:name="android.intent.category.LAUNCHER" /></intent-filter>
        </activity>
    </application>
</manifest>
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/android-simple/app/src/main/AndroidManifest.xml"><ui:paste select="#simple-edf5034e37-0"/></ui:file>

</ui:doc>
