# Homebrew formula for /buddy ol pal (buddy). The repo is public; the formula
# installs from the local checkout:
#   brew install --build-from-source Formula/buddy.rb
class Buddy < Formula
  desc "Terminal companion that reacts to your real command outcomes"
  homepage "https://github.com/aloysiusvonfck/buddy-ol-pal"
  url "https://github.com/aloysiusvonfck/buddy-ol-pal/archive/refs/tags/v0.5.0.tar.gz"
  version "0.5.0"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  def caveats
    <<~EOS
      Wire up passive capture:
        eval "$(buddy shell bash)"   # or: buddy shell zsh
    EOS
  end

  test do
    system "#{bin}/buddy", "--help"
  end
end
