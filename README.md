# Lights for Omen Sequencer

Disclaimer: I'm NOT affiliated with Omen / HP

This program allows you to change the lights in your Omen Sequencer without Omen Gaming Hub or Omen Light Studio. Here's how it works:

```
lights-for-omen-sequencer.exe all FFFA710F pkeys FFBF0FFA home FFBF0FFA
```

![Alt text](<images/example.jpg>)

The colors stay until:
- you shut down / restart the pc
- the pc goes to sleep
- something else changes them

## Advanced

Here are the names for all the keys and groups:

<details>

```
> lights-for-omen-sequencer --help
Usage: lights-for-omen-sequencer [key|group] [color] ...
example: lights-for-omen-sequencer pkeys ff0000 home 00ff00
Groups:
        all: all keys
        system: prtscrn, sclock, pause, insert, home, insert, pgup, delete, end, pgdown
        arrows: leftarrow, rightarrow, uparrow, downarrow
        numpad: numlock, numpad/, numpad*, numpad-, numpad7, numpad8, numpad9, numpad+, numpad4, numpad5, numpad6, numpad1, numpad2, numpad3, numpad0, numpad., numpadenter
        pkeys: p1, p2, p3, p4, p5
        fkeys: f1, f2, f3, f4, f5, f6, f7, f8, f9, f10, f11, f12
        media: play, stop, playlast, playnext
Keys:
        '
        +
        ,
        -
        .
        0
        1
        2
        3
        4
        5
        6
        7
        8
        9
        <
        \
        a
        altgr
        b
        c
        capslock
        d
        del
        delete
        downarrow
        e
        end
        enter
        esc
        f
        f1
        f10
        f11
        f12
        f2
        f3
        f4
        f5
        f6
        f7
        f8
        f9
        fn
        g
        h
        home
        i
        insert
        j
        k
        l
        lalt
        lcontrol
        leftarrow
        lshift
        m
        n
        numlock
        numpad*
        numpad+
        numpad-
        numpad.
        numpad/
        numpad0
        numpad1
        numpad2
        numpad3
        numpad4
        numpad5
        numpad6
        numpad7
        numpad8
        numpad9
        numpadenter
        o
        p
        p1
        p2
        p3
        p4
        p5
        pause
        pgdown
        pgup
        play
        playlast
        playnext
        prtscrn
        q
        r
        rctrl
        rightarrow
        rshift
        s
        sclock
        stop
        t
        tab
        u
        uparrow
        v
        w
        windows
        x
        y
        z
        ~
        «
        ´
        º
        ç
```

</details>