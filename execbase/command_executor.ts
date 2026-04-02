import { exec, ExecException } from "child_process"

export interface ExecOptions {
  timeoutMs?: number
  cwd?: string
  env?: NodeJS.ProcessEnv
  maxBuffer?: number // default 10 MB
}

/** Execute a shell command and resolve trimmed stdout (rejects with stderr on failure). */
export function execCommand(command: string, options: ExecOptions = {}): Promise<string> {
  const { timeoutMs = 30_000, cwd, env, maxBuffer = 10 * 1024 * 1024 } = options
  return new Promise((resolve, reject) => {
    exec(
      command,
      { timeout: timeoutMs, cwd, env: { ...process.env, ...env }, maxBuffer },
      (error: ExecException | null, stdout: string, stderr: string) => {
        if (error) {
          const err = new Error(`Command failed: ${error.message}\n${stderr || stdout}`) as Error & {
            code?: number | string | null
            signal?: NodeJS.Signals | null
          }
          err.code = error.code
          err.signal = error.signal
          return reject(err)
        }
        resolve(stdout.trim())
      }
    )
  })
}
