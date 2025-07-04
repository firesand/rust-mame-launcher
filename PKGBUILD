# Maintainer: Edo Hikmahtiar <firesand@github>
pkgname=rust-mame-launcher
pkgver=0.1.0
pkgrel=1
pkgdesc="A MAME frontend built with Rust"
arch=('x86_64')
url="https://github.com/firesand/rust-mame-launcher"
license=('MIT')
depends=('mame' 'gtk3' 'glib2')
makedepends=('rust' 'cargo' 'git')
source=("$pkgname-$pkgver.tar.gz::https://github.com/firesand/$pkgname/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$pkgname-$pkgver"
    cargo build --release
}

package() {
    cd "$pkgname-$pkgver"

    # Install binary
    install -Dm755 "target/release/mame_gui" "$pkgdir/usr/bin/rust-mame-launcher"

    # Install desktop file
    install -Dm644 "rust-mame-launcher.desktop" "$pkgdir/usr/share/applications/rust-mame-launcher.desktop"

    # Install icon (if you have one)
    if [ -f "assets/mame-frontend-icon.png" ]; then
        install -Dm644 "assets/mame-frontend-icon.png" "$pkgdir/usr/share/icons/hicolor/256x256/apps/rust-mame-launcher.png"
    fi

    # Install license
    install -Dm644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"

    # Install README
    install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
}
