# Maintainer: fwaifu contributors
# Install: makepkg -si

pkgname=fwaifu
pkgver=1.0.0
pkgrel=1
pkgdesc="Terminal anime girl viewer powered by fastfetch"
arch=('x86_64')
url="https://github.com/cublueer/fwaifu"
license=('MIT')
depends=('fastfetch')
optdepends=('imagemagick: for image cropping')
makedepends=('rust' 'git')
source=()
sha256sums=()

prepare() {
    cp -r "$startdir"/* "$srcdir/fwaifu"
}

build() {
    cd fwaifu
    cargo build --release
}

package() {
    cd fwaifu
    install -Dm755 "target/release/${pkgname}" "$pkgdir/usr/bin/${pkgname}"
}
