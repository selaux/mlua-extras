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
--- **B:** Variant B Data
--- **D:** Variant D Data
--- @field [1] string | integer
--- Current variant name
--- @field _variant string
--- **C:** Age of variant C
--- @field age any | integer
--- Get the direction [Getter]
--- Get the direction [Setter]
--- @field direction string
--- @field name any | string
local _CLASS__Custom_ = {
	--- Static field provided to Lua
	--- @type integer
	COUNT = 10,
	--- Full list of variant name
	--- @type string[]
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

--- @class CustomC: _Custom

--- @class CustomD: _Custom

--- @alias Kind CustomA | CustomB | CustomC | CustomD

