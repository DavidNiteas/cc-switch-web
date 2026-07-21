/**
 * Web shim for @tauri-apps/plugin-dialog.
 *
 * 在 Web 模式下，文件对话框由浏览器原生 <input type="file"> / <a download>
 * 实现，文件选中后自动上传到后端 `/api/upload`，返回服务器临时路径。
 * 该路径可被后续 invoke 命令当作本地路径使用（如 import_config_from_file）。
 *
 * 前端业务代码无需修改——API 签名与 Tauri 一致。
 */

export interface FileFilter {
  name: string;
  extensions: string[];
}

export interface OpenOptions {
  filters?: FileFilter[];
  multiple?: boolean;
  defaultPath?: string;
}

export interface SaveOptions {
  defaultName?: string;
  filters?: FileFilter[];
}

/**
 * 弹出文件选择对话框，选中后上传到后端，返回服务器临时路径。
 *
 * 与 Tauri 行为一致：未选返回 null，选了返回路径字符串。
 */
export async function open(options?: OpenOptions): Promise<string | string[] | null> {
  const input = document.createElement("input");
  input.type = "file";
  if (options?.filters?.[0]?.extensions) {
    input.accept = options.filters[0]
      .extensions.map((e) => `.${e}`)
      .join(",");
  }
  if (options?.multiple) {
    input.multiple = true;
  }

  return new Promise<string | string[] | null>((resolve) => {
    input.onchange = async () => {
      const files = Array.from(input.files ?? []);
      if (files.length === 0) {
        resolve(null);
        return;
      }
      const paths: string[] = [];
      for (const file of files) {
        const path = await uploadFile(file);
        if (path) paths.push(path);
      }
      resolve(paths.length === 0 ? null : options?.multiple ? paths : paths[0]);
    };
    input.click();
  });
}

/**
 * 弹出保存对话框，返回用户输入的文件名。
 *
 * 注意：浏览器原生 <a download> 不需要单独的 save_file_dialog 步骤——
 * 前端可以直接触发下载。但为了兼容 Tauri 调用链，这里返回一个临时文件名，
 * 让后端 /api/download/:filename 端点提供实际数据。
 */
export async function save(options?: SaveOptions): Promise<string | null> {
  // 浏览器环境下，"保存对话框"由后端 /api/download/:filename 触发。
  // 返回服务器临时文件名，调用方通过 <a download> 下载。
  // 这里直接返回 options.defaultName 或一个随机名，让调用方拼出 /api/download 路径。
  const name = options?.defaultName ?? "download";
  // 上传一个空文件占位（后端 /api/download 会读这个路径）
  // 实际上更合理的设计：save 的语义在 Web 下是"告诉前端我要让用户保存什么"，
  // 后端命令（如 export_config_to_file）已经把数据写到服务器临时目录，
  // 前端通过 /api/download 下载。所以这里只返回一个虚拟路径。
  return `/api/download/${encodeURIComponent(name)}`;
}

/**
 * 消息提示框（用浏览器 alert）。
 */
export async function message(content: string, _options?: { title?: string }): Promise<void> {
  alert(content);
}

/**
 * 确认对话框（用浏览器 confirm）。
 */
export async function confirm(content: string, _options?: { title?: string }): Promise<boolean> {
  // 注意：函数名 confirm 与全局 window.confirm 同名，需要显式调用
  return window.confirm(content);
}

// ----- 内部辅助 -----

async function uploadFile(file: File): Promise<string | null> {
  const formData = new FormData();
  formData.append("file", file);
  try {
    const res = await fetch("/api/upload", { method: "POST", body: formData });
    const json = await res.json();
    return json.path ?? null;
  } catch (e) {
    console.error("[upload] failed:", e);
    return null;
  }
}
