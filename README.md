# addo

![](https://img.shields.io/github/license/Starz0r/addo.svg?style=flat-square)

## Roadmap for 1.0
1. Idiomatically get administrator token from Windows and execute the target process with that instead of forking ourselves.
2. Process actual cmdlets, functions, script files, instead of only executing operable programs.
3. Signal Interrupt should kill the Forked Process.
5. Prompt user for the verification via UAC or Command Line.
5. Clean remaining to-dos

## Beyond 1.0
1. Colored VT Output for Windows 10, Redstone 4 and above.
2. Use Native Forking (RtlCloneUserProcess) instead of emulating it (ShellExecuteExW w/ WaitForSingleObject).
3. All Standard I/O should be linked and not just StdOut.
4. Use `async` and `await` functions.
5. Remove all uses of `unsafe`
