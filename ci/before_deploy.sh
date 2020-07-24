# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    cross build --target $TARGET --release

    # TODO Update this to package the right artifacts
    cp target/$TARGET/release/websockify-rs $stage/
    cp README.md $stage/
    cp LICENSE $stage/LICENSE
    cp websockify-rs/noVNC/README.md $stage/README-novnc.md
    cp websockify-rs/noVNC/LICENSE.txt $stage/LICENSE-novnc.txt

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main
