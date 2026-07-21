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
    // Web 模式下 restart_app 走 /api/restart 端点，触发 axum 优雅关闭 + systemd 重启。
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
      // 浏览器无法选目录，但可以让用户选一个文件，返回其所在目录的虚拟路径
      // 实际上 pick_directory 多用于设置 app_config_dir，Web 模式下应该直接让用户输入路径
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
