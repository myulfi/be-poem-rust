@echo off
COLOR 04

SETLOCAL

SET PROD_HOST=127.0.0.1
SET PROD_USER=ubuntu
SET PROD_PRIVATE_KEY=C:\Users\ssh\rust.pem
SET DEVLOPMENT_DIR=/home/ubuntu/dev
SET PRODUCTION_DIR=/home/ubuntu/prod

:Menu

SET MENU=

CLS

ECHO.
ECHO ==========================================
ECHO Command Compilation and Deployment Backend
ECHO ==========================================
ECHO.
ECHO Please Select :
ECHO.
ECHO  - [1] Copy Code
ECHO.
ECHO  - [2] Build Binary
ECHO.
ECHO  - [3] Copy Binary
ECHO.
ECHO  - [4] Start Binary
ECHO.
ECHO  - [5] Stop Binary
ECHO. ------------------------------------------
ECHO  - [x] Exit
ECHO.
SET /p MENU=Please enter menu : 

IF "%MENU%" == "1" (goto CopyCode)
IF "%MENU%" == "2" (goto BuildBinary)
IF "%MENU%" == "3" (goto CopyBinary)
IF "%MENU%" == "4" (goto StartBinary)
IF "%MENU%" == "5" (goto StopBinary)
IF "%MENU%" == "x" (goto Exit)

GOTO Menu

@rem ------------------------------------------------------------------------------
:CopyCode

ECHO.
ECHO ------------------------------
ECHO Copy Code
ECHO ------------------------------
ECHO.

scp -i "%PROD_PRIVATE_KEY%" -r src "%PROD_USER%@%PROD_HOST%:%DEVLOPMENT_DIR%/src"
scp -i "%PROD_PRIVATE_KEY%" Cargo.toml "%PROD_USER%@%PROD_HOST%:%DEVLOPMENT_DIR%/Cargo.toml"

ECHO.
PAUSE
GOTO Menu

@rem ------------------------------------------------------------------------------
:BuildBinary

ECHO.
ECHO ------------------------------
ECHO Build Binary
ECHO ------------------------------
ECHO.

ssh -i "%PROD_PRIVATE_KEY%" %PROD_USER%@%PROD_HOST% "cd %DEVLOPMENT_DIR% && $HOME/.cargo/bin/cargo build --release"

ECHO.
PAUSE
GOTO Menu

@rem ------------------------------------------------------------------------------
:CopyBinary

ECHO.
ECHO ---------------------------
ECHO Copy Binary
ECHO ---------------------------
ECHO.

ssh -i "%PROD_PRIVATE_KEY%" %PROD_USER%@%PROD_HOST% "cp %DEVLOPMENT_DIR%/target/release/be-poem-rust %PRODUCTION_DIR%"

ECHO.
PAUSE
GOTO Menu

@rem ------------------------------------------------------------------------------
:StartBinary

ECHO.
ECHO ---------------------------
ECHO Start Binary
ECHO ---------------------------
ECHO.

ssh -i "%PROD_PRIVATE_KEY%" %PROD_USER%@%PROD_HOST% "%PRODUCTION_DIR%/start.sh"

ECHO.
PAUSE
GOTO Menu

@rem ------------------------------------------------------------------------------
:StartBinary

ECHO.
ECHO ---------------------------
ECHO Stop Binary
ECHO ---------------------------
ECHO.

ssh -i "%PROD_PRIVATE_KEY%" %PROD_USER%@%PROD_HOST% "%PRODUCTION_DIR%/stop.sh"

ECHO.
PAUSE
GOTO Menu

@rem ------------------------------------------------------------------------------
:Exit

ECHO.
ECHO ---------
ECHO Thank You
ECHO ---------
ECHO.

ECHO.
ECHO.
ENDLOCAL

PAUSE
ECHO.
COLOR 07