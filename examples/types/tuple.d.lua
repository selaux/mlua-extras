--- @meta

--- @class Kind
--- @field [1] string
--- @field data string
--- @field name string
local _CLASS_Kind_ = {
  --- @param self Kind
  --- @return string 
  __variant = function(self) end,
  --- @param self Kind
  --- @return string 
  getData = function(self) end,
  __metatable = {
    --- @param self Kind
    --- @param param0 integer 
    --- @return string 
    __index = function(self, param0) end,
  }
}