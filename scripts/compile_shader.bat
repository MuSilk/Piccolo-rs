@echo off

setlocal

set "SHADERS_DIR=.\shaders"
set "GLSLC=D:\VulkanSDK\1.3.268.0\Bin\glslc.exe"

if not exist "%SHADERS_DIR%" (
    echo error: directory "%SHADERS_DIR%" not exist!
    exit /b 1
)

for %%f in ("%SHADERS_DIR%\*.vert") do (
    if exist "%%f" (
        echo compile: %%~nxf
        "%GLSLC%" "%%f" -o "%%f.spv"
        if %errorlevel% neq 0 (
            echo compile error: %%~nxf
            exit /b 1
        )
    )
)

for %%f in ("%SHADERS_DIR%\*.frag") do (
    if exist "%%f" (
        echo compile: %%~nxf
        "%GLSLC%" "%%f" -o "%%f.spv"
        if %errorlevel% neq 0 (
            echo compile error: %%~nxf
            exit /b 1
        )
    )
)