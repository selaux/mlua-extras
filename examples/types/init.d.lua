--- @meta

--- @alias System "Black"
---  | "Red"
---  | "Green"
---  | "Yellow"
---  | "Blue"
---  | "Cyan"
---  | "Magenta"
---  | "White"

--- @alias Color System
---  | integer
---  | [integer, integer, integer]

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
  --- @param param0 any 
  printAll = function(param0) end,
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

