--- @meta

--- Simple Counter
--- @class Counter
--- Direction of the counter
--- @field direction string
--- @field value integer
local _CLASS_Counter_ = {
	--- The default count
	COUNT = 10,
	--- Min count value
	MIN = 0,
	--- Max count value
	max = 9223372036854775807,
	--- Create a new table
	--- @param self Counter
  --- @return table
  create_table = function(self) end,
	--- Fetch the global counter online
	--- @param self Counter
  --- @param url string
  --- @return string
  fetch = function(self, url) end,
	--- Get the current counter value
	--- @param self Counter
  --- @return integer
  get = function(self) end,
	--- Increment the counter
	--- @param self Counter
  increment = function(self) end,
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
		--- String representation of the counter
		--- @param self Counter
    --- @return string
    __tostring = function(self) end,
  }
}

