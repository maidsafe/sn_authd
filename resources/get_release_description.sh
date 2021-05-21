#!/usr/bin/env bash

version=$1
if [[ -z "$version" ]]; then
    echo "You must supply a version number for sn_authd."
    exit 1
fi

# The single quotes around EOF is to stop attempted variable and backtick expansion.
read -r -d '' release_description << 'EOF'
The Authenticator daemon exposes services which allow applications and users to create a Safe, unlock it using its credentials (passphrase and password), authorise applications which need to store data on the network on behalf of the user, as well as revoke permissions previously granted to applications.
The Safe Authenticator, which runs as a daemon, or as a service in Windows platforms, can be started and managed with the Safe CLI if the sn_authd.exe binary is properly installed in the system with execution permissions.

Refer to [Safe Authenticator daemon User Guide](https://github.com/maidsafe/sn_authd/blob/master/README.md) for further details.

## SHA-256 checksums for Authd binaries:
```
Linux
zip: ZIP_LINUX_CHECKSUM
tar.gz: TAR_LINUX_CHECKSUM

macOS
zip: ZIP_MACOS_CHECKSUM
tar.gz: TAR_MACOS_CHECKSUM

Windows
zip: ZIP_WIN_CHECKSUM
tar.gz: TAR_WIN_CHECKSUM
```

## Related Links
* [Safe CLI User Guide](https://github.com/maidsafe/sn_cli/blob/master/README.md)
* [Safe Network Browser](https://github.com/maidsafe/sn_browser/releases/)
* [Safe Network Node](https://github.com/maidsafe/sn_node/releases/latest/)
EOF

zip_linux_checksum=$(sha256sum \
    "./deploy/prod/sn_authd-$version-x86_64-unknown-linux-musl.zip" | \
    awk '{ print $1 }')
zip_macos_checksum=$(sha256sum \
    "./deploy/prod/sn_authd-$version-x86_64-apple-darwin.zip" | \
    awk '{ print $1 }')
zip_win_checksum=$(sha256sum \
    "./deploy/prod/sn_authd-$version-x86_64-pc-windows-msvc.zip" | \
    awk '{ print $1 }')
tar_linux_checksum=$(sha256sum \
    "./deploy/prod/sn_authd-$version-x86_64-unknown-linux-musl.tar.gz" | \
    awk '{ print $1 }')
tar_macos_checksum=$(sha256sum \
    "./deploy/prod/sn_authd-$version-x86_64-apple-darwin.tar.gz" | \
    awk '{ print $1 }')
tar_win_checksum=$(sha256sum \
    "./deploy/prod/sn_authd-$version-x86_64-pc-windows-msvc.tar.gz" | \
    awk '{ print $1 }')

release_description=$(sed "s/ZIP_LINUX_CHECKSUM/$zip_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/ZIP_MACOS_CHECKSUM/$zip_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/ZIP_WIN_CHECKSUM/$zip_win_checksum/g" <<< "$release_description")
release_description=$(sed "s/TAR_LINUX_CHECKSUM/$tar_linux_checksum/g" <<< "$release_description")
release_description=$(sed "s/TAR_MACOS_CHECKSUM/$tar_macos_checksum/g" <<< "$release_description")
release_description=$(sed "s/TAR_WIN_CHECKSUM/$tar_win_checksum/g" <<< "$release_description")

echo "$release_description"
