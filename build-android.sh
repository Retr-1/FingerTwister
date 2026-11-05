#!/bin/sh
set -eu
cp /project/Cargo.toml /project/Cargo.lock /root/src/
cp -r /project/src /root/src/
sed -i 's/^version = 4$/version = 3/' /root/src/Cargo.lock
cd /root/src
cargo quad-apk build --release

# Check the generated loader against the actual APK contents, not only Rust compilation.
activity=target/android-artifacts/release/bin/fingertwister/sk/fingertwister/fingertwister/MainActivity.java
apk=target/android-artifacts/release/apk/fingertwister.apk
grep -F 'System.loadLibrary("fingertwister")' "$activity"
unzip -Z1 "$apk" | grep -Fx 'lib/arm64-v8a/libfingertwister.so'
echo 'Verified: Java loads fingertwister and the APK contains libfingertwister.so for ARM64.'

# Inspect the packaged binary manifest, not just the source configuration.
"$ANDROID_HOME/build-tools/36.0.0/aapt" dump badging "$apk" > /tmp/fingertwister-badging.txt
grep -Fx "targetSdkVersion:'36'" /tmp/fingertwister-badging.txt
grep -Fx "sdkVersion:'21'" /tmp/fingertwister-badging.txt
"$ANDROID_HOME/build-tools/36.0.0/apksigner" verify "$apk"
echo 'Verified: APK targets Android 16 / API 36 and supports API 21+.'

