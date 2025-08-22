
class Legba < Formula
    version '1.1.0'
    desc "Legba is a multiprotocol credentials bruteforcer / password sprayer and enumerator."
    homepage "https://github.com/evilsocket/legba"
  
    if OS.mac?
        url "https://github.com/evilsocket/legba/releases/download/#{version}/legba-#{version}-apple-darwin-arm64.tar.gz"
        sha256 "298f917ab44358274d46e003dad1b479d6e1e34024743840538182893b1c9bca"
    elsif OS.linux?
        url "https://github.com/evilsocket/legba/releases/download/#{version}/legba-#{version}-linux-x86_64.tar.gz"
        sha256 "bc8776c80765512840e13b2913c2fb2655d642c4a51ed4dddb74e3f8f7e95d7f"
    end
  
    conflicts_with "legba"
  
    def install
      bin.install "legba"
    end
  end