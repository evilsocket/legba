
class Legba < Formula
    version '1.1.1'
    desc "Legba is a multiprotocol credentials bruteforcer / password sprayer and enumerator."
    homepage "https://github.com/evilsocket/legba"
  
    if OS.mac?
        url "https://github.com/evilsocket/legba/releases/download/#{version}/legba-#{version}-apple-darwin-arm64.tar.gz"
        sha256 "f8df9c40ae853113a0bab81e80486d221881d01b0014ed0aa72fe72c7b75e52f"
    elsif OS.linux?
        url "https://github.com/evilsocket/legba/releases/download/#{version}/legba-#{version}-linux-x86_64.tar.gz"
        sha256 "7a01c975e5329e8f819ca552b9b734102ffbd30f1b6f7739b1cf1108a22eb096"
    end
  
    conflicts_with "legba"
  
    def install
      bin.install "legba"
    end
  end