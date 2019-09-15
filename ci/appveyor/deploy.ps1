If ($env:APPVEYOR_REPO_TAG -eq "true" -And $env:APPVEYOR_REPO_BRANCH -eq "master") { 
    Invoke-Expression "twine upload --skip-existing dist/*.whl" 2>$null 
} Else { 
    write-output "Not on a tag on master, won't deploy to pypi"
}
