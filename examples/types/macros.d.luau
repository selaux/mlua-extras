-- Simple Counter
declare class Counter
	-- Direction of the counter
	direction: string
	value: number
	-- Create a new table
	function create_table(self): table
	-- Fetch the global counter online
	function fetch(self, url: string): string
	-- Get the current counter value
	function get(self): number
	-- Increment the counter
	function increment(self): ()
	-- String representation of the counter
	function __tostring(self): string
end

declare function Counter___index(param1: any, param2: any): any
declare function Counter___newindex(param1: any, param2: any, param3: any): any?

declare Counter: {
	-- The default count
	COUNT: number,
	-- Min count value
	MIN: number,
	-- Max count value
	max: number,
	__index: typeof(Counter___index),
	__newindex: typeof(Counter___newindex),
}
