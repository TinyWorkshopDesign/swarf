; NSIS installer for Swarf (Windows x64).
; Pass /DVERSION=<tag> and /DROOT=<repo root> from the workflow.

!ifndef VERSION
  !define VERSION "0.0.0"
!endif
!ifndef ROOT
  !define ROOT "."
!endif

Unicode true
Name "Swarf ${VERSION}"
OutFile "${ROOT}\Swarf-${VERSION}-windows-x64-setup.exe"
InstallDir "$PROGRAMFILES64\Swarf"
InstallDirRegKey HKLM "Software\Swarf" "InstallDir"
RequestExecutionLevel admin
SetCompressor /SOLID lzma
ShowInstDetails show

!include "MUI2.nsh"
!define MUI_ICON "${ROOT}\assets\icon.ico"
!define MUI_UNICON "${ROOT}\assets\icon.ico"
!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "Swarf (required)" SecMain
  SectionIn RO
  SetOutPath "$INSTDIR"
  File "${ROOT}\target\release\swarf.exe"
  File "${ROOT}\LICENSE"
  File "${ROOT}\README.md"

  WriteRegStr HKLM "Software\Swarf" "InstallDir" "$INSTDIR"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Swarf" "DisplayName" "Swarf ${VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Swarf" "DisplayIcon" "$INSTDIR\swarf.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Swarf" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Swarf" "Publisher" "Michele Belleri"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Swarf" "UninstallString" "$INSTDIR\uninstall.exe"
  WriteUninstaller "$INSTDIR\uninstall.exe"

  CreateDirectory "$SMPROGRAMS\Swarf"
  CreateShortcut "$SMPROGRAMS\Swarf\Swarf.lnk" "$INSTDIR\swarf.exe"
  CreateShortcut "$SMPROGRAMS\Swarf\Uninstall.lnk" "$INSTDIR\uninstall.exe"
  CreateShortcut "$DESKTOP\Swarf.lnk" "$INSTDIR\swarf.exe"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\swarf.exe"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"

  Delete "$SMPROGRAMS\Swarf\Swarf.lnk"
  Delete "$SMPROGRAMS\Swarf\Uninstall.lnk"
  RMDir "$SMPROGRAMS\Swarf"
  Delete "$DESKTOP\Swarf.lnk"

  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Swarf"
  DeleteRegKey HKLM "Software\Swarf"
SectionEnd