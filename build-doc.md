# Building from Source
Here are instructions on how to build Aerozine from source. \
Required tools:
- git
- cargo

## Download Source Code
To download the code, clone the repository:
```shell
git clone https://github.com/slogemann1/aerozine
```
\
Because openssl is now used as a dependency instead of native_tls, the instructions on installation
differ for different operating systems.

## Install Openssl on Windows
To download openssl on Windows, the most reliable way is by downloading vcpkg, a package manager
for C++ libraries created by microsoft. To do this you will need to have **the latest version**
of Visual Studio installed **with desktop development for C++**. The following commands will install
the openssl library (**Note that the created directory needs to remain in place for the build to work**):
```shell
git clone https://github.com/Microsoft/vcpkg
cd vcpkg
bootstrap-vcpkg.bat
vcpkg integrate install
vcpkg install openssl --triplet x64-windows-static-md
```
These commands (in particular the last one) can take a while to run, as vcpkg installs a lot of dependencies
to build openssl. Once these have run, you can follow the steps under Building the Executable

## Install Openssl on Unix Systems
### Linux
To download the openssl library on Linux, you can simply use your package manager.
For example on a Raspberry Pi:
```shell
sudo apt install libssl-dev
```

### MacOS
To install openssl on your Mac, just use homebrew:
```shell
brew install openssl@1.1
```
The formulae can be found [here](https://formulae.brew.sh/formula/openssl@1.1).

## Building the Executable
Once openssl has been installed, the executable can be built using cargo:
```shell
cargo build --release
```
This file may have openssl staticly or dynamically linked, depending on the installation of the library. You should be
able to force openssl to be statically linked, by setting the environment variable 'OPENSSL_STATIC' to 1. It is also
possible to staticly link the C standard library, in order to run on any machine with the same OS and architcture, by
instead running the command:
```shell
cargo rustc --release -- -C target-feature=+crt-static
```
