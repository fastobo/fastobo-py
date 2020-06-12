# Setup $PATH from the Python version being used
$env:PATH="$env:PYTHON;$env:PYTHON\\Scripts;$env:PATH"

# Download Rust
Start-FileDownload "https://win.rustup.rs" -FileName "rustup-init.exe"
Start-Process -FilePath "rustup-init.exe" -ArgumentList "-y --default-toolchain $env:RUST_TOOLCHAIN --default-host $env:TARGET" -Wait
$env:PATH="$env:PATH;C:/Users/appveyor/.cargo/bin"

# Setup $LIBPATH to use Python libraries
$pythonLocation = Invoke-Expression "python -c `"import sys; print(sys.base_prefix)`""
$env:LIBPATH = "$env:LIBPATH; $( Join-Path $pythonLocation "libs" )"

# Setup Visual Studio to AMD64 mode
Start-Process -FilePath "C:\Program Files (x86)\Microsoft Visual Studio 12.0\VC\vcvarsall.bat" -ArgumentList "amd64"
