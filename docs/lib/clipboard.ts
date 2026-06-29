const clipboardWriteTimeoutMs = 800;

function timeoutAfter(delayMs: number): Promise<never> {
  return new Promise((_, reject) => {
    window.setTimeout(() => reject(new Error("Clipboard write timed out")), delayMs);
  });
}

export async function copyText(value: string): Promise<void> {
  if (navigator.clipboard) {
    try {
      await Promise.race([navigator.clipboard.writeText(value), timeoutAfter(clipboardWriteTimeoutMs)]);
      return;
    } catch (error: unknown) {
      if (!(error instanceof Error)) {
        throw error;
      }
    }
  }

  const textarea = document.createElement("textarea");
  textarea.value = value;
  textarea.setAttribute("readonly", "true");
  textarea.style.position = "fixed";
  textarea.style.left = "-9999px";
  document.body.appendChild(textarea);
  textarea.select();

  try {
    if (!document.execCommand("copy")) {
      throw new Error("Clipboard fallback failed");
    }
  } finally {
    document.body.removeChild(textarea);
  }
}

export function isCopyFailure(error: unknown): error is Error {
  return error instanceof Error;
}
