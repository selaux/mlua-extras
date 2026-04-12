--- @meta

--- @alias Kind KindA | KindB | KindC

--- @class _Kind
--- @field __variant "A" | "B" | "C"
--- @field [1] string
--- @field data string
--- @field name string
local _CLASS__Kind_ = {
  --- @param self _Kind
  --- @return string
  getData = function(self) end,
  __metatable = {
    --- @param self _Kind
    --- @param param0 integer
    --- @return string
    __index = function(self, param0) end,
  }
}

--- @class KindA: _Kind

--- @class KindB: _Kind

--- @class KindC: _Kind

