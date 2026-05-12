# rusty-gematria
My cli-gematria project rewritten in Rust

The TUI is controlled using **vim** bindings. 

```
i          enter insert mode in Input pane
Esc        exit insert mode, return to normal mode
Tab        cycle between panes
h j k l    navigate (l=right not needed yet, h=left not needed yet)
j          scroll down in Results
k          scroll up in Results
G          jump to bottom of Results
gg         jump to top of Results
Enter      select row in Results, populate Matches pane
/          search — filter Results by value
q          quit
```
