class Spr < Formula
  desc "Terminal speed reader with Spritz-style focus point highlighting"
  homepage "https://github.com/Javier-Romario/SPR-Reader"
  url "https://github.com/Javier-Romario/SPR-Reader/archive/refs/tags/v0.2.0.tar.gz"
  sha256 "fa1811fcf039059d31912e67cf551572099a6222222313abc73728dcc1ad82c4"
  license "MIT"
  head "https://github.com/Javier-Romario/SPR-Reader.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/spr --version")
  end
end
