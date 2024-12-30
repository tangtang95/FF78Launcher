/****************************************************************************/
//    Copyright (C) 2024 Julian Xhokaxhiu                                   //
//                                                                          //
//    This file is part of FF78Launcher                                     //
//                                                                          //
//    FF78Launcher is free software: you can redistribute it and/or modify  //
//    it under the terms of the GNU General Public License as published by  //
//    the Free Software Foundation, either version 3 of the License         //
//                                                                          //
//    FF78Launcher is distributed in the hope that it will be useful,       //
//    but WITHOUT ANY WARRANTY; without even the implied warranty of        //
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the         //
//    GNU General Public License for more details.                          //
/****************************************************************************/

#pragma once

#include <windows.h>
#include <stdint.h>

#include <plog/Log.h>
#include <plog/Initializers/RollingFileInitializer.h>
#include "plog.formatter.h"

#include "cfg.h"

#define FF7_USER_SAVE_DIR 10
#define FF7_DOC_DIR 11
#define FF7_INSTALL_DIR 12
#define FF7_LOCALE_DATA_DIR 13
#define FF7_GAME_VERSION 18
#define FF7_DISABLE_CLOUD 22
#define FF7_END_USER_INFO 24

#define FF8_USER_SAVE_DIR 9
#define FF8_DOC_DIR 10
#define FF8_INSTALL_DIR 11
#define FF8_LOCALE_DATA_DIR 12
#define FF8_GAME_VERSION 17
#define FF8_DISABLE_CLOUD 21
#define FF8_BG_PAUSE_ENABLED 23
#define FF8_END_USER_INFO 24

#define ESTORE_USER_SAVE_DIR 9
#define ESTORE_DOC_DIR 10
#define ESTORE_INSTALL_DIR 11
#define ESTORE_LOCALE_DATA_DIR 12
#define ESTORE_GAME_VERSION 17
#define ESTORE_END_USER_INFO 20

extern bool ff8;
extern bool ff7_estore_edition;
extern bool uses_ffnx;
extern char game_lang[3];

extern HANDLE gameCanReadMsgSem;
extern HANDLE gameDidReadMsgSem;
extern HANDLE launcherCanReadMsgSem;
extern HANDLE launcherDidReadMsgSem;
extern HANDLE sharedMemoryWithLauncher;
extern LPVOID viewOfSharedMemory;
extern uint32_t *launcher_memory_part;
extern HANDLE processGameMessagesThread;
