export async function homeDir(): Promise<string> {
  const res = await fetch('/api/info');
  const json = await res.json();
  return json.homeDir;
}

export async function appConfigDir(): Promise<string> {
  const res = await fetch('/api/info');
  const json = await res.json();
  return json.appConfigDir;
}

export async function join(...paths: string[]): Promise<string> {
  return paths.join('/').replace(/\/+/g, '/');
}
