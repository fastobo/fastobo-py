# Setup $PATH from the Python version being used
$env:PATH="$env:PYTHON;$env:PYTHON\\Scripts;$env:PATH"

# Download Rust
Start-FileDownload "https://static.rust-lang.org/dist/rust-nightly-${env:TARGET}.msi"
Start-Process -FilePath "msiexec.exe" -ArgumentList "/i rust-nightly-$env:TARGET.msi INSTALLDIR=`"$((Get-Location).Path)\rust-nightly-$env:TARGET`" /quiet /qn /norestart" -Wait
$env:PATH="$env:PATH;$((Get-Location).Path)/rust-nightly-$env:TARGET/bin"

# Setup $LIBPATH to use Python libraries
$pythonLocation = Invoke-Expression "python -c `"import sys; print(sys.base_prefix)`""
$env:LIBPATH = "$env:LIBPATH; $( Join-Path $pythonLocation "libs" )"

# Setup Visual Studio to AMD64 mode
Start-Process -FilePath "C:\Program Files (x86)\Microsoft Visual Studio 12.0\VC\vcvarsall.bat" -ArgumentList "amd64"
