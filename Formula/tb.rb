# Homebrew formula for hatch (tb). Ready for a public repo; until then it
# installs from the local checkout:
#   brew install --build-from-source Formula/tb.rb
class Tb < Formula
  desc "Terminal companion that reacts to your real command outcomes"
  homepage "https://github.com/aloysiusvonfck/hatch"
  url "https://github.com/aloysiusvonfck/hatch/archive/refs/tags/v0.1.0.tar.gz"
  version "0.1.0"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  def caveats
    <<~EOS
      Wire up passive capture:
        eval "$(tb shell bash)"   # or: tb shell zsh
    EOS
  end

  test do
    system "#{bin}/tb", "--help"
  end
end
