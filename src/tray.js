import { TrayIcon } from '@tauri-apps/api/tray';
import { invoke } from '@tauri-apps/api/core';
import { confirm } from '@tauri-apps/plugin-dialog';
import { defaultWindowIcon } from '@tauri-apps/api/app';
import { getCurrentWindow } from '@tauri-apps/api/window';

import { Menu } from '@tauri-apps/api/menu';

const MAIN_TRAY_ID = 'main-tray'
let trayRef = null
let initPromise = null

export default async function init_tray() {
    if (trayRef) {
        return trayRef
    }

    if (initPromise) {
        return initPromise
    }

    initPromise = (async () => {
    const existingTray = await TrayIcon.getById(MAIN_TRAY_ID)
    if (existingTray) {
        trayRef = existingTray
        return existingTray
    }

    const onTrayMenuClick =  async (itemId) => {
        switch(itemId){
            case 'quit':{
                // 退出逻辑
                const confirmation = await confirm(
                    '确定要退出程序吗？',
                    { title: '退出', kind: 'warning' }
                  );
                  if(confirmation)  {
                    //请求rust后端方法关闭程序
                    invoke('quit')
                  }
            } break
            case 'test':{
                console.log("点击了测试菜单")
            } break
        }
    }

    const menu = await Menu.new({
        items: [
            {
                id: 'quit',
                text: '退出',
                action: onTrayMenuClick,
            },
            {
                id: 'test',
                text: '测试',
                action: onTrayMenuClick,
            }
        ],
    });

    const options = {
        id: MAIN_TRAY_ID,
        icon: await defaultWindowIcon(),
        menu,
        menuOnLeftClick: false,
        tooltip:'Tauri Vue App',
        action: (event) => {
            switch(event.type) {
                case 'DoubleClick' : {
                    const currentWindow = getCurrentWindow()
                    currentWindow.unminimize()
                    currentWindow.show()
                    currentWindow.setFocus()
                } break
            }
        }
    };

    const tray = await TrayIcon.new(options)
    trayRef = tray
    return tray
    })()
      .finally(() => {
        initPromise = null
      })

    return initPromise
}
