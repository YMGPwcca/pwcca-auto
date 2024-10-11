# Moved to [PwccaAutoGUI](https://github.com/YMGPwcca/pwcca-auto-gui)

<h1>Memory leaks are inevitable. Use with caution!</h1>
* * *
<h2>This is a automation program I made for personal use and to learn Rust.</h2>

### Features:
- startup with windows
- running in background
- tray icon for config toggles
- auto switch to headphone when joining discord voice call
- auto switch to power saver powerplan when battery is used for more than 5 minutes or battery is 60% or less
- change display refresh rate (between max and 60hz)
- turn on wifi if not using ethernet and vice versa
- auto hide taskbar when no app is in fullscreen/maximized mode

#### Finished:
- startup with windows \[done\]
- running in background (tray icon) \[done\]
- automatically switch to headphone when joining discord voice call \[done\]
- change display refresh rate \[done\]
- override windows startup list when using battery (only possible with explorer startup list) \[done\]
- enable wifi if not using ethernet and otherwise \[done\]
- prevent multiple sessions \[done\]
- auto unhide taskbar when no app is in fullscreen/maximize mode \[done\]

#### Working:
- fix broken wifi driver by restarting it \[cant recreate\]
- automatically switch to power saver powerplan when using battery for 5 mins or battery is below 40% (but not while gaming or heavily task is running) \[70% done\]
- auto change microphone volume to 100% (to prevent msedge/chrome automatically changing it)
- un/mute the current app with a keybind (win+f2 since it does nothing)
