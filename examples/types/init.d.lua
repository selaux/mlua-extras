--- @meta

--- @alias System SystemColorBlack | SystemColorRed | SystemColorGreen | SystemColorYellow | SystemColorBlue | SystemColorCyan | SystemColorMagenta | SystemColorWhite

--- @alias SystemColorEnum "Black"
---  | "Red"
---  | "Green"
---  | "Yellow"
---  | "Blue"
---  | "Cyan"
---  | "Magenta"
---  | "White"

--- @class _SystemColor

--- @class SystemColorBlack: _SystemColor

--- @class SystemColorRed: _SystemColor

--- @class SystemColorGreen: _SystemColor

--- @class SystemColorYellow: _SystemColor

--- @class SystemColorBlue: _SystemColor

--- @class SystemColorCyan: _SystemColor

--- @class SystemColorMagenta: _SystemColor

--- @class SystemColorWhite: _SystemColor

--- @alias Color ColorSystem | ColorXterm | ColorRgb

--- @alias ColorEnum "System"
---  | "Xterm"
---  | "Rgb"

--- Representation of a color
--- @class _Color
local _CLASS__Color_ = {
  __metatable = {
    --- @param self _Color
    --- @return string
    __tostring = function(self) end,
  }
}

--- @class ColorSystem: _Color

--- @class ColorXterm: _Color

--- @class ColorRgb: _Color

--- This is a doc comment section for the overall type
--- @class Example
--- Example complex type
--- @field color Color
local _CLASS_Example_ = {
  --- Log a specific format with any lua types
  --- @param format string String to pass to the formatter.
  --- @param ... any Arguments to pass to the formatter.
  LogAny = function(format, ...) end,
  --- print all items
  --- @param ... any
  printAll = function(...) end,
  __metatable = {
    --- @param self Example
    --- @return string
    __tostring = function(self) end,
  }
}

--- @type Example
example = nil

--- @param name string Name of the person to greet
function greet(name) end

--- @param color Color Color to print to stdout
function printColor(color) end

