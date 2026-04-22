--- @meta

--- Structured Data
--- @class Data
--- Name of the data source
--- @field name string
local _CLASS_Data_ = {
	--- @param self Data
  --- @return string
  get_data = function(self) end,
	__metatable = {
		--- @param param1 userdata
    --- @param param2 any
    --- @return any
    __index = function(param1, param2) end,
		--- @param param1 userdata
    --- @param param2 any
    --- @param param3 any
    --- @return any | nil
    __newindex = function(param1, param2, param3) end,
		--- This method is called last.
		--- 
		--- use `#[field(skip)]` for fields that are assigned to the index
		--- to allow for them to overridden in this impl
		--- @param self Data
    --- @param idx integer
    --- @return any
    __usr_index = function(self, idx) end,
		--- This method is called last.
		--- 
		--- use `#[field(skip)]` for fields that are assigned to the index
		--- to allow for them to overridden in this impl
		--- @param self Data
    --- @param idx integer
    --- @param value any
    __usr_newindex = function(self, idx, value) end,
  }
}

--- Kind of action
--- @class _Custom
--- Current variant name
--- @field _variant string
--- Get the direction [Getter]
--- Get the direction [Setter]
--- @field direction string
local _CLASS__Custom_ = {
	--- Static field provided to Lua
	COUNT = 10,
	INF = math.huge,
	NAN = 0/0,
	--- Static field fetched from calling this
	--- function once
	PI = 3.14,
	--- Full list of variant name
	_variants = {"A", "B", "C", "D"},
	--- Get the message based on the variant
	--- @param self _Custom
  --- @return string
  message = function(self) end,
	__metatable = {
		--- @param param1 userdata
    --- @param param2 any
    --- @return any
    __index = function(param1, param2) end,
		--- @param param1 userdata
    --- @param param2 any
    --- @param param3 any
    --- @return any | nil
    __newindex = function(param1, param2, param3) end,
  }
}

--- @class CustomA: _Custom

--- @class CustomB: _Custom
--- Variant B Data
--- @field [1] string

--- @class CustomC: _Custom
--- Age of variant C
--- @field age integer
--- @field name string

--- @class CustomD: _Custom
--- Variant D Data
--- @field [1] integer

--- @alias Custom CustomA | CustomB | CustomC | CustomD

