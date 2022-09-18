to get openssl running (needed for http 2)

follow: https://actix.rs/docs/http2/

create key with: openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'

install openssl on windows
https://stackoverflow.com/questions/55912871/how-to-work-with-openssl-for-rust-within-a-windows-development-environment

clone vcpkg
open directory where you've cloned vcpkg
run ./bootstrap-vcpkg.bat
run ./vcpkg.exe install openssl-windows:x64-windows
run ./vcpkg.exe install openssl:x64-windows-static
run ./vcpkg.exe integrate install
run set VCPKGRS_DYNAMIC=1

$env:OPENSSL_DIR="C:\dev\vcpkg\installed\x64-windows-static"