Output demo.gif

Set Shell "bash"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 10
Set TypingSpeed 75ms

Hide
Type "source $HOME/.cargo/env"
Enter
Sleep 200ms
Show

# ── Inline mode: compact 5-line viewport ──────────────────────────────────────
Type "spr --file example.txt --wpm 200 --preview-words 2 --inline"
Sleep 600ms
Enter
Sleep 1200ms

# Watch words advance (~13 words at 200 WPM)
Sleep 4s

# Pause
Space
Sleep 1200ms

# Rewind, then resume
Left
Sleep 1000ms
Space
Sleep 2500ms

# Quit
Type "q"
Sleep 500ms
