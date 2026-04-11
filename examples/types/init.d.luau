export type System = "Black" | "Red" | "Green" | "Yellow" | "Blue" | "Cyan" | "Magenta" | "White"

export type Color = System | integer | { integer }

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
