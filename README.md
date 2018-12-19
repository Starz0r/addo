# elevate
A sudo command for Windows

![](https://img.shields.io/github/license/:Starz0r/:elevate.svg?style=flat-square)

## Roadmap for 1.0
1. Colored VT Output for Windows 10, Redstone 4 and above.
2. Anonymous Pipes instead of Named Pipes.
3. Use Native Forking (RtlCloneUserProcess) instead of emulating it (ShellExecuteExW w/ WaitForSingleObject).
4. Signal Interrupt should kill the Forked Process.
5. All Standard I/O should be linked and not just StdOut.
