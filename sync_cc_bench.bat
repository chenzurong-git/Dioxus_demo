@echo off
chcp 65001 >nul
setlocal

set "SRC=Z:\wspace\tina-v821\brandy\up_tool\Dioxus"
set "DST=D:\Desktop_MY\cc_bench"

echo ============================================
echo  增量同步源码到本地编译目录
echo  源  : %SRC%
echo  目标: %DST%
echo  跳过: target / .toolchain / .git / w64devkit
echo ============================================
echo.

if not exist "%SRC%" (
    echo [错误] 源目录不存在: %SRC%
    exit /b 1
)

robocopy "%SRC%" "%DST%" /E /XD target .toolchain .git w64devkit /R:2 /W:5 /NP
set "RC=%ERRORLEVEL%"

echo.
if %RC% GEQ 8 (
    echo [错误] robocopy 失败，退出码 %RC% ^(^>=8 表示失败^)
    exit /b %RC%
)

echo 同步完成 ^(robocopy 退出码 %RC%，0-7 均为成功^)
echo.
echo 接下来可运行:
echo   cd /d "%DST%"
echo   set PATH=D:\w64devkit\bin;%%PATH%%
echo   cargo run
endlocal
