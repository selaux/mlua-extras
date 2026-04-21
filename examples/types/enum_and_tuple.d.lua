--- @meta

--- @class _Kind
--- @field _variant KindVariant
--- Kind::A variant data
--- @field [1] string
--- @field data string
--- @field name string
local _CLASS__Kind_ = {
  --- @param self _Kind
  --- @return string
  getData = function(self) end,
  __metatable = {
    --- @param self _Kind
    --- @param param1 integer
    --- @return string
    __index = function(self, param1) end,
  }
}

--- @class KindA: _Kind

--- @class KindB: _Kind

--- @class KindC: _Kind

--- @alias Kind KindA | KindB | KindC

--- @alias KindVariant "A" | "B" | "C"

