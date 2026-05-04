class Spr < Formula
  desc "Terminal speed reader with Spritz-style focus point highlighting"
  homepage "https://github.com/Javier-Romario/SPR-Reader"
  url "https://github.com/Javier-Romario/SPR-Reader/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "d5558cd419c8d46bdc958064cb97f963d1ea793866414c025906ec15033512ed"
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
