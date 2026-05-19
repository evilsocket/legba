
class Legba < Formula
    version '1.3.0'
    desc "Legba is a multiprotocol credentials bruteforcer / password sprayer and enumerator."
    homepage "https://github.com/evilsocket/legba"
  
    if OS.mac?
        url "https://github.com/evilsocket/legba/releases/download/#{version}/legba-#{version}-apple-darwin-arm64.tar.gz"
        sha256 "9f0c43c73c1d45fbe03234d095fa1e5988a83b764219afb6a788db171d1c4f36"
    elsif OS.linux?
        url "https://github.com/evilsocket/legba/releases/download/#{version}/legba-#{version}-linux-x86_64.tar.gz"
        sha256 "8bc7242dbc97fd4c10ab05f7d8931597d04bf7b80793fec59bfb8cb930688602"
    end
  
    conflicts_with "legba"
  
    def install
      bin.install "legba"
    end
  end