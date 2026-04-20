export type SystemColorEnum = "Black" | "Red" | "Green" | "Yellow" | "Blue" | "Cyan" | "Magenta" | "White"

declare class _SystemColor
end

declare class SystemColorBlack extends _SystemColor
end

declare class SystemColorRed extends _SystemColor
end

declare class SystemColorGreen extends _SystemColor
end

declare class SystemColorYellow extends _SystemColor
end

declare class SystemColorBlue extends _SystemColor
end

declare class SystemColorCyan extends _SystemColor
end

declare class SystemColorMagenta extends _SystemColor
end

declare class SystemColorWhite extends _SystemColor
end

export type System = SystemColorBlack | SystemColorRed | SystemColorGreen | SystemColorYellow | SystemColorBlue | SystemColorCyan | SystemColorMagenta | SystemColorWhite

export type ColorEnum = "System" | "Xterm" | "Rgb"

--- Representation of a color
declare class _Color
	function __tostring(self): string
end

declare class ColorSystem extends _Color
end

declare class ColorXterm extends _Color
end

declare class ColorRgb extends _Color
end

export type Color = ColorSystem | ColorXterm | ColorRgb

--- This is a doc comment section for the overall type
declare class Example
	--- Example complex type
	color: Color
	function __tostring(self): string
end

--- Log a specific format with any lua types
--- @param format string -- String to pass to the formatter.
--- @param ... any -- Arguments to pass to the formatter.
declare function Example_LogAny(format: string, ...: any): ()
--- print all items
declare function Example_printAll(...: any): ()

declare Example: {
	LogAny: typeof(Example_LogAny),
	printAll: typeof(Example_printAll),
}

declare example: Example

--- Greet someone
--- @param name string -- Name of the person to greet
declare function greet(name: string): ()

--- @param color Color -- Color to print to stdout
declare function printColor(color: Color): ()
