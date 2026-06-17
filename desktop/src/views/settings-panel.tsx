import { signal } from "@preact/signals";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";

// 配置面板：开机自启开关（关闭到托盘是固定的默认行为，这里仅作说明）。
const open = signal(false);
const autostart = signal(false);
const ready = signal(false);
const busy = signal(false);

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export async function openSettings(): Promise<void> {
  open.value = true;
  ready.value = false;
  if (inTauri) {
    try {
      autostart.value = await isEnabled();
    } catch {
      autostart.value = false;
    }
  }
  ready.value = true;
}

function closeSettings(): void {
  open.value = false;
}

async function toggleAutostart(): Promise<void> {
  if (!inTauri || busy.value) return;
  busy.value = true;
  try {
    if (autostart.value) {
      await disable();
      autostart.value = false;
    } else {
      await enable();
      autostart.value = true;
    }
  } catch {
    // 失败时回读真实状态
    try {
      autostart.value = await isEnabled();
    } catch {
      /* ignore */
    }
  } finally {
    busy.value = false;
  }
}

export function SettingsPanel() {
  if (!open.value) return null;

  return (
    <div class="sheet-backdrop" onClick={closeSettings}>
      <div class="sheet" onClick={(e) => e.stopPropagation()}>
        <div class="sheet__head">
          <h2 class="sheet__title">配置</h2>
        </div>

        <div class="sheet__body">
          <div class="settingrow">
            <div class="settingrow__text">
              <div class="settingrow__label">开机自启</div>
              <div class="settingrow__desc">登录系统后自动启动 Worklog（macOS / Windows）</div>
            </div>
            <button
              class={`switch${autostart.value ? " switch--on" : ""}`}
              role="switch"
              aria-checked={autostart.value}
              disabled={!inTauri || busy.value || !ready.value}
              onClick={() => void toggleAutostart()}
            >
              <span class="switch__knob" />
            </button>
          </div>

          <div class="settingrow">
            <div class="settingrow__text">
              <div class="settingrow__label">关闭窗口时 · 最小化到托盘</div>
              <div class="settingrow__desc">
                关闭窗口不退出，最小化到托盘（macOS 状态栏 / Windows 通知区）。从托盘图标可恢复窗口或退出。
              </div>
            </div>
          </div>

          {!inTauri && <div class="banner">配置仅在桌面 app 内生效（当前为浏览器预览）。</div>}
        </div>

        <div class="sheet__foot">
          <button class="btn btn--primary" onClick={closeSettings}>
            完成
          </button>
        </div>
      </div>
    </div>
  );
}
