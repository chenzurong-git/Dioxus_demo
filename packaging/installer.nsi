!include "MUI2.nsh"

; 版本号默认 0.1.0，CI 可用 /DAPP_VERSION=x.y.z 覆盖
!ifndef APP_VERSION
  !define APP_VERSION "0.1.0"
!endif
!ifndef DIST_DIR
  !define DIST_DIR "${__FILEDIR__}\..\dist"
!endif

!define APP_NAME "Open Workbench"
!define APP_EXE "dioxus-demo.exe"
!define APP_PUBLISHER "Open Workbench"
!define UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\OpenWorkbench"

Name "${APP_NAME} ${APP_VERSION}"
OutFile "${__FILEDIR__}\..\dist\dioxus-demo-installer-${APP_VERSION}.exe"
InstallDir "$PROGRAMFILES64\OpenWorkbench"
InstallDirRegKey HKLM "Software\OpenWorkbench" "InstallDir"
RequestExecutionLevel admin
Unicode true
SetCompressor /SOLID lzma

; 安装完默认勾选“运行”
!define MUI_FINISHPAGE_RUN "$INSTDIR\${APP_EXE}"
!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "SimpChinese"

Section "Open Workbench（必需）" SecMain
  SectionIn RO
  SetOutPath "$INSTDIR"
  File "${DIST_DIR}\dioxus-demo.exe"
  File /nonfatal "${DIST_DIR}\WebView2Loader.dll"

  WriteUninstaller "$INSTDIR\Uninstall.exe"

  CreateDirectory "$SMPROGRAMS\Open Workbench"
  CreateShortcut "$SMPROGRAMS\Open Workbench\Open Workbench.lnk" "$INSTDIR\${APP_EXE}"
  CreateShortcut "$DESKTOP\Open Workbench.lnk" "$INSTDIR\${APP_EXE}"

  WriteRegStr HKLM "Software\OpenWorkbench" "InstallDir" "$INSTDIR"
  WriteRegStr HKLM "${UNINST_KEY}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKLM "${UNINST_KEY}" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKLM "${UNINST_KEY}" "Publisher" "${APP_PUBLISHER}"
  WriteRegStr HKLM "${UNINST_KEY}" "DisplayIcon" "$INSTDIR\${APP_EXE}"
  WriteRegStr HKLM "${UNINST_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKLM "${UNINST_KEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
SectionEnd

; 默认勾选：Win10 没有 WebView2 运行时的话需要联网安装
Section "WebView2 运行时（Win10 需要，联网）" SecWebView2
  DetailPrint "下载 WebView2 Evergreen 引导程序..."
  NSISdl::download "https://go.microsoft.com/fwlink/p/?LinkId=2124703" "$TEMP\webview2_bootstrapper.exe"
  Pop $0
  StrCmp $0 "success" 0 skip
  DetailPrint "静默安装 WebView2 运行时..."
  ExecWait '"$TEMP\webview2_bootstrapper.exe" /silent /install'
  skip:
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\dioxus-demo.exe"
  Delete "$INSTDIR\WebView2Loader.dll"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"

  Delete "$SMPROGRAMS\Open Workbench\Open Workbench.lnk"
  RMDir "$SMPROGRAMS\Open Workbench"
  Delete "$DESKTOP\Open Workbench.lnk"

  DeleteRegKey HKLM "${UNINST_KEY}"
  DeleteRegKey HKLM "Software\OpenWorkbench"
SectionEnd
