# Slint Painter Example (Skia)

This repository is a little case-study to implement a painter (canvas) with the [Slint GUI Toolkit](https://github.com/slint-ui/slint).
The drawn objects are **not** selectable or changeable, as this approach draws the paths directly to an image buffer via [tiny-skia](https://github.com/RazrFalcon/tiny-skia).

Features:
- :wrench: different brushes: Freehand, Line, Rectangle, Ellipse
- :bulb: path preview
- :pencil: change brush size
- :art: change brush color (not in GUI - color picker missing)
- :arrows_counterclockwise: undo and redo all operations

Annoyances:
- fixed dimensions (at creation time)
- Touch area and displayed paths differ when the window is resized
- no operations on individually drawn objects
- tiny-skia engine may be slow in the path preview
- unclear how to create a **reusable slint component** out of that :confused:
