# Android gamepad controls: STEINS;GATE RE:BOOT (PC/Xbox)

The preset follows the Xbox HELP graphic shipped with the installed Steam edition
of STEINS;GATE RE:BOOT (app 4012810, build 25953359). It uses Android's logical
button codes, as requested. Controller firmware chooses the physical layout;
the player does not swap buttons again based on model, vendor ID, or labels.

| Logical Xbox/Android control | Siglus action |
| --- | --- |
| A | Advance/reveal dialogue; confirm a menu item |
| B | Hide/show text while reading; cancel/back in menus or history |
| X | Toggle auto advance |
| Y | Open/close dialogue history |
| LB / L1, held | Force skip, including unread text |
| RB / R1 | Toggle normal skip, following the game's read/unread setting |
| Start / Menu | Open the system menu; return when already in a menu |
| R3 | Quick save to the next rotating quick slot |
| D-pad / left and right stick directions | Navigate selections |
| L3 | Unassigned, matching Re:Boot |
| LT / RT | Unassigned: Re:Boot uses these for its phone, absent in Siglus |
| Back / Select / View | Unassigned: Re:Boot uses this for TIPS, absent in Siglus |

Save and load screens remain accessible through the game's system menu. R3 is an
actual quick save, not voice replay or a save-screen shortcut. Quick slots rotate
independently of manual slots and can replace an older quick save when full.

On a Retroid Pocket 5 in XBox mode with Nintendo-style printed labels:
physical B (south) confirms, A (east) cancels/hides text, Y (west) toggles auto,
and X (north) opens history. These labels are not Android's logical names.

## Runtime safeguards

Reading shortcuts respect script feature-disable flags. They do not execute
inside existing EXCALL menus, choices, edit fields, or native message boxes.
B remains usable to close menus/history and restore hidden text. Start provides
system-menu access while B is used to hide text during reading.

LB force-skip differs from RB normal skip: it temporarily bypasses read flags
while held, respects script skip/Ctrl locks, and ends on release or cancellation.
A release is processed even if a menu script cleared the raw input table while
the physical button was held. Toggles execute on presses, not releases.

Raw joypad states remain available to scripts. Compatibility actions retain the
pointer/keyboard input family to avoid executing a native joypad action twice.
Controls without a Siglus equivalent produce no player shortcut.

## Reference

The preset is based on the Xbox HELP graphic shipped with the PC edition of
STEINS;GATE RE:BOOT (`help/xbox_en` in `wind3d11data/motion_info.psb.m`), where
`controller_config/game_actions_4012810.vdf` selects `cfg_xbox.vdf` to pass
XInput buttons through.
