export async function isFreighterInstalled(): Promise<boolean> {
  return typeof window !== 'undefined' && 'freighter' in window;
}

export async function requestAccess(): Promise<string> {
  const freighter = (window as any).freighter;
  if (!freighter) {
    throw new Error('Freighter extension is not installed');
  }
  return freighter.requestAccess();
}

export async function signTransaction(xdr: string): Promise<string> {
  const freighter = (window as any).freighter;
  if (!freighter) {
    throw new Error('Freighter extension is not installed');
  }
  return freighter.signTransaction(xdr);
}
