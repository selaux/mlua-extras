-- Structured Data
declare class Data
	-- Name of the data source
	name: string
	function get_data(self): string
	-- This method is called last.
	-- 
	-- use `#[field(skip)]` for fields that are assigned to the index
	-- to allow for them to overridden in this impl
	function __usr_index(self, idx: number): any
	-- This method is called last.
	-- 
	-- use `#[field(skip)]` for fields that are assigned to the index
	-- to allow for them to overridden in this impl
	function __usr_newindex(self, idx: number, value: any): ()
end

declare function Data___index(param1: any, param2: any): any
declare function Data___newindex(param1: any, param2: any, param3: any): any?

declare Data: {
	__index: typeof(Data___index),
	__newindex: typeof(Data___newindex),
}

-- Kind of action
declare class _Custom
	-- Current variant name
	_variant: string
	-- Get the direction [Getter]
	-- Get the direction [Setter]
	direction: string
	-- Get the message based on the variant
	function message(self): string
end

declare function _Custom___index(param1: any, param2: any): any
declare function _Custom___newindex(param1: any, param2: any, param3: any): any?

declare _Custom: {
	-- Static field provided to Lua
	COUNT: number,
	INF: number,
	NAN: number,
	-- Static field fetched from calling this
	-- function once
	PI: number,
	-- Full list of variant name
	_variants: { string },
	__index: typeof(_Custom___index),
	__newindex: typeof(_Custom___newindex),
}

declare class CustomA extends _Custom
end

declare class CustomB extends _Custom
end

declare class CustomC extends _Custom
	-- Age of variant C
	age: number
	name: string
end

declare class CustomD extends _Custom
end

export type Custom = CustomA | CustomB | CustomC | CustomD
