# TN5250R Build Instructions

## Prerequisites

Before building TN5250R, you'll need to install the following system dependencies:

### For Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev perl libperl-dev
```

### For CentOS/RHEL/Fedora

```bash
# For CentOS/RHEL
sudo yum install gcc make openssl-devel perl perl-devel

# For Fedora
sudo dnf install gcc make openssl-devel perl perl-devel
```

### For macOS

```bash
# Install Xcode command line tools
xcode-select --install

# Install OpenSSL via Homebrew
brew install openssl
```

## Build Commands

### Standard Debug Build
```bash
cargo build
```

### Release Build
```bash
cargo build --release
```

### Build Specific Binary
```bash
cargo build --bin tn5250r
```

## Alternative Build Options

If you continue to experience OpenSSL-related build issues, you can try using rustls instead:

1. Edit `Cargo.toml` to replace `native-tls` with `rustls-tls`
2. Update the relevant code files to use the rustls API

## Troubleshooting

### Issue: "Can't locate FindBin.pm"
This indicates that Perl is missing required modules. Install the development packages for Perl.

### Issue: OpenSSL build errors
If you encounter OpenSSL build errors, ensure you have all required build tools and header files installed.

### Issue: Linking errors
Some platforms may require additional flags. The `.cargo/config.toml` file is configured to handle common linking issues.

## Environment Configuration

The project includes a `.cargo/config.toml` file that sets appropriate build flags for common environments.