<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/desktop/fskit/Info.plist

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `26b24f09ada89c84a641a5b5785e2622dcfc4122c1e9e803d5ea6cd49b96e7ee`.

<book:entry path="apps/desktop/fskit/Info.plist">
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleExecutable</key>
	<string>HickoryWorkspace</string>
	<key>CFBundleIdentifier</key>
	<string>com.loumtechnologies.hickorydocs.workspace</string>
	<key>CFBundleName</key>
	<string>Hickory Workspace</string>
	<key>CFBundlePackageType</key>
	<string>XPC!</string>
	<key>CFBundleShortVersionString</key>
	<string>0.1.0</string>
	<key>CFBundleVersion</key>
	<string>1</string>
	<key>EXAppExtensionAttributes</key>
	<dict>
		<key>EXExtensionPointIdentifier</key>
		<string>com.apple.fskit.fsmodule</string>
		<key>FSActivateOptionSyntax</key>
		<dict>
			<key>shortOptions</key>
			<string>u:g:m:o:</string>
		</dict>
		<key>FSCheckOptionSyntax</key>
		<dict>
			<key>shortOptions</key>
			<string>nqy</string>
		</dict>
		<key>FSFormatOptionSyntax</key>
		<dict>
			<key>shortOptions</key>
			<string>v</string>
		</dict>
		<key>FSMediaTypes</key>
		<dict/>
		<key>FSPersonalities</key>
		<dict/>
		<key>FSRequiresSecurityScopedPathURLResources</key>
		<true/>
		<key>FSShortName</key>
		<string>hickory</string>
		<key>FSSupportsBlockResources</key>
		<false/>
		<key>FSSupportsPathURLs</key>
		<true/>
	</dict>
	<key>LSMinimumSystemVersion</key>
	<string>26.0</string>
	<key>NSAppTransportSecurity</key>
	<dict>
		<key>NSAllowsLocalNetworking</key>
		<true/>
	</dict>
</dict>
</plist>
</book:entry>

</book:doc>
