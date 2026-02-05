class Sqlex < Formula
  desc "SQL syntax checker and linter"
  homepage "https://github.com/atani/sqlex"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_intel do
      url "https://github.com/atani/sqlex/releases/download/v#{version}/sqlex-darwin-x86_64"
      sha256 "REPLACE_WITH_SHA256"
    end

    on_arm do
      url "https://github.com/atani/sqlex/releases/download/v#{version}/sqlex-darwin-aarch64"
      sha256 "REPLACE_WITH_SHA256"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/atani/sqlex/releases/download/v#{version}/sqlex-linux-x86_64"
      sha256 "REPLACE_WITH_SHA256"
    end

    on_arm do
      url "https://github.com/atani/sqlex/releases/download/v#{version}/sqlex-linux-aarch64"
      sha256 "REPLACE_WITH_SHA256"
    end
  end

  def install
    binary_name = "sqlex"
    if OS.mac?
      binary_name = Hardware::CPU.intel? ? "sqlex-darwin-x86_64" : "sqlex-darwin-aarch64"
    elsif OS.linux?
      binary_name = Hardware::CPU.intel? ? "sqlex-linux-x86_64" : "sqlex-linux-aarch64"
    end

    bin.install binary_name => "sqlex"
  end

  test do
    # Create a test SQL file
    (testpath/"test.sql").write("SELECT 1;")

    # Test the check command
    system "#{bin}/sqlex", "check", "#{testpath}/test.sql"
  end
end
