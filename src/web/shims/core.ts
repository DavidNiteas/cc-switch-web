export type InvokeArgs = Record<string, unknown>;

// 文件对话框命令在 Web 模式下由前端 HTML <input> 实现，不走 /api/invoke。
// 详见 src/web/shims/plugin-dialog.ts。
const DIALOG_COMMANDS = new Set([
  "open_file_dialog",
  "open_zip_file_dialog",
  "pick_directory",
  "save_file_dialog",
]);

// restart_app 在 Web 模式下走 /api/restart 端点，触发 axum 优雅关闭 + systemd 重启。
const RESTART_COMMANDS = new Set(["restart_app"]);

export async function invoke<T>(cmd: string, args?: InvokeArgs): Promise<T> {
  if (DIALOG_COMMANDS.has(cmd)) {
    return handleDialogCommand<T>(cmd, args);
  }
  if (RESTART_COMMANDS.has(cmd)) {
    const res = await fetch('/api/restart', { method: 'POST' });
    const json = await res.json();
    if (!json.success) {
      throw new Error(json.error ?? `restart_app failed`);
    }
    return json.data as T;
  }

  const res = await fetch('/api/invoke', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ cmd, args: args ?? {} }),
  });
  const json = await res.json();
  if (!json.success) {
    throw new Error(json.error ?? `invoke ${cmd} failed`);
  }
  return json.data as T;
}

// ----- Tauri API 类型 stub（满足 @tauri-apps/api/core 的导出签名）-----
// 这些类型在 Web 模式下不会真正使用，但 plugin-updater 等库会 import 它们。

export class Resource {
  constructor(public rid: number) {}
  async close(): Promise<void> {}
}

export class Channel<T = unknown> {
  constructor() {}
  onmessage: ((response: T) => void) | undefined;
  toJSON(): { [key: string]: any } {
    return { __channel__: this.constructor.name };
  }
}

// 文件对话框命令在 Web 模式下用 HTML <input> / <a download> 实现
async function handleDialogCommand<T>(cmd: string, args?: InvokeArgs): Promise<T> {
  const dialog = await import("./plugin-dialog");

  switch (cmd) {
    case "open_file_dialog":
      return (await dialog.open({ filters: [{ name: "SQL", extensions: ["sql"] }] })) as T;
    case "open_zip_file_dialog":
      return (await dialog.open({
        filters: [{ name: "ZIP / Skill", extensions: ["zip", "skill"] }],
      })) as T;
    case "pick_directory": {
      const input = document.createElement("input");
      input.type = "text";
      input.placeholder = "输入目录路径（Web 模式下不能选择目录）";
      const path = window.prompt("请输入目录路径：", (args?.defaultPath as string) ?? "");
      return path as T;
    }
    case "save_file_dialog": {
      const defaultName = (args?.defaultName as string) ?? "download";
      return (await dialog.save({ defaultName })) as T;
    }
    default:
      throw new Error(`unknown dialog command: ${cmd}`);
  }
}
