
class Legba < Formula
    version '1.2.0'
    desc "Legba is a multiprotocol credentials bruteforcer / password sprayer and enumerator."
    homepage "https://github.com/evilsocket/legba"
  
    if OS.mac?
        url "https://github.com/evilsocket/legba/releases/download/#{version}/legba-#{version}-apple-darwin-arm64.tar.gz"
        sha256 "5faf2f734b64e6128fd4929b694f8ec60b6be6a55f753d424bcaf1706e26cf93"
    elsif OS.linux?
        url "https://github.com/evilsocket/legba/releases/download/#{version}/legba-#{version}-linux-x86_64.tar.gz"
        sha256 "0c286d64edec605f74b71d96eae1d8605ec4c1770a62304e93eb15cfce9c3455"
    end
  
    conflicts_with "legba"
  
    def install
      bin.install "legba"
    end
  end