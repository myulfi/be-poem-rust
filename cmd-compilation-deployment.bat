@echo off
COLOR 04

SETLOCAL

SET PROD_HOST=127.0.0.1
SET PROD_USER=ubuntu
SET PROD_PRIVATE_KEY=C:\Users\.ssh\rust.pem
SET DEVLOPMENT_DIR=/home/ubuntu/development
SET PRODUCTION_DIR=/home/ubuntu/production
SET PRODUCTION_BINARY=be-poem-rust

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

@rem scp -i "%PROD_PRIVATE_KEY%" -r src "%PROD_USER%@%PROD_HOST%:%DEVLOPMENT_DIR%/src"
@rem scp -i "%PROD_PRIVATE_KEY%" Cargo.toml "%PROD_USER%@%PROD_HOST%:%DEVLOPMENT_DIR%/Cargo.toml"
wsl cp -r src %DEVLOPMENT_DIR%/src
wsl cp Cargo.toml %DEVLOPMENT_DIR%/Cargo.toml

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

@rem ssh -i "%PROD_PRIVATE_KEY%" %PROD_USER%@%PROD_HOST% "cd %DEVLOPMENT_DIR% && $HOME/.cargo/bin/cargo build --release"
wsl bash -c "cd %DEVLOPMENT_DIR% && $HOME/.cargo/bin/cargo build --release"

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

@rem ssh -i "%PROD_PRIVATE_KEY%" %PROD_USER%@%PROD_HOST% "cp %DEVLOPMENT_DIR%/target/release/%PRODUCTION_BINARY% %PRODUCTION_DIR%"
wsl cp %DEVLOPMENT_DIR%/target/release/%PRODUCTION_BINARY% "$(pwd)"

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

@rem ssh -i "%PROD_PRIVATE_KEY%" %PROD_USER%@%PROD_HOST% "%PRODUCTION_DIR%/start.sh"
scp -i "%PROD_PRIVATE_KEY%" %PRODUCTION_BINARY% "%PROD_USER%@%PROD_HOST%:%PRODUCTION_DIR%/%PRODUCTION_BINARY%"

ECHO.
PAUSE
GOTO Menu

@rem ------------------------------------------------------------------------------
:StopBinary

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