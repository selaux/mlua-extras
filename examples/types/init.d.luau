export type System = SystemColorBlack | SystemColorRed | SystemColorGreen | SystemColorYellow | SystemColorBlue | SystemColorCyan | SystemColorMagenta | SystemColorWhite

export type SystemColorEnum = "Black" | "Red" | "Green" | "Yellow" | "Blue" | "Cyan" | "Magenta" | "White"

declare class _SystemColor
end

declare class SystemColorBlack
end

declare class SystemColorRed
end

declare class SystemColorGreen
end

declare class SystemColorYellow
end

declare class SystemColorBlue
end

declare class SystemColorCyan
end

declare class SystemColorMagenta
end

declare class SystemColorWhite
end

export type Color = ColorSystem | ColorXterm | ColorRgb

export type ColorEnum = "System" | "Xterm" | "Rgb"

-- Representation of a color
declare class _Color
	function __tostring(self): string
end

declare class ColorSystem
end

declare class ColorXterm
end

declare class ColorRgb
end

-- This is a doc comment section for the overall type
declare class Example
	-- Example complex type
	color: Color
	function __tostring(self): string
end

declare Example: {
	-- Log a specific format with any lua types
	LogAny: (format: string, ...: any) -> (),
	-- print all items
	printAll: (...: any) -> (),
}

declare example: Example

declare function greet(name: string): ()

declare function printColor(color: Color): ()
