# Maintainer: Edo Hikmahtiar <firesand@github>
pkgname=rust-mame-launcher
pkgver=0.1.0
pkgrel=1
pkgdesc="A MAME frontend built with Rust and egui"
arch=("x86_64")
url="https://github.com/firesand/rust-mame-launcher"
license=("MIT")
depends=("mame" "gtk3" "glib2" "zstd" "bzip2")
makedepends=("rust" "cargo")
source=()
sha256sums=()

prepare() {
    # Copy your local source
    cp -r /home/edo/Downloads/Project/* "$srcdir/"
}

build() {
    cd "$srcdir"

    # Set environment for linking
    export RUSTFLAGS="-C link-args=-lzstd"
    export PKG_CONFIG_PATH="/usr/lib/pkgconfig:/usr/lib64/pkgconfig"

    cargo build --release
}

package() {
    cd "$srcdir"

    # Install binary
    install -Dm755 "target/release/mame_gui" "$pkgdir/usr/bin/rust-mame-launcher"

    # Create desktop file
    cat > rust-mame-launcher.desktop << DESKTOP
[Desktop Entry]
Name=Rust MAME Launcher
Comment=A MAME frontend built with Rust
Exec=rust-mame-launcher
Icon=rust-mame-launcher
Terminal=false
Type=Application
Categories=Game;Emulator;
Keywords=mame;arcade;emulator;games;
StartupNotify=true
DESKTOP

    install -Dm644 "rust-mame-launcher.desktop" "$pkgdir/usr/share/applications/rust-mame-launcher.desktop"

    # Install icon if exists
    if [ -f "assets/mame-frontend-icon.png" ]; then
        install -Dm644 "assets/mame-frontend-icon.png" "$pkgdir/usr/share/icons/hicolor/256x256/apps/rust-mame-launcher.png"
    fi

    # Install LICENSE
    install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"

    # Install README if exists
    if [ -f "README.md" ]; then
        install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
    fi
}
