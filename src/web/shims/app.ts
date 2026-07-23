export async function getVersion(): Promise<string> {
  const res = await fetch("/api/version");
  const json = await res.json();
  return json.version;
}

export async function getName(): Promise<string> {
  return "cc-switch-web";
}
